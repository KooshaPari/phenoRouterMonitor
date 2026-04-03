//! Observability traits for phenotype ecosystem.
//!
//! Provides standard interfaces for metrics collection, tracing, and logging
//! across all phenotype crates.

use std::fmt::Debug;

/// Metrics hook for cache and operation observability.
///
/// Implement this trait to receive cache hit/miss events and other
/// operational metrics from phenotype crates.
///
/// # Example
/// ```
/// use phenotype_port_traits::observability::MetricsHook;
/// use std::sync::atomic::{AtomicU64, Ordering};
///
/// #[derive(Debug)]
/// struct Counter {
///     hits: AtomicU64,
///     misses: AtomicU64,
/// }
///
/// impl MetricsHook for Counter {
///     fn record_hit(&self, tier: &str) {
///         self.hits.fetch_add(1, Ordering::Relaxed);
///     }
///
///     fn record_miss(&self, tier: &str) {
///         self.misses.fetch_add(1, Ordering::Relaxed);
///     }
/// }
/// ```
pub trait MetricsHook: Send + Sync + Debug {
    /// Record a cache hit at the specified tier (e.g., "l1", "l2").
    fn record_hit(&self, tier: &str);

    /// Record a cache miss at the specified tier (e.g., "l1", "l2").
    fn record_miss(&self, tier: &str);
}

/// No-op metrics hook for when observability is not needed.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoOpMetrics;

impl MetricsHook for NoOpMetrics {
    fn record_hit(&self, _tier: &str) {
        // Intentionally no-op
    }

    fn record_miss(&self, _tier: &str) {
        // Intentionally no-op
    }
}

/// A simple counter-based metrics hook for testing.
#[derive(Debug, Default)]
pub struct CounterMetrics {
    hits: std::sync::atomic::AtomicU64,
    misses: std::sync::atomic::AtomicU64,
}

impl CounterMetrics {
    /// Create a new counter metrics instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the current hit count.
    pub fn hits(&self) -> u64 {
        self.hits.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Get the current miss count.
    pub fn misses(&self) -> u64 {
        self.misses.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Reset all counters to zero.
    pub fn reset(&self) {
        self.hits.store(0, std::sync::atomic::Ordering::Relaxed);
        self.misses.store(0, std::sync::atomic::Ordering::Relaxed);
    }
}

impl MetricsHook for CounterMetrics {
    fn record_hit(&self, _tier: &str) {
        self.hits.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    fn record_miss(&self, _tier: &str) {
        self.misses.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noop_metrics_does_not_panic() {
        let metrics = NoOpMetrics;
        metrics.record_hit("l1");
        metrics.record_miss("l2");
        // Should not panic
    }

    #[test]
    fn counter_metrics_tracks_hits() {
        let metrics = CounterMetrics::new();
        metrics.record_hit("l1");
        metrics.record_hit("l1");
        metrics.record_hit("l2");
        assert_eq!(metrics.hits(), 3);
    }

    #[test]
    fn counter_metrics_tracks_misses() {
        let metrics = CounterMetrics::new();
        metrics.record_miss("l1");
        metrics.record_miss("l2");
        assert_eq!(metrics.misses(), 2);
    }

    #[test]
    fn counter_metrics_reset_works() {
        let metrics = CounterMetrics::new();
        metrics.record_hit("l1");
        metrics.record_miss("l1");
        metrics.reset();
        assert_eq!(metrics.hits(), 0);
        assert_eq!(metrics.misses(), 0);
    }
}
