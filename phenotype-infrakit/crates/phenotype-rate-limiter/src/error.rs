//! Rate limiting errors

use std::error::Error;
use thiserror::Error;

/// Rate limiting error types
#[derive(Error, Debug, Clone, PartialEq)]
pub enum RateLimitError {
    /// Rate limited
    #[error("rate limited, retry after {retry_after_ms}ms")]
    RateLimited { retry_after_ms: u64 },

    /// Backend error
    #[error("backend error: {0}")]
    BackendError(String),

    /// Configuration error
    #[error("config error: {0}")]
    ConfigError(String),
}

/// Result type for rate limiting operations
pub type RateLimitResult<T> = std::result::Result<T, RateLimitError>;

impl RateLimitError {
    /// Create a rate limited error
    pub fn rate_limited(retry_after_ms: u64) -> Self {
        Self::RateLimited { retry_after_ms }
    }

    /// Check if this is a rate limit error
    pub fn is_rate_limited(&self) -> bool {
        matches!(self, Self::RateLimited { .. })
    }

    /// Get retry after in milliseconds
    pub fn retry_after_ms(&self) -> Option<u64> {
        match self {
            Self::RateLimited { retry_after_ms } => Some(*retry_after_ms),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_error() {
        let err = RateLimitError::rate_limited(1000);
        assert!(err.is_rate_limited());
        assert_eq!(err.retry_after_ms(), Some(1000));
    }

    #[test]
    fn test_display() {
        let err = RateLimitError::BackendError("down".to_string());
        assert!(err.to_string().contains("down"));
    }

    #[test]
    fn test_source() {
        let err = RateLimitError::BackendError("test".to_string());
        assert!(err.source().is_none());
    }
}
