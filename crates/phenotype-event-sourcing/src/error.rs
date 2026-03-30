//! Error types for the event sourcing system.

use phenotype_error_core::ErrorKind;
use serde::Serialize;

/// Result type for event sourcing operations.
pub type Result<T> = std::result::Result<T, EventSourcingError>;

/// Wrapper error type for event sourcing operations.
#[derive(Debug, Clone, Serialize)]
pub struct EventSourcingError(pub ErrorKind);

impl std::fmt::Display for EventSourcingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for EventSourcingError {}

impl EventSourcingError {
    /// Create a new error
    pub fn new(msg: impl Into<String>) -> Self {
        Self(ErrorKind::internal(msg))
    }

    /// Aggregate not found
    pub fn aggregate_not_found(id: impl Into<String>) -> Self {
        Self(ErrorKind::not_found(format!("aggregate: {}", id.into())))
    }

    /// Event not found
    pub fn event_not_found(id: impl Into<String>) -> Self {
        Self(ErrorKind::not_found(format!("event: {}", id.into())))
    }

    /// Serialization error
    pub fn serialization(msg: impl Into<String>) -> Self {
        Self(ErrorKind::serialization(msg))
    }

    /// Snapshot error
    pub fn snapshot(msg: impl Into<String>) -> Self {
        Self(ErrorKind::storage(format!("snapshot: {}", msg.into())))
    }

    /// Replay error
    pub fn replay(msg: impl Into<String>) -> Self {
        Self(ErrorKind::internal(format!("replay: {}", msg.into())))
    }

    /// Internal error
    pub fn internal(msg: impl Into<String>) -> Self {
        Self(ErrorKind::internal(msg))
    }
}

impl From<ErrorKind> for EventSourcingError {
    fn from(kind: ErrorKind) -> Self {
        Self(kind)
    }
}

impl From<std::io::Error> for EventSourcingError {
    fn from(e: std::io::Error) -> Self {
        Self(ErrorKind::from(e))
    }
}

impl From<serde_json::Error> for EventSourcingError {
    fn from(e: serde_json::Error) -> Self {
        Self(ErrorKind::from(e))
    }
}

/// Event store errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum EventStoreError {
    #[error("event not found: {0}")]
    NotFound(String),

    #[error("storage error: {0}")]
    StorageError(String),

    #[error("sequence gap: expected {expected}, got {actual}")]
    SequenceGap { expected: i64, actual: i64 },
}

/// Hash verification errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum HashError {
    #[error("hash chain broken at sequence {sequence}")]
    ChainBroken { sequence: i64 },

    #[error("invalid hash length: expected 64 hex chars, got {0}")]
    InvalidHashLength(usize),

    #[error("hash mismatch at sequence {sequence}")]
    HashMismatch { sequence: i64 },
}

impl From<EventStoreError> for EventSourcingError {
    fn from(e: EventStoreError) -> Self {
        Self(e.to_string().into())
    }
}

impl From<HashError> for EventSourcingError {
    fn from(e: HashError) -> Self {
        Self(e.to_string().into())
    }
}
