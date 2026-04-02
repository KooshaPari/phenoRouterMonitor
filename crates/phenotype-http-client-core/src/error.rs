//! HTTP transport error types.

use thiserror::Error;

/// Unified error type for HTTP transport operations.
#[derive(Debug, Error)]
pub enum TransportError {
    #[error("request failed: {0}")]
    Request(String),

    #[error("timeout: {0}")]
    Timeout(String),

    #[error("connection error: {0}")]
    Connection(String),

    #[error("authentication error: {0}")]
    Authentication(String),

    #[error("rate limited: retry after {retry_after}s")]
    RateLimited { retry_after: u64 },

    #[error("server error: {status} - {message}")]
    Server { status: u16, message: String },

    #[error("not found: {0}")]
    NotFound(String),

    #[error("serialization error: {0}")]
    Serialization(String),

    #[error("unknown: {0}")]
    Unknown(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

impl TransportError {
    /// Check if this error should trigger a retry.
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            TransportError::Timeout(_)
                | TransportError::Connection(_)
                | TransportError::RateLimited { .. }
                | TransportError::Server { status: 429 | 500..=599, .. }
        )
    }

    /// Get the error kind for categorization.
    pub fn kind(&self) -> ErrorKind {
        match self {
            TransportError::Request(_) => ErrorKind::Request,
            TransportError::Timeout(_) => ErrorKind::Timeout,
            TransportError::Connection(_) => ErrorKind::Connection,
            TransportError::Authentication(_) => ErrorKind::Authentication,
            TransportError::RateLimited { .. } => ErrorKind::RateLimited,
            TransportError::Server { .. } => ErrorKind::Server,
            TransportError::NotFound(_) => ErrorKind::NotFound,
            TransportError::Serialization(_) => ErrorKind::Serialization,
            TransportError::Unknown(_) => ErrorKind::Unknown,
            TransportError::Io(_) => ErrorKind::Io,
        }
    }
}

/// Error kind categorization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    Request,
    Timeout,
    Connection,
    Authentication,
    RateLimited,
    Server,
    NotFound,
    Serialization,
    Unknown,
    Io,
}
