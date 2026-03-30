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

    /// Run an async operation with retries and exponential backoff.
    pub async fn execute<F, Fut, T, E>(&self, mut f: F) -> Result<T, E>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<T, E>>,
        E: std::error::Error + Send + Sync + 'static,
    {
        let mut backoff = ExponentialBackoff::new(self.base_delay, self.max_delay);

        for attempt in 0..self.max_attempts {
            match f().await {
                Ok(result) => return Ok(result),
                Err(e) if attempt + 1 >= self.max_attempts => return Err(e),
                Err(_) => {
                    if let Some(mut delay) = backoff.next_delay() {
                        if self.jitter {
                            let extra_ms = (attempt.wrapping_mul(7919) % 50) as u64;
                            delay += Duration::from_millis(extra_ms);
                        }
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        unreachable!("loop returns or errors within max_attempts")
    }
}

/// Exponential backoff calculator.
#[derive(Debug, Clone)]
pub struct ExponentialBackoff {
    current_delay: Duration,
    base_delay: Duration,
    max_delay: Duration,
    multiplier: f64,
}

impl ExponentialBackoff {
    /// Create a new backoff calculator.
    pub fn new(base_delay: Duration, max_delay: Duration) -> Self {
        Self {
            current_delay: base_delay,
            base_delay,
            max_delay,
            multiplier: 2.0,
        }
    }

    /// Next delay before the following retry attempt.
    pub fn next_delay(&mut self) -> Option<Duration> {
        let delay = self.current_delay;
        let next_ms = (self.current_delay.as_millis() as f64 * self.multiplier) as u64;
        self.current_delay = Duration::from_millis(next_ms).min(self.max_delay);
        if delay > self.max_delay {
            return None;
        }
        Some(delay)
    }

    /// Reset the backoff to its initial state.
    pub fn reset(&mut self) {
        self.current_delay = self.base_delay;
    }
}

impl Default for ExponentialBackoff {
    fn default() -> Self {
        Self::new(DEFAULT_BASE_DELAY, DEFAULT_MAX_DELAY)
    }
}
