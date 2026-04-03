//! Error types for phenotype-event-sourcing.

use thiserror::Error;

pub type Result<T> = std::result::Result<T, EventSourcingError>;

#[derive(Debug, Clone, Error)]
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

#[derive(Debug, Clone, Error)]
pub enum HashError {
    #[error("invalid hash length: expected 64 hex chars (32 bytes), got {0}")]
    InvalidHashLength(usize),

    #[error("chain broken at sequence {sequence}")]
    ChainBroken { sequence: i64 },

    #[error("hex decode error: {0}")]
    HexDecode(String),
}

impl From<HashError> for EventSourcingError {
    fn from(err: HashError) -> Self {
        match err {
            HashError::InvalidHashLength(len) => {
                EventSourcingError::Hash(format!("invalid hash length: {}", len))
            }
            HashError::ChainBroken { sequence } => EventSourcingError::ChainBroken { sequence },
            HashError::HexDecode(msg) => {
                EventSourcingError::Hash(format!("hex decode error: {}", msg))
            }
        }
    }
}

impl From<serde_json::Error> for EventSourcingError {
    fn from(err: serde_json::Error) -> Self {
        EventSourcingError::Serialization(err.to_string())
    }
}
