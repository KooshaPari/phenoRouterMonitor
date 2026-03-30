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
#[derive(Debug, Clone)]
pub enum EventStoreError {
    NotFound(String),
    StorageError(String),
    SequenceGap { expected: i64, actual: i64 },
}

impl std::fmt::Display for EventStoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(s) => write!(f, "event not found: {s}"),
            Self::StorageError(s) => write!(f, "storage error: {s}"),
            Self::SequenceGap { expected, actual } => {
                write!(f, "sequence gap: expected {expected}, got {actual}")
            }
        }
    }
}

impl std::error::Error for EventStoreError {}

/// Hash verification errors
#[derive(Debug, Clone)]
pub enum HashError {
    ChainBroken { sequence: i64 },
    InvalidHashLength(usize),
    HashMismatch { sequence: i64 },
}

impl std::fmt::Display for HashError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ChainBroken { sequence } => write!(f, "hash chain broken at sequence {sequence}"),
            Self::InvalidHashLength(len) => {
                write!(f, "invalid hash length: expected 64 hex chars, got {len}")
            }
            Self::HashMismatch { sequence } => write!(f, "hash mismatch at sequence {sequence}"),
        }
    }
}

impl std::error::Error for HashError {}

impl From<EventStoreError> for EventSourcingError {
    fn from(e: EventStoreError) -> Self {
        Self(e.to_string())
    }
}

impl From<HashError> for EventSourcingError {
    fn from(e: HashError) -> Self {
        Self(e.to_string())
    }
}
