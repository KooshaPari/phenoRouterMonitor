//! Configuration error types

use thiserror::Error;

/// Result type for configuration operations
pub type Result<T> = std::result::Result<T, ConfigError>;

/// Configuration errors
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to load configuration: {0}")]
    LoadError(String),

    #[error("Failed to parse configuration: {0}")]
    ParseError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Watch error: {0}")]
    WatchError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("TOML error: {0}")]
    TomlError(#[from] toml::de::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    // Traces to: FR-ROUTER-018 (Configuration errors)
    #[test]
    fn test_load_error() {
        let err = ConfigError::LoadError("file not found".to_string());
        assert!(err.to_string().contains("file not found"));
    }

    // Traces to: FR-ROUTER-018
    #[test]
    fn test_validation_error() {
        let err = ConfigError::ValidationError("invalid timeout".to_string());
        assert!(err.to_string().contains("invalid timeout"));
    }
}
