//! Rate limiting strategies

use crate::error::{RateLimitError, RateLimitResult};
use async_trait::async_trait;

/// Rate limiting strategy
#[async_trait]
pub trait RateLimitStrategy: Send + Sync {
    /// Check if a request is allowed
    async fn check(&self, key: &str) -> RateLimitResult<()>;

    /// Record a request
    async fn record(&self, key: &str) -> RateLimitResult<()>;

    /// Get remaining capacity for a key
    async fn remaining(&self, key: &str) -> RateLimitResult<u64>;
}

/// Token bucket strategy configuration
#[derive(Debug, Clone)]
pub struct TokenBucketConfig {
    /// Maximum capacity
    pub capacity: u64,
    /// Refill rate per second
    pub refill_rate: u64,
}

impl TokenBucketConfig {
    /// Create a new token bucket config
    pub fn new(capacity: u64, refill_rate: u64) -> Self {
        Self {
            capacity,
            refill_rate,
        }
    }
}

/// Sliding window strategy configuration
#[derive(Debug, Clone)]
pub struct SlidingWindowConfig {
    /// Maximum requests in window
    pub capacity: u64,
    /// Window size in milliseconds
    pub window_size_ms: u64,
}

impl SlidingWindowConfig {
    /// Create a new sliding window config
    pub fn new(capacity: u64, window_size_ms: u64) -> Self {
        Self {
            capacity,
            window_size_ms,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_bucket_config() {
        let config = TokenBucketConfig::new(100, 10);
        assert_eq!(config.capacity, 100);
        assert_eq!(config.refill_rate, 10);
    }

    #[test]
    fn test_sliding_window_config() {
        let config = SlidingWindowConfig::new(100, 60000);
        assert_eq!(config.capacity, 100);
        assert_eq!(config.window_size_ms, 60000);
    }
}
