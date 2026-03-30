//! Error types for configuration loading and validation.

use std::path::PathBuf;
use thiserror::Error;

/// Result type for configuration operations.
pub type Result<T> = std::result::Result<T, ConfigError>;

/// Comprehensive error type for configuration loading and validation.
#[derive(Error, Debug)]
pub enum ConfigError {
    /// File I/O error with context.
    #[error("Failed to read config file {path}: {reason}")]
    FileRead {
        /// Path to the file that failed to read.
        path: PathBuf,
        /// Underlying error reason.
        reason: String,
    },

    /// File not found error.
    #[error("Config file not found: {path}")]
    FileNotFound {
        /// Path that was not found.
        path: PathBuf,
    },

    /// TOML parsing error.
    #[error("Failed to parse TOML config: {reason}")]
    TomlParse {
        /// Details about what went wrong.
        reason: String,
    },

    /// YAML parsing error.
    #[error("Failed to parse YAML config: {reason}")]
    YamlParse {
        /// Details about what went wrong.
        reason: String,
    },

    /// JSON parsing error.
    #[error("Failed to parse JSON config: {reason}")]
    JsonParse {
        /// Details about what went wrong.
        reason: String,
    },

    /// Deserialization error (type mismatch, missing field, etc.).
    #[error("Failed to deserialize config: {reason}")]
    Deserialize {
        /// Details about the deserialization failure.
        reason: String,
    },

    /// Unsupported file format.
    #[error("Unsupported config file format: {format}. Supported: toml, yaml, yml, json")]
    UnsupportedFormat {
        /// The unsupported format extension or type.
        format: String,
    },

    /// Configuration validation error.
    #[error("Config validation failed: {reason}")]
    Validation {
        /// Details about the validation failure.
        reason: String,
    },

    /// Missing required configuration or field.
    #[error("Missing required configuration: {field}")]
    MissingRequired {
        /// The required field or configuration key.
        field: String,
    },

    /// Environment or system error.
    #[error("Environment error: {reason}")]
    Environment {
        /// Details about the environment error.
        reason: String,
    },

    /// Generic configuration error with custom message.
    #[error("Configuration error: {0}")]
    Other(String),
}

impl ConfigError {
    /// Create a file read error.
    pub fn file_read(path: impl Into<PathBuf>, reason: impl Into<String>) -> Self {
        Self::FileRead {
            path: path.into(),
            reason: reason.into(),
        }
    }

    /// Create a file not found error.
    pub fn file_not_found(path: impl Into<PathBuf>) -> Self {
        Self::FileNotFound {
            path: path.into(),
        }
    }

    /// Create a TOML parsing error.
    pub fn toml_parse(reason: impl Into<String>) -> Self {
        Self::TomlParse {
            reason: reason.into(),
        }
    }

    /// Create a YAML parsing error.
    pub fn yaml_parse(reason: impl Into<String>) -> Self {
        Self::YamlParse {
            reason: reason.into(),
        }
    }

    /// Create a JSON parsing error.
    pub fn json_parse(reason: impl Into<String>) -> Self {
        Self::JsonParse {
            reason: reason.into(),
        }
    }

    /// Create a deserialization error.
    pub fn deserialize(reason: impl Into<String>) -> Self {
        Self::Deserialize {
            reason: reason.into(),
        }
    }

    /// Create an unsupported format error.
    pub fn unsupported_format(format: impl Into<String>) -> Self {
        Self::UnsupportedFormat {
            format: format.into(),
        }
    }

    /// Create a validation error.
    pub fn validation(reason: impl Into<String>) -> Self {
        Self::Validation {
            reason: reason.into(),
        }
    }

    /// Create a missing required field error.
    pub fn missing_required(field: impl Into<String>) -> Self {
        Self::MissingRequired {
            field: field.into(),
        }
    }

    /// Create an environment error.
    pub fn environment(reason: impl Into<String>) -> Self {
        Self::Environment {
            reason: reason.into(),
        }
    }
}

impl From<std::io::Error> for ConfigError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::NotFound => {
                Self::FileNotFound {
                    path: PathBuf::from("<unknown>"),
                }
            }
            _ => Self::Other(err.to_string()),
        }
    }
}

impl From<serde_json::Error> for ConfigError {
    fn from(err: serde_json::Error) -> Self {
        Self::JsonParse {
            reason: err.to_string(),
        }
    }
}

impl From<serde_yaml::Error> for ConfigError {
    fn from(err: serde_yaml::Error) -> Self {
        Self::YamlParse {
            reason: err.to_string(),
        }
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(err: toml::de::Error) -> Self {
        Self::TomlParse {
            reason: err.to_string(),
        }
    }
}
