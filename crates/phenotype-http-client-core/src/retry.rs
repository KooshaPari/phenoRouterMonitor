//! HTTP retry policy with exponential backoff.

use std::time::Duration;
use crate::error::TransportError;

/// Retry policy configuration.
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Maximum number of retries.
    pub max_retries: u32,
    /// Initial delay between retries.
    pub initial_delay: Duration,
    /// Maximum delay between retries.
    pub max_delay: Duration,
    /// Multiplier for exponential backoff.
    pub backoff_multiplier: f64,
    /// Jitter factor (0.0 to 1.0).
    pub jitter: f64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            jitter: 0.1,
        }
    }
}

impl RetryPolicy {
    /// Create a new retry policy with custom configuration.
    pub fn new(
        max_retries: u32,
        initial_delay: Duration,
        max_delay: Duration,
        backoff_multiplier: f64,
        jitter: f64,
    ) -> Self {
        Self {
            max_retries,
            initial_delay,
            max_delay,
            backoff_multiplier,
            jitter,
        }
    }

    /// Calculate the delay for the given attempt number.
    pub fn delay_for(&self, attempt: u32) -> Duration {
        let base_delay = self.initial_delay * self.backoff_multiplier.powi(attempt as i32 - 1);
        let capped_delay = base_delay.min(self.max_delay);

        // Add jitter
        let jitter_range = capped_delay.mul_f64(self.jitter);
        let jitter_ns = (rand_ratio() * jitter_range.as_nanos() as f64) as u64;

        capped_delay + Duration::from_nanos(jitter_ns)
    }

    /// Check if we should retry the given error.
    pub fn should_retry(&self, error: &TransportError, attempt: u32) -> bool {
        if attempt >= self.max_retries {
            return false;
        }

        error.is_retryable()
    }
}

/// Simple pseudo-random number generator for jitter.
fn rand_ratio() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    (nanos % 1000) as f64 / 1000.0
}

/// Retry future with policy.
pub async fn retry_with_policy<F, Fut, T>(
    policy: &RetryPolicy,
    mut f: F,
) -> Result<T, TransportError>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, TransportError>>,
{
    let mut attempt = 0u32;

    loop {
        match f().await {
            Ok(result) => return Ok(result),
            Err(error) => {
                if !policy.should_retry(&error, attempt) {
                    return Err(error);
                }

                let delay = policy.delay_for(attempt);
                tracing::debug!("retry attempt {} after {:?}: {}", attempt, delay, error);
                tokio::time::sleep(delay).await;
                attempt += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_policy_default() {
        let policy = RetryPolicy::default();
        assert_eq!(policy.max_retries, 3);
    }

    #[test]
    fn test_delay_calculation() {
        let policy = RetryPolicy::new(3, Duration::from_millis(100), Duration::from_secs(30), 2.0, 0.0);

        // Attempt 0: 100ms
        let delay0 = policy.delay_for(0);
        assert_eq!(delay0, Duration::from_millis(100));

        // Attempt 1: 200ms
        let delay1 = policy.delay_for(1);
        assert_eq!(delay1, Duration::from_millis(200));

        // Attempt 2: 400ms
        let delay2 = policy.delay_for(2);
        assert_eq!(delay2, Duration::from_millis(400));
    }

    #[test]
    fn test_should_retry() {
        let policy = RetryPolicy::default();

        // Timeout is retryable
        let timeout = TransportError::Timeout("timeout".into());
        assert!(policy.should_retry(&timeout, 0));
        assert!(policy.should_retry(&timeout, 1));
        assert!(policy.should_retry(&timeout, 2));

        // But not after max retries
        assert!(!policy.should_retry(&timeout, 3));

        // Validation error is not retryable
        let validation = TransportError::Validation("invalid".into());
        assert!(!policy.should_retry(&validation, 0));
    }
}
