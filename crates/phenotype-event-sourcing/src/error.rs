//! Error types for phenotype-event-sourcing.

use phenotype_error_core::ErrorKind;
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

impl From<EventSourcingError> for ErrorKind {
    fn from(e: EventSourcingError) -> Self {
        match e {
            EventSourcingError::Serialization(m) => Self::serialization(m),
            EventSourcingError::Deserialization(m) => Self::serialization(m),
            EventSourcingError::Hash(m) => Self::internal(m),
            EventSourcingError::ChainBroken { sequence } => {
                Self::validation(format!("hash chain broken at sequence {sequence}"))
            }
            EventSourcingError::EntityNotFound(m) => Self::not_found(m),
            EventSourcingError::InvalidEvent(m) => Self::validation(m),
            EventSourcingError::Internal(m) => Self::internal(m),
        }
    }
}

#[cfg(test)]
mod error_kind_tests {
    use super::*;

    // Traces to: FR-PHENO-001
    #[test]
    fn event_sourcing_error_maps_entity_not_found_to_kind() {
        let e = EventSourcingError::EntityNotFound("agg/1".into());
        let k: ErrorKind = e.into();
        assert_eq!(k.kind(), "NotFound");
    }

    // Traces to: FR-PHENO-001
    #[test]
    fn event_sourcing_error_maps_serialization_to_kind() {
        let e = EventSourcingError::Serialization("bad json".into());
        let k: ErrorKind = e.into();
        assert_eq!(k.kind(), "Serialization");
    }
}
