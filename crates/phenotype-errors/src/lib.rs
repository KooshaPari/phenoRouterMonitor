//! # Phenotype Errors
//!
//! Unified error types for the Phenotype ecosystem.

pub use thiserror::Error;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Operation failed: {0}")]
    Failed(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    // Traces to: FR-PHENO-001
    #[test]
    fn error_display_failed() {
        let e = Error::Failed("disk full".into());
        assert_eq!(e.to_string(), "Operation failed: disk full");
    }

    // Traces to: FR-PHENO-001
    #[test]
    fn error_display_not_found() {
        let e = Error::NotFound("user/42".into());
        assert_eq!(e.to_string(), "Not found: user/42");
    }

    // Traces to: FR-PHENO-001
    #[test]
    fn error_display_timeout() {
        let e = Error::Timeout("5s elapsed".into());
        assert_eq!(e.to_string(), "Timeout: 5s elapsed");
    }

    // Traces to: FR-PHENO-001
    #[test]
    fn error_display_unauthorized() {
        let e = Error::Unauthorized("bad token".into());
        assert_eq!(e.to_string(), "Unauthorized: bad token");
    }

    // Traces to: FR-PHENO-001
    #[test]
    fn error_display_internal() {
        let e = Error::Internal("segfault".into());
        assert_eq!(e.to_string(), "Internal error: segfault");
    }

    // Traces to: FR-PHENO-001
    #[test]
    fn error_is_std_error() {
        let e: Box<dyn std::error::Error> = Box::new(Error::Failed("x".into()));
        assert!(e.to_string().contains("Operation failed"));
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
        let r: Result<i32> = Err(Error::NotFound("missing".into()));
        assert!(r.is_err());
    }

    // Traces to: FR-PHENO-001
    #[test]
    fn error_debug_format() {
        let e = Error::Failed("test".into());
        let dbg = format!("{:?}", e);
        assert!(dbg.contains("Failed"));
    }
}
