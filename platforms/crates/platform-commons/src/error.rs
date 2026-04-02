//! Cross-platform error types for the Phenotype ecosystem.

use thiserror::Error;

/// Result type alias using PlatformError.
pub type Result<T> = std::result::Result<T, Error>;

/// Cross-platform error type.
///
/// This is a simplified error designed to work across all platforms.
/// Each platform may extend this with their own error domain.
#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("operation failed: {0}")]
    OperationFailed(String),

    #[error("internal error: {0}")]
    Internal(String),
}

impl Error {
    /// Creates a new invalid input error.
    pub fn invalid_input<S: Into<String>>(msg: S) -> Self {
        Self::InvalidInput(msg.into())
    }

    /// Creates a new not found error.
    pub fn not_found<S: Into<String>>(msg: S) -> Self {
        Self::NotFound(msg.into())
    }

    /// Creates a new operation failed error.
    pub fn operation_failed<S: Into<String>>(msg: S) -> Self {
        Self::OperationFailed(msg.into())
    }

    /// Creates a new internal error.
    pub fn internal<S: Into<String>>(msg: S) -> Self {
        Self::Internal(msg.into())
    }
}
