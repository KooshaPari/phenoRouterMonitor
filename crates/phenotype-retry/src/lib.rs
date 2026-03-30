//! # Phenotype Retry Library
//!
//! Async retry helpers with exponential backoff (Tokio).

pub mod builder;
pub mod error;

pub use builder::{ExponentialBackoff, RetryBuilder};
pub use error::RetryError;

/// Backwards-compatible alias for [`ExponentialBackoff`].
pub type Backoff = ExponentialBackoff;

pub use std::time::Duration;

/// Default retry builder with sensible defaults.
pub fn retry() -> RetryBuilder {
    RetryBuilder::default()
}

/// Create a retry builder with custom max attempts.
pub fn retry_with_attempts(max_attempts: u32) -> RetryBuilder {
    RetryBuilder::default().max_attempts(max_attempts)
}

/// Create a retry builder with custom base delay.
pub fn retry_with_delay(base_delay: Duration) -> RetryBuilder {
    RetryBuilder::default().base_delay(base_delay)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[tokio::test]
    async fn test_retry_success() {
        static CALL_COUNT: AtomicU32 = AtomicU32::new(0);

        let result = retry()
            .max_attempts(3)
            .base_delay(Duration::from_millis(10))
            .execute(|| async {
                CALL_COUNT.fetch_add(1, Ordering::SeqCst);
                Ok::<_, RetryError>("success")
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(CALL_COUNT.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_retry_failure_then_success() {
        static CALL_COUNT: AtomicU32 = AtomicU32::new(0);

        let result = retry()
            .max_attempts(3)
            .base_delay(Duration::from_millis(10))
            .execute(|| async {
                let count = CALL_COUNT.fetch_add(1, Ordering::SeqCst);
                if count < 1 {
                    Err(RetryError::Transient("try again".into()))
                } else {
                    Ok("success")
                }
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
        assert_eq!(CALL_COUNT.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn test_retry_exhausted() {
        static CALL_COUNT: AtomicU32 = AtomicU32::new(0);

        let result: Result<(), RetryError> = retry()
            .max_attempts(3)
            .base_delay(Duration::from_millis(10))
            .execute(|| async {
                CALL_COUNT.fetch_add(1, Ordering::SeqCst);
                Err(RetryError::Transient("always fail".into()))
            })
            .await;

        assert!(result.is_err());
        assert_eq!(CALL_COUNT.load(Ordering::SeqCst), 3);
    }
}
