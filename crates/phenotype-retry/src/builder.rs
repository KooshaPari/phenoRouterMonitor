//! Retry builder for configuring retry behavior.

use std::future::Future;
use std::time::Duration;

use crate::error::RetryError;

/// Maximum number of retry attempts
const DEFAULT_MAX_ATTEMPTS: u32 = 3;

/// Base delay between retries
const DEFAULT_BASE_DELAY: Duration = Duration::from_millis(100);

/// Maximum delay between retries
const DEFAULT_MAX_DELAY: Duration = Duration::from_secs(30);

/// Multiplier for exponential backoff
const DEFAULT_MULTIPLIER: f64 = 2.0;

/// Builder for configuring retry behavior
#[derive(Default, Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub jitter: bool,
    pub timeout: Option<Duration>,
    pub multiplier: f64,
}

impl RetryConfig {
    /// Create a new retry config with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set maximum number of retry attempts
    pub fn max_attempts(mut self, attempts: u32) -> Self {
        self.max_attempts = attempts;
        self
    }

    /// Set base delay between retries
    pub fn base_delay(mut self, delay: Duration) -> Self {
        self.base_delay = delay;
        self
    }

    /// Set maximum delay between retries
    pub fn max_delay(mut self, delay: Duration) -> Self {
        self.max_delay = delay;
        self
    }

    /// Enable jitter for randomization
    pub fn with_jitter(mut self) -> Self {
        self.jitter = true;
        self
    }

    /// Set timeout for the entire operation
    pub fn timeout(mut self, duration: Duration) -> Self {
        self.timeout = Some(duration);
        self
    }

    /// Execute an async operation with retry logic
    pub async fn execute<F, Fut, T, E>(&self, mut f: F) -> Result<T, RetryError>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<T, RetryError>>,
    {
        let mut backoff = ExponentialBackoff::new(self.base_delay, self.max_delay, self.multiplier);

        for attempt in 0..self.max_attempts {
            match f().await {
                Ok(result) => return Ok(result),
                Err(e) if attempt == self.max_attempts - 1 => return Err(e),
                Err(_) => {
                    if let Some(delay) = backoff.next_delay() {
                        if self.jitter {
                            let jitter = Duration::from_millis(rand::random::<u64>() % 50);
                            tokio::time::sleep(delay + jitter).await;
                        } else {
                            tokio::time::sleep(delay).await;
                        }
                    }
                }
            }
        }

        Err(RetryError::MaxAttemptsExceeded)
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
    pub fn new(base_delay: Duration, max_delay: Duration, multiplier: f64) -> Self {
        Self {
            current_delay: base_delay,
            base_delay,
            max_delay,
            multiplier,
        }
    }

    /// Get the next delay value
    pub fn next_delay(&mut self) -> Option<Duration> {
        if self.current_delay > self.max_delay {
            return None;
        }

        let delay = self.current_delay;
        let next = (self.current_delay.as_millis() as f64 * self.multiplier) as u64;
        self.current_delay = Duration::from_millis(next).min(self.max_delay);
        Some(delay)
    }

    /// Reset the backoff to initial state
    pub fn reset(&mut self) {
        self.current_delay = self.base_delay;
    }
}

impl Default for ExponentialBackoff {
    fn default() -> Self {
        Self::new(DEFAULT_BASE_DELAY, DEFAULT_MAX_DELAY, DEFAULT_MULTIPLIER)
    }
}
