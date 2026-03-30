//! # Phenotype Errors
//!
//! Unified error types for the Phenotype ecosystem.
//!
//! This crate re-exports the canonical error type `CoreError` from
//! `phenotype-error-core` along with the convenience `Result` type.

pub use phenotype_error_core::{CoreError, Result};

/// Alias for the canonical error type from error-core.
pub type Error = CoreError;

#[cfg(test)]
mod tests {
    use super::*;

    // Traces to: FR-PHENO-001
    #[test]
    fn re_exported_core_error_not_found() {
        let err = CoreError::not_found("user/42");
        assert_eq!(err.to_string(), "not found: user/42");
    }

    // Traces to: FR-PHENO-001
    #[test]
    fn re_exported_core_error_validation() {
        let err = CoreError::validation("invalid input");
        assert_eq!(err.to_string(), "validation error: invalid input");
    }

    // Traces to: FR-PHENO-001
    #[test]
    fn error_alias() {
        let err: Error = CoreError::internal("failed");
        assert_eq!(err.to_string(), "internal error: failed");
    }

    // Traces to: FR-PHENO-001
    #[test]
    fn result_type_ok() {
        let r: Result<i32> = Ok(42);
        assert_eq!(r.unwrap(), 42);
    }

    // Traces to: FR-PHENO-001
    #[test]
    fn result_type_err() {
        let r: Result<i32> = Err(CoreError::not_found("missing"));
        assert!(r.is_err());
    }
}
