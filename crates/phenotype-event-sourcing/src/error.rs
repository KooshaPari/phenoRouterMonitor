//! Error types for phenotype-event-sourcing.

use thiserror::Error;

/// Result type for event sourcing operations.
pub type Result<T> = std::result::Result<T, EventSourcingError>;

/// Errors that can occur during event sourcing operations.
#[derive(Debug, Error)]
pub enum EventSourcingError {
    #[error("serialization error: {0}")]
    Serialization(String),

    #[error("deserialization error: {0}")]
    Deserialization(String),

    #[error("hash error: {0}")]
    Hash(String),

    #[error("chain broken at sequence {sequence}")]
    ChainBroken { sequence: i64 },

    #[error("entity not found: {0}")]
    EntityNotFound(String),

    #[error("invalid event: {0}")]
    InvalidEvent(String),

    #[error("replay error: {0}")]
    Replay(String),

    #[error("internal error: {0}")]
    Internal(String),
}

impl EventSourcingError {
    pub fn serialization(msg: impl Into<String>) -> Self {
        Self::Serialization(msg.into())
    }

    pub fn deserialization(msg: impl Into<String>) -> Self {
        Self::Deserialization(msg.into())
    }

    pub fn hash(msg: impl Into<String>) -> Self {
        Self::Hash(msg.into())
    }

    pub fn chain_broken(sequence: i64) -> Self {
        Self::ChainBroken { sequence }
    }

    pub fn entity_not_found(id: impl Into<String>) -> Self {
        Self::EntityNotFound(id.into())
    }

    pub fn invalid_event(msg: impl Into<String>) -> Self {
        Self::InvalidEvent(msg.into())
    }

    pub fn replay(msg: impl Into<String>) -> Self {
        Self::Replay(msg.into())
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }
}

/// Hash-related errors.
#[derive(Debug, Error)]
pub enum HashError {
    #[error("invalid hash length: expected 64, got {0}")]
    InvalidHashLength(usize),

    #[error("chain broken at sequence {sequence}")]
    ChainBroken { sequence: i64 },
}

impl HashError {
    pub fn invalid_hash_length(len: usize) -> Self {
        Self::InvalidHashLength(len)
    }

    pub fn chain_broken(sequence: i64) -> Self {
        Self::ChainBroken { sequence }
    }
}
