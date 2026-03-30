//! Phenotype Errors - Error types for Phenotype crates.

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

/// Application-level errors.
#[derive(Debug, Error)]
pub enum Error {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("validation failed: {0}")]
    Validation(String),
    #[error("operation failed: {0}")]
    Failed(String),
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("serialization error: {0}")]
    Serialization(String),
    #[error("storage error: {0}")]
    Storage(String),
    #[error("invalid state: {0}")]
    InvalidState(String),
}
