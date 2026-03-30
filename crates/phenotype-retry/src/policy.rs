//! Retry policies for configuring retry behavior.
//!
//! This module defines policies that control how retries should be executed,
//! including maximum attempts, timeouts, and jitter.

use crate::backoff::Backoff;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for retry behavior.
///
/// Specifies the conditions under which an operation should be retried,
/// including maximum attempts, timeout duration, and backoff strategy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts (not including the initial attempt).
    /// For example, max_attempts=3 means: initial attempt + 3 retries = 4 total attempts.
    pub max_attempts: u32,

    /// Total timeout duration for all attempts combined.
    /// If None, there is no overall timeout.
    pub timeout: Option<Duration>,

    /// Backoff strategy between retries.
    pub backoff: Backoff,

    /// Whether to add random jitter to delays to prevent thundering herd.
    /// If true, actual delay = delay * (1.0 ± jitter_factor)
    pub use_jitter: bool,

    /// Jitter factor as a fraction (0.0 to 1.0).
    /// Default is 0.1 (10% variance).
    pub jitter_factor: f64,
}

impl RetryPolicy {
    /// Create a new retry policy with default values.
    ///
    /// # Default values:
    /// - max_attempts: 3
    /// - timeout: None (no timeout)
    /// - backoff: Exponential(100ms base, 2.0 multiplier, 30s max)
    /// - use_jitter: false
    /// - jitter_factor: 0.1
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the maximum number of retry attempts.
    ///
    /// # Arguments
    ///
    /// * `max_attempts` - Number of retries (not including initial attempt)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use phenotype_retry::RetryPolicy;
    ///
    /// let policy = RetryPolicy::new().with_max_attempts(5);
    /// assert_eq!(policy.max_attempts, 5);
    /// ```
    pub fn with_max_attempts(mut self, max_attempts: u32) -> Self {
        self.max_attempts = max_attempts;
        self
    }

    /// Set the overall timeout duration.
    ///
    /// # Arguments
    ///
    /// * `timeout` - Total duration for all attempts combined
    ///
    /// # Examples
    ///
    /// ```rust
    /// use phenotype_retry::RetryPolicy;
    /// use std::time::Duration;
    ///
    /// let policy = RetryPolicy::new().with_timeout(Duration::from_secs(30));
    /// assert_eq!(policy.timeout, Some(Duration::from_secs(30)));
    /// ```
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Set the backoff strategy.
    ///
    /// # Arguments
    ///
    /// * `backoff` - The backoff strategy to use
    ///
    /// # Examples
    ///
    /// ```rust
    /// use phenotype_retry::{RetryPolicy, Backoff};
    ///
    /// let policy = RetryPolicy::new()
    ///     .with_backoff(Backoff::fixed(100));
    /// ```
    pub fn with_backoff(mut self, backoff: Backoff) -> Self {
        self.backoff = backoff;
        self
    }

    /// Enable jitter to prevent thundering herd.
    ///
    /// # Arguments
    ///
    /// * `use_jitter` - Whether to apply jitter
    /// * `jitter_factor` - Jitter variance as a fraction (0.0 to 1.0)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use phenotype_retry::RetryPolicy;
    ///
    /// let policy = RetryPolicy::new().with_jitter(true, 0.1);
    /// assert!(policy.use_jitter);
    /// assert_eq!(policy.jitter_factor, 0.1);
    /// ```
    pub fn with_jitter(mut self, use_jitter: bool, jitter_factor: f64) -> Self {
        self.use_jitter = use_jitter;
        self.jitter_factor = jitter_factor.max(0.0).min(1.0); // Clamp to [0.0, 1.0]
        self
    }

    /// Calculate the delay for a specific attempt with optional jitter.
    ///
    /// # Arguments
    ///
    /// * `attempt` - Zero-indexed attempt number
    pub fn get_delay(&self, attempt: u32) -> Duration {
        let base_delay = self.backoff.delay_ms(attempt);
        let delay = if self.use_jitter && self.jitter_factor > 0.0 {
            let jitter = Self::compute_jitter(base_delay as f64, self.jitter_factor);
            (base_delay as f64 * jitter).ceil() as u64
        } else {
            base_delay
        };
        Duration::from_millis(delay)
    }

    /// Compute jitter multiplier using the full jitter algorithm.
    /// Returns a value in [1 - jitter_factor, 1 + jitter_factor]
    fn compute_jitter(base_delay: f64, jitter_factor: f64) -> f64 {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        // Generate random value in [0, 1]
        let random = rng.gen::<f64>();
        // Map to [1 - jitter_factor, 1 + jitter_factor]
        1.0 - jitter_factor + (2.0 * jitter_factor * random)
    }

    /// Check if we should retry based on the attempt number.
    ///
    /// # Arguments
    ///
    /// * `attempt` - Zero-indexed attempt number
    pub fn should_retry(&self, attempt: u32) -> bool {
        attempt < self.max_attempts
    }

    /// Check if the retry has exceeded the timeout (if set).
    ///
    /// # Arguments
    ///
    /// * `elapsed` - Time elapsed since the first attempt
    pub fn is_timeout_exceeded(&self, elapsed: Duration) -> bool {
        self.timeout.map(|t| elapsed >= t).unwrap_or(false)
    }
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            timeout: None,
            backoff: Backoff::default(),
            use_jitter: false,
            jitter_factor: 0.1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Backoff;

    #[test]
    fn test_default_policy() {
        let policy = RetryPolicy::default();
        assert_eq!(policy.max_attempts, 3);
        assert_eq!(policy.timeout, None);
        assert!(!policy.use_jitter);
        assert_eq!(policy.jitter_factor, 0.1);
    }

    #[test]
    fn test_with_max_attempts() {
        let policy = RetryPolicy::new().with_max_attempts(5);
        assert_eq!(policy.max_attempts, 5);
    }

    #[test]
    fn test_with_timeout() {
        let timeout = Duration::from_secs(30);
        let policy = RetryPolicy::new().with_timeout(timeout);
        assert_eq!(policy.timeout, Some(timeout));
    }

    #[test]
    fn test_with_backoff() {
        let backoff = Backoff::fixed(200);
        let policy = RetryPolicy::new().with_backoff(backoff);
        assert_eq!(policy.get_delay(0).as_millis(), 200);
    }

    #[test]
    fn test_with_jitter() {
        let policy = RetryPolicy::new().with_jitter(true, 0.2);
        assert!(policy.use_jitter);
        assert_eq!(policy.jitter_factor, 0.2);
    }

    #[test]
    fn test_jitter_factor_clamping() {
        let policy = RetryPolicy::new().with_jitter(true, 1.5); // > 1.0
        assert_eq!(policy.jitter_factor, 1.0); // Clamped to 1.0

        let policy = RetryPolicy::new().with_jitter(true, -0.5); // < 0.0
        assert_eq!(policy.jitter_factor, 0.0); // Clamped to 0.0
    }

    #[test]
    fn test_should_retry() {
        let policy = RetryPolicy::new().with_max_attempts(3);
        assert!(policy.should_retry(0));
        assert!(policy.should_retry(1));
        assert!(policy.should_retry(2));
        assert!(!policy.should_retry(3)); // 4th attempt (0-indexed: 3)
    }

    #[test]
    fn test_is_timeout_exceeded() {
        let policy = RetryPolicy::new().with_timeout(Duration::from_secs(10));

        assert!(!policy.is_timeout_exceeded(Duration::from_secs(5)));
        assert!(!policy.is_timeout_exceeded(Duration::from_secs(10))); // Exactly at timeout
        assert!(policy.is_timeout_exceeded(Duration::from_secs(11)));
    }

    #[test]
    fn test_timeout_none() {
        let policy = RetryPolicy::new();
        assert!(!policy.is_timeout_exceeded(Duration::from_secs(1000)));
    }

    #[test]
    fn test_get_delay_without_jitter() {
        let policy = RetryPolicy::new()
            .with_backoff(Backoff::fixed(100))
            .with_jitter(false, 0.0);

        assert_eq!(policy.get_delay(0).as_millis(), 100);
        assert_eq!(policy.get_delay(5).as_millis(), 100);
    }

    #[test]
    fn test_get_delay_with_jitter() {
        let policy = RetryPolicy::new()
            .with_backoff(Backoff::fixed(1000))
            .with_jitter(true, 0.2); // ±20%

        // Run multiple times to see variance
        let mut delays = Vec::new();
        for _ in 0..20 {
            let delay_ms = policy.get_delay(0).as_millis() as f64;
            delays.push(delay_ms);
            // With 1000ms base and 20% jitter, should be in [800, 1200]
            assert!(delay_ms >= 800.0 && delay_ms <= 1200.0);
        }

        // Check that there's actual variance (not all the same)
        let min = delays.iter().copied().fold(f64::INFINITY, f64::min);
        let max = delays.iter().copied().fold(0.0, f64::max);
        assert!(max - min > 0.0); // Should have some variance
    }

    #[test]
    fn test_builder_chaining() {
        let policy = RetryPolicy::new()
            .with_max_attempts(5)
            .with_timeout(Duration::from_secs(60))
            .with_backoff(Backoff::exponential(100, 2.0, Some(10_000)))
            .with_jitter(true, 0.15);

        assert_eq!(policy.max_attempts, 5);
        assert_eq!(policy.timeout, Some(Duration::from_secs(60)));
        assert!(policy.use_jitter);
        assert_eq!(policy.jitter_factor, 0.15);
    }

    #[test]
    fn test_policy_serialization() {
        let policy = RetryPolicy::new()
            .with_max_attempts(5)
            .with_backoff(Backoff::exponential(100, 2.0, Some(10_000)));

        let json = serde_json::to_string(&policy).unwrap();
        let deserialized: RetryPolicy = serde_json::from_str(&json).unwrap();

        assert_eq!(policy.max_attempts, deserialized.max_attempts);
        assert_eq!(policy.get_delay(2), deserialized.get_delay(2));
    }
}
