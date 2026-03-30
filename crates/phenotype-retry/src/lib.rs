//! # Phenotype Retry Library
//!
//! Provides retry patterns with exponential backoff.
//!
//! ## Features
//!
//! - Exponential backoff with configurable base delay
//! - Jitter for randomization via deterministic spread
//! - Maximum retry attempts limit
//! - Error types for retry operations
//!
//! ## Usage
//!
//! ```rust,ignore
//! use phenotype_retry::retry;
//! use std::time::Duration;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let result = retry()
//!         .max_attempts(3)
//!         .base_delay(Duration::from_millis(100))
//!         .execute(|| async {
//!             // Your async operation here
//!             Ok(())
//!         })
//!         .await;
//!     Ok(result?)
//! }
//! ```

pub mod builder;
pub mod error;

pub use builder::RetryBuilder;
pub use error::RetryError;

// Re-export commonly used types
pub use std::time::Duration;

/// Default retry builder with sensible defaults
pub fn retry() -> RetryBuilder {
    RetryBuilder::default()
}

/// Create a retry builder with custom max attempts
pub fn retry_with_attempts(max_attempts: u32) -> RetryBuilder {
    RetryBuilder::default().max_attempts(max_attempts)
}

/// Create a retry builder with custom base delay
pub fn retry_with_delay(base_delay: Duration) -> RetryBuilder {
    RetryBuilder::default().base_delay(base_delay)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_retry_success() {
        let attempt_count = Arc::new(AtomicU32::new(0));
        let attempt_count_clone = attempt_count.clone();

        let result: Result<&str, RetryError> = retry()
            .max_attempts(3)
            .base_delay(Duration::from_millis(10))
            .execute(|| {
                let count = attempt_count_clone.clone();
                async move {
                    count.fetch_add(1, Ordering::SeqCst);
                    Ok::<_, RetryError>("success")
                }
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(attempt_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_retry_failure_then_success() {
        let attempt_count = Arc::new(AtomicU32::new(0));
        let attempt_count_clone = attempt_count.clone();

        let result: Result<&str, RetryError> = retry()
            .max_attempts(3)
            .base_delay(Duration::from_millis(10))
            .execute(|| {
                let count = attempt_count_clone.clone();
                async move {
                    let attempt_num = count.fetch_add(1, Ordering::SeqCst);
                    if attempt_num < 1 {
                        Err(RetryError::Transient("try again".into()))
                    } else {
                        Ok("success")
                    }
                }
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
        assert_eq!(attempt_count.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn test_retry_exhausted() {
        let attempt_count = Arc::new(AtomicU32::new(0));
        let attempt_count_clone = attempt_count.clone();

        let result: Result<&str, RetryError> = retry()
            .max_attempts(3)
            .base_delay(Duration::from_millis(10))
            .execute(|| {
                let count = attempt_count_clone.clone();
                async move {
                    count.fetch_add(1, Ordering::SeqCst);
                    Err(RetryError::Transient("always fail".into()))
                }
            })
            .await;

        assert!(result.is_err());
        assert_eq!(attempt_count.load(Ordering::SeqCst), 3);
    }
}
