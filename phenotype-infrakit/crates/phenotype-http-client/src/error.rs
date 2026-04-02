//! HTTP client errors

use thiserror::Error;

/// HTTP client errors
#[derive(Debug, Error)]
pub enum HttpClientError {
    #[error("request failed: {0}")]
    RequestFailed(String),

    #[error("timeout after {0:?}")]
    Timeout(String),

    #[error("connection error: {0}")]
    ConnectionError(String),

    #[error("invalid URL: {0}")]
    InvalidUrl(String),

    #[error("serialization error: {0}")]
    SerializationError(String),

    #[error("deserialization error: {0}")]
    DeserializationError(String),
}

/// Result type alias
pub type Result<T> = std::result::Result<T, HttpClientError>;
