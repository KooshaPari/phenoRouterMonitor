//! # Phenotype Errors
//!
//! Unified error types for the Phenotype ecosystem.

use thiserror::Error;

// Re-export ErrorKind from phenotype-error-core for unified error handling
pub use phenotype_error_core::ErrorKind;

/// Canonical error type alias for type annotations.
///
/// Preferred over using `ErrorKind` directly in public APIs.
pub type Error = ErrorKind;

/// Convenience result type using the canonical `ErrorKind`.
pub type Result<T> = std::result::Result<T, ErrorKind>;

/// Backward compatibility alias for ErrorKind.
pub use ErrorKind as CoreError;

impl ErrorKind {
    /// Returns the kind name for this error as a static string.
    pub fn kind(&self) -> &'static str {
        match self {
            ErrorKind::NotFound(_) => "NotFound",
            ErrorKind::Serialization(_) => "Serialization",
            ErrorKind::Validation(_) => "Validation",
            ErrorKind::Conflict(_) => "Conflict",
            ErrorKind::Timeout(_) => "Timeout",
            ErrorKind::Storage(_) => "Storage",
            ErrorKind::Network(_) => "Network",
            ErrorKind::Configuration(_) => "Configuration",
            ErrorKind::Permission(_) => "Permission",
            ErrorKind::Internal(_) => "Internal",
            ErrorKind::InvalidInput(_) => "InvalidInput",
            ErrorKind::Io(_) => "Io",
        }
    }

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

    /// Create a conflict error.
    pub fn conflict(msg: impl Into<String>) -> Self {
        Self::Conflict(msg.into())
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

    /// Create a configuration error.
    pub fn configuration(msg: impl Into<String>) -> Self {
        Self::Configuration(msg.into())
    }

    /// Create a permission error.
    pub fn permission(msg: impl Into<String>) -> Self {
        Self::Permission(msg.into())
    }

    /// Create an internal error.
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }

    /// Create an invalid input error.
    pub fn invalid_input(msg: impl Into<String>) -> Self {
        Self::InvalidInput(msg.into())
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
    fn test_error_kind_not_found() {
        let err = ErrorKind::not_found("user/42");
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn test_result_type_ok() {
        let r: Result<i32> = Ok(42);
        assert_eq!(r.unwrap(), 42);
    }

    #[test]
    fn test_result_type_err() {
        let r: Result<i32> = Err(ErrorKind::not_found("missing"));
        assert!(r.is_err());
    }

    #[test]
    fn test_error_constructors() {
        assert_eq!(ErrorKind::serialization("parse failed").kind(), "Serialization");
        assert_eq!(ErrorKind::validation("invalid email").kind(), "Validation");
        assert_eq!(ErrorKind::conflict("already exists").kind(), "Conflict");
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
        let err = ErrorKind::from(io_err);
        assert_eq!(err.kind(), "Io");
    }

    #[test]
    fn test_error_alias() {
        let e: Error = ErrorKind::internal("test");
        assert_eq!(e.kind(), "Internal");
    }

    #[test]
    fn test_core_error_alias() {
        let err: CoreError = ErrorKind::permission("denied");
        assert_eq!(err.kind(), "Permission");
    }
}
