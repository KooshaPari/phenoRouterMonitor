//! Error types.
pub use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("operation failed: {0}")]
    OperationFailed(String),
    #[error("not found: {0}")]
    NotFound(String),
}

pub type Result<T> = std::result::Result<T, Error>;
