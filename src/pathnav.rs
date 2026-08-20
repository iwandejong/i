use std::path::{Path, PathBuf};

/// Given a starting root and the raw text typed by the user (everything
/// after the `@`), walk through any leading, unambiguous path segments
/// (".." to go up, "~" for home, a leading "/" for filesystem root, or an
/// exact existing child directory name) and return the directory those
/// segments resolve to, plus whatever's left over to fuzzy-match.
///
/// Examples (starting root = /home/iwan/code/project):
///   "test"        -> (project, "test")
///   "../test"     -> (code, "test")
///   "../../test"  -> (iwan, "test")
///   "../"         -> (code, "")
///   "src/comp"    -> (project/src, "comp")   [only if "src" exists]
///   "~/Doc"       -> (/home/iwan, "Doc")
///   "/etc/ngi"    -> (/etc, "ngi")
pub fn resolve(start_root: &Path, typed: &str) -> (PathBuf, String) {
    let mut root = start_root.to_path_buf();
    let mut rest = typed;

    // Leading "~" jumps home, only meaningful as the very first thing typed.
    if let Some(stripped) = rest.strip_prefix('~') {
        if let Some(home) = dirs::home_dir() {
            root = home;
        }
        rest = stripped.trim_start_matches('/');
        if !rest.contains('/') {
            return (root, rest.to_string());
        }
    }

    // Leading "/" jumps to filesystem root.
    if let Some(stripped) = rest.strip_prefix('/') {
        root = PathBuf::from("/");
        rest = stripped;
    }

    while let Some(slash_pos) = rest.find('/') {
        let segment = &rest[..slash_pos];

        let next_root = match segment {
            "" | "." => root.clone(),
            ".." => root
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or(root.clone()),
            name => {
                let candidate = root.join(name);
                if candidate.is_dir() {
                    candidate
                } else {
                    // Doesn't resolve to a real directory — stop parsing
                    // path segments and treat everything from here on as
                    // the fuzzy query instead.
                    break;
                }
            }
        };

        root = next_root;
        rest = &rest[slash_pos + 1..];
    }

    (root, rest.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_query_stays_at_root() {
        let start = PathBuf::from("/a/b/c");
        let (root, q) = resolve(&start, "test");
        assert_eq!(root, PathBuf::from("/a/b/c"));
        assert_eq!(q, "test");
    }

    #[test]
    fn dotdot_pops_root() {
        let start = PathBuf::from("/a/b/c");
        let (root, q) = resolve(&start, "../test");
        assert_eq!(root, PathBuf::from("/a/b"));
        assert_eq!(q, "test");
    }

    #[test]
    fn double_dotdot() {
        let start = PathBuf::from("/a/b/c");
        let (root, q) = resolve(&start, "../../test");
        assert_eq!(root, PathBuf::from("/a"));
        assert_eq!(q, "test");
    }

    #[test]
    fn trailing_slash_empty_query() {
        let start = PathBuf::from("/a/b/c");
        let (root, q) = resolve(&start, "../");
        assert_eq!(root, PathBuf::from("/a/b"));
        assert_eq!(q, "");
    }

    #[test]
    fn root_jump() {
        // "/etc" isn't guaranteed to exist on every platform (e.g. Windows
        // CI), so resolve segments against a real directory instead: a temp
        // dir keyed by test name, matching walker.rs's fixture convention.
        // resolve()'s leading-"/" handling maps to PathBuf::from("/"), which
        // on Windows is a driveless root (resolved against the *current*
        // drive) — so the fixture must live under the current drive too.
        // std::env::temp_dir() is often on a different drive than the repo
        // checkout (e.g. CI runs from D:\ while temp is on C:\), which made
        // every segment fail to resolve as a real directory. Use
        // current_dir()'s drive instead, then strip the drive prefix
        // ("C:") before comparing.
        let real_dir = std::env::current_dir()
            .unwrap()
            .join("target")
            .join("i_pathnav_root_jump_fixture");
        std::fs::create_dir_all(&real_dir).unwrap();
        let forward = real_dir.to_str().unwrap().replace('\\', "/");
        let rootless = match forward.split_once(':') {
            Some((_, rest)) => rest.to_string(),
            None => forward,
        };

        let start = PathBuf::from("/a/b/c");
        let (root, q) = resolve(&start, &format!("{rootless}/ngi"));
        assert_eq!(root, PathBuf::from(&rootless));
        assert_eq!(q, "ngi");
    }
}
