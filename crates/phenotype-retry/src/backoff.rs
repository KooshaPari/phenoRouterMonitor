//! Backoff strategies for retry logic.
//!
//! This module defines various backoff strategies that determine the delay between retry attempts.

use serde::{Deserialize, Serialize};
use std::num::NonZeroU64;

/// Backoff strategy for retry delays.
///
/// Defines how the delay between retries should be calculated.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Backoff {
    /// Fixed delay between retries.
    ///
    /// Each retry will wait exactly `delay_ms` milliseconds.
    Fixed {
        /// Fixed delay in milliseconds.
        delay_ms: u64,
    },

    /// Exponential backoff with base.
    ///
    /// Delay = base_ms * (multiplier ^ attempt_number)
    /// Attempts are 0-indexed, so the first retry has delay = base_ms * multiplier^0
    Exponential {
        /// Initial delay in milliseconds.
        base_ms: u64,
        /// Multiplication factor for each retry.
        multiplier: f64,
        /// Optional maximum delay cap in milliseconds.
        max_ms: Option<u64>,
    },

    /// Linear backoff.
    ///
    /// Delay = base_ms + (step_ms * attempt_number)
    /// Attempts are 0-indexed.
    Linear {
        /// Initial delay in milliseconds.
        base_ms: u64,
        /// Step increment per retry in milliseconds.
        step_ms: u64,
        /// Optional maximum delay cap in milliseconds.
        max_ms: Option<u64>,
    },

    /// Fibonacci backoff.
    ///
    /// Uses the Fibonacci sequence to calculate delays.
    /// sequence: 1, 1, 2, 3, 5, 8, 13, 21, ...
    /// Delay = fib(attempt_number) * base_ms
    Fibonacci {
        /// Base multiplier for Fibonacci values in milliseconds.
        base_ms: u64,
        /// Optional maximum delay cap in milliseconds.
        max_ms: Option<u64>,
    },
}

impl Backoff {
    /// Create a fixed backoff strategy.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use phenotype_retry::Backoff;
    ///
    /// let backoff = Backoff::fixed(100);
    /// assert_eq!(backoff.delay_ms(0), 100);
    /// assert_eq!(backoff.delay_ms(5), 100); // Always 100ms
    /// ```
    pub fn fixed(delay_ms: u64) -> Self {
        Backoff::Fixed { delay_ms }
    }

    /// Create an exponential backoff strategy.
    ///
    /// # Arguments
    ///
    /// * `base_ms` - Initial delay in milliseconds
    /// * `multiplier` - Multiplication factor for each retry (typically 2.0)
    /// * `max_ms` - Optional maximum delay cap
    ///
    /// # Examples
    ///
    /// ```rust
    /// use phenotype_retry::Backoff;
    ///
    /// let backoff = Backoff::exponential(100, 2.0, Some(10_000));
    /// assert_eq!(backoff.delay_ms(0), 100);    // 100 * 2^0 = 100
    /// assert_eq!(backoff.delay_ms(1), 200);    // 100 * 2^1 = 200
    /// assert_eq!(backoff.delay_ms(2), 400);    // 100 * 2^2 = 400
    /// ```
    pub fn exponential(base_ms: u64, multiplier: f64, max_ms: Option<u64>) -> Self {
        Backoff::Exponential {
            base_ms,
            multiplier,
            max_ms,
        }
    }

    /// Create a linear backoff strategy.
    ///
    /// # Arguments
    ///
    /// * `base_ms` - Initial delay in milliseconds
    /// * `step_ms` - Step increment per retry
    /// * `max_ms` - Optional maximum delay cap
    ///
    /// # Examples
    ///
    /// ```rust
    /// use phenotype_retry::Backoff;
    ///
    /// let backoff = Backoff::linear(100, 50, None);
    /// assert_eq!(backoff.delay_ms(0), 100);    // 100 + 50*0 = 100
    /// assert_eq!(backoff.delay_ms(1), 150);    // 100 + 50*1 = 150
    /// assert_eq!(backoff.delay_ms(2), 200);    // 100 + 50*2 = 200
    /// ```
    pub fn linear(base_ms: u64, step_ms: u64, max_ms: Option<u64>) -> Self {
        Backoff::Linear {
            base_ms,
            step_ms,
            max_ms,
        }
    }

    /// Create a Fibonacci backoff strategy.
    ///
    /// # Arguments
    ///
    /// * `base_ms` - Base multiplier for Fibonacci values in milliseconds
    /// * `max_ms` - Optional maximum delay cap
    ///
    /// # Examples
    ///
    /// ```rust
    /// use phenotype_retry::Backoff;
    ///
    /// let backoff = Backoff::fibonacci(100, None);
    /// assert_eq!(backoff.delay_ms(0), 100);    // fib(0) * 100 = 1 * 100
    /// assert_eq!(backoff.delay_ms(1), 100);    // fib(1) * 100 = 1 * 100
    /// assert_eq!(backoff.delay_ms(2), 200);    // fib(2) * 100 = 2 * 100
    /// assert_eq!(backoff.delay_ms(3), 300);    // fib(3) * 100 = 3 * 100
    /// ```
    pub fn fibonacci(base_ms: u64, max_ms: Option<u64>) -> Self {
        Backoff::Fibonacci { base_ms, max_ms }
    }

    /// Calculate delay in milliseconds for a given attempt number.
    ///
    /// # Arguments
    ///
    /// * `attempt` - Zero-indexed attempt number
    pub fn delay_ms(&self, attempt: u32) -> u64 {
        match self {
            Backoff::Fixed { delay_ms } => *delay_ms,
            Backoff::Exponential {
                base_ms,
                multiplier,
                max_ms,
            } => {
                let delay = (*base_ms as f64) * multiplier.powi(attempt as i32);
                let delay = delay.ceil() as u64;
                max_ms.map(|m| delay.min(m)).unwrap_or(delay)
            }
            Backoff::Linear {
                base_ms,
                step_ms,
                max_ms,
            } => {
                let delay = base_ms + (step_ms * attempt as u64);
                max_ms.map(|m| delay.min(m)).unwrap_or(delay)
            }
            Backoff::Fibonacci { base_ms, max_ms } => {
                let fib_value = fibonacci_value(attempt);
                let delay = base_ms * fib_value as u64;
                max_ms.map(|m| delay.min(m)).unwrap_or(delay)
            }
        }
    }

    /// Get the maximum possible delay (if capped) or None.
    pub fn max_delay(&self) -> Option<u64> {
        match self {
            Backoff::Fixed { delay_ms } => Some(*delay_ms),
            Backoff::Exponential { max_ms, .. } => *max_ms,
            Backoff::Linear { max_ms, .. } => *max_ms,
            Backoff::Fibonacci { max_ms, .. } => *max_ms,
        }
    }
}

/// Calculate the nth Fibonacci number.
/// F(0) = 1, F(1) = 1, F(2) = 2, F(3) = 3, F(4) = 5, ...
fn fibonacci_value(n: u32) -> u32 {
    match n {
        0 | 1 => 1,
        _ => {
            let mut a = 1u32;
            let mut b = 1u32;
            for _ in 2..=n {
                let next = a.saturating_add(b);
                a = b;
                b = next;
            }
            b
        }
    }
}

impl Default for Backoff {
    /// Default backoff: exponential with 100ms base, 2.0 multiplier, 30s max.
    fn default() -> Self {
        Backoff::exponential(100, 2.0, Some(30_000))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_backoff() {
        let backoff = Backoff::fixed(100);
        assert_eq!(backoff.delay_ms(0), 100);
        assert_eq!(backoff.delay_ms(1), 100);
        assert_eq!(backoff.delay_ms(10), 100);
    }

    #[test]
    fn test_exponential_backoff_basic() {
        let backoff = Backoff::exponential(100, 2.0, None);
        assert_eq!(backoff.delay_ms(0), 100);
        assert_eq!(backoff.delay_ms(1), 200);
        assert_eq!(backoff.delay_ms(2), 400);
        assert_eq!(backoff.delay_ms(3), 800);
        assert_eq!(backoff.delay_ms(4), 1600);
    }

    #[test]
    fn test_exponential_backoff_with_cap() {
        let backoff = Backoff::exponential(100, 2.0, Some(500));
        assert_eq!(backoff.delay_ms(0), 100);
        assert_eq!(backoff.delay_ms(1), 200);
        assert_eq!(backoff.delay_ms(2), 400);
        assert_eq!(backoff.delay_ms(3), 500); // Capped
        assert_eq!(backoff.delay_ms(10), 500); // Capped
    }

    #[test]
    fn test_exponential_backoff_non_integer_multiplier() {
        let backoff = Backoff::exponential(100, 1.5, None);
        assert_eq!(backoff.delay_ms(0), 100);
        assert_eq!(backoff.delay_ms(1), 150);
        assert_eq!(backoff.delay_ms(2), 225);
        assert_eq!(backoff.delay_ms(3), 338); // 100 * 1.5^3 = 337.5, ceil = 338
    }

    #[test]
    fn test_linear_backoff() {
        let backoff = Backoff::linear(100, 50, None);
        assert_eq!(backoff.delay_ms(0), 100);
        assert_eq!(backoff.delay_ms(1), 150);
        assert_eq!(backoff.delay_ms(2), 200);
        assert_eq!(backoff.delay_ms(3), 250);
        assert_eq!(backoff.delay_ms(5), 350);
    }

    #[test]
    fn test_linear_backoff_with_cap() {
        let backoff = Backoff::linear(100, 50, Some(300));
        assert_eq!(backoff.delay_ms(0), 100);
        assert_eq!(backoff.delay_ms(1), 150);
        assert_eq!(backoff.delay_ms(2), 200);
        assert_eq!(backoff.delay_ms(3), 250);
        assert_eq!(backoff.delay_ms(4), 300); // Capped
        assert_eq!(backoff.delay_ms(10), 300); // Capped
    }

    #[test]
    fn test_fibonacci_backoff() {
        let backoff = Backoff::fibonacci(100, None);
        assert_eq!(backoff.delay_ms(0), 100);  // fib(0) = 1
        assert_eq!(backoff.delay_ms(1), 100);  // fib(1) = 1
        assert_eq!(backoff.delay_ms(2), 200);  // fib(2) = 2
        assert_eq!(backoff.delay_ms(3), 300);  // fib(3) = 3
        assert_eq!(backoff.delay_ms(4), 500);  // fib(4) = 5
        assert_eq!(backoff.delay_ms(5), 800);  // fib(5) = 8
        assert_eq!(backoff.delay_ms(6), 1300); // fib(6) = 13
        assert_eq!(backoff.delay_ms(7), 2100); // fib(7) = 21
    }

    #[test]
    fn test_fibonacci_backoff_with_cap() {
        let backoff = Backoff::fibonacci(100, Some(1000));
        assert_eq!(backoff.delay_ms(0), 100);
        assert_eq!(backoff.delay_ms(4), 500);
        assert_eq!(backoff.delay_ms(5), 800);
        assert_eq!(backoff.delay_ms(6), 1000); // Capped
        assert_eq!(backoff.delay_ms(10), 1000); // Capped
    }

    #[test]
    fn test_fibonacci_saturation() {
        // Test that fibonacci doesn't overflow
        let backoff = Backoff::fibonacci(1, Some(u64::MAX));
        // Just ensure these don't panic
        let _ = backoff.delay_ms(30);
        let _ = backoff.delay_ms(50);
    }

    #[test]
    fn test_backoff_default() {
        let backoff = Backoff::default();
        // Default is exponential(100, 2.0, Some(30_000))
        assert_eq!(backoff.delay_ms(0), 100);
        assert_eq!(backoff.delay_ms(1), 200);
        // Further attempts get capped
        assert_eq!(backoff.delay_ms(20), 30_000);
    }

    #[test]
    fn test_backoff_max_delay() {
        assert_eq!(Backoff::fixed(100).max_delay(), Some(100));
        assert_eq!(Backoff::exponential(100, 2.0, Some(5000)).max_delay(), Some(5000));
        assert_eq!(Backoff::exponential(100, 2.0, None).max_delay(), None);
        assert_eq!(Backoff::linear(100, 50, Some(1000)).max_delay(), Some(1000));
        assert_eq!(Backoff::fibonacci(100, Some(5000)).max_delay(), Some(5000));
    }

    #[test]
    fn test_backoff_serialization() {
        let backoff = Backoff::exponential(100, 2.0, Some(10_000));
        let json = serde_json::to_string(&backoff).unwrap();
        let deserialized: Backoff = serde_json::from_str(&json).unwrap();
        assert_eq!(backoff.delay_ms(0), deserialized.delay_ms(0));
        assert_eq!(backoff.delay_ms(3), deserialized.delay_ms(3));
    }

    #[test]
    fn test_exponential_backoff_zero_base() {
        let backoff = Backoff::exponential(0, 2.0, None);
        assert_eq!(backoff.delay_ms(0), 0);
        assert_eq!(backoff.delay_ms(1), 0);
    }

    #[test]
    fn test_linear_backoff_zero_base() {
        let backoff = Backoff::linear(0, 100, None);
        assert_eq!(backoff.delay_ms(0), 0);
        assert_eq!(backoff.delay_ms(1), 100);
        assert_eq!(backoff.delay_ms(5), 500);
    }
}
