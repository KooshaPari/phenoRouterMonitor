//! HTTP retry policy with exponential backoff and jitter.

use crate::error::TransportError;
use std::time::Duration;

/// Retry policy configuration.
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub backoff_multiplier: f64,
    pub use_jitter: bool,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            use_jitter: true,
        }
    }
}

impl RetryPolicy {
    /// Calculate the delay for the given attempt number (0-indexed) with optional jitter.
    pub fn delay_for(&self, attempt: u32) -> Duration {
        // Exponential backoff: initial_delay * multiplier^attempt
        let base_millis = self
            .initial_delay
            .as_millis() as f64
            * self.backoff_multiplier.powi(attempt as i32);
        
        let capped_millis = base_millis.min(self.max_delay.as_millis() as f64);
        
        // Apply jitter (±25% of calculated delay) if enabled
        let final_millis = if self.use_jitter {
            let jitter_factor = 0.75 + (random_fraction() * 0.5);
            capped_millis * jitter_factor
        } else {
            capped_millis
        };

        Duration::from_millis(final_millis as u64)
    }

    /// Check if we should retry the given error at the given attempt.
    pub fn should_retry(&self, error: &TransportError, attempt: u32) -> bool {
        if attempt >= self.max_attempts {
            return false;
        }
        error.is_retryable()
    }
}

/// Generate a pseudo-random fraction in [0.0, 1.0).
/// Uses a simple deterministic approach for testability.
#[inline]
fn random_fraction() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos() as f64;
    
    (nanos % 1_000_000_000.0) / 1_000_000_000.0
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
        assert!(policy.use_jitter);
    }

    #[test]
    fn delay_calculation_exponential() {
        let policy = RetryPolicy {
            max_attempts: 5,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            use_jitter: false,
        };
        
        // Attempt 0: 100ms * 2^0 = 100ms
        let delay_0 = policy.delay_for(0);
        assert_eq!(delay_0, Duration::from_millis(100));
        
        // Attempt 1: 100ms * 2^1 = 200ms
        let delay_1 = policy.delay_for(1);
        assert_eq!(delay_1, Duration::from_millis(200));
        
        // Attempt 2: 100ms * 2^2 = 400ms
        let delay_2 = policy.delay_for(2);
        assert_eq!(delay_2, Duration::from_millis(400));
        
        // Attempt 3: 100ms * 2^3 = 800ms
        let delay_3 = policy.delay_for(3);
        assert_eq!(delay_3, Duration::from_millis(800));
        
        // Attempt 4: 100ms * 2^4 = 1600ms
        let delay_4 = policy.delay_for(4);
        assert_eq!(delay_4, Duration::from_millis(1600));
    }

    #[test]
    fn delay_calculation_capped_by_max() {
        let policy = RetryPolicy {
            max_attempts: 10,
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 2.0,
            use_jitter: false,
        };
        
        // Attempt 0: 1s
        assert_eq!(policy.delay_for(0), Duration::from_secs(1));
        
        // Attempt 3: 1s * 2^3 = 8s
        assert_eq!(policy.delay_for(3), Duration::from_secs(8));
        
        // Attempt 4: 1s * 2^4 = 16s, but capped at 10s
        assert_eq!(policy.delay_for(4), Duration::from_secs(10));
        
        // Attempt 10: still capped at 10s
        assert_eq!(policy.delay_for(10), Duration::from_secs(10));
    }

    #[test]
    fn jitter_applies_variation() {
        let policy = RetryPolicy {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            use_jitter: true,
        };
        
        // Get multiple samples and verify they vary (with high probability)
        let mut delays = Vec::new();
        for _ in 0..10 {
            delays.push(policy.delay_for(0));
        }
        
        // Check that not all delays are identical (jitter is working)
        let unique_delays: std::collections::HashSet<_> = delays.iter().collect();
        assert!(
            unique_delays.len() > 1,
            "jitter should produce varying delays"
        );
        
        // All delays should be within ±25% of 100ms
        for delay in delays {
            let millis = delay.as_millis() as f64;
            assert!(
                millis >= 75.0 && millis <= 125.0,
                "delay {} outside jitter range [75, 125]",
                millis
            );
        }
    }

    #[test]
    fn should_retry_retryable() {
        let policy = RetryPolicy::default();
        let err = TransportError::Timeout("timeout".into());
        assert!(policy.should_retry(&err, 0));
        assert!(policy.should_retry(&err, 1));
        assert!(policy.should_retry(&err, 2));
        assert!(!policy.should_retry(&err, 3)); // Exceeds max_attempts
    }

    #[test]
    fn should_not_retry_non_retryable() {
        let policy = RetryPolicy::default();
        let err = TransportError::NotFound("missing".into());
        assert!(!policy.should_retry(&err, 0));
    }

    #[test]
    fn should_not_retry_auth_errors() {
        let policy = RetryPolicy::default();
        let auth_err = TransportError::Auth("invalid".into());
        assert!(!policy.should_retry(&auth_err, 0));
        
        let auth_err2 = TransportError::Authentication("denied".into());
        assert!(!policy.should_retry(&auth_err2, 0));
    }

    #[test]
    fn max_delay_respected() {
        let policy = RetryPolicy {
            max_attempts: 100,
            initial_delay: Duration::from_millis(1),
            max_delay: Duration::from_millis(500),
            backoff_multiplier: 10.0,
            use_jitter: false,
        };
        
        // Even with huge multiplier, delay should be capped
        for attempt in 0..100 {
            let delay = policy.delay_for(attempt);
            assert!(
                delay <= Duration::from_millis(500),
                "delay {} exceeds max at attempt {}",
                delay.as_millis(),
                attempt
            );
        }
    }
}
