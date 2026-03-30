//! Error types for retry operations.

use thiserror::Error;

/// Errors that can occur during retry operations
#[derive(Debug, Error)]
pub enum RetryError {
    /// Maximum retry attempts exceeded
    #[error("maximum retry attempts ({max_attempts}) exceeded")]
    MaxAttemptsExceeded {
        /// The number of attempts made
        attempts: u32,
        /// The maximum allowed attempts
        max_attempts: u32,
    },

    /// Operation timed out
    #[error("operation timed out after {timeout:?}")]
    Timeout {
        /// The timeout duration
        timeout: std::time::Duration,
    },

    /// Operation was cancelled
    #[error("operation was cancelled")]
    Cancelled,

    /// Underlying error from the operation
    #[error("operation failed: {0}")]
    OperationFailed(String),

    /// Maximum delay exceeded
    #[error("maximum delay ({max_delay:?}) exceeded")]
    MaxDelayExceeded {
        /// The maximum delay
        max_delay: std::time::Duration,
    },
}

impl RetryError {
    /// Create an error for max attempts exceeded
    pub fn max_attempts_exceeded(attempts: u32, max_attempts: u32) -> Self {
        Self::MaxAttemptsExceeded {
            attempts,
            max_attempts,
        }
    }

    /// Create an error for timeout
    pub fn timeout(duration: std::time::Duration) -> Self {
        Self::Timeout { timeout: duration }
    }

    /// Create an error for operation failure
    pub fn operation_failed<E: std::error::Error>(e: E) -> Self {
        Self::OperationFailed(e.to_string())
    }

    /// Check if this is a retriable error
    pub fn is_retriable(&self) -> bool {
        matches!(
            self,
            Self::OperationFailed(_) | Self::MaxDelayExceeded { .. }
        )
    }
}

impl From<std::time::Elapsed> for RetryError {
    fn from(_: std::time::Elapsed) -> Self {
        Self::Cancelled
    }
}
