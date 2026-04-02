//! Configuration error types

use thiserror::Error;

/// Configuration error type
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Failed to parse configuration
    #[error("parse error: {0}")]
    Parse(String),

    /// Configuration key not found
    #[error("key not found: {0}")]
    NotFound(String),

    /// Invalid configuration value
    #[error("invalid value for {key}: {message}")]
    InvalidValue { key: String, message: String },

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// TOML error
    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),
}

/// Result type alias for configuration operations
pub type Result<T> = std::result::Result<T, ConfigError>;
