//! Configuration loading errors

use thiserror::Error;

/// Configuration error types
#[derive(Error, Debug, Clone)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("YAML error: {0}")]
   Yaml(#[from] serde_yaml::Error),

    #[error("configuration key not found: {0}")]
    KeyNotFound(String),

    #[error("invalid configuration: {0}")]
    Invalid(String),
}

/// Configuration result type
pub type Result<T> = std::result::Result<T, ConfigError>;
