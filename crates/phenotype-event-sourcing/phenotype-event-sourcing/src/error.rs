//! Error types for the event sourcing system.

/// Result type for event sourcing operations.
pub type Result<T> = std::result::Result<T, EventSourcingError>;

#[derive(Debug, thiserror::Error)]
pub enum EventSourcingError {
    #[error("Store error: {0}")]
    Store(#[from] EventStoreError),

    #[error("Hash error: {0}")]
    Hash(#[from] HashError),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
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
