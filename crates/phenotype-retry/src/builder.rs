//! Retry builder for configuring retry behavior.

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

use pin_project::pin_project;
use tenacity::{Backoff, RetryForError, SystemClock};

use crate::error::RetryError;
use crate::RetryBuilder;

/// Maximum number of retry attempts
const DEFAULT_MAX_ATTEMPTS: u32 = 3;

/// Base delay between retries
const DEFAULT_BASE_DELAY: Duration = Duration::from_millis(100);

/// Maximum delay between retries
const DEFAULT_MAX_DELAY: Duration = Duration::from_secs(30);

/// Builder for configuring retry behavior
#[derive(Default, Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub jitter: bool,
    pub timeout: Option<Duration>,
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
    pub async fn execute<F, Fut, T, E>(&self, f: F) -> Result<T, E>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<T, E>>,
        E: std::error::Error + Send + Sync + 'static,
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
