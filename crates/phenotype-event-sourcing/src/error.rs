//! Error types for event sourcing.

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, EventSourcingError>;

#[derive(Error, Debug, Serialize, Deserialize, Clone)]
pub enum EventSourcingError {
    #[error("Invalid hash: {0}")]
    InvalidHash(String),
    #[error("Serialization error")]
    SerializationError,
}
