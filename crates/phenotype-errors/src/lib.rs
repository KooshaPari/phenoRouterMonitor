//! # Phenotype Errors
//!
//! Unified error types for the Phenotype ecosystem.

pub use phenotype_error_core::{ErrorKind, Result};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_kind_not_found() {
        let err = ErrorKind::not_found("user/42");
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn error_kind_kind_method() {
        let err = ErrorKind::not_found("user/42");
        assert_eq!(err.kind(), "NotFound");
    }

    #[test]
    fn result_type_ok() {
        let r: Result<i32> = Ok(42);
        assert_eq!(r.unwrap(), 42);
    }

    #[test]
    fn result_type_err() {
        let r: Result<i32> = Err(ErrorKind::not_found("missing"));
        assert!(r.is_err());
    }
}
