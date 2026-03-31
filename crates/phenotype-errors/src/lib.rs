//! # Phenotype Errors
//!
//! Unified error types for the Phenotype ecosystem.

pub use phenotype_error_core::{ApiError, DomainError, RepositoryError, ConfigError, StorageError};

/// Convenience result type using ApiError.
pub type Result<T> = std::result::Result<T, ApiError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_result_type_ok() {
        let r: Result<i32> = Ok(42);
        assert_eq!(r.unwrap(), 42);
    }

    #[test]
    fn test_domain_error() {
        let err = DomainError::invalid_state("test");
        assert!(err.to_string().contains("invalid_state"));
    }

    #[test]
    fn test_repository_error() {
        let err = RepositoryError::not_found("entity/123");
        assert!(err.to_string().contains("not_found"));
    }
}
