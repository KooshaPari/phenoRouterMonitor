//! Configuration error types

use thiserror::Error;

/// Configuration errors
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("file not found: {0}")]
    FileNotFound(String),

    #[error("parse error: {0}")]
    Parse(String),

    #[error("missing required key: {0}")]
    MissingKey(String),

    #[error("invalid type for key '{key}': expected {expected}, got {actual}")]
    InvalidType {
        key: String,
        expected: String,
        actual: String,
    },

    #[error("validation error: {0}")]
    Validation(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),
}

/// Configuration error kinds for programmatic handling
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigErrorKind {
    FileNotFound,
    Parse,
    MissingKey,
    InvalidType,
    Validation,
    Io,
    Json,
    Toml,
}

impl ConfigError {
    /// Get the kind of this error
    pub fn kind(&self) -> ConfigErrorKind {
        match self {
            Self::FileNotFound(_) => ConfigErrorKind::FileNotFound,
            Self::Parse(_) => ConfigErrorKind::Parse,
            Self::MissingKey(_) => ConfigErrorKind::MissingKey,
            Self::InvalidType { .. } => ConfigErrorKind::InvalidType,
            Self::Validation(_) => ConfigErrorKind::Validation,
            Self::Io(_) => ConfigErrorKind::Io,
            Self::Json(_) => ConfigErrorKind::Json,
            Self::Toml(_) => ConfigErrorKind::Toml,
        }
    }
}

/// Result type alias for config operations
pub type Result<T> = std::result::Result<T, ConfigError>;
