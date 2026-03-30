//! # Phenotype Config Core
//!
//! Minimal, composable config loading for Phenotype crates.
//!
//! Loads TOML config from a cascade of sources:
//! 1. System config (`/etc/phenotype/<name>.toml`)
//! 2. User config (`~/.config/phenotype/<name>.toml`)
//! 3. Project config (`./<name>.toml`)
//! 4. Custom paths via `with_path()`

use serde::de::DeserializeOwned;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("toml parse error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("config not found at any search path")]
    NotFound,
}

pub type Result<T> = std::result::Result<T, ConfigError>;

/// Config loader with cascading source resolution.
pub struct ConfigLoader {
    name: String,
    search_paths: Vec<PathBuf>,
}

impl ConfigLoader {
    /// Create a new config loader for the given config name (e.g. "agileplus").
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        let mut search_paths = Vec::new();

        // System config
        search_paths.push(PathBuf::from(format!("/etc/phenotype/{name}.toml")));

        // User config
        if let Some(config_dir) = dirs::config_dir() {
            search_paths.push(config_dir.join("phenotype").join(format!("{name}.toml")));
        }

        // Project config (cwd)
        search_paths.push(PathBuf::from(format!("{name}.toml")));

        Self { name, search_paths }
    }

    /// Add a custom search path.
    pub fn with_path(mut self, path: impl Into<PathBuf>) -> Self {
        let pos = self.search_paths.len().saturating_sub(1);
        self.search_paths.insert(pos, path.into());
        self
    }

    /// Load and deserialize config from the first found file.
    pub fn load<T: DeserializeOwned>(&self) -> Result<T> {
        for path in &self.search_paths {
            if path.exists() {
                let content = std::fs::read_to_string(path)?;
                let config: T = toml::from_str(&content)?;
                return Ok(config);
            }
        }
        Err(ConfigError::NotFound)
    }

    /// Load from a specific file path.
    pub fn load_from<T: DeserializeOwned>(path: &Path) -> Result<T> {
        let content = std::fs::read_to_string(path)?;
        let config: T = toml::from_str(&content)?;
        Ok(config)
    }

    /// Return the list of paths that will be searched.
    pub fn search_paths(&self) -> &[PathBuf] {
        &self.search_paths
    }

    /// Get the config name.
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use std::io::Write;

    #[derive(Debug, Deserialize, PartialEq)]
    struct TestConfig {
        name: String,
        port: u16,
    }

    #[test]
    fn load_from_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.toml");
        let mut f = std::fs::File::create(&path).unwrap();
        writeln!(f, "name = \"myapp\"\nport = 8080").unwrap();

        let config: TestConfig = ConfigLoader::load_from(&path).unwrap();
        assert_eq!(config.name, "myapp");
        assert_eq!(config.port, 8080);
    }

    #[test]
    fn not_found() {
        let loader = ConfigLoader::new("nonexistent-config-xyz");
        let result: Result<TestConfig> = loader.load();
        assert!(matches!(result, Err(ConfigError::NotFound)));
    }

    #[test]
    fn custom_search_path() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("custom.toml");
        let mut f = std::fs::File::create(&path).unwrap();
        writeln!(f, "name = \"custom\"\nport = 3000").unwrap();

        let loader = ConfigLoader::new("custom").with_path(dir.path().join("custom.toml"));
        let config: TestConfig = loader.load().unwrap();
        assert_eq!(config.name, "custom");
        assert_eq!(config.port, 3000);
    }

    #[test]
    fn search_paths_populated() {
        let loader = ConfigLoader::new("test");
        assert!(!loader.search_paths().is_empty());
        assert_eq!(loader.name(), "test");
    }
}
