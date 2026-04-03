//! Error types

use thiserror::Error;

#[derive(Error, Debug)]
pub enum PlatformError {
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("operation failed: {0}")]
    OperationFailed(String),
    #[error("internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, PlatformError>;
