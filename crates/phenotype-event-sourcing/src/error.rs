//! Error types for phenotype-event-sourcing

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
