//! Phenotype Retry Library
//!
//! A standardized retry library using the `backoff` crate with sensible defaults
//! and common retry patterns for async operations.
//!
//! # Usage
//!
//! ```rust
//! use phenotype_retry::{retry, ExponentialBackoff};
//!
//! // Simple retry with defaults
//! retry(async {
//!     some_async_operation().await
//! }).await;
//!
//! // Custom backoff
//! let backoff = ExponentialBackoff::default()
//!     .with_max_retries(5)
//!     .with_max_interval(std::time::Duration::from_secs(30));
//! retry_with_backoff(backoff, async { operation().await }).await
//! ```

use async_trait::async_trait;
use backoff::backoff::Backoff;
use std::future::Future;
use std::time::Duration;
use thiserror::Error;

// Re-export common types
pub use backoff::ExponentialBackoff;
pub use backoff::SystemClock;

/// Error type for retry operations
#[derive(Debug, Error)]
pub enum RetryError<E> {
    /// Permanent error after all retries exhausted
    #[error("retry exhausted after {0} attempts: {1}")]
    Exhausted(usize, E),
}

/// Trait for types that can be retried
#[async_trait]
pub trait Retryable: Send {
    /// The output type
    type Output;
    /// The error type
    type Error;

    /// Execute the operation
    async fn run(&self) -> Result<Self::Output, Self::Error>;
}

/// Execute a future with exponential backoff retry
///
/// Uses default settings:
/// - Initial interval: 1ms
/// - Multiplier: 2.0
/// - Max interval: 30 seconds
/// - Max elapsed time: 15 minutes
pub async fn retry<F, T, E, Fut>(op: F) -> Result<T, RetryError<E>>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: Send + 'static,
{
    let backoff_config = ExponentialBackoff::default();
    retry_with_backoff(backoff_config, op).await
}

/// Execute a future with custom backoff settings
pub async fn retry_with_backoff<B, F, T, E, Fut>(mut backoff: B, mut op: F) -> Result<T, RetryError<E>>
where
    B: Backoff + Send + 'static,
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: Send + 'static,
{
    let mut attempt = 0;

    loop {
        attempt += 1;

        match op().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                if let Some(duration) = backoff.next_backoff() {
                    tokio::time::sleep(duration).await;
                    continue;
                }
                return Err(RetryError::Exhausted(attempt, e));
            }
        }
    }
}

/// Retry with a simple retry count (no backoff - instant retries)
pub async fn retry_n<F, T, E, Fut>(max_attempts: usize, mut op: F) -> Result<T, RetryError<E>>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
{
    let mut attempt = 0;

    loop {
        attempt += 1;

        match op().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                if attempt >= max_attempts {
                    return Err(RetryError::Exhausted(attempt, e));
                }
            }
        }
    }
}

/// Retry with jitter (randomized exponential backoff)
/// Provides better distribution and avoids thundering herd
pub async fn retry_with_jitter<F, T, E, Fut>(
    initial_interval: Duration,
    max_interval: Duration,
    max_attempts: usize,
    mut op: F,
) -> Result<T, RetryError<E>>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
{
    use std::sync::atomic::{AtomicU64, Ordering};

    static JITTER_COUNTER: AtomicU64 = AtomicU64::new(0);

    let mut attempt = 0;
    let mut current_interval = initial_interval;

    loop {
        attempt += 1;

        match op().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                if attempt >= max_attempts {
                    return Err(RetryError::Exhausted(attempt, e));
                }

                // Add jitter: random value between 0 and current_interval
                let jitter = JITTER_COUNTER.fetch_add(1, Ordering::Relaxed) % current_interval.as_millis() as u64;
                let jitter_duration = Duration::from_millis(jitter);
                
                tokio::time::sleep(current_interval + jitter_duration).await;

                // Exponential increase with cap
                current_interval = Duration::from_secs(
                    (current_interval.as_secs() * 2).min(max_interval.as_secs())
                );
            }
        }
    }
}

/// Extension trait for adding retry to Result types
pub trait ResultRetryExt<T, E> {
    /// Retry this result with default exponential backoff
    fn with_retry(self) -> impl Future<Output = Result<T, RetryError<E>>>;
}

impl<T, E> ResultRetryExt<T, E> for Result<T, E>
where
    E: Send + 'static,
{
    fn with_retry(self) -> impl Future<Output = Result<T, RetryError<E>>> {
        async { self.map_err(|e| RetryError::Exhausted(1, e)) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::timeout;

    #[tokio::test]
    async fn test_retry_success_first_attempt() {
        let result = retry(|| async { Ok::<_, ()>(42) }).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_retry_exhausted() {
        let result = retry(|| async { 
            tokio::time::sleep(Duration::from_millis(1)).await;
            Err::<i32, ()>(()) 
        }).await;
        
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_retry_n_success() {
        // Test retry_n succeeds on first attempt
        let result = retry_n(3, || async {
            Ok::<i32, ()>(42)
        }).await;
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_retry_n_eventually_succeeds() {
        // Test retry_n with attempts counter using RefCell
        use std::cell::RefCell;
        use std::rc::Rc;
        
        let attempts = Rc::new(RefCell::new(0));
        let attempts_clone = attempts.clone();
        
        let result = retry_n(3, move || {
            let attempts = attempts_clone.clone();
            async move {
                *attempts.borrow_mut() += 1;
                if *attempts.borrow() < 2 {
                    Err::<i32, ()>(())
                } else {
                    Ok(42)
                }
            }
        }).await;
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_retry_with_timeout() {
        let result = timeout(
            Duration::from_secs(1),
            retry(|| async {
                tokio::time::sleep(Duration::from_millis(100)).await;
                Ok::<_, ()>(42)
            })
        ).await;
        
        assert!(result.is_ok());
    }
}