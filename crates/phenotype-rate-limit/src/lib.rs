//! Token bucket rate limiter for API clients.
//!
//! Provides a generic, async-friendly token bucket implementation
//! for rate limiting HTTP clients.
//!
//! # Example
//!
//! ```rust
//! use phenotype_rate_limit::TokenBucket;
//! use std::sync::Arc;
//! use std::time::Duration;
//!
//! // Create a bucket with 60 tokens, refilling at 1/second
//! let bucket = Arc::new(TokenBucket::new(60.0, 1.0));
//!
//! // Try to acquire a token
//! assert!(bucket.try_acquire());
//! ```

use std::time::{Duration, Instant};

/// Token bucket for rate limiting.
///
/// Refills tokens at a constant rate. A token is consumed on each `try_acquire()` call.
#[derive(Debug, Clone)]
pub struct TokenBucket {
    tokens: f64,
    max_tokens: f64,
    refill_rate: f64,
    last_refill: Instant,
}

impl TokenBucket {
    /// Create a new bucket with `max_tokens` capacity and `refill_rate` tokens/second.
    pub fn new(max_tokens: f64, refill_rate: f64) -> Self {
        Self {
            tokens: max_tokens,
            max_tokens,
            refill_rate,
            last_refill: Instant::now(),
        }
    }

    /// Try to acquire a token without blocking.
    pub fn try_acquire(&mut self) -> bool {
        self.refill();
        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }

    /// Returns the time until a token is available.
    pub fn time_until_available(&self) -> Duration {
        if self.tokens >= 1.0 {
            Duration::ZERO
        } else {
            let needed = 1.0 - self.tokens;
            Duration::from_secs_f64(needed / self.refill_rate)
        }
    }

    /// Returns the current number of available tokens.
    pub fn available(&self) -> f64 {
        let elapsed = Instant::now().duration_since(self.last_refill).as_secs_f64();
        (self.tokens + elapsed * self.refill_rate).min(self.max_tokens)
    }

    fn refill(&mut self) {
        let elapsed = Instant::now().duration_since(self.last_refill).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.max_tokens);
        self.last_refill = Instant::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bucket_acquire() {
        let mut bucket = TokenBucket::new(5.0, 1.0);
        assert!(bucket.try_acquire());
        assert!(bucket.try_acquire());
        assert!(bucket.try_acquire());
    }

    #[test]
    fn test_bucket_depleted() {
        let mut bucket = TokenBucket::new(2.0, 0.0);
        assert!(bucket.try_acquire());
        assert!(bucket.try_acquire());
        assert!(!bucket.try_acquire());
    }

    #[test]
    fn test_time_until_available() {
        let bucket = TokenBucket::new(0.0, 1.0);
        assert!(bucket.time_until_available() > Duration::ZERO);
    }
}
