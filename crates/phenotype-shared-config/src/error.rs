//! Configuration error types.
//!
//! Provides structured error types for configuration loading, parsing, and validation.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Errors that can occur during configuration operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigError {
    /// Failed to read a configuration file.
    #[serde(rename = "file_read")]
    FileRead { path: Option<String>, reason: String },

    /// Configuration file was not found.
    #[serde(rename = "file_not_found")]
    FileNotFound { path: Option<String> },

    /// TOML parsing failed.
    #[serde(rename = "toml_parse")]
    TomlParse { path: Option<String>, reason: String },

    /// JSON parsing failed.
    #[serde(rename = "json_parse")]
    JsonParse { path: Option<String>, reason: String },

    /// YAML parsing failed.
    #[serde(rename = "yaml_parse")]
    YamlParse { path: Option<String>, reason: String },

    /// Validation failed.
    #[serde(rename = "validation")]
    Validation { reason: String },

    /// Invalid configuration format.
    #[serde(rename = "invalid_format")]
    InvalidFormat { expected: String, found: Option<String> },

    /// Missing required field.
    #[serde(rename = "missing_field")]
    MissingField { field: String },

    /// IO error occurred.
    #[serde(rename = "io")]
    Io { reason: String },

    /// Directory resolution error.
    #[serde(rename = "directory")]
    Directory { reason: String },

    /// Custom error with context.
    #[serde(rename = "custom")]
    Custom { context: String, reason: String },
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::FileRead { path, reason } => {
                if let Some(p) = path {
                    write!(f, "failed to read '{}': {}", p, reason)
                } else {
                    write!(f, "failed to read config: {}", reason)
                }
            }
            ConfigError::FileNotFound { path } => {
                if let Some(p) = path {
                    write!(f, "config file not found: '{}'", p)
                } else {
                    write!(f, "config file not found")
                }
            }
            ConfigError::TomlParse { path, reason } => {
                if let Some(p) = path {
                    write!(f, "TOML parse error in '{}': {}", p, reason)
                } else {
                    write!(f, "TOML parse error: {}", reason)
                }
            }
            ConfigError::JsonParse { path, reason } => {
                if let Some(p) = path {
                    write!(f, "JSON parse error in '{}': {}", p, reason)
                } else {
                    write!(f, "JSON parse error: {}", reason)
                }
            }
            ConfigError::YamlParse { path, reason } => {
                if let Some(p) = path {
                    write!(f, "YAML parse error in '{}': {}", p, reason)
                } else {
                    write!(f, "YAML parse error: {}", reason)
                }
            }
            ConfigError::Validation { reason } => write!(f, "validation failed: {}", reason),
            ConfigError::InvalidFormat { expected, found } => {
                if let Some(found) = found {
                    write!(f, "invalid format: expected {}, found {}", expected, found)
                } else {
                    write!(f, "invalid format: expected {}", expected)
                }
            }
            ConfigError::MissingField { field } => write!(f, "missing required field: {}", field),
            ConfigError::Io { reason } => write!(f, "IO error: {}", reason),
            ConfigError::Directory { reason } => write!(f, "directory error: {}", reason),
            ConfigError::Custom { context, reason } => write!(f, "{}: {}", context, reason),
        }
    }
}

impl std::error::Error for ConfigError {}

impl ConfigError {
    pub fn file_read(path: impl Into<PathBuf>, reason: impl Into<String>) -> Self {
        Self::FileRead { path: Some(path.into().to_string_lossy().into_owned()), reason: reason.into() }
    }

    pub fn file_not_found(path: impl Into<PathBuf>) -> Self {
        Self::FileNotFound { path: Some(path.into().to_string_lossy().into_owned()) }
    }

    pub fn toml_parse(path: impl Into<PathBuf>, reason: impl Into<String>) -> Self {
        Self::TomlParse { path: Some(path.into().to_string_lossy().into_owned()), reason: reason.into() }
    }

    pub fn json_parse(reason: impl Into<String>) -> Self {
        Self::JsonParse { path: None, reason: reason.into() }
    }

    pub fn yaml_parse(reason: impl Into<String>) -> Self {
        Self::YamlParse { path: None, reason: reason.into() }
    }

    pub fn validation(reason: impl Into<String>) -> Self {
        Self::Validation { reason: reason.into() }
    }

    pub fn invalid_format(expected: impl Into<String>) -> Self {
        Self::InvalidFormat { expected: expected.into(), found: None }
    }

    pub fn missing_field(field: impl Into<String>) -> Self {
        Self::MissingField { field: field.into() }
    }

    pub fn io(reason: impl Into<String>) -> Self {
        Self::Io { reason: reason.into() }
    }

    pub fn directory(reason: impl Into<String>) -> Self {
        Self::Directory { reason: reason.into() }
    }

    pub fn custom(context: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::Custom { context: context.into(), reason: reason.into() }
    }
}

impl From<std::io::Error> for ConfigError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::NotFound => Self::FileNotFound { path: None },
            _ => Self::Io { reason: err.to_string() },
        }
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(err: toml::de::Error) -> Self {
        Self::TomlParse { path: None, reason: err.to_string() }
    }
}

impl From<serde_json::Error> for ConfigError {
    fn from(err: serde_json::Error) -> Self {
        Self::JsonParse { path: None, reason: err.to_string() }
    }
}

#[cfg(feature = "yaml")]
impl From<serde_yaml::Error> for ConfigError {
    fn from(err: serde_yaml::Error) -> Self {
        Self::YamlParse { path: None, reason: err.to_string() }
    }
}

/// Result type alias for configuration operations.
pub type Result<T> = std::result::Result<T, ConfigError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = ConfigError::file_not_found("/etc/app.toml");
        assert!(err.to_string().contains("/etc/app.toml"));
    }

    #[test]
    fn test_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let config_err: ConfigError = io_err.into();
        matches!(config_err, ConfigError::FileNotFound { .. });
    }
}
