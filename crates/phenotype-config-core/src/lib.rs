//! Phenotype Config Core - Cascading TOML config loader with env overrides.
//!
//! Provides a cascading configuration system that searches for TOML files
//! in standard locations and merges with environment variable overrides.
//!
//! # Search Order (lowest to highest priority)
//!
//! 1. System config: `/etc/phenotype/config.toml`
//! 2. User config: `~/.config/phenotype/config.toml`
//! 3. Project config: `./config.toml`
//! 4. Env vars: `PHENOTYPE_*` prefix

use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML parse error: {0}")]
    Parse(#[from] toml::de::Error),

    #[error("config key not found: {0}")]
    KeyNotFound(String),
}

/// Result type for config operations.
pub type Result<T> = std::result::Result<T, ConfigError>;

/// Configuration value that can come from file or environment.
#[derive(Debug, Clone, Deserialize)]
pub struct ConfigValue {
    pub value: toml::Value,
    pub source: ConfigSource,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConfigSource {
    System,
    User,
    Project,
    Env,
}

/// Cascading TOML config loader with environment overrides.
///
/// Searches for config files in standard locations and merges them,
/// with environment variables taking highest priority.
#[derive(Debug, Clone)]
pub struct ConfigLoader {
    values: HashMap<String, ConfigValue>,
}

impl Default for ConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigLoader {
    /// Creates a new config loader with default search paths.
    pub fn new() -> Self {
        let mut loader = Self {
            values: HashMap::new(),
        };
        loader.load();
        loader
    }

    /// Load configs from standard search paths.
    pub fn load(&mut self) {
        // System config
        if let Some(path) = Self::system_config_path() {
            self.load_file(&path, ConfigSource::System);
        }

        // User config
        if let Some(path) = Self::user_config_path() {
            self.load_file(&path, ConfigSource::User);
        }

        // Project config
        if let Some(path) = Self::project_config_path() {
            self.load_file(&path, ConfigSource::Project);
        }

        // Environment variables
        self.load_env();
    }

    /// Load a TOML file into the config.
    fn load_file(&mut self, path: &PathBuf, source: ConfigSource) {
        if path.exists() {
            match fs::read_to_string(path) {
                Ok(contents) => match toml::from_str::<toml::Value>(&contents) {
                    Ok(value) => self.merge_value(value, source),
                    Err(e) => {
                        eprintln!("warning: failed to parse {}: {}", path.display(), e);
                    }
                },
                Err(e) => {
                    eprintln!("warning: failed to read {}: {}", path.display(), e);
                }
            }
        }
    }

    /// Merge a TOML value into the config.
    fn merge_value(&mut self, value: toml::Value, source: ConfigSource) {
        if let toml::Value::Table(table) = value {
            for (key, val) in table {
                self.values.insert(
                    key.clone(),
                    ConfigValue {
                        value: val,
                        source,
                    },
                );
            }
        }
    }

    /// Load configuration from environment variables.
    fn load_env(&mut self) {
        let prefix = "PHENOTYPE_";
        for (key, val) in env::vars() {
            if key.starts_with(prefix) {
                let config_key = key.strip_prefix(prefix).unwrap().to_lowercase();
                if let Ok(value) = val.parse::<toml::Value>() {
                    self.values.insert(
                        config_key,
                        ConfigValue {
                            value,
                            source: ConfigSource::Env,
                        },
                    );
                }
            }
        }
    }

    /// Get the system config path.
    fn system_config_path() -> Option<PathBuf> {
        #[cfg(unix)]
        {
            Some(PathBuf::from("/etc/phenotype/config.toml"))
        }
        #[cfg(not(unix))]
        {
            None
        }
    }

    /// Get the user config path.
    fn user_config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("phenotype").join("config.toml"))
    }

    /// Get the project config path.
    fn project_config_path() -> Option<PathBuf> {
        Some(PathBuf::from("./config.toml"))
    }

    /// Get a string config value.
    pub fn get_str(&self, key: &str) -> Option<&str> {
        self.values.get(key).and_then(|v| v.value.as_str())
    }

    /// Get a config value as a specific type.
    pub fn get<T: Deserialize>(&self, key: &str) -> Result<T> {
        self.values
            .get(key)
            .ok_or_else(|| ConfigError::KeyNotFound(key.to_string()))
            .and_then(|v| serde_json::from_value(v.value.clone()).map_err(ConfigError::Io))
    }

    /// Check if a key exists.
    pub fn contains(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }

    /// Get the source of a config value.
    pub fn source(&self, key: &str) -> Option<ConfigSource> {
        self.values.get(key).map(|v| v.source)
    }

    /// Get all keys.
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.values.keys().map(|k| k.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_loader() {
        let loader = ConfigLoader::new();
        // Should not panic even if no config files exist
        assert!(loader.keys().count() >= 0);
    }

    #[test]
    fn test_project_config_path() {
        let path = ConfigLoader::project_config_path();
        assert!(path.is_some());
        assert_eq!(path.unwrap(), PathBuf::from("./config.toml"));
    }
}
