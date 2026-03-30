//! Error types

pub type Result<T> = std::result::Result<T, EventSourcingError>;

#[derive(Debug, thiserror::Error)]
pub enum EventSourcingError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("invalid hash chain")]
    InvalidChain,
    #[error("IO error: {0}")]
    Io(String),
    #[error("serialization error: {0}")]
    Serialization(String),
    #[error("internal error: {0}")]
    Internal(String),
}

impl EventSourcingError {
    pub fn not_found<S: Into<String>>(msg: S) -> Self { Self::NotFound(msg.into()) }
    pub fn internal<S: Into<String>>(msg: S) -> Self { Self::Internal(msg.into()) }
}
