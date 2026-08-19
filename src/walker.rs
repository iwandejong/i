use std::fs;
use std::path::{Path, PathBuf};

/// Recursively collect directory paths under `root`, pruning any directory
/// whose name appears in `excludes` (pruned dirs are neither listed nor
/// descended into), optionally skipping dot-directories, optionally capped
/// at `max_depth` levels below root.
pub fn build_index(
    root: &Path,
    excludes: &[String],
    include_hidden: bool,
    max_depth: Option<usize>,
    max_entries: usize,
) -> Vec<PathBuf> {
    let mut out = Vec::new();
    walk(
        root,
        root,
        excludes,
        include_hidden,
        max_depth,
        max_entries,
        0,
        &mut out,
    );
    out
}

// Private recursive helper threading fixed per-call context (root, excludes,
// caps) plus the accumulating `out` — bundling that into a struct would add
// a type for one call site's benefit.
#[allow(clippy::too_many_arguments)]
fn walk(
    root: &Path,
    dir: &Path,
    excludes: &[String],
    include_hidden: bool,
    max_depth: Option<usize>,
    max_entries: usize,
    depth: usize,
    out: &mut Vec<PathBuf>,
) {
    if out.len() >= max_entries {
        return;
    }
    if let Some(max) = max_depth {
        if depth > max {
            return;
        }
    }

    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return, // permission denied, gone, etc — skip quietly
    };

    for entry in entries.flatten() {
        if out.len() >= max_entries {
            return;
        }
        let file_type = match entry.file_type() {
            Ok(ft) => ft,
            Err(_) => continue,
        };
        // Skip symlinks entirely to avoid cycles and double-counting.
        if file_type.is_symlink() || !file_type.is_dir() {
            continue;
        }

        let path = entry.path();
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n,
            None => continue,
        };

        if !include_hidden && name.starts_with('.') {
            continue;
        }
        if excludes.iter().any(|e| e == name) {
            continue;
        }

        if path != root {
            out.push(path.clone());
        }
        walk(
            root,
            &path,
            excludes,
            include_hidden,
            max_depth,
            max_entries,
            depth + 1,
            out,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Each test gets its own subtree under a fresh temp dir so tests can
    // run in parallel without clobbering each other's fixtures.
    fn fixture(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("i_walker_test_{name}"));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn hidden_dirs_skipped_by_default_but_included_when_asked() {
        let root = fixture("hidden");
        fs::create_dir_all(root.join(".config/nested")).unwrap();
        fs::create_dir_all(root.join("visible")).unwrap();

        let excluded = build_index(&root, &[], false, None, 50_000);
        assert!(excluded.contains(&root.join("visible")));
        assert!(!excluded.iter().any(|p| p.starts_with(root.join(".config"))));

        let included = build_index(&root, &[], true, None, 50_000);
        assert!(included.contains(&root.join(".config")));
        assert!(included.contains(&root.join(".config/nested")));
    }

    #[test]
    fn max_depth_caps_recursion() {
        let root = fixture("depth");
        fs::create_dir_all(root.join("a/b/c/d")).unwrap();

        let shallow = build_index(&root, &[], false, Some(1), 50_000);
        assert!(shallow.contains(&root.join("a")));
        assert!(shallow.contains(&root.join("a/b")));
        assert!(!shallow.contains(&root.join("a/b/c")));

        let deep = build_index(&root, &[], false, Some(10), 50_000);
        assert!(deep.contains(&root.join("a/b/c/d")));
    }

    #[test]
    fn max_entries_caps_total_collected() {
        let root = fixture("entries");
        for i in 0..10 {
            fs::create_dir_all(root.join(format!("dir{i}"))).unwrap();
        }

        let out = build_index(&root, &[], false, None, 3);
        assert!(out.len() <= 3);
    }

    #[test]
    fn excluded_names_are_pruned_not_just_hidden() {
        let root = fixture("excludes");
        fs::create_dir_all(root.join("node_modules/pkg")).unwrap();
        fs::create_dir_all(root.join("src")).unwrap();

        let out = build_index(&root, &["node_modules".to_string()], false, None, 50_000);
        assert!(out.contains(&root.join("src")));
        assert!(!out.iter().any(|p| p.starts_with(root.join("node_modules"))));
    }
}
