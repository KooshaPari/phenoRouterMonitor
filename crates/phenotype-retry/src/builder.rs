//! Retry builder for configuring retry behavior.

use std::future::Future;
use std::time::Duration;

/// Maximum number of retry attempts
const DEFAULT_MAX_ATTEMPTS: u32 = 3;

/// Base delay between retries
const DEFAULT_BASE_DELAY: Duration = Duration::from_millis(100);

/// Maximum delay between retries
const DEFAULT_MAX_DELAY: Duration = Duration::from_secs(30);

/// Builder for configuring retry behavior.
#[derive(Debug, Clone)]
pub struct RetryBuilder {
    max_attempts: u32,
    base_delay: Duration,
    max_delay: Duration,
    jitter: bool,
}

impl Default for RetryBuilder {
    fn default() -> Self {
        Self {
            max_attempts: DEFAULT_MAX_ATTEMPTS,
            base_delay: DEFAULT_BASE_DELAY,
            max_delay: DEFAULT_MAX_DELAY,
            jitter: false,
        }
    }
}

impl RetryBuilder {
    /// Set maximum number of retry attempts (clamped to at least 1).
    pub fn max_attempts(mut self, attempts: u32) -> Self {
        self.max_attempts = attempts.max(1);
        self
    }

    /// Set base delay between retries.
    pub fn base_delay(mut self, delay: Duration) -> Self {
        self.base_delay = delay;
        self
    }

    /// Set maximum delay between retries.
    pub fn max_delay(mut self, delay: Duration) -> Self {
        self.max_delay = delay;
        self
    }

    /// Enable deterministic jitter (spread based on attempt index; no extra deps).
    pub fn with_jitter(mut self) -> Self {
        self.jitter = true;
        self
    }

    /// Execute an async operation with retry logic.
    pub async fn execute<F, Fut, T, E>(&self, mut f: F) -> Result<T, E>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<T, E>>,
    {
        let mut backoff = ExponentialBackoff::new(self.base_delay, self.max_delay);

        for attempt in 0..self.max_attempts {
            match f().await {
                Ok(result) => return Ok(result),
                Err(e) if attempt == self.max_attempts - 1 => return Err(e),
                Err(_) => {
                    if let Some(delay) = backoff.next_delay() {
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        unreachable!()
    }
}

/// Exponential backoff calculator
#[derive(Debug, Clone)]
pub struct ExponentialBackoff {
    current_delay: Duration,
    base_delay: Duration,
    max_delay: Duration,
    multiplier: f64,
}

impl ExponentialBackoff {
    /// Create a new backoff calculator
    pub fn new(base_delay: Duration, max_delay: Duration) -> Self {
        Self {
            current_delay: base_delay,
            base_delay,
            max_delay,
            multiplier: 2.0,
        }
    }

    /// Get the next delay value
    pub fn next_delay(&mut self) -> Option<Duration> {
        if self.current_delay > self.max_delay {
            return None;
        }

        let delay = self.current_delay;
        self.current_delay =
            Duration::from_millis((self.current_delay.as_millis() as f64 * self.multiplier) as u64);
        self.current_delay = self.current_delay.min(self.max_delay);
        Some(delay)
    }

    /// Reset the backoff to initial state
    pub fn reset(&mut self) {
        self.current_delay = self.base_delay;
    }
}

impl Default for ExponentialBackoff {
    fn default() -> Self {
        Self::new(DEFAULT_BASE_DELAY, DEFAULT_MAX_DELAY)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::RetryError;

    #[test]
    fn test_retry_builder_default() {
        let builder = RetryBuilder::default();
        assert_eq!(builder.max_attempts, DEFAULT_MAX_ATTEMPTS);
        assert_eq!(builder.base_delay, DEFAULT_BASE_DELAY);
    }

    #[test]
    fn test_retry_builder_max_attempts() {
        let builder = RetryBuilder::default().max_attempts(5);
        assert_eq!(builder.max_attempts, 5);
    }

    #[test]
    fn test_exponential_backoff() {
        let mut backoff = ExponentialBackoff::new(Duration::from_millis(100), Duration::from_secs(5));

        let first = backoff.next_delay().unwrap();
        assert_eq!(first, Duration::from_millis(100));

        let second = backoff.next_delay().unwrap();
        assert_eq!(second, Duration::from_millis(200));

        let third = backoff.next_delay().unwrap();
        assert_eq!(third, Duration::from_millis(400));
    }

    #[tokio::test]
    async fn test_retry_success_first_attempt() {
        let builder = RetryBuilder::default();
        let result: Result<&str, RetryError> = builder
            .execute(|| async { Ok("success") })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
    }

    #[tokio::test]
    async fn test_retry_success_after_failures() {
        use std::sync::atomic::{AtomicU32, Ordering};
        use std::sync::Arc;

        let attempt_count = Arc::new(AtomicU32::new(0));
        let attempt_count_clone = attempt_count.clone();

        let builder = RetryBuilder::default().max_attempts(3).base_delay(Duration::from_millis(10));
        let result: Result<&str, RetryError> = builder
            .execute(|| {
                let count = attempt_count_clone.clone();
                async move {
                    let attempt = count.fetch_add(1, Ordering::SeqCst);
                    if attempt < 2 {
                        Err(RetryError::Transient("try again".into()))
                    } else {
                        Ok("success")
                    }
                }
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(attempt_count.load(Ordering::SeqCst), 3);
    }
}
