//! Phenotype Error Core - Shared error types.

use thiserror::Error;

/// Common error types for Phenotype crates.
#[derive(Error, Debug)]
pub enum ErrorKind {
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
}

pub type Result<T> = std::result::Result<T, ErrorKind>;
