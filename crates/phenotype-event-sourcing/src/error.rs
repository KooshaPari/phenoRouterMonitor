//! Common error types for event sourcing.

pub use phenotype_error_core::Error;

#[derive(Debug, thiserror::Error)]
pub enum EventSourcingError {
    #[error("hash error: {0}")]
    Hash(String),
    #[error("serialization error: {0}")]
    Serialization(String),
    #[error("storage error: {0}")]
    Storage(String),
    #[error("event not found: {0}")]
    EventNotFound(String),
    #[error("version conflict: {0}")]
    VersionConflict(String),
}
