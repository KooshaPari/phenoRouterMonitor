//! Retry policies and strategies
//!
//! Provides automatic retry policy detection and execution.

use std::time::Duration;

/// Retry policy configuration
#[derive(Debug, Clone, Copy)]
pub enum RetryPolicy {
    /// No retry
    None,
    /// Fixed delay between retries
    Fixed {
        /// Maximum number of attempts
        max_attempts: u32,
        /// Delay between attempts
        delay: Duration,
    },
    /// Exponential backoff with jitter
    Exponential {
        /// Maximum number of attempts
        max_attempts: u32,
        /// Base delay (doubles each attempt)
        base: Duration,
        /// Maximum delay cap
        max_delay: Duration,
    },
    /// Linear backoff
    Linear {
        /// Maximum number of attempts
        max_attempts: u32,
        /// Step increment
        step: Duration,
    },
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self::None
    }
}

impl RetryPolicy {
    /// Get the maximum number of attempts
    pub fn max_attempts(&self) -> u32 {
        match self {
            Self::None => 1,
            Self::Fixed { max_attempts, .. } => *max_attempts,
            Self::Exponential { max_attempts, .. } => *max_attempts,
            Self::Linear { max_attempts, .. } => *max_attempts,
        }
    }
    
    /// Calculate delay for a given attempt (1-indexed)
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        match self {
            Self::None => Duration::ZERO,
            Self::Fixed { delay, .. } => *delay,
            Self::Exponential { base, max_delay, .. } => {
                let delay = Duration::from_secs(2_u64.saturating_pow(attempt - 1)) * *base;
                std::cmp::min(delay, *max_delay)
            }
            Self::Linear { step, .. } => *step * attempt,
        }
    }
}

/// Trait for types that can be retried
pub trait Retryable: std::fmt::Debug + Send + Sync {
    /// Get the retry policy for this error
    fn retry_policy(&self) -> RetryPolicy;
    
    /// Check if this error should be retried
    fn is_retryable(&self) -> bool {
        !matches!(self.retry_policy(), RetryPolicy::None)
    }
}

/// Retry strategy for executing operations with automatic retry
#[derive(Debug, Clone)]
pub struct RetryStrategy {
    policy: RetryPolicy,
}

impl RetryStrategy {
    /// Create a new retry strategy
    pub fn new(policy: RetryPolicy) -> Self {
        Self { policy }
    }
    
    /// Execute an operation with retries
    pub async fn execute<F, Fut, T, E>(&self, op: F) -> Result<T, E>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
        E: Retryable,
    {
        let max_attempts = self.policy.max_attempts();
        
        for attempt in 1..=max_attempts {
            match op().await {
                Ok(result) => return Ok(result),
                Err(error) if attempt < max_attempts && error.is_retryable() => {
                    let delay = self.policy.delay_for_attempt(attempt);
                    tokio::time::sleep(delay).await;
                }
                Err(error) => return Err(error),
            }
        }
        
        unreachable!("loop should have returned in last iteration")
    }
}

impl Default for RetryStrategy {
    fn default() -> Self {
        Self::new(RetryPolicy::None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fixed_policy() {
        let policy = RetryPolicy::Fixed {
            max_attempts: 3,
            delay: Duration::from_millis(100),
        };
        
        assert_eq!(policy.max_attempts(), 3);
        assert_eq!(policy.delay_for_attempt(1), Duration::from_millis(100));
        assert_eq!(policy.delay_for_attempt(2), Duration::from_millis(100));
    }
    
    #[test]
    fn test_exponential_policy() {
        let policy = RetryPolicy::Exponential {
            max_attempts: 3,
            base: Duration::from_millis(100),
            max_delay: Duration::from_secs(1),
        };
        
        assert_eq!(policy.max_attempts(), 3);
        assert_eq!(policy.delay_for_attempt(1), Duration::from_millis(100));
        assert_eq!(policy.delay_for_attempt(2), Duration::from_millis(200));
        assert_eq!(policy.delay_for_attempt(3), Duration::from_millis(400));
    }
}
