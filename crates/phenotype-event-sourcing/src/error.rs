//! Error types for phenotype-event-sourcing

/// Event sourcing errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventSourcingError {
    AggregateNotFound(String),
    EventNotFound(String),
    Serialization(String),
    HashMismatch,
    Snapshot(String),
    Replay(String),
    VersionConflict,
    InvalidEventSequence,
    Internal(String),
}

impl std::fmt::Display for EventSourcingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AggregateNotFound(s) => write!(f, "aggregate not found: {}", s),
            Self::EventNotFound(s) => write!(f, "event not found: {}", s),
            Self::Serialization(s) => write!(f, "serialization error: {}", s),
            Self::HashMismatch => write!(f, "hash mismatch"),
            Self::Snapshot(s) => write!(f, "snapshot error: {}", s),
            Self::Replay(s) => write!(f, "replay error: {}", s),
            Self::VersionConflict => write!(f, "version conflict"),
            Self::InvalidEventSequence => write!(f, "invalid event sequence"),
            Self::Internal(s) => write!(f, "internal error: {}", s),
        }
    }
}

impl std::error::Error for EventSourcingError {}

impl EventSourcingError {
    pub fn aggregate_not_found(id: impl Into<String>) -> Self { Self::AggregateNotFound(id.into()) }
    pub fn event_not_found(id: impl Into<String>) -> Self { Self::EventNotFound(id.into()) }
    pub fn serialization(msg: impl Into<String>) -> Self { Self::Serialization(msg.into()) }
    pub fn snapshot(msg: impl Into<String>) -> Self { Self::Snapshot(msg.into()) }
    pub fn replay(msg: impl Into<String>) -> Self { Self::Replay(msg.into()) }
    pub fn internal(msg: impl Into<String>) -> Self { Self::Internal(msg.into()) }
}

impl From<serde_json::Error> for EventSourcingError {
    fn from(e: serde_json::Error) -> Self { Self::serialization(e.to_string()) }
}
