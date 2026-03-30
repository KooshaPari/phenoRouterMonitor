//! Interval utilities with backoff strategies.

use crate::Duration;
use std::fmt;

/// Backoff strategy for interval retry behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackoffStrategy {
    /// Fixed interval between attempts
    Fixed,
    /// Exponential backoff: interval = base * multiplier^attempt
    Exponential { multiplier: u32 },
    /// Linear backoff: interval = base + (step * attempt)
    Linear { step: u64 },
}

impl fmt::Display for BackoffStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Fixed => write!(f, "fixed"),
            Self::Exponential { multiplier } => write!(f, "exponential(x{})", multiplier),
            Self::Linear { step } => write!(f, "linear(+{}s)", step),
        }
    }
}

/// Interval configuration with backoff support
#[derive(Debug, Clone, Copy)]
pub struct Interval {
    base: Duration,
    strategy: BackoffStrategy,
    max_interval: Option<Duration>,
}

impl Interval {
    /// Create a new fixed interval
    #[must_use]
    pub fn fixed(base: Duration) -> Self {
        Self {
            base,
            strategy: BackoffStrategy::Fixed,
            max_interval: None,
        }
    }

    /// Create a new exponential backoff interval
    #[must_use]
    pub fn exponential(base: Duration, multiplier: u32) -> Self {
        Self {
            base,
            strategy: BackoffStrategy::Exponential { multiplier },
            max_interval: None,
        }
    }

    /// Create a new linear backoff interval
    #[must_use]
    pub fn linear(base: Duration, step: u64) -> Self {
        Self {
            base,
            strategy: BackoffStrategy::Linear { step },
            max_interval: None,
        }
    }

    /// Set a maximum interval cap
    #[must_use]
    pub fn with_max(mut self, max: Duration) -> Self {
        self.max_interval = Some(max);
        self
    }

    /// Calculate the next wait duration for the given attempt number (0-indexed)
    #[must_use]
    pub fn next_wait(&self, attempt: u32) -> Duration {
        let wait = match self.strategy {
            BackoffStrategy::Fixed => self.base,
            BackoffStrategy::Exponential { multiplier } => {
                let multiplier = u64::from(multiplier).pow(attempt);
                let secs = self.base.as_secs().saturating_mul(multiplier);
                Duration::from_secs(secs)
            }
            BackoffStrategy::Linear { step } => {
                let secs = self
                    .base
                    .as_secs()
                    .saturating_add(step.saturating_mul(u64::from(attempt)));
                Duration::from_secs(secs)
            }
        };

        if let Some(max) = self.max_interval {
            if wait.as_secs() > max.as_secs() {
                return max;
            }
        }
        wait
    }

    #[must_use]
    pub fn strategy(&self) -> BackoffStrategy {
        self.strategy
    }

    #[must_use]
    pub fn base(&self) -> Duration {
        self.base
    }

    #[must_use]
    pub fn max(&self) -> Option<Duration> {
        self.max_interval
    }
}

impl fmt::Display for Interval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} ", self.base, self.strategy)?;
        if let Some(max) = self.max_interval {
            write!(f, "(max: {})", max)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interval_fixed() {
        let interval = Interval::fixed(Duration::from_secs(5));
        assert_eq!(interval.next_wait(0), Duration::from_secs(5));
        assert_eq!(interval.next_wait(1), Duration::from_secs(5));
    }

    #[test]
    fn interval_exponential() {
        let interval = Interval::exponential(Duration::from_secs(1), 2);
        assert_eq!(interval.next_wait(0), Duration::from_secs(1));
        assert_eq!(interval.next_wait(1), Duration::from_secs(2));
    }

    #[test]
    fn interval_exponential_with_max() {
        let interval =
            Interval::exponential(Duration::from_secs(1), 2).with_max(Duration::from_secs(10));
        assert_eq!(interval.next_wait(0), Duration::from_secs(1));
        assert_eq!(interval.next_wait(1), Duration::from_secs(2));
    }

    #[test]
    fn interval_linear() {
        let interval = Interval::linear(Duration::from_secs(1), 2);
        assert_eq!(interval.next_wait(0), Duration::from_secs(1));
        assert_eq!(interval.next_wait(1), Duration::from_secs(3));
        assert_eq!(interval.next_wait(2), Duration::from_secs(5));
    }
}
