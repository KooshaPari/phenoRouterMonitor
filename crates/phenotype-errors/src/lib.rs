//! # Phenotype Errors
//!
//! Re-exports error types from phenotype-error-core for backward compatibility.
//!
//! This crate provides a compatibility layer for crates that depend on `phenotype_errors`.

// Re-export all error types from phenotype-error-core
pub use phenotype_error_core::{ErrorContext, ErrorExt, ErrorKind, ErrorKindInner, ErrorTimestamp};

/// Convenience alias — the primary error type for the Phenotype ecosystem.
pub type PhenotypeError = ErrorKind;

/// Convenience alias for backward compatibility.
#[deprecated(since = "0.2.0", note = "Use PhenotypeError (ErrorKind) instead")]
pub type PhenoError = ErrorKind;
