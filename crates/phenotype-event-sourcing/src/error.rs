//! Error types for phenotype-event-sourcing.

use thiserror::Error;

/// Result type for event sourcing operations.
pub type Result<T> = std::result::Result<T, EventSourcingError>;

#[derive(Debug, Error)]
pub enum EventSourcingError {
    #[error(transparent)]
    Store(#[from] EventStoreError),

    #[error(transparent)]
    Hash(#[from] HashError),

    #[error(transparent)]
    Serialization(#[from] serde_json::Error),

    #[error("aggregate not found: {0}")]
    AggregateNotFound(String),

    #[error("event not found: {0}")]
    EventNotFound(String),

    #[error("hash mismatch: expected {expected}, got {actual}")]
    HashMismatch { expected: String, actual: String },

    #[error("version conflict: expected {expected}, got {actual}")]
    VersionConflict { expected: u64, actual: u64 },

    #[error("internal error: {0}")]
    Internal(String),
}

impl EventSourcingError {
    pub fn aggregate_not_found(id: impl Into<String>) -> Self {
        Self::AggregateNotFound(id.into())
    }

    pub fn hash_mismatch(expected: impl Into<String>, actual: impl Into<String>) -> Self {
        Self::HashMismatch {
            expected: expected.into(),
            actual: actual.into(),
        }
    }
}

impl From<std::io::Error> for EventSourcingError {
    fn from(e: std::io::Error) -> Self {
        Self::Internal(e.to_string())
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

// Conversion to unified phenotype error hierarchy
impl From<EventSourcingError> for phenotype_errors::PhenotypeError {
    fn from(err: EventSourcingError) -> Self {
        match err {
            EventSourcingError::AggregateNotFound(s) => {
                phenotype_errors::PhenotypeError::NotFound(s)
            }
            EventSourcingError::EventNotFound(s) => {
                phenotype_errors::PhenotypeError::NotFound(s)
            }
            EventSourcingError::Serialization(s) => {
                phenotype_errors::PhenotypeError::Serialization(s)
            }
            EventSourcingError::HashMismatch => {
                phenotype_errors::PhenotypeError::InvalidState("hash mismatch".to_string())
            }
            EventSourcingError::Snapshot(s) => {
                phenotype_errors::PhenotypeError::InvalidState(s)
            }
            EventSourcingError::VersionConflict => {
                phenotype_errors::PhenotypeError::Conflict("version conflict".to_string())
            }
            EventSourcingError::InvalidEventSequence => {
                phenotype_errors::PhenotypeError::InvalidState("invalid event sequence".to_string())
            }
            EventSourcingError::Internal(s) => phenotype_errors::PhenotypeError::Internal(s),
            EventSourcingError::Replay(s) => {
                phenotype_errors::PhenotypeError::InvalidState(s)
            }
        }
    }
}
