//! Error types for the event sourcing system.

use thiserror::Error;

/// Result type for event sourcing operations.
pub type Result<T> = std::result::Result<T, EventSourcingError>;

/// Top-level error for event sourcing operations.
#[derive(Debug, Error)]
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

    #[error("version conflict")]
    VersionConflict,

    #[error("invalid event sequence")]
    InvalidEventSequence,

    #[error("internal error: {0}")]
    Internal(String),

    #[error("replay error: {0}")]
    Replay(String),

    #[error(transparent)]
    Store(#[from] EventStoreError),

    #[error(transparent)]
    Hash(#[from] HashError),
}

impl From<serde_json::Error> for EventSourcingError {
    fn from(e: serde_json::Error) -> Self {
        EventSourcingError::Serialization(e.to_string())
    }
}

impl serde::Serialize for EventSourcingError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
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
