//! Unified error type for all port operations.
//!
//! Consolidates the two separate error enums (`inbound::Error` and `outbound::Error`)
//! into a single `PortError` type.

use thiserror::Error;

/// Unified error type for all port operations.
#[derive(Error, Debug, Clone)]
pub enum PortError {
    /// Resource not found.
    #[error("not found: {0}")]
    NotFound(String),

    /// Resource already exists.
    #[error("already exists: {0}")]
    AlreadyExists(String),

    /// Validation failed.
    #[error("validation failed: {0}")]
    Validation(String),

    /// Conflict detected.
    #[error("conflict: {0}")]
    Conflict(String),

    /// Permission denied.
    #[error("permission denied: {0}")]
    PermissionDenied(String),

    /// Connection error.
    #[error("connection error: {0}")]
    Connection(String),

    /// Timeout.
    #[error("timeout")]
    Timeout,

    /// Serialization error.
    #[error("serialization error: {0}")]
    Serialization(String),

    /// Operation failed.
    #[error("operation failed: {0}")]
    OperationFailed(String),

    /// Internal error.
    #[error("internal error: {0}")]
    Internal(String),

    /// Not implemented.
    #[error("not implemented: {0}")]
    NotImplemented(String),
}

impl PortError {
    /// Create a not-found error.
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }

    /// Create a validation error.
    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(msg.into())
    }

    /// Create a conflict error.
    pub fn conflict(msg: impl Into<String>) -> Self {
        Self::Conflict(msg.into())
    }

    /// Create a permission denied error.
    pub fn permission_denied(msg: impl Into<String>) -> Self {
        Self::PermissionDenied(msg.into())
    }

    /// Create a connection error.
    pub fn connection(msg: impl Into<String>) -> Self {
        Self::Connection(msg.into())
    }

    /// Check if this error is retryable.
    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::Timeout | Self::Connection(_) | Self::Internal(_))
    }

    /// Check if this is a client error (4xx equivalent).
    pub fn is_client_error(&self) -> bool {
        matches!(
            self,
            Self::NotFound(_)
                | Self::AlreadyExists(_)
                | Self::Validation(_)
                | Self::Conflict(_)
                | Self::PermissionDenied(_)
        )
    }
}

/// Result type for port operations.
pub type Result<T> = std::result::Result<T, PortError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_not_found() {
        let err = PortError::not_found("user 123");
        assert!(matches!(err, PortError::NotFound(_)));
        assert_eq!(err.to_string(), "not found: user 123");
    }

    #[test]
    fn error_validation() {
        let err = PortError::validation("email invalid");
        assert!(err.is_client_error());
        assert!(!err.is_retryable());
    }
}
