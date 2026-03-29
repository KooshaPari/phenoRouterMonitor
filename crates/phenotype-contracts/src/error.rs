//! Common error types for Phenotype contracts.

use thiserror::Error;

/// Result type for contract operations.
pub type Result<T> = std::result::Result<T, ContractError>;

/// Common errors that can occur in Phenotype contracts.
#[derive(Debug, Clone, Error)]
pub enum ContractError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}
