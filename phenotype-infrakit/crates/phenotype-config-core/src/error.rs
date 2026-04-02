//! Configuration error types

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("file not found: {path}")]
    FileNotFound { path: String },

    #[error("parse error: {source}")]
    ParseError {
        #[from]
        source: serde_json::Error,
    },

    #[error("missing key: {key}")]
    MissingKey { key: String },

    #[error("validation: {message}")]
    ValidationError { message: String },
}

impl ConfigError {
    /// Create a file not found error
    pub fn file_not_found(path: impl Into<String>) -> Self {
        Self::FileNotFound { path: path.into() }
    }

    /// Create a missing key error
    pub fn missing_key(key: impl Into<String>) -> Self {
        Self::MissingKey { key: key.into() }
    }

    /// Create a validation error
    pub fn validation(message: impl Into<String>) -> Self {
        Self::ValidationError {
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_not_found() {
        let err = ConfigError::file_not_found("/path/to/file.toml");
        assert!(err.to_string().contains("/path/to/file.toml"));
    }

    #[test]
    fn test_missing_key() {
        let err = ConfigError::missing_key("database.host");
        assert!(err.to_string().contains("database.host"));
    }

    #[test]
    fn test_validation() {
        let err = ConfigError::validation("must be positive");
        assert!(err.to_string().contains("must be positive"));
    }
}
