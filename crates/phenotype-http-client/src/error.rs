//! HTTP client error types.

use thiserror::Error;

/// Errors that can occur during HTTP operations.
#[derive(Debug, Error)]
pub enum HttpClientError {
    #[error("request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("unexpected status: {0} {1}")]
    UnexpectedStatus(u16, String),

    #[error("timeout after {0:?}")]
    Timeout(Duration),

    #[error("invalid URL: {0}")]
    InvalidUrl(String),

    #[error("deserialization failed: {0}")]
    Deserialization(String),

    #[error("connection failed: {0}")]
    Connection(String),
}

/// Result type alias for HTTP operations.
pub type HttpResult<T> = Result<T, HttpClientError>;

use std::time::Duration;

impl HttpClientError {
    /// Check if the error is retryable.
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::Timeout(_)
                | Self::Connection(_)
                | Self::UnexpectedStatus(429, _)
                | Self::UnexpectedStatus(500, _)
                | Self::UnexpectedStatus(502, _)
                | Self::UnexpectedStatus(503, _)
                | Self::UnexpectedStatus(504, _)
        )
    }

    /// Get status code if this is a status error.
    pub fn status_code(&self) -> Option<u16> {
        match self {
            Self::UnexpectedStatus(code, _) => Some(*code),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeout_is_retryable() {
        let err = HttpClientError::Timeout(Duration::from_secs(30));
        assert!(err.is_retryable());
    }

    #[test]
    fn test_429_is_retryable() {
        let err = HttpClientError::UnexpectedStatus(429, "Too Many Requests".into());
        assert!(err.is_retryable());
    }

    #[test]
    fn test_404_is_not_retryable() {
        let err = HttpClientError::UnexpectedStatus(404, "Not Found".into());
        assert!(!err.is_retryable());
    }
}
