//! Configuration error types.

use thiserror::Error;

/// Errors that can occur during configuration operations.
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("configuration error: {0}")]
    Custom(String),

    #[error("file not found: {path}")]
    FileNotFound { path: String },

    #[error("failed to parse {format}: {reason}")]
    Parse { format: &'static str, reason: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON parse error: {0}")]
    JsonParse { path: Option<String>, reason: String },

    #[error("TOML parse error: {0}")]
    TomlParse { path: Option<String>, reason: String },

    #[error("YAML parse error: {0}")]
    YamlParse { path: Option<String>, reason: String },
}

/// Result type alias for configuration operations.
pub type Result<T> = std::result::Result<T, ConfigError>;

impl ConfigError {
    /// Create a custom error message.
    pub fn custom(context: &str, message: impl Into<String>) -> Self {
        Self::Custom(format!("{}: {}", context, message.into()))
    }

    /// Create a JSON parse error.
    pub fn json_parse(reason: impl Into<String>) -> Self {
        Self::JsonParse {
            path: None,
            reason: reason.into(),
        }
    }

    /// Create a TOML parse error.
    pub fn toml_parse(reason: impl Into<String>) -> Self {
        Self::TomlParse {
            path: None,
            reason: reason.into(),
        }
    }

    /// Create a YAML parse error.
    pub fn yaml_parse(reason: impl Into<String>) -> Self {
        Self::YamlParse {
            path: None,
            reason: reason.into(),
        }
    }
}
