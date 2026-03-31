//! # Phenotype Errors
//!
//! Unified error types for the Phenotype ecosystem.

pub use phenotype_error_core::ApiError;
pub use phenotype_error_core::ConfigError;
pub use phenotype_error_core::DomainError;
pub use phenotype_error_core::RepositoryError;
pub use phenotype_error_core::StorageError;

/// Convenience result type using the canonical ApiError.
pub type Result<T> = std::result::Result<T, ApiError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_error() {
        let err = DomainError::Validation("invalid".into());
        assert!(err.to_string().contains("validation failed"));
    }

    #[test]
    fn test_result_type_ok() {
        let r: Result<i32> = Ok(42);
        assert_eq!(r.unwrap(), 42);
    }

    #[test]
    fn test_result_type_err() {
        let r: Result<i32> = Err(ApiError::BadRequest("bad".into()));
        assert!(r.is_err());
    }

    #[test]
    fn test_api_error_status_code() {
        let err = ApiError::NotFound {
            resource: "user".into(),
            id: "42".into(),
        };
        assert_eq!(err.status_code(), 404);
    }

    #[test]
    fn test_storage_error_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = StorageError::from(io_err);
        assert!(err.to_string().contains("I/O error"));
    }

    #[test]
    fn test_config_error_parse() {
        let err = ConfigError::Parse {
            format: "json".into(),
            reason: "invalid".into(),
        };
        assert!(err.to_string().contains("parse error"));
    }
}
