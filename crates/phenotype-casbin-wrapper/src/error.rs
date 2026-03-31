//! Error types for the Casbin wrapper.

use thiserror::Error;

/// Errors that can occur during Casbin operations.
#[derive(Debug, Error)]
pub enum CasbinWrapperError {
    #[error("Casbin enforcement failed: {0}")]
    EnforcementFailed(String),

    #[error("Policy operation failed: {0}")]
    PolicyError(String),

    #[error("Model loading failed: {0}")]
    ModelError(String),

    #[error("Adapter initialization failed: {0}")]
    InitError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Unsupported model type: {0}")]
    UnsupportedModel(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

impl From<casbin::Error> for CasbinWrapperError {
    fn from(err: casbin::Error) -> Self {
        CasbinWrapperError::PolicyError(err.to_string())
    }
}
