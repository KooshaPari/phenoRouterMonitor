//! Core errors.
pub use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("internal: {0}")]
    Internal(String),
}
