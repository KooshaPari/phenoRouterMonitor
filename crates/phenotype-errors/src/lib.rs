//! # Phenotype Errors
//!
//! Unified error types for the Phenotype ecosystem.

/// Result type alias
pub type Result<T> = std::result::Result<T, Error>;

/// Error enum for Phenotype operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Not found error
    #[error("not found: {0}")]
    NotFound(String),

    /// Validation error
    #[error("validation failed: {0}")]
    Validation(String),

    /// Conflict error
    #[error("conflict: {0}")]
    Conflict(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error
    #[error("serialization error: {0}")]
    Serialization(String),

    /// Internal error
    #[error("internal error: {0}")]
    Internal(String),
}

impl Error {
    pub fn not_found<S: Into<String>>(msg: S) -> Self {
        Self::NotFound(msg.into())
    }

    pub fn validation<S: Into<String>>(msg: S) -> Self {
        Self::Validation(msg.into())
    }

    pub fn conflict<S: Into<String>>(msg: S) -> Self {
        Self::Conflict(msg.into())
    }

    pub fn internal<S: Into<String>>(msg: S) -> Self {
        Self::Internal(msg.into())
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::Serialization(e.to_string())
    }
}
