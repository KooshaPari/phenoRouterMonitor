//! Comprehensive test suite for phenotype-retry.
//!
//! This module contains integration tests covering all backoff strategies,
//! retry policies, and executor functionality.

use crate::{
    backoff::Backoff, executor::execute_with_retry, policy::RetryPolicy, RetryError,
};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

// ============================================================================
// Backoff Strategy Tests
// ============================================================================

#[test]
fn test_all_backoff_variants_implement_delay_ms() {
    let fixed = Backoff::fixed(100);
    assert_eq!(fixed.delay_ms(0), 100);

    let exp = Backoff::exponential(100, 2.0, None);
    assert_eq!(exp.delay_ms(0), 100);

    let lin = Backoff::linear(100, 50, None);
    assert_eq!(lin.delay_ms(0), 100);

    let fib = Backoff::fibonacci(100, None);
    assert_eq!(fib.delay_ms(0), 100);
}

#[test]
fn test_backoff_max_delay_consistency() {
    // Fixed should report consistent max delay
    let fixed = Backoff::fixed(500);
    for i in 0..10 {
        assert_eq!(fixed.delay_ms(i), 500);
    }

    // Exponential with cap should respect cap
    let exp = Backoff::exponential(100, 2.0, Some(1000));
    assert!(exp.delay_ms(10) <= 1000);

    // Linear with cap should respect cap
    let lin = Backoff::linear(100, 100, Some(500));
    assert!(lin.delay_ms(10) <= 500);

    // Fibonacci with cap should respect cap
    let fib = Backoff::fibonacci(100, Some(800));
    assert!(fib.delay_ms(10) <= 800);
}

#[test]
fn test_exponential_growth_vs_linear() {
    let exp = Backoff::exponential(10, 2.0, None);
    let lin = Backoff::linear(10, 20, None);

    // After a few attempts, exponential should be much larger
    let exp_10 = exp.delay_ms(5); // 10 * 2^5 = 320
    let lin_10 = lin.delay_ms(5); // 10 + 20*5 = 110

    assert!(exp_10 > lin_10);
}

#[test]
fn test_fibonacci_growth_pattern() {
    let fib = Backoff::fibonacci(10, None);

    let mut prev = fib.delay_ms(0);
    for i in 1..20 {
        let curr = fib.delay_ms(i);
        // Each value should be >= previous (monotonic for Fibonacci)
        assert!(curr >= prev);
        prev = curr;
    }
}

// ============================================================================
// Policy Tests
// ============================================================================

#[test]
fn test_policy_builder_pattern() {
    let policy = RetryPolicy::new()
        .with_max_attempts(5)
        .with_timeout(Duration::from_secs(30))
        .with_backoff(Backoff::exponential(100, 2.0, Some(5000)))
        .with_jitter(true, 0.15);

    assert_eq!(policy.max_attempts, 5);
    assert_eq!(policy.timeout, Some(Duration::from_secs(30)));
    assert!(policy.use_jitter);
    assert_eq!(policy.jitter_factor, 0.15);
}

#[test]
fn test_policy_should_retry_boundary() {
    let policy = RetryPolicy::new().with_max_attempts(3);

    // Should retry attempts 0, 1, 2
    for i in 0..3 {
        assert!(policy.should_retry(i));
    }
    // Should not retry attempt 3 (exhausted)
    assert!(!policy.should_retry(3));
    assert!(!policy.should_retry(100));
}

#[test]
fn test_policy_timeout_edge_cases() {
    let policy = RetryPolicy::new().with_timeout(Duration::from_millis(100));

    assert!(!policy.is_timeout_exceeded(Duration::from_millis(0)));
    assert!(!policy.is_timeout_exceeded(Duration::from_millis(99)));
    assert!(!policy.is_timeout_exceeded(Duration::from_millis(100)));
    assert!(policy.is_timeout_exceeded(Duration::from_millis(101)));
}

#[test]
fn test_policy_no_timeout() {
    let policy = RetryPolicy::new();

    // Should never timeout if not set
    assert!(!policy.is_timeout_exceeded(Duration::from_secs(1000)));
    assert!(!policy.is_timeout_exceeded(Duration::from_secs(999999)));
}

#[test]
fn test_jitter_variance() {
    let policy = RetryPolicy::new()
        .with_backoff(Backoff::fixed(1000))
        .with_jitter(true, 0.2);

    let mut delays = Vec::new();
    for _ in 0..30 {
        let delay_ms = policy.get_delay(0).as_millis() as u64;
        delays.push(delay_ms);
        // Should be in [800, 1200] with 20% jitter
        assert!(delay_ms >= 800 && delay_ms <= 1200);
    }

    // Check that we have variance
    let min = *delays.iter().min().unwrap();
    let max = *delays.iter().max().unwrap();
    assert!(max > min); // Should not all be the same
}

#[test]
fn test_no_jitter() {
    let policy = RetryPolicy::new()
        .with_backoff(Backoff::fixed(500))
        .with_jitter(false, 0.5);

    let delay1 = policy.get_delay(0);
    let delay2 = policy.get_delay(0);
    let delay3 = policy.get_delay(0);

    assert_eq!(delay1, delay2);
    assert_eq!(delay2, delay3);
    assert_eq!(delay1.as_millis(), 500);
}

// ============================================================================
// Executor Tests
// ============================================================================

#[tokio::test]
async fn test_success_immediate() {
    let policy = RetryPolicy::new();

    let result = execute_with_retry::<_, _, _, String>(
        || async { Ok(100) },
        policy,
        None,
    )
    .await;

    assert_eq!(result.unwrap(), 100);
}

#[tokio::test]
async fn test_retry_on_error() {
    let counter = Arc::new(AtomicU32::new(0));
    let counter_clone = Arc::clone(&counter);

    let policy = RetryPolicy::new()
        .with_max_attempts(3)
        .with_backoff(Backoff::fixed(10));

    let result = execute_with_retry(
        move || {
            let count = Arc::clone(&counter_clone);
            async move {
                let current = count.fetch_add(1, Ordering::SeqCst);
                if current < 2 {
                    Err::<String, _>("error")
                } else {
                    Ok("success".to_string())
                }
            }
        },
        policy,
        None,
    )
    .await;

    assert_eq!(result.unwrap(), "success");
    assert_eq!(counter.load(Ordering::SeqCst), 3);
}

#[tokio::test]
async fn test_error_exhaustion() {
    let policy = RetryPolicy::new()
        .with_max_attempts(2)
        .with_backoff(Backoff::fixed(5));

    let result = execute_with_retry::<_, _, _, String>(
        || async { Err("permanent error".to_string()) },
        policy,
        None,
    )
    .await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.attempts, 3); // Initial + 2 retries
    assert!(!err.timeout_exceeded);
    assert!(err.last_error.contains("permanent error"));
}

#[tokio::test]
async fn test_custom_should_retry_predicate() {
    #[derive(Debug)]
    enum CustomError {
        Transient,
        Permanent,
    }

    impl std::fmt::Display for CustomError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                CustomError::Transient => write!(f, "transient"),
                CustomError::Permanent => write!(f, "permanent"),
            }
        }
    }

    let counter = Arc::new(AtomicU32::new(0));
    let counter_clone = Arc::clone(&counter);

    let should_retry_fn = |err: &CustomError| matches!(err, CustomError::Transient);

    let policy = RetryPolicy::new()
        .with_max_attempts(5)
        .with_backoff(Backoff::fixed(5));

    let result = execute_with_retry(
        move || {
            let count = Arc::clone(&counter_clone);
            async move {
                let current = count.fetch_add(1, Ordering::SeqCst);
                if current == 0 {
                    Err(CustomError::Transient)
                } else {
                    Err(CustomError::Permanent)
                }
            }
        },
        policy,
        Some(should_retry_fn),
    )
    .await;

    assert!(result.is_err());
    // Should have 2 attempts: 1 for Transient, 1 for Permanent (which stops)
    assert_eq!(counter.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn test_timeout_mechanism() {
    let policy = RetryPolicy::new()
        .with_max_attempts(10)
        .with_timeout(Duration::from_millis(100))
        .with_backoff(Backoff::fixed(50));

    let start = Instant::now();

    let result = execute_with_retry::<_, _, _, String>(
        || async { Err("error".to_string()) },
        policy,
        None,
    )
    .await;

    let elapsed = start.elapsed();

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.timeout_exceeded);
    // Should timeout around 100ms (may overshoot slightly due to last attempt)
    assert!(elapsed.as_millis() >= 80);
}

#[tokio::test]
async fn test_exponential_backoff_timing() {
    let policy = RetryPolicy::new()
        .with_max_attempts(3)
        .with_backoff(Backoff::exponential(20, 2.0, None));

    let start = Instant::now();

    let result = execute_with_retry::<_, _, _, String>(
        || async { Err("error".to_string()) },
        policy,
        None,
    )
    .await;

    let elapsed = start.elapsed();

    assert!(result.is_err());
    // Expected delays: 20ms (attempt 0) + 40ms (attempt 1) = 60ms minimum
    assert!(elapsed.as_millis() >= 55);
}

#[tokio::test]
async fn test_zero_max_attempts_single_try() {
    let counter = Arc::new(AtomicU32::new(0));
    let counter_clone = Arc::clone(&counter);

    let policy = RetryPolicy::new()
        .with_max_attempts(0)
        .with_backoff(Backoff::fixed(1));

    let result = execute_with_retry(
        move || {
            let count = Arc::clone(&counter_clone);
            async move {
                count.fetch_add(1, Ordering::SeqCst);
                Err::<(), _>("error")
            }
        },
        policy,
        None,
    )
    .await;

    assert!(result.is_err());
    // With max_attempts=0, only the initial attempt should run
    assert_eq!(counter.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn test_linear_backoff_progression() {
    let policy = RetryPolicy::new()
        .with_max_attempts(4)
        .with_backoff(Backoff::linear(10, 10, None));

    let start = Instant::now();

    let result = execute_with_retry::<_, _, _, String>(
        || async { Err("error".to_string()) },
        policy,
        None,
    )
    .await;

    let elapsed = start.elapsed();

    assert!(result.is_err());
    // Expected delays: 10 + 20 + 30 + 40 = 100ms minimum
    assert!(elapsed.as_millis() >= 95);
}

#[tokio::test]
async fn test_retry_error_structure() {
    let policy = RetryPolicy::new()
        .with_max_attempts(2)
        .with_backoff(Backoff::fixed(5));

    let result = execute_with_retry::<_, _, _, String>(
        || async { Err("test error".to_string()) },
        policy,
        None,
    )
    .await;

    let err = result.unwrap_err();

    assert_eq!(err.attempts, 3); // 1 initial + 2 retries
    assert_eq!(err.last_error, "test error");
    assert!(!err.timeout_exceeded);
}

#[tokio::test]
async fn test_success_stops_retries() {
    let counter = Arc::new(AtomicU32::new(0));
    let counter_clone = Arc::clone(&counter);

    let policy = RetryPolicy::new()
        .with_max_attempts(10) // More than enough
        .with_backoff(Backoff::fixed(5));

    let result = execute_with_retry(
        move || {
            let count = Arc::clone(&counter_clone);
            async move {
                let current = count.fetch_add(1, Ordering::SeqCst);
                if current < 2 {
                    Err::<String, _>("error")
                } else {
                    Ok(format!("success at attempt {}", current))
                }
            }
        },
        policy,
        None,
    )
    .await;

    assert!(result.is_ok());
    // Should stop immediately after success, not continue retrying
    assert_eq!(counter.load(Ordering::SeqCst), 3);
}

#[tokio::test]
async fn test_multiple_error_types() {
    #[derive(Debug)]
    enum Error {
        TypeA,
        TypeB,
        TypeC,
    }

    impl std::fmt::Display for Error {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Error::TypeA => write!(f, "type A"),
                Error::TypeB => write!(f, "type B"),
                Error::TypeC => write!(f, "type C"),
            }
        }
    }

    let counter = Arc::new(AtomicU32::new(0));
    let counter_clone = Arc::clone(&counter);

    let should_retry_fn = |err: &Error| !matches!(err, Error::TypeC);

    let policy = RetryPolicy::new()
        .with_max_attempts(10)
        .with_backoff(Backoff::fixed(5));

    let result = execute_with_retry(
        move || {
            let count = Arc::clone(&counter_clone);
            async move {
                let current = count.fetch_add(1, Ordering::SeqCst);
                match current {
                    0 => Err(Error::TypeA),
                    1 => Err(Error::TypeB),
                    2 => Err(Error::TypeC), // This should not retry
                    _ => unreachable!(),
                }
            }
        },
        policy,
        Some(should_retry_fn),
    )
    .await;

    assert!(result.is_err());
    assert_eq!(counter.load(Ordering::SeqCst), 3);
}

#[tokio::test]
async fn test_large_number_of_retries() {
    let policy = RetryPolicy::new()
        .with_max_attempts(100)
        .with_backoff(Backoff::fixed(1));

    let counter = Arc::new(AtomicU32::new(0));
    let counter_clone = Arc::clone(&counter);

    let result = execute_with_retry(
        move || {
            let count = Arc::clone(&counter_clone);
            async move {
                let current = count.fetch_add(1, Ordering::SeqCst);
                if current < 50 {
                    Err::<(), _>("error")
                } else {
                    Ok(())
                }
            }
        },
        policy,
        None,
    )
    .await;

    assert!(result.is_ok());
    assert_eq!(counter.load(Ordering::SeqCst), 51); // 50 fails + 1 success
}

#[tokio::test]
async fn test_fibonacci_backoff_with_retries() {
    let policy = RetryPolicy::new()
        .with_max_attempts(5)
        .with_backoff(Backoff::fibonacci(10, Some(500)));

    let start = Instant::now();

    let result = execute_with_retry::<_, _, _, String>(
        || async { Err("error".to_string()) },
        policy,
        None,
    )
    .await;

    let elapsed = start.elapsed();

    assert!(result.is_err());
    // Fibonacci: 10 + 10 + 20 + 30 + 50 = 120ms minimum
    assert!(elapsed.as_millis() >= 110);
}

#[test]
fn test_retry_error_serialization() {
    let err = RetryError {
        attempts: 5,
        last_error: "test error".to_string(),
        timeout_exceeded: false,
    };

    let json = serde_json::to_string(&err).unwrap();
    let deserialized: RetryError = serde_json::from_str(&json).unwrap();

    assert_eq!(err.attempts, deserialized.attempts);
    assert_eq!(err.last_error, deserialized.last_error);
    assert_eq!(err.timeout_exceeded, deserialized.timeout_exceeded);
}
