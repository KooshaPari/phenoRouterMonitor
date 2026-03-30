//! Contract-specific error types.
//!
//! Traces to: FR-PHENO-001

use phenotype_error_core::ErrorKind;
use thiserror::Error;

/// Errors that can occur when working with contracts.
#[derive(Debug, Clone, Error)]
pub enum ContractError {
    /// An operation on a port failed.
    #[error("port error: {0}")]
    Port(String),

    /// A domain invariant was violated.
    #[error("invariant violation: {0}")]
    InvariantViolation(String),

    /// The requested entity was not found.
    #[error("not found: {0}")]
    NotFound(String),

    /// A conflict occurred (e.g. optimistic concurrency).
    #[error("conflict: {0}")]
    Conflict(String),

    /// Serialization or deserialization failed.
    #[error("serialization: {0}")]
    Serialization(String),

    /// An underlying infrastructure error.
    #[error(transparent)]
    Infrastructure(ErrorKind),
}

impl From<ErrorKind> for ContractError {
    fn from(err: ErrorKind) -> Self {
        Self::Infrastructure(err)
    }
}

/// Convenience result type for contract operations.
pub type Result<T> = std::result::Result<T, ContractError>;

#[cfg(test)]
mod tests {
    use super::*;

    // Traces to: FR-PHENO-001
    #[test]
    fn contract_error_display() {
        let err = ContractError::NotFound("user:42".into());
        assert_eq!(err.to_string(), "not found: user:42");
    }

    // Traces to: FR-PHENO-001
    #[test]
    fn contract_error_from_error_kind() {
        let kind = ErrorKind::not_found("item");
        let err: ContractError = kind.into();
        assert!(matches!(err, ContractError::Infrastructure(_)));
    }
}
