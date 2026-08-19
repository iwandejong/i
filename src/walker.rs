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
