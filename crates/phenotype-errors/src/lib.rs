//! # Phenotype Errors
//!
//! Unified error types for the Phenotype ecosystem.
//!
//! This crate re-exports `ErrorKind` from `phenotype-error-core` as the
//! canonical error type. The legacy `Error` enum is retained for
//! backward compatibility but should not be used in new code.

pub use phenotype_error_core::{Error, ErrorContext, ErrorExt, ErrorKind, ErrorKindInner};
pub use thiserror::Error;

/// Convenience result type using the canonical `ErrorKind`.
pub type Result<T> = std::result::Result<T, ErrorKind>;

#[cfg(test)]
mod tests {
    use super::*;

    // Traces to: FR-PHENO-001
    #[test]
    fn re_exported_error_kind_not_found() {
        let err = ErrorKind::not_found("user/42");
        assert_eq!(err.kind(), "NotFound");
        assert!(err.to_string().contains("not found"));
    }

    // Traces to: FR-PHENO-001
    #[test]
    fn re_exported_error_kind_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: ErrorKind = io_err.into();
        assert_eq!(err.kind(), "NotFound");
    }

    // Traces to: FR-PHENO-001
    #[test]
    fn re_exported_error_context_chain() {
        let err = ErrorKind::not_found("user");
        let ctx = err.chain("while fetching");
        assert!(ctx.to_string().contains("while fetching"));
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
        let r: Result<i32> = Err(ErrorKind::not_found("missing"));
        assert!(r.is_err());
    }
}
