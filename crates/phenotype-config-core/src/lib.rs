//! Configuration loading utilities for the Phenotype ecosystem.

use serde::de::DeserializeOwned;
use serde::Serialize;
use std::path::Path;
use thiserror::Error;

/// Configuration error type.
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("file not found: {0}")]
    NotFound(String),
    #[error("parse error: {0}")]
    Parse(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type for configuration operations.
pub type Result<T> = std::result::Result<T, ConfigError>;

/// Loads JSON configuration from a file.
pub fn load_json<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let content = std::fs::read_to_string(path)?;
    serde_json::from_str(&content).map_err(|e| ConfigError::Parse(e.to_string()))
}

/// Loads TOML configuration from a file.
pub fn load_toml<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let content = std::fs::read_to_string(path)?;
    toml::from_str(&content).map_err(|e| ConfigError::Parse(e.to_string()))
}

/// Loads YAML configuration from a file.
pub fn load_yaml<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let content = std::fs::read_to_string(path)?;
    serde_yaml::from_str(&content).map_err(|e| ConfigError::Parse(e.to_string()))
}

/// Auto-detects format from file extension and loads configuration.
pub fn load_auto<T: DeserializeOwned>(path: &Path) -> Result<T> {
    match path.extension().and_then(|e| e.to_str()) {
        Some("json") => load_json(path),
        Some("toml") => load_toml(path),
        Some("yaml") | Some("yml") => load_yaml(path),
        Some(ext) => Err(ConfigError::Parse(format!(
            "unsupported extension: {}",
            ext
        ))),
        None => Err(ConfigError::Parse("no file extension".to_string())),
    }
}

/// Saves JSON configuration to a file.
pub fn save_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let content =
        serde_json::to_string_pretty(value).map_err(|e| ConfigError::Parse(e.to_string()))?;
    std::fs::write(path, content)?;
    Ok(())
}

/// Saves TOML configuration to a file.
pub fn save_toml<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let content = toml::to_string_pretty(value).map_err(|e| ConfigError::Parse(e.to_string()))?;
    std::fs::write(path, content)?;
    Ok(())
}

/// Saves YAML configuration to a file.
pub fn save_yaml<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let content = serde_yaml::to_string(value).map_err(|e| ConfigError::Parse(e.to_string()))?;
    std::fs::write(path, content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize, Debug, PartialEq, Serialize)]
    struct TestConfig {
        name: String,
        value: i32,
    }

    #[test]
    fn test_load_json() {
        let dir = std::env::temp_dir();
        let path = dir.join("test_cfg.json");
        std::fs::write(&path, r#"{"name":"test","value":42}"#).unwrap();
        let config: TestConfig = load_json(&path).unwrap();
        assert_eq!(config.name, "test");
        assert_eq!(config.value, 42);
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_load_toml() {
        let dir = std::env::temp_dir();
        let path = dir.join("test_cfg.toml");
        std::fs::write(&path, "name = \"test\"\nvalue = 42\n").unwrap();
        let config: TestConfig = load_toml(&path).unwrap();
        assert_eq!(config.name, "test");
        assert_eq!(config.value, 42);
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_save_and_load_json() {
        let dir = std::env::temp_dir();
        let path = dir.join("test_save.json");
        let config = TestConfig {
            name: "saved".to_string(),
            value: 100,
        };
        save_json(&path, &config).unwrap();
        let loaded: TestConfig = load_json(&path).unwrap();
        assert_eq!(loaded.name, "saved");
        assert_eq!(loaded.value, 100);
        std::fs::remove_file(&path).ok();
    }
}
