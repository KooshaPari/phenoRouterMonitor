//! Async retry executor with customizable error handling.
//!
//! This module provides the main retry execution logic with support for custom
//! error predicates to determine if a specific error should trigger a retry.

use crate::policy::RetryPolicy;
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::time::Instant;
use thiserror::Error;
use tracing::{debug, warn, error as tracing_error};

/// Error type for retry operations.
#[derive(Debug, Clone, Error, Serialize, Deserialize)]
#[error("Retry failed after {attempts} attempts")]
pub struct RetryError {
    /// Number of attempts made.
    pub attempts: u32,

    /// The last error encountered.
    pub last_error: String,

    /// Whether the timeout was exceeded.
    pub timeout_exceeded: bool,
}

/// Type alias for a custom error predicate function.
///
/// Returns true if the error should trigger a retry, false otherwise.
pub type ShouldRetryFn<E> = fn(&E) -> bool;

/// Execute a closure with automatic retries according to the policy.
///
/// # Arguments
///
/// * `closure` - A closure that returns a future
/// * `policy` - Retry policy configuration
/// * `should_retry` - Optional custom predicate to determine if a specific error should be retried.
///                    If None, all errors are retried.
///
/// # Returns
///
/// * `Ok(T)` if the operation succeeds
/// * `Err(RetryError)` if all retries are exhausted or timeout is exceeded
///
/// # Examples
///
/// ```rust,ignore
/// use phenotype_retry::{execute_with_retry, RetryPolicy, Backoff};
/// use std::time::Duration;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let policy = RetryPolicy::new()
///         .with_max_attempts(3)
///         .with_backoff(Backoff::exponential(100, 2.0, Some(10_000)));
///
///     let result = execute_with_retry(
///         || async { Ok::<_, String>("success") },
///         policy,
///         None,
///     ).await?;
///
///     Ok(())
/// }
/// ```
pub async fn execute_with_retry<F, Fut, T, E>(
    mut closure: F,
    policy: RetryPolicy,
    should_retry: Option<ShouldRetryFn<E>>,
) -> Result<T, RetryError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Display + std::fmt::Debug,
{
    let start = Instant::now();
    let mut attempt = 0u32;
    let mut last_error: Option<String> = None;

    loop {
        // Check if we've exceeded the overall timeout
        if policy.is_timeout_exceeded(start.elapsed()) {
            let error_msg = last_error.unwrap_or_else(|| "timeout exceeded".to_string());
            tracing_error!("Retry timeout exceeded after {} attempts", attempt);
            return Err(RetryError {
                attempts: attempt,
                last_error: error_msg,
                timeout_exceeded: true,
            });
        }

        // Execute the closure
        match closure().await {
            Ok(result) => {
                if attempt > 0 {
                    debug!("Operation succeeded on attempt {}", attempt + 1);
                }
                return Ok(result);
            }
            Err(err) => {
                let error_msg = format!("{}", err);
                last_error = Some(error_msg.clone());

                // Check if we should retry this specific error
                let should_retry_err = should_retry.map(|f| f(&err)).unwrap_or(true);

                if !should_retry_err {
                    warn!("Error not retryable: {}", error_msg);
                    return Err(RetryError {
                        attempts: attempt + 1,
                        last_error: error_msg,
                        timeout_exceeded: false,
                    });
                }

                // Check if we've exhausted our retry attempts
                if !policy.should_retry(attempt) {
                    tracing_error!("Retry exhausted after {} attempts: {}", attempt, error_msg);
                    return Err(RetryError {
                        attempts: attempt + 1,
                        last_error: error_msg,
                        timeout_exceeded: false,
                    });
                }

                // Wait before the next retry
                let delay = policy.get_delay(attempt);
                debug!(
                    "Attempt {} failed with: {}. Waiting {:?} before retry {}",
                    attempt + 1,
                    error_msg,
                    delay,
                    attempt + 2
                );
                tokio::time::sleep(delay).await;
                attempt += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Backoff;

    #[tokio::test]
    async fn test_success_on_first_attempt() {
        let policy = RetryPolicy::new();

        let result = execute_with_retry::<_, _, _, String>(
            || async { Ok(42) },
            policy,
            None,
        )
        .await;

        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_success_after_retries() {
        let mut attempts = 0;
        let policy = RetryPolicy::new()
            .with_max_attempts(3)
            .with_backoff(Backoff::fixed(10));

        let result = execute_with_retry(
            || {
                let current = attempts;
                attempts += 1;
                async move {
                    if current < 2 {
                        Err("not yet")
                    } else {
                        Ok("success")
                    }
                }
            },
            policy,
            None,
        )
        .await;

        assert_eq!(result.unwrap(), "success");
        assert_eq!(attempts, 3); // Initial + 2 retries
    }

    #[tokio::test]
    async fn test_exhausted_retries() {
        let policy = RetryPolicy::new()
            .with_max_attempts(2)
            .with_backoff(Backoff::fixed(5));

        let result = execute_with_retry::<_, _, _, &str>(
            || async { Err("always fails") },
            policy,
            None,
        )
        .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.attempts, 3); // Initial + 2 retries
        assert!(!err.timeout_exceeded);
    }

    #[tokio::test]
    async fn test_custom_should_retry_predicate() {
        let mut attempts = 0;
        let policy = RetryPolicy::new()
            .with_max_attempts(5)
            .with_backoff(Backoff::fixed(5));

        #[derive(Debug)]
        enum MyError {
            Retryable,
            NonRetryable,
        }

        impl std::fmt::Display for MyError {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    MyError::Retryable => write!(f, "retryable"),
                    MyError::NonRetryable => write!(f, "non-retryable"),
                }
            }
        }

        let should_retry = |err: &MyError| matches!(err, MyError::Retryable);

        let result = execute_with_retry(
            || {
                let current = attempts;
                attempts += 1;
                async move {
                    if current < 1 {
                        Err(MyError::Retryable)
                    } else {
                        Err(MyError::NonRetryable)
                    }
                }
            },
            policy,
            Some(should_retry),
        )
        .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.attempts, 2); // One retry on Retryable, then fails on NonRetryable
        assert!(!err.timeout_exceeded);
    }

    #[tokio::test]
    async fn test_timeout_exceeded() {
        use std::time::Duration;

        let policy = RetryPolicy::new()
            .with_max_attempts(10)
            .with_timeout(Duration::from_millis(50))
            .with_backoff(Backoff::fixed(100)); // 100ms delays

        let result = execute_with_retry::<_, _, _, String>(
            || async {
                // This will always fail
                Err::<i32, String>("error".to_string())
            },
            policy,
            None,
        )
        .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.timeout_exceeded);
    }

    #[tokio::test]
    async fn test_retry_with_exponential_backoff() {
        let policy = RetryPolicy::new()
            .with_max_attempts(3)
            .with_backoff(Backoff::exponential(10, 2.0, None));

        let start = std::time::Instant::now();
        let result = execute_with_retry::<_, _, _, String>(
            || async { Err("fail".to_string()) },
            policy,
            None,
        )
        .await;

        assert!(result.is_err());
        let elapsed = start.elapsed();

        // Expected delays: 10ms + 20ms + 40ms = 70ms minimum
        // But we need some tolerance for execution time
        assert!(elapsed.as_millis() >= 60); // Allow some margin
    }

    #[tokio::test]
    async fn test_retry_error_message() {
        let policy = RetryPolicy::new()
            .with_max_attempts(1)
            .with_backoff(Backoff::fixed(5));

        let result = execute_with_retry::<_, _, _, String>(
            || async { Err("custom error message".to_string()) },
            policy,
            None,
        )
        .await;

        let err = result.unwrap_err();
        assert!(err.last_error.contains("custom error message"));
    }

    #[tokio::test]
    async fn test_zero_max_attempts() {
        let policy = RetryPolicy::new()
            .with_max_attempts(0)
            .with_backoff(Backoff::fixed(5));

        let result = execute_with_retry::<_, _, _, String>(
            || async { Err("error".to_string()) },
            policy,
            None,
        )
        .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.attempts, 1); // Only the initial attempt
    }

    #[tokio::test]
    async fn test_multiple_successes_use_initial_attempt() {
        let call_count = std::sync::atomic::AtomicU32::new(0);
        let policy = RetryPolicy::new();

        let result = execute_with_retry(
            || {
                let counter = &call_count;
                async move {
                    counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    Ok::<_, String>(42)
                }
            },
            policy,
            None,
        )
        .await;

        assert_eq!(result.unwrap(), 42);
        assert_eq!(
            call_count.load(std::sync::atomic::Ordering::SeqCst),
            1
        );
    }

    #[tokio::test]
    async fn test_retry_error_display() {
        let err = RetryError {
            attempts: 5,
            last_error: "test error".to_string(),
            timeout_exceeded: false,
        };

        let msg = format!("{}", err);
        assert!(msg.contains("5"));
        assert!(msg.contains("Retry failed"));
    }

    #[tokio::test]
    async fn test_complex_retry_scenario() {
        let mut call_sequence = Vec::new();
        let policy = RetryPolicy::new()
            .with_max_attempts(4)
            .with_backoff(Backoff::linear(5, 5, None));

        let result = execute_with_retry(
            || {
                let seq = &call_sequence;
                async move {
                    let current = seq.len();
                    seq.push(current);

                    if current < 2 {
                        Err("transient error")
                    } else if current == 2 {
                        Err("different error")
                    } else {
                        Ok("success")
                    }
                }
            },
            policy,
            None,
        )
        .await;

        assert_eq!(result.unwrap(), "success");
        assert_eq!(call_sequence.len(), 4); // 0, 1, 2, 3
    }

    #[tokio::test]
    async fn test_should_retry_all_errors_by_default() {
        let mut count = 0;
        let policy = RetryPolicy::new()
            .with_max_attempts(3)
            .with_backoff(Backoff::fixed(5));

        let result = execute_with_retry(
            || {
                let c = count;
                count += 1;
                async move {
                    Err::<(), &str>(if c == 0 { "a" } else { "b" })
                }
            },
            policy,
            None,
        )
        .await;

        assert!(result.is_err());
        assert_eq!(count, 4); // Initial + 3 retries
    }
}
