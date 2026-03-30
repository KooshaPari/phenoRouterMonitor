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

    #[error("snapshot error: {0}")]
    Snapshot(String),

    #[error("replay error: {0}")]
    Replay(String),

    #[error("version conflict: expected {expected}, got {actual}")]
    VersionConflict { expected: u64, actual: u64 },

    #[error("invalid event sequence at position {position}")]
    InvalidEventSequence { position: u64 },

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

    #[error("duplicate sequence: {0}")]
    DuplicateSequence(String),

    #[error("storage error: {0}")]
    StorageError(String),

    #[error("invalid hash: {0}")]
    InvalidHash(String),

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
            EventSourcingError::Serialization(e) => {
                phenotype_errors::PhenotypeError::Serialization(e.to_string())
            }
            EventSourcingError::HashMismatch { expected, actual } => {
                phenotype_errors::PhenotypeError::InvalidState(format!(
                    "hash mismatch: expected {}, got {}",
                    expected, actual
                ))
            }
            EventSourcingError::Snapshot(s) => {
                phenotype_errors::PhenotypeError::InvalidState(s)
            }
            EventSourcingError::VersionConflict { expected, actual } => {
                phenotype_errors::PhenotypeError::Conflict(format!(
                    "version conflict: expected {}, got {}",
                    expected, actual
                ))
            }
            EventSourcingError::InvalidEventSequence { position } => {
                phenotype_errors::PhenotypeError::InvalidState(format!(
                    "invalid event sequence at position {}",
                    position
                ))
            }
            EventSourcingError::Internal(s) => phenotype_errors::PhenotypeError::Internal(s),
            EventSourcingError::Store(e) => {
                // Map EventStoreError variants
                match e {
                    EventStoreError::NotFound(s) => {
                        phenotype_errors::PhenotypeError::NotFound(s)
                    }
                    EventStoreError::DuplicateSequence(s) => {
                        phenotype_errors::PhenotypeError::Conflict(s)
                    }
                    EventStoreError::StorageError(s) => {
                        phenotype_errors::PhenotypeError::StorageFailure(s)
                    }
                    EventStoreError::InvalidHash(s) => {
                        phenotype_errors::PhenotypeError::InvalidState(s)
                    }
                    EventStoreError::SequenceGap { expected, actual } => {
                        phenotype_errors::PhenotypeError::InvalidState(format!(
                            "sequence gap: expected {}, got {}",
                            expected, actual
                        ))
                    }
                }
            }
            EventSourcingError::Hash(e) => {
                // Map HashError variants
                match e {
                    HashError::ChainBroken { sequence } => {
                        phenotype_errors::PhenotypeError::InvalidState(format!(
                            "hash chain broken at sequence {}",
                            sequence
                        ))
                    }
                    HashError::InvalidHashLength(len) => {
                        phenotype_errors::PhenotypeError::InvalidState(format!(
                            "invalid hash length: expected 32 bytes (64 hex digits), got {}",
                            len
                        ))
                    }
                    HashError::HashMismatch { sequence } => {
                        phenotype_errors::PhenotypeError::InvalidState(format!(
                            "hash mismatch at sequence {}",
                            sequence
                        ))
                    }
                }
            }
            EventSourcingError::Replay(s) => phenotype_errors::PhenotypeError::InvalidState(s),
        }
    }
}
