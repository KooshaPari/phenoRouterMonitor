//! # Phenotype Errors
//!
//! Re-exports error types from phenotype-error-core for backward compatibility.
//!
//! This crate provides a compatibility layer for crates that depend on `phenotype_errors`.

// Re-export all error types from phenotype-error-core
pub use phenotype_error_core::Error;

// Convenience type aliases for backward compatibility
#[deprecated(since = "0.2.0", note = "Use phenotype_error_core::Error instead")]
pub type PhenoError = Error;

#[deprecated(since = "0.2.0", note = "Use phenotype_error_core::Error::NotFound instead")]
pub fn not_found<S: Into<String>>(msg: S) -> Error {
    Error::not_found(msg)
}

#[deprecated(since = "0.2.0", note = "Use phenotype_error_core::Error::Validation instead")]
pub fn validation<S: Into<String>>(msg: S) -> Error {
    Error::validation(msg)
}

#[deprecated(since = "0.2.0", note = "Use phenotype_error_core::Error::Conflict instead")]
pub fn conflict<S: Into<String>>(msg: S) -> Error {
    Error::conflict(msg)
}
