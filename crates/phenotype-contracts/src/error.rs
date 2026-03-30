//! Common error types for Phenotype contracts.

pub use phenotype_error_core::Error;

/// Result type for contract operations.
pub type Result<T> = std::result::Result<T, ContractError>;

/// Common errors that can occur in Phenotype contracts.
pub type ContractError = Error;

// Backward compatibility aliases
#[deprecated(since = "0.2.0", note = "Use Error instead")]
pub type OldContractError = Error;
