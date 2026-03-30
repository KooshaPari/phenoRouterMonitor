//! # Phenotype Errors
//!
//! Unified error types for the Phenotype ecosystem.

pub use phenotype_error_core::ErrorKind;

/// Canonical error type alias for type annotations.
///
/// Preferred over using `ErrorKind` directly in public APIs.
pub type Error = ErrorKind;

/// Convenience result type using the canonical `ErrorKind`.
pub type Result<T> = std::result::Result<T, ErrorKind>;

/// Backward compatibility alias for ErrorKind.
pub use ErrorKind as CoreError;

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
