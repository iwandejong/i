use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct Config {
    /// Directory *names* to prune entirely (never descended into, never listed).
    pub excludes: Vec<String>,
    /// Whether to descend into dot-directories (.config, .cache, etc).
    pub include_hidden: bool,
    /// Safety cap on recursion depth below whatever root is currently active.
    /// Keeps an accidental `@../../../../../..` from wandering into huge
    /// system trees. None = unlimited.
    pub max_depth: Option<usize>,
    /// Safety cap on how many directories a single walk will collect before
    /// it gives up early (still shows what it found).
    pub max_entries: usize,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            excludes: vec![
                ".git",
                ".hg",
                ".svn",
                "node_modules",
                ".venv",
                "venv",
                "__pycache__",
                "target",
                ".cache",
                ".npm",
                ".cargo",
                ".rustup",
                ".Trash",
                "dist",
                "build",
                "proc",
                "sys",
                "dev",
                "Library",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
            include_hidden: false,
            max_depth: Some(5),
            max_entries: 50_000,
        }
    }
}

impl Config {
    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("i")
            .join("config.json")
    }

    pub fn load() -> Self {
        Self::load_from(&Self::config_path())
    }

    fn load_from(path: &Path) -> Self {
        match std::fs::read_to_string(path) {
            Ok(contents) => match serde_json::from_str::<Config>(&contents) {
                Ok(cfg) => cfg,
                Err(e) => {
                    eprintln!(
                        "i: failed to parse {}: {} — using defaults",
                        path.display(),
                        e
                    );
                    Config::default()
                }
            },
            Err(_) => {
                // No config yet — write a lean starter file (just "{}", so
                // every field still comes from Config::default() via
                // #[serde(default)]) rather than dumping the full default
                // struct. That way future default changes in a new version
                // of `i` keep applying instead of getting frozen the moment
                // this file was created. Best-effort: an unwritable config
                // dir shouldn't stop the tool from working.
                if let Some(parent) = path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                let _ = std::fs::write(path, "{}\n");
                Config::default()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_path_ends_with_i_config_json() {
        let path = Config::config_path();
        assert_eq!(path.file_name().unwrap(), "config.json");
        assert_eq!(path.parent().unwrap().file_name().unwrap(), "i");
    }

    #[test]
    fn default_has_sane_caps() {
        let cfg = Config::default();
        assert!(!cfg.include_hidden);
        assert_eq!(cfg.max_depth, Some(5));
        assert!(cfg.max_entries > 0);
        assert!(cfg.excludes.iter().any(|e| e == "node_modules"));
        assert!(cfg.excludes.iter().any(|e| e == "Library"));
    }

    #[test]
    fn partial_json_fills_missing_fields_from_defaults() {
        // A user overriding just one field (as the README documents as
        // supported) must not lose the rest of the defaults.
        let cfg: Config = serde_json::from_str(r#"{"include_hidden": true}"#).unwrap();
        assert!(cfg.include_hidden);
        assert_eq!(cfg.max_depth, Config::default().max_depth);
        assert_eq!(cfg.excludes, Config::default().excludes);
    }

    #[test]
    fn full_json_round_trips() {
        let original = Config::default();
        let json = serde_json::to_string(&original).unwrap();
        let parsed: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.excludes, original.excludes);
        assert_eq!(parsed.include_hidden, original.include_hidden);
        assert_eq!(parsed.max_depth, original.max_depth);
        assert_eq!(parsed.max_entries, original.max_entries);
    }

    #[test]
    fn missing_config_creates_lean_starter_file() {
        let path = std::env::temp_dir()
            .join("i_config_test_missing")
            .join("config.json");
        let _ = std::fs::remove_file(&path);

        let cfg = Config::load_from(&path);
        assert_eq!(cfg.excludes, Config::default().excludes);

        // The file on disk should be lean ("{}"), not a full dump of every
        // default field — so a future default change still applies.
        let written = std::fs::read_to_string(&path).unwrap();
        assert_eq!(written.trim(), "{}");

        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn existing_config_is_not_overwritten() {
        let path = std::env::temp_dir()
            .join("i_config_test_existing")
            .join("config.json");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, r#"{"include_hidden": true}"#).unwrap();

        let cfg = Config::load_from(&path);
        assert!(cfg.include_hidden);

        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }
}
