//! Loader errors

use thiserror::Error;

/// Errors that can occur during configuration loading
#[derive(Error, Debug)]
pub enum LoaderError {
    #[error("file not found: {path}")]
    FileNotFound { path: String },

    #[error("read error: {path}")]
    ReadError {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("JSON parse: {source}")]
    JsonParseError {
        #[from]
        source: serde_json::Error,
    },

    #[error("TOML parse: {source}")]
    TomlParseError {
        #[from]
        source: toml::de::Error,
    },

    #[error("unsupported: {format}")]
    UnsupportedFormat { format: String },

    #[error("validation: {message}")]
    ValidationError { message: String },
}

impl LoaderError {
    /// Create a file not found error
    pub fn file_not_found(path: impl Into<String>) -> Self {
        Self::FileNotFound { path: path.into() }
    }

    /// Create a read error
    pub fn read_error(path: impl Into<String>, source: std::io::Error) -> Self {
        Self::ReadError {
            path: path.into(),
            source,
        }
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
        let err = LoaderError::file_not_found("/missing/file.toml");
        assert!(err.to_string().contains("/missing/file.toml"));
    }

    #[test]
    fn test_read_error() {
        let err = LoaderError::read_error(
            "/path",
            std::io::Error::new(std::io::ErrorKind::NotFound, "oops"),
        );
        assert!(err.to_string().contains("/path"));
    }

    #[test]
    fn test_unsupported_format() {
        let err = LoaderError::UnsupportedFormat {
            format: "yaml".to_string(),
        };
        assert!(err.to_string().contains("yaml"));
    }

    #[test]
    fn test_validation_error() {
        let err = LoaderError::validation("must be valid JSON");
        assert!(err.to_string().contains("must be valid JSON"));
    }
}
