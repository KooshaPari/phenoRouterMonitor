//! Centralized error types for Phenotype ecosystem.

use thiserror::Error;

/// Core error type for the Phenotype ecosystem.
#[derive(Debug, Error)]
pub enum CoreError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("serialization error: {0}")]
    Serialization(String),

    #[error("validation error: {0}")]
    Validation(String),

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("timeout: {0}")]
    Timeout(String),

    #[error("storage error: {0}")]
    Storage(String),

    #[error("network error: {0}")]
    Network(String),

    #[error("configuration error: {0}")]
    Configuration(String),

    #[error("permission denied: {0}")]
    Permission(String),

    #[error("internal error: {0}")]
    Internal(String),

    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type alias using CoreError.
pub type Result<T> = std::result::Result<T, CoreError>;

impl CoreError {
    /// Create a not found error.
    pub fn not_found(entity: impl Into<String>) -> Self {
        Self::NotFound(entity.into())
    }

    /// Create a serialization error.
    pub fn serialization(msg: impl Into<String>) -> Self {
        Self::Serialization(msg.into())
    }

    /// Create a validation error.
    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(msg.into())
    }

    /// Create a timeout error.
    pub fn timeout(msg: impl Into<String>) -> Self {
        Self::Timeout(msg.into())
    }

    /// Create a storage error.
    pub fn storage(msg: impl Into<String>) -> Self {
        Self::Storage(msg.into())
    }

    /// Create a network error.
    pub fn network(msg: impl Into<String>) -> Self {
        Self::Network(msg.into())
    }

    /// Create an internal error.
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }

    /// Create an IO error.
    pub fn io(err: std::io::Error) -> Self {
        Self::from(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = CoreError::not_found("User");
        assert_eq!(err.to_string(), "not found: User");

        let err = CoreError::validation("invalid email");
        assert_eq!(err.to_string(), "validation error: invalid email");
    }

    #[test]
    fn test_result_usage() {
        fn fallible() -> Result<i32> {
            Err(CoreError::not_found("item"))
        }

        let result = fallible();
        assert!(result.is_err());
    }
}
