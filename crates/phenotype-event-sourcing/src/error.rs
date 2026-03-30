//! Error types for phenotype-event-sourcing.

use thiserror::Error;

pub type Result<T> = std::result::Result<T, EventSourcingError>;

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
    #[error("internal error: {0}")]
    Internal(String),
}

#[derive(Debug, Error)]
pub enum HashError {
    #[error("invalid hash length: expected 64, got {0}")]
    InvalidHashLength(usize),
    #[error("chain broken at sequence {sequence}")]
    ChainBroken { sequence: i64 },
}
