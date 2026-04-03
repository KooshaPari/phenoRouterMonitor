//! Plugin error types.

use thiserror::Error;

/// Errors that can occur in plugin operations.
#[derive(Error, Debug)]
pub enum PluginError {
    #[error("Plugin initialization failed: {0}")]
    Initialization(String),

    #[error("Plugin `{0}` not found in registry")]
    NotFound(String),

    #[error("Plugin `{0}` already registered")]
    AlreadyRegistered(String),

    #[error("Entity already exists: {0}")]
    AlreadyExists(String),

    #[error("Operation failed: {0}")]
    Operation(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Plugin execution error: {0}")]
    Execution(String),

    #[error("Validation error: {0}")]
    Validation(String),
}

/// Result type alias for plugin operations.
pub type PluginResult<T> = Result<T, PluginError>;
