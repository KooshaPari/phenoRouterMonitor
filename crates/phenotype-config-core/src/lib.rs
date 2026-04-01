//! Core configuration types and traits for Phenotype crates.
//!
//! This crate provides shared configuration abstractions used across
//! the Phenotype ecosystem.

use serde::{de::DeserializeOwned, Serialize};
use std::path::Path;

/// Configuration source priority (higher = takes precedence).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Priority(u8);

impl Priority {
    /// Lowest priority (default).
    pub const LOWEST: u8 = 0;
    /// Default configuration priority.
    pub const DEFAULT: u8 = 50;
    /// Environment variable priority.
    pub const ENV: u8 = 75;
    /// Highest priority (user overrides).
    pub const HIGHEST: u8 = 100;

    /// Creates a new priority with the given value.
    pub const fn new(value: u8) -> Self {
        Self(value)
    }

    /// Returns the raw priority value.
    pub const fn value(&self) -> u8 {
        self.0
    }
}

impl From<u8> for Priority {
    fn from(value: u8) -> Self {
        Self::new(value)
    }
}

impl From<Priority> for u8 {
    fn from(p: Priority) -> Self {
        p.0
    }
}

/// Configuration source that can provide values.
#[derive(Debug, Clone)]
pub struct ConfigSource {
    /// Source name (e.g., "env", "file", "default").
    pub name: String,
    /// Priority of this source.
    pub priority: Priority,
}

impl ConfigSource {
    /// Creates a new configuration source.
    pub fn new(name: impl Into<String>, priority: Priority) -> Self {
        Self {
            name: name.into(),
            priority,
        }
    }

    /// Creates a source with default priority.
    pub fn default_source(name: impl Into<String>) -> Self {
        Self::new(name, Priority::DEFAULT)
    }
}

/// Trait for configuration loaders.
pub trait ConfigLoader: Send + Sync {
    /// The error type for loading.
    type Error: std::error::Error + Send + Sync + 'static;

    /// loads configuration from this source.
    fn load<T: DeserializeOwned + Serialize>(&self) -> Result<T, Self::Error>;

    /// Returns the source name.
    fn source_name(&self) -> &str;
}

/// Trait for configuration validation.
pub trait ValidateConfig {
    /// Validates the configuration.
    fn validate(&self) -> Result<(), ConfigValidationError>;
}

/// Configuration validation error.
#[derive(Debug, thiserror::Error)]
#[error("configuration validation failed: {0}")]
pub struct ConfigValidationError(pub String);

impl ConfigValidationError {
    /// Creates a new validation error.
    pub fn new(msg: impl Into<String>) -> Self {
        Self(msg.into())
    }
}

/// Environment-based configuration source.
#[derive(Debug, Clone, Default)]
pub struct EnvConfig {
    prefix: Option<String>,
}

impl EnvConfig {
    /// Creates a new environment configuration source.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new environment configuration source with a prefix.
    pub fn with_prefix(prefix: impl Into<String>) -> Self {
        Self {
            prefix: Some(prefix.into()),
        }
    }

    /// Gets a value from the environment.
    pub fn get(&self, key: &str) -> Option<String> {
        let key = match &self.prefix {
            Some(p) => format!("{}_{}", p, key),
            None => key.to_uppercase(),
        };
        std::env::var(key).ok()
    }
}

impl ConfigLoader for EnvConfig {
    type Error = std::env::VarError;

    fn load<T: DeserializeOwned + Serialize>(&self) -> Result<T, Self::Error> {
        // For simple env loading, we parse the entire env as JSON
        let env_vars: std::collections::HashMap<String, String> =
            std::env::vars().collect();

        let json = serde_json::to_string(&env_vars)?;

        serde_json::from_str(&json).map_err(|_| std::env::VarError::NotPresent)
    }

    fn source_name(&self) -> &str {
        "environment"
    }
}

/// File-based configuration source.
#[derive(Debug, Clone)]
pub struct FileConfig {
    path: std::path::PathBuf,
    format: ConfigFormat,
}

/// Supported configuration file formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConfigFormat {
    #[default]
    Json,
    Toml,
    Yaml,
}

impl ConfigFormat {
    /// Detects format from file extension.
    pub fn from_path(path: &Path) -> Option<Self> {
        match path.extension()?.to_str()? {
            "json" => Some(Self::Json),
            "toml" => Some(Self::Toml),
            "yaml" | "yml" => Some(Self::Yaml),
            _ => None,
        }
    }
}

impl FileConfig {
    /// Creates a new file configuration source.
    pub fn new(path: impl Into<std::path::PathBuf>) -> Self {
        let path = path.into();
        let format = ConfigFormat::from_path(&path).unwrap_or(ConfigFormat::Json);
        Self { path, format }
    }

    /// Loads configuration from the file.
    pub fn load<T: DeserializeOwned + Serialize>(&self) -> Result<T, FileConfigError> {
        let content = std::fs::read_to_string(&self.path)?;

        match self.format {
            ConfigFormat::Json => serde_json::from_str(&content).map_err(FileConfigError::Parse),
            ConfigFormat::Toml => toml::from_str(&content).map_err(FileConfigError::Parse),
            ConfigFormat::Yaml => serde_yaml::from_str(&content).map_err(FileConfigError::Parse),
        }
    }
}

/// File configuration error.
#[derive(Debug, thiserror::Error)]
pub enum FileConfigError {
    #[error("failed to read file: {0}")]
    Io(#[from] std::io::Error),

    #[error("failed to parse: {0}")]
    Parse(String),
}

impl From<serde_json::Error> for FileConfigError {
    fn from(e: serde_json::Error) -> Self {
        Self::Parse(e.to_string())
    }
}

impl From<toml::de::Error> for FileConfigError {
    fn from(e: toml::de::Error) -> Self {
        Self::Parse(e.to_string())
    }
}

/// Merges multiple configuration sources.
pub fn merge_configs<T: DeserializeOwned + Serialize + Default>(
    sources: &[&dyn ConfigLoader],
) -> Result<T, Box<dyn std::error::Error>>
where
    <dyn ConfigLoader>::Error: 'static,
{
    let mut merged = serde_json::Map::new();

    for source in sources {
        match source.load::<serde_json::Value>() {
            Ok(value) => {
                if let serde_json::Value::Object(map) = value {
                    merged.extend(map);
                }
            }
            Err(_) => continue,
        }
    }

    serde_json::from_value(serde_json::Value::Object(merged))
    serde_json::from_value(serde_json::Value::Object(merged))
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>))

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_ordering() {
        let low = Priority::LOWEST;
        let high = Priority::HIGHEST;
        assert!(low < high);
    }

    #[test]
    fn test_env_config() {
        std::env::set_var("TEST_VAR", "test_value");
        let config = EnvConfig::new();
        assert_eq!(config.get("TEST_VAR"), Some("test_value".to_string()));
        std::env::remove_var("TEST_VAR");
    }

    #[test]
    fn test_config_format_detection() {
        assert_eq!(
            ConfigFormat::from_path(std::path::Path::new("config.json")),
            Some(ConfigFormat::Json)
        );
        assert_eq!(
            ConfigFormat::from_path(std::path::Path::new("config.toml")),
            Some(ConfigFormat::Toml)
        );
        assert_eq!(
            ConfigFormat::from_path(std::path::Path::new("config.yaml")),
            Some(ConfigFormat::Yaml)
        );
    }
}
