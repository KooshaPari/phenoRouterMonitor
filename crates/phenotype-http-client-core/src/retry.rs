//! HTTP retry policy with exponential backoff.

use crate::error::TransportError;
use std::time::Duration;

/// Retry policy configuration.
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub backoff_multiplier: f64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
        }
    }
}

impl RetryPolicy {
    /// Calculate the delay for the given attempt number (0-indexed).
    pub fn delay_for(&self, attempt: u32) -> Duration {
        let base = self.initial_delay.mul_f64(self.backoff_multiplier.powi(attempt as i32));
        base.min(self.max_delay)
    }

    /// Check if we should retry the given error at the given attempt.
    pub fn should_retry(&self, error: &TransportError, attempt: u32) -> bool {
        if attempt >= self.max_attempts {
            return false;
        }
        error.is_retryable()
    }
}

/// Execute a closure with retry logic.
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
                tracing::debug!(attempt, ?delay, %error, "retrying request");
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
    fn default_policy() {
        let policy = RetryPolicy::default();
        assert_eq!(policy.max_attempts, 3);
    }

    #[test]
    fn delay_calculation() {
        let policy = RetryPolicy {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
        };
        assert_eq!(policy.delay_for(0), Duration::from_millis(100));
        assert_eq!(policy.delay_for(1), Duration::from_millis(200));
        assert_eq!(policy.delay_for(2), Duration::from_millis(400));
    }

    #[test]
    fn should_retry_retryable() {
        let policy = RetryPolicy::default();
        let err = TransportError::Timeout("timeout".into());
        assert!(policy.should_retry(&err, 0));
        assert!(!policy.should_retry(&err, 3));
    }

    #[test]
    fn should_not_retry_non_retryable() {
        let policy = RetryPolicy::default();
        let err = TransportError::NotFound("missing".into());
        assert!(!policy.should_retry(&err, 0));
    }
}
