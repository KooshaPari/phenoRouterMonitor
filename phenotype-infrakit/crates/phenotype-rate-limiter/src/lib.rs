//! Rate limiting traits and implementations

use async_trait::async_trait;
use std::time::{Duration, Instant};

pub mod error;
pub mod strategy;

pub use error::{RateLimitError, RateLimitResult};

/// Rate limiter trait for token bucket and sliding window implementations
#[async_trait]
pub trait RateLimiter: Send + Sync {
    /// Try to acquire a permit, returns Ok(()) if allowed, Err if rate limited
    async fn try_acquire(&self) -> RateLimitResult<()>;

    /// Get the retry-after duration if rate limited
    fn retry_after(&self) -> Duration;

    /// Get the current available permits
    fn available_permits(&self) -> u64;
}

/// Token bucket rate limiter
pub struct TokenBucket {
    capacity: u64,
    tokens: u64,
    refill_rate: u64, // tokens per second
    last_refill: Instant,
}

impl TokenBucket {
    /// Create a new token bucket
    pub fn new(capacity: u64, refill_rate: u64) -> Self {
        Self {
            capacity,
            tokens: capacity,
            refill_rate,
            last_refill: Instant::now(),
        }
    }

    fn refill(&mut self) {
        let elapsed = self.last_refill.elapsed().as_secs_f64();
        let new_tokens = (elapsed * self.refill_rate as f64) as u64;
        self.tokens = (self.tokens + new_tokens).min(self.capacity);
        self.last_refill = Instant::now();
    }
}

#[async_trait]
impl RateLimiter for TokenBucket {
    async fn try_acquire(&self) -> RateLimitResult<()> {
        let mut bucket = Self::new(self.capacity, self.refill_rate);
        bucket.refill();
        if bucket.tokens > 0 {
            bucket.tokens -= 1;
            Ok(())
        } else {
            Err(RateLimitError::rate_limited(1000))
        }
    }

    fn retry_after(&self) -> Duration {
        Duration::from_millis(1000)
    }

    fn available_permits(&self) -> u64 {
        let mut bucket = Self::new(self.capacity, self.refill_rate);
        bucket.refill();
        bucket.tokens
    }
}

/// Sliding window rate limiter
pub struct SlidingWindow {
    capacity: u64,
    window_size: Duration,
    requests: Vec<Instant>,
}

impl SlidingWindow {
    /// Create a new sliding window
    pub fn new(capacity: u64, window_size: Duration) -> Self {
        Self {
            capacity,
            window_size,
            requests: Vec::new(),
        }
    }

    fn clean_old_requests(&mut self) {
        let cutoff = Instant::now() - self.window_size;
        self.requests.retain(|&t| t > cutoff);
    }
}

#[async_trait]
impl RateLimiter for SlidingWindow {
    async fn try_acquire(&self) -> RateLimitResult<()> {
        let mut window = Self::new(self.capacity, self.window_size);
        window.requests = self.requests.clone();
        window.clean_old_requests();
        if window.requests.len() < window.capacity as usize {
            window.requests.push(Instant::now());
            Ok(())
        } else {
            Err(RateLimitError::rate_limited(1000))
        }
    }

    fn retry_after(&self) -> Duration {
        self.window_size
    }

    fn available_permits(&self) -> u64 {
        let mut window = Self::new(self.capacity, self.window_size);
        window.requests = self.requests.clone();
        window.clean_old_requests();
        (self.capacity - window.requests.len() as u64).max(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_token_bucket_initial() {
        let bucket = TokenBucket::new(5, 1);
        assert_eq!(bucket.available_permits(), 5);
    }

    #[tokio::test]
    async fn test_sliding_window_initial() {
        let window = SlidingWindow::new(10, Duration::from_secs(60));
        assert_eq!(window.available_permits(), 10);
    }
}
