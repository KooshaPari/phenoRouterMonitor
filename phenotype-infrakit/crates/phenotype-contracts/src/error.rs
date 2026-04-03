//! Common error types for Phenotype contracts.

pub use phenotype_error_core::DomainError;

/// Result type for contract operations.
pub type Result<T> = std::result::Result<T, DomainError>;

/// Convenience helpers for constructing contract errors.
pub struct ErrorKind;

impl ErrorKind {
    /// Create a not-found error with a descriptive message.
    pub fn not_found(msg: String) -> DomainError {
        DomainError::NotFound {
            entity: "entity".to_string(),
            id: msg,
        }
    }
}
