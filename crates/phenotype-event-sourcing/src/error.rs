//! Error types for the event sourcing system.
//!
//! Uses phenotype-error-core for foundational error types.

use phenotype_error_core::CoreError;

/// Result type for event sourcing operations.
pub type Result<T> = std::result::Result<T, EventSourcingError>;

/// Wrapper error type for event sourcing operations.
/// Maps domain-specific errors to CoreError variants.
#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct EventSourcingError(pub CoreError);

impl From<CoreError> for EventSourcingError {
    fn from(e: CoreError) -> Self {
        EventSourcingError(e)
    }
}

impl From<EventStoreError> for EventSourcingError {
    fn from(e: EventStoreError) -> Self {
        EventSourcingError(CoreError::Failed(e.to_string()))
    }
}

impl From<HashError> for EventSourcingError {
    fn from(e: HashError) -> Self {
        EventSourcingError(CoreError::Failed(e.to_string()))
    }
}

impl From<serde_json::Error> for EventSourcingError {
    fn from(e: serde_json::Error) -> Self {
        EventSourcingError(CoreError::Failed(format!("Serialization error: {}", e)))
    }
}

impl serde::Serialize for EventSourcingError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum EventStoreError {
    #[error("Event not found: {0}")]
    NotFound(String),

    #[error("Duplicate sequence: {0}")]
    DuplicateSequence(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Invalid hash: {0}")]
    InvalidHash(String),

    #[error("Sequence gap: expected {expected}, got {actual}")]
    SequenceGap { expected: i64, actual: i64 },
}

#[derive(Debug, thiserror::Error)]
pub enum HashError {
    #[error("Hash chain broken at sequence {sequence}")]
    ChainBroken { sequence: i64 },

    #[error("Invalid hash length: expected 32, got {0}")]
    InvalidHashLength(usize),

    #[error("Hash mismatch at sequence {sequence}")]
    HashMismatch { sequence: i64 },
}
