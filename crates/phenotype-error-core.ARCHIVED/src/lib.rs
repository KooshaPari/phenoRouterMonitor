//! Shared error types for Phenotype crates.
//!
//! This crate provides a unified error type system across the Phenotype monorepo,
//! consolidating common error patterns and reducing code duplication.
//!
//! # ErrorKind
//!
//! [`ErrorKind`] is the primary error enum that covers common error scenarios:
//! - `NotFound` - Resource not found
//! - `Validation` - Input validation failure
//! - `Timeout` - Operation timeout
//! - `Internal` - Internal server error
//! - `Serialization` - JSON/TOML serialization error
//! - `Storage` - Storage backend error
//! - `Connection` - Network/connection error
//! - `Config` - Configuration error
//! - `PermissionDenied` - Permission/authorization error
//! - `Conflict` - Resource conflict (e.g., duplicate)
//! - `AlreadyExists` - Resource already exists
//! - `ParseError` - Parsing error
//! - `NetworkError` - Network error
//! - `AuthError` - Authentication error
//!
//! # Domain-Specific Wrappers
//!
//! Some crates have domain-specific error needs and should create wrapper enums
//! that include `ErrorKind` for common cases. For example:
//! - Event sourcing: preserve `DuplicateSequence`, `SequenceGap`, `InvalidHash`
//! - Policy engine: preserve `RegexCompilation`, `InvalidConfiguration`
//!
//! # Example
//!
//! ```rust
//! use phenotype_error_core::ErrorKind;
//!
//! fn validate_name(name: &str) -> Result<(), ErrorKind> {
//!     if name.is_empty() {
//!         return Err(ErrorKind::Validation("name cannot be empty".to_string()));
//!     }
//!     Ok(())
//! }
//! ```

/// Result type alias for operations returning ErrorKind.
pub type Result<T> = std::result::Result<T, ErrorKind>;

/// Unified error type for Phenotype crates.
///
/// This enum consolidates common error patterns across the monorepo.
/// Domain-specific crates should create wrapper enums if they need additional variants.
#[derive(Debug, Clone, thiserror::Error)]
pub enum ErrorKind {
    /// Resource not found.
    #[error("not found: {0}")]
    NotFound(String),

    /// Serialization/deserialization error.
    #[error("serialization error: {0}")]
    Serialization(String),

    /// Input validation error.
    #[error("validation error: {0}")]
    Validation(String),

    /// Operation timeout.
    #[error("timeout: {0}")]
    Timeout(String),

    /// Internal server error.
    #[error("internal error: {0}")]
    Internal(String),

    /// Storage backend error.
    #[error("storage error: {0}")]
    Storage(String),

    /// Network/connection error.
    #[error("connection error: {0}")]
    Connection(String),

    /// Configuration error.
    #[error("config error: {0}")]
    Config(String),

    /// Permission denied/authorization error.
    #[error("permission denied: {0}")]
    PermissionDenied(String),

    /// Resource conflict (e.g., duplicate key).
    #[error("conflict: {0}")]
    Conflict(String),

    /// Resource already exists.
    #[error("already exists: {0}")]
    AlreadyExists(String),

    /// Parsing error.
    #[error("parse error: {0}")]
    ParseError(String),

    /// Network-level error.
    #[error("network error: {0}")]
    NetworkError(String),

    /// Authentication error.
    #[error("authentication error: {0}")]
    AuthError(String),
}

// Implement additional conversions from standard library error types.

impl From<std::io::Error> for ErrorKind {
    fn from(err: std::io::Error) -> Self {
        ErrorKind::Storage(err.to_string())
    }
}

impl From<serde_json::Error> for ErrorKind {
    fn from(err: serde_json::Error) -> Self {
        ErrorKind::Serialization(err.to_string())
    }
}

impl From<regex::Error> for ErrorKind {
    fn from(err: regex::Error) -> Self {
        ErrorKind::ParseError(err.to_string())
    }
}

impl From<toml::de::Error> for ErrorKind {
    fn from(err: toml::de::Error) -> Self {
        ErrorKind::Serialization(err.to_string())
    }
}

impl From<&str> for ErrorKind {
    fn from(msg: &str) -> Self {
        ErrorKind::Internal(msg.to_string())
    }
}

impl From<String> for ErrorKind {
    fn from(msg: String) -> Self {
        ErrorKind::Internal(msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = ErrorKind::NotFound("user123".to_string());
        assert_eq!(err.to_string(), "not found: user123");
    }

    #[test]
    fn test_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: ErrorKind = io_err.into();
        assert!(matches!(err, ErrorKind::Storage(_)));
    }

    #[test]
    fn test_error_from_json() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid").unwrap_err();
        let err: ErrorKind = json_err.into();
        assert!(matches!(err, ErrorKind::Serialization(_)));
    }

    #[test]
    fn test_error_from_str() {
        let err: ErrorKind = "something went wrong".into();
        assert!(matches!(err, ErrorKind::Internal(_)));
    }

    #[test]
    fn test_result_type() {
        fn might_fail() -> Result<i32> {
            Err(ErrorKind::Validation("expected positive number".to_string()))
        }
        assert!(might_fail().is_err());
    }
}
