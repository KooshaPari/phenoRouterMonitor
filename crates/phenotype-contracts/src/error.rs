//! Common error types for Phenotype contracts.

pub use phenotype_error_core::ErrorKind;

/// Result type for contract operations.
pub type Result<T> = std::result::Result<T, ContractError>;

/// Common errors that can occur in Phenotype contracts.
pub type ContractError = ErrorKind;
