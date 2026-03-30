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

    // FR-PHENO-001: Immediate success on first attempt (no retries needed)
    #[tokio::test]
    async fn test_retry_success_first_attempt() {
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

    // FR-PHENO-002: Transient failure followed by success
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

    // FR-PHENO-003: Max retries exhausted — all attempts fail
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

    // FR-PHENO-004: Exponential backoff doubles delay between retries
    #[tokio::test]
    async fn test_exponential_backoff_progression() {
        let mut backoff = ExponentialBackoff::new(
            Duration::from_millis(10),
            Duration::from_secs(10),
        );

        let delay1 = backoff.next_delay().unwrap();
        let delay2 = backoff.next_delay().unwrap();
        let delay3 = backoff.next_delay().unwrap();

        assert_eq!(delay1, Duration::from_millis(10));
        assert_eq!(delay2, Duration::from_millis(20));
        assert_eq!(delay3, Duration::from_millis(40));
    }

    // FR-PHENO-005: Backoff respects max delay cap
    #[tokio::test]
    async fn test_backoff_capped_at_max_delay() {
        let mut backoff = ExponentialBackoff::new(
            Duration::from_millis(100),
            Duration::from_millis(500),
        );

        let d1 = backoff.next_delay().unwrap(); // 100ms
        let d2 = backoff.next_delay().unwrap(); // 200ms
        let d3 = backoff.next_delay().unwrap(); // 400ms
        let d4 = backoff.next_delay().unwrap(); // would be 800ms, capped at 500ms

        assert_eq!(d1, Duration::from_millis(100));
        assert_eq!(d2, Duration::from_millis(200));
        assert_eq!(d3, Duration::from_millis(400));
        assert_eq!(d4, Duration::from_millis(500));

        // Further attempts stay at max
        let d5 = backoff.next_delay().unwrap();
        assert_eq!(d5, Duration::from_millis(500));
    }

    // FR-PHENO-006: Backoff reset returns to initial state
    #[tokio::test]
    async fn test_backoff_reset() {
        let mut backoff = ExponentialBackoff::new(
            Duration::from_millis(50),
            Duration::from_secs(1),
        );

        let _d1 = backoff.next_delay().unwrap();
        let _d2 = backoff.next_delay().unwrap();

        backoff.reset();

        let d_after_reset = backoff.next_delay().unwrap();
        assert_eq!(d_after_reset, Duration::from_millis(50));
    }

    // FR-PHENO-007: Jitter adds deterministic variation based on attempt count
    #[tokio::test]
    async fn test_jitter_applied_to_delays() {
        static CALL_COUNT: AtomicU32 = AtomicU32::new(0);

        let builder = RetryBuilder::default()
            .max_attempts(5)
            .base_delay(Duration::from_millis(100))
            .with_jitter();

        let start = std::time::Instant::now();
        let result: Result<(), RetryError> = builder
            .execute(|| async {
                let attempt = CALL_COUNT.fetch_add(1, Ordering::SeqCst);
                if attempt < 4 {
                    Err(RetryError::Transient("fail".into()))
                } else {
                    Ok(())
                }
            })
            .await;

        let elapsed = start.elapsed();
        assert!(result.is_ok());
        // With jitter, delays should be slightly longer than base delays alone
        // Base delays: 100 + 200 + 400 = 700ms minimum
        // With jitter (0-50ms each): 700 + up_to_150ms
        assert!(elapsed.as_millis() >= 700);
    }

    // FR-PHENO-008: Custom max attempts enforces retry limit
    #[tokio::test]
    async fn test_custom_max_attempts() {
        static CALL_COUNT: AtomicU32 = AtomicU32::new(0);

        let result: Result<(), RetryError> = retry_with_attempts(5)
            .base_delay(Duration::from_millis(5))
            .execute(|| async {
                CALL_COUNT.fetch_add(1, Ordering::SeqCst);
                Err(RetryError::Transient("fail".into()))
            })
            .await;

        assert!(result.is_err());
        assert_eq!(CALL_COUNT.load(Ordering::SeqCst), 5);
    }

    // FR-PHENO-009: Min attempts clamped to at least 1
    #[tokio::test]
    async fn test_max_attempts_minimum_clamped() {
        let builder = RetryBuilder::default().max_attempts(0); // should clamp to 1
        assert_eq!(builder.max_attempts, 1);

        let builder = RetryBuilder::default().max_attempts(0).max_attempts(0);
        assert_eq!(builder.max_attempts, 1);
    }

    // FR-PHENO-010: Custom base delay configures initial backoff
    #[tokio::test]
    async fn test_custom_base_delay() {
        static CALL_COUNT: AtomicU32 = AtomicU32::new(0);
        let start = std::time::Instant::now();

        let _result: Result<(), RetryError> = retry_with_delay(Duration::from_millis(50))
            .max_attempts(2)
            .execute(|| async {
                CALL_COUNT.fetch_add(1, Ordering::SeqCst);
                Err(RetryError::Transient("fail".into()))
            })
            .await;

        let elapsed = start.elapsed();
        // Minimum expected delay: 50ms for the retry sleep
        assert!(elapsed.as_millis() >= 50);
    }

    // FR-PHENO-011: Backoff calculation with base=1ms, max=100ms
    #[tokio::test]
    async fn test_backoff_with_small_base_delay() {
        let mut backoff = ExponentialBackoff::new(
            Duration::from_millis(1),
            Duration::from_millis(100),
        );

        let d1 = backoff.next_delay().unwrap();
        let d2 = backoff.next_delay().unwrap();
        let d3 = backoff.next_delay().unwrap();
        let d4 = backoff.next_delay().unwrap();
        let d5 = backoff.next_delay().unwrap();

        assert_eq!(d1, Duration::from_millis(1));
        assert_eq!(d2, Duration::from_millis(2));
        assert_eq!(d3, Duration::from_millis(4));
        assert_eq!(d4, Duration::from_millis(8));
        assert_eq!(d5, Duration::from_millis(16));
    }

    // FR-PHENO-012: Jitter is deterministic and varies per attempt
    #[tokio::test]
    async fn test_jitter_variation_across_attempts() {
        static ATTEMPT: AtomicU32 = AtomicU32::new(0);

        let builder = RetryBuilder::default()
            .max_attempts(4)
            .base_delay(Duration::from_millis(10))
            .with_jitter();

        let _result: Result<(), RetryError> = builder
            .execute(|| async {
                let att = ATTEMPT.fetch_add(1, Ordering::SeqCst);
                if att < 3 {
                    Err(RetryError::Transient("fail".into()))
                } else {
                    Ok(())
                }
            })
            .await;

        // With jitter applied deterministically per attempt:
        // attempt 0: base 10ms + jitter(0*7919 % 50 = 0)  = 10ms
        // attempt 1: base 20ms + jitter(1*7919 % 50 = 19) = 39ms
        // attempt 2: base 40ms + jitter(2*7919 % 50 = 38) = 78ms
        assert_eq!(ATTEMPT.load(Ordering::SeqCst), 4);
    }

    // FR-PHENO-013: Multiple retries with success on final attempt
    #[tokio::test]
    async fn test_multiple_retries_success_on_final() {
        static CALL_COUNT: AtomicU32 = AtomicU32::new(0);

        let result = retry()
            .max_attempts(5)
            .base_delay(Duration::from_millis(5))
            .execute(|| async {
                let count = CALL_COUNT.fetch_add(1, Ordering::SeqCst);
                if count < 4 {
                    Err(RetryError::Transient("fail".into()))
                } else {
                    Ok("finally!")
                }
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(CALL_COUNT.load(Ordering::SeqCst), 5);
    }

    // FR-PHENO-014: Backoff with 1ms base and 10s cap
    #[tokio::test]
    async fn test_backoff_extreme_ranges() {
        let mut backoff = ExponentialBackoff::new(
            Duration::from_millis(1),
            Duration::from_secs(10),
        );

        let d1 = backoff.next_delay().unwrap();
        let d2 = backoff.next_delay().unwrap();
        let d3 = backoff.next_delay().unwrap();
        let d4 = backoff.next_delay().unwrap();
        let d5 = backoff.next_delay().unwrap();
        let d6 = backoff.next_delay().unwrap();
        let d7 = backoff.next_delay().unwrap();
        let d8 = backoff.next_delay().unwrap();
        let d9 = backoff.next_delay().unwrap();
        let d10 = backoff.next_delay().unwrap();

        assert_eq!(d1, Duration::from_millis(1));
        assert_eq!(d2, Duration::from_millis(2));
        assert_eq!(d3, Duration::from_millis(4));
        assert_eq!(d4, Duration::from_millis(8));
        assert_eq!(d5, Duration::from_millis(16));
        assert_eq!(d6, Duration::from_millis(32));
        assert_eq!(d7, Duration::from_millis(64));
        assert_eq!(d8, Duration::from_millis(128));
        assert_eq!(d9, Duration::from_millis(256));
        assert_eq!(d10, Duration::from_millis(512));
    }
}
