//! Error types for the event sourcing system.

use serde::{Serialize, Serializer};
use thiserror::Error;

/// Result type for event sourcing operations.
pub type Result<T> = std::result::Result<T, EventSourcingError>;

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
        Self::Serialization(e.to_string())
    }
}

impl Serialize for EventSourcingError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl EventSourcingError {
    pub fn aggregate_not_found(id: impl Into<String>) -> Self {
        Self::AggregateNotFound(id.into())
    }

    pub fn event_not_found(id: impl Into<String>) -> Self {
        Self::EventNotFound(id.into())
    }

    pub fn serialization(msg: impl Into<String>) -> Self {
        Self::Serialization(msg.into())
    }

    pub fn snapshot(msg: impl Into<String>) -> Self {
        Self::Snapshot(msg.into())
    }

    pub fn replay(msg: impl Into<String>) -> Self {
        Self::Replay(msg.into())
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
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

impl From<EventStoreError> for phenotype_errors::Error {
    fn from(e: EventStoreError) -> Self {
        match e {
            EventStoreError::NotFound(s) => Self::not_found(s),
            EventStoreError::StorageError(s) => Self::internal(s),
            EventStoreError::SequenceGap { .. } => Self::internal(e.to_string()),
        }
    }
}

impl From<HashError> for phenotype_errors::Error {
    fn from(e: HashError) -> Self {
        Self::internal(e.to_string())
    }
}

impl From<EventSourcingError> for phenotype_errors::Error {
    fn from(err: EventSourcingError) -> Self {
        match err {
            EventSourcingError::AggregateNotFound(s) => Self::not_found(s),
            EventSourcingError::EventNotFound(s) => Self::not_found(s),
            EventSourcingError::Serialization(s) => Self::internal(format!("serialization error: {s}")),
            EventSourcingError::HashMismatch => Self::internal("hash mismatch"),
            EventSourcingError::Snapshot(s) => Self::internal(s),
            EventSourcingError::VersionConflict => Self::conflict("version conflict"),
            EventSourcingError::InvalidEventSequence => Self::internal("invalid event sequence"),
            EventSourcingError::Internal(s) => Self::internal(s),
            EventSourcingError::Replay(s) => Self::internal(s),
            EventSourcingError::Store(e) => e.into(),
            EventSourcingError::Hash(e) => e.into(),
        }
    }
}
