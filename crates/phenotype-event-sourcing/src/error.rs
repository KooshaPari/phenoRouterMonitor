//! Error types for the event sourcing system.
//!
//! Uses phenotype-error-core for foundational error types.
//! Error types for phenotype-event-sourcing
//!
//! These error types are specific to event sourcing operations.
//! They can be converted to the unified `PhenotypeError` using `From` implementations.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Result type for event sourcing operations
pub type Result<T> = std::result::Result<T, EventSourcingError>;

/// Event sourcing errors
#[derive(Debug, Error, Serialize, Deserialize, Clone)]
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

// Conversion to unified PhenotypeError (when phenotype_errors is available)
#[cfg(feature = "with-phenotype-errors")]
impl From<EventSourcingError> for phenotype_errors::PhenotypeError {
    fn from(err: EventSourcingError) -> Self {
        use phenotype_errors::PhenotypeError as PE;
        match err {
            EventSourcingError::AggregateNotFound(id) => 
                PE::NotFound(format!("aggregate: {}", id)),
            EventSourcingError::EventNotFound(id) => 
                PE::NotFound(format!("event: {}", id)),
            EventSourcingError::Serialization(msg) => 
                PE::Serialization(msg),
            EventSourcingError::HashMismatch => 
                PE::InvalidState("hash mismatch".into()),
            EventSourcingError::Snapshot(msg) => 
                PE::InvalidState(format!("snapshot: {}", msg)),
            EventSourcingError::Replay(msg) => 
                PE::InvalidState(format!("replay: {}", msg)),
            EventSourcingError::VersionConflict => 
                PE::Conflict("version conflict".into()),
            EventSourcingError::InvalidEventSequence => 
                PE::InvalidState("invalid event sequence".into()),
            EventSourcingError::Internal(msg) => 
                PE::Internal(msg),
        }
    }
}
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
