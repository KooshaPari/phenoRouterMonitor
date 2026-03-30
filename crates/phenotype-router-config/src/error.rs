//! Error types for router configuration operations

use thiserror::Error;

/// Result type for router configuration operations
pub type Result<T> = std::result::Result<T, RouterConfigError>;

/// Router configuration errors
#[derive(Debug, Clone, Error)]
pub enum RouterConfigError {
    #[error("IO error: {0}")]
    Io(String),

    #[error("Configuration parse error: {0}")]
    ParseError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Configuration not found: {0}")]
    NotFound(String),

    #[error("Watch error: {0}")]
    WatchError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

impl From<std::io::Error> for RouterConfigError {
    fn from(err: std::io::Error) -> Self {
        RouterConfigError::Io(err.to_string())
    }
}

impl From<toml::de::Error> for RouterConfigError {
    fn from(err: toml::de::Error) -> Self {
        RouterConfigError::ParseError(err.to_string())
    }
}

impl From<serde_json::Error> for RouterConfigError {
    fn from(err: serde_json::Error) -> Self {
        RouterConfigError::SerializationError(err.to_string())
    }
}

impl From<notify::Error> for RouterConfigError {
    fn from(err: notify::Error) -> Self {
        RouterConfigError::WatchError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let config_err: RouterConfigError = io_err.into();
        assert!(config_err.to_string().contains("IO error"));
    }

    #[test]
    fn test_validation_error_creation() {
        let err = RouterConfigError::ValidationError("port must be > 0".to_string());
        assert!(err.to_string().contains("Validation error"));
    }
}
