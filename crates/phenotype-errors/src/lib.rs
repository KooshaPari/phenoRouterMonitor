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
