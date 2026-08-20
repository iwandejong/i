// Smoke test: build a fake tree, confirm node_modules is pruned and a
// deeply nested "test" dir is still found (the "deeper than this level"
// requirement).
use std::fs;

#[path = "../src/walker.rs"]
mod walker;

#[test]
fn excludes_pruned_and_deep_dirs_still_found() {
    let root = std::env::temp_dir().join("i_playground");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("projectA/src/components")).unwrap();
    fs::create_dir_all(root.join("projectA/tests")).unwrap();
    fs::create_dir_all(root.join("projectA/node_modules/foo/test")).unwrap();
    fs::create_dir_all(root.join("projectB/tests_deep/nested/testing_utils")).unwrap();

    let excludes = vec!["node_modules".to_string(), ".git".to_string()];
    let entries = walker::build_index(&root, &excludes, false, None, 50_000);

    let strs: Vec<String> = entries
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    // node_modules itself and everything under it must be pruned.
    assert!(!strs.iter().any(|s| s.contains("node_modules")));

    // But a deeply nested dir several levels down, in a non-excluded
    // subtree, must still show up.
    assert!(strs.iter().any(|s| s.ends_with("testing_utils")));
    assert!(strs.iter().any(|s| s.ends_with("projectA/tests")));

    let _ = fs::remove_dir_all(&root);
}
