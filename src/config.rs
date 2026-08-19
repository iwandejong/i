use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
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
                ".git", ".hg", ".svn", "node_modules", ".venv", "venv",
                "__pycache__", "target", ".cache", ".npm", ".cargo",
                ".rustup", ".Trash", "dist", "build", "proc", "sys", "dev",
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
        let path = Self::config_path();
        match std::fs::read_to_string(&path) {
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
            Err(_) => Config::default(),
        }
    }
}
