//! Error types for phenotype-event-sourcing

use thiserror::Error;

/// Result type for event sourcing operations.
pub type Result<T> = std::result::Result<T, EventSourcingError>;

/// Event sourcing errors
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum EventSourcingError {
    #[error("aggregate not found: {0}")]
    AggregateNotFound(String),

    #[error("event not found: {0}")]
    EventNotFound(String),

    #[error("serialization error: {0}")]
    Serialization(String),

    #[error("hash mismatch")]
    HashMismatch,

    #[error("snapshot error: {0}")]
    Snapshot(String),

    #[error("replay error: {0}")]
    Replay(String),

    #[error("version conflict")]
    VersionConflict,

    #[error("invalid event sequence")]
    InvalidEventSequence,

    #[error("internal error: {0}")]
    Internal(String),
}

impl EventSourcingError {
    pub fn aggregate_not_found(id: impl Into<String>) -> Self { Self::AggregateNotFound(id.into()) }
    pub fn event_not_found(id: impl Into<String>) -> Self { Self::EventNotFound(id.into()) }
    pub fn serialization(msg: impl Into<String>) -> Self { Self::Serialization(msg.into()) }
    pub fn snapshot(msg: impl Into<String>) -> Self { Self::Snapshot(msg.into()) }
    pub fn replay(msg: impl Into<String>) -> Self { Self::Replay(msg.into()) }
    pub fn internal(msg: impl Into<String>) -> Self { Self::Internal(msg.into()) }
}

#[derive(Debug, Error)]
pub enum EventStoreError {
    #[error("event not found: {0}")]
    NotFound(String),

    #[error("storage error: {0}")]
    StorageError(String),

    #[error("sequence gap: expected {expected}, got {actual}")]
    SequenceGap { expected: i64, actual: i64 },
}

#[derive(Debug, Error)]
pub enum HashError {
    #[error("hash chain broken at sequence {sequence}")]
    ChainBroken { sequence: i64 },

    #[error("invalid hash length: expected 32 bytes (64 hex digits), got {0}")]
    InvalidHashLength(usize),

    #[error("hash mismatch at sequence {sequence}")]
    HashMismatch { sequence: i64 },
}

impl From<EventSourcingError> for phenotype_errors::Error {
    fn from(err: EventSourcingError) -> Self {
        match err {
            EventSourcingError::AggregateNotFound(s) => phenotype_errors::Error::not_found(s),
            EventSourcingError::EventNotFound(s) => phenotype_errors::Error::not_found(s),
            EventSourcingError::Serialization(s) => phenotype_errors::Error::internal(format!("serialization error: {}", s)),
            EventSourcingError::HashMismatch => phenotype_errors::Error::internal("hash mismatch"),
            EventSourcingError::Snapshot(s) => phenotype_errors::Error::internal(s),
            EventSourcingError::VersionConflict => phenotype_errors::Error::conflict("version conflict"),
            EventSourcingError::InvalidEventSequence => {
                phenotype_errors::Error::internal("invalid event sequence")
            }
            EventSourcingError::Internal(s) => phenotype_errors::Error::internal(s),
            EventSourcingError::Replay(s) => phenotype_errors::Error::internal(s),
        }
    }
}
