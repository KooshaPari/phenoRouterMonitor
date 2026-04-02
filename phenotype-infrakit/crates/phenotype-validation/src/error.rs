//! Error types for validation

use thiserror::Error;

/// Result type for validation operations
pub type Result<T> = std::result::Result<T, ValidationError>;

/// Validation error types
#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Validation failed: {0}")]
    Invalid(String),

    #[error("Schema error: {0}")]
    Schema(String),

    #[error("Context error: {0}")]
    Context(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
