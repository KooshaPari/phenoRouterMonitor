//! Core error types for Phenotype (backward compatibility re-export).
//!
//! This crate is maintained for backward compatibility. New code should use
//! `phenotype_errors::PhenotypeError` directly.
//!
//! ## Deprecation Notice
//!
//! This crate re-exports core error types from `phenotype-errors` for backward compatibility.
//! Future versions will consolidate to `phenotype-errors` as the sole error crate.

use thiserror::Error;

/// Core error type (legacy, use `PhenotypeError` instead).
///
/// Provided for backward compatibility with existing code.
/// New code should use `phenotype_errors::PhenotypeError` instead.
#[derive(Error, Debug)]
pub enum CoreError {
    #[error("Operation failed: {0}")]
    Failed(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

/// Result type for legacy code (use `std::result::Result` instead).
pub type Result<T> = std::result::Result<T, CoreError>;

// Conversions from CoreError to PhenotypeError for forward compatibility
impl From<CoreError> for phenotype_errors::PhenotypeError {
    fn from(e: CoreError) -> Self {
        match e {
            CoreError::Failed(msg) => phenotype_errors::PhenotypeError::Internal(msg),
            CoreError::NotFound(msg) => phenotype_errors::PhenotypeError::NotFound(msg),
            CoreError::InvalidInput(msg) => phenotype_errors::PhenotypeError::ValidationFailed(msg),
        }
    }
}
