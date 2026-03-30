// Metrics tracking for LLM providers

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Cost tracking per provider
#[derive(Debug, Clone)]
pub struct CostTracker {
    total_cost_usd: Arc<AtomicU64>, // Stored as millionths of USD
    request_count: Arc<AtomicU64>,
}

impl CostTracker {
    pub fn new() -> Self {
        Self {
            total_cost_usd: Arc::new(AtomicU64::new(0)),
            request_count: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn record_cost(&self, cost: f64) {
        let microdollars = (cost * 1_000_000.0) as u64;
        self.total_cost_usd.fetch_add(microdollars, Ordering::Relaxed);
        self.request_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn total_cost(&self) -> f64 {
        self.total_cost_usd.load(Ordering::Relaxed) as f64 / 1_000_000.0
    }

    pub fn request_count(&self) -> u64 {
        self.request_count.load(Ordering::Relaxed)
    }

    pub fn average_cost(&self) -> f64 {
        let count = self.request_count();
        if count == 0 {
            0.0
        } else {
            self.total_cost() / count as f64
        }
    }

    pub fn reset(&self) {
        self.total_cost_usd.store(0, Ordering::Relaxed);
        self.request_count.store(0, Ordering::Relaxed);
    }
}

impl Default for CostTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Latency tracking per provider
#[derive(Debug, Clone)]
pub struct LatencyTracker {
    total_latency_ms: Arc<AtomicU64>,
    request_count: Arc<AtomicU64>,
    min_latency_ms: Arc<AtomicU64>,
    max_latency_ms: Arc<AtomicU64>,
}

impl LatencyTracker {
    pub fn new() -> Self {
        Self {
            total_latency_ms: Arc::new(AtomicU64::new(0)),
            request_count: Arc::new(AtomicU64::new(0)),
            min_latency_ms: Arc::new(AtomicU64::new(u64::MAX)),
            max_latency_ms: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn record_latency(&self, latency_ms: u64) {
        self.total_latency_ms.fetch_add(latency_ms, Ordering::Relaxed);
        self.request_count.fetch_add(1, Ordering::Relaxed);

        // Update min
        loop {
            let current_min = self.min_latency_ms.load(Ordering::Relaxed);
            if latency_ms >= current_min {
                break;
            }
            match self.min_latency_ms.compare_exchange(
                current_min,
                latency_ms,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(_) => continue,
            }
        }

        // Update max
        loop {
            let current_max = self.max_latency_ms.load(Ordering::Relaxed);
            if latency_ms <= current_max {
                break;
            }
            match self.max_latency_ms.compare_exchange(
                current_max,
                latency_ms,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(_) => continue,
            }
        }
    }

    pub fn average_latency_ms(&self) -> u64 {
        let count = self.request_count.load(Ordering::Relaxed);
        if count == 0 {
            0
        } else {
            self.total_latency_ms.load(Ordering::Relaxed) / count
        }
    }

    pub fn min_latency_ms(&self) -> Option<u64> {
        let min = self.min_latency_ms.load(Ordering::Relaxed);
        if min == u64::MAX {
            None
        } else {
            Some(min)
        }
    }

    pub fn max_latency_ms(&self) -> Option<u64> {
        let max = self.max_latency_ms.load(Ordering::Relaxed);
        if max == 0 {
            None
        } else {
            Some(max)
        }
    }

    pub fn request_count(&self) -> u64 {
        self.request_count.load(Ordering::Relaxed)
    }

    pub fn reset(&self) {
        self.total_latency_ms.store(0, Ordering::Relaxed);
        self.request_count.store(0, Ordering::Relaxed);
        self.min_latency_ms.store(u64::MAX, Ordering::Relaxed);
        self.max_latency_ms.store(0, Ordering::Relaxed);
    }
}

impl Default for LatencyTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Comprehensive provider metrics
#[derive(Debug, Clone)]
pub struct ProviderMetrics {
    pub name: String,
    pub cost_tracker: CostTracker,
    pub latency_tracker: LatencyTracker,
    pub success_count: Arc<AtomicU64>,
    pub failure_count: Arc<AtomicU64>,
}

impl ProviderMetrics {
    pub fn new(name: String) -> Self {
        Self {
            name,
            cost_tracker: CostTracker::new(),
            latency_tracker: LatencyTracker::new(),
            success_count: Arc::new(AtomicU64::new(0)),
            failure_count: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn record_success(&self, latency_ms: u64, cost: f64) {
        self.success_count.fetch_add(1, Ordering::Relaxed);
        self.latency_tracker.record_latency(latency_ms);
        self.cost_tracker.record_cost(cost);
    }

    pub fn record_failure(&self) {
        self.failure_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn success_rate(&self) -> f64 {
        let success = self.success_count.load(Ordering::Relaxed);
        let failure = self.failure_count.load(Ordering::Relaxed);
        let total = success + failure;

        if total == 0 {
            0.0
        } else {
            success as f64 / total as f64
        }
    }

    pub fn total_requests(&self) -> u64 {
        self.success_count.load(Ordering::Relaxed) + self.failure_count.load(Ordering::Relaxed)
    }

    pub fn summary(&self) -> String {
        format!(
            "Provider: {}, Requests: {}, Success Rate: {:.2}%, Avg Latency: {}ms, Total Cost: ${:.4}",
            self.name,
            self.total_requests(),
            self.success_rate() * 100.0,
            self.latency_tracker.average_latency_ms(),
            self.cost_tracker.total_cost()
        )
    }

    pub fn reset(&self) {
        self.cost_tracker.reset();
        self.latency_tracker.reset();
        self.success_count.store(0, Ordering::Relaxed);
        self.failure_count.store(0, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cost_tracker() {
        let tracker = CostTracker::new();
        tracker.record_cost(0.001);
        tracker.record_cost(0.002);

        assert_eq!(tracker.request_count(), 2);
        assert!(f64::abs(tracker.total_cost() - 0.003) < 0.00001);
        assert!(f64::abs(tracker.average_cost() - 0.0015) < 0.00001);
    }

    #[test]
    fn test_latency_tracker() {
        let tracker = LatencyTracker::new();
        tracker.record_latency(100);
        tracker.record_latency(200);
        tracker.record_latency(150);

        assert_eq!(tracker.request_count(), 3);
        assert_eq!(tracker.min_latency_ms(), Some(100));
        assert_eq!(tracker.max_latency_ms(), Some(200));
        assert_eq!(tracker.average_latency_ms(), 150);
    }

    #[test]
    fn test_provider_metrics() {
        let metrics = ProviderMetrics::new("openai".to_string());
        metrics.record_success(100, 0.01);
        metrics.record_success(150, 0.02);
        metrics.record_failure();

        assert_eq!(metrics.total_requests(), 3);
        assert_eq!(metrics.success_count.load(Ordering::Relaxed), 2);
        assert_eq!(metrics.failure_count.load(Ordering::Relaxed), 1);
        assert!(f64::abs(metrics.success_rate() - (2.0 / 3.0)) < 0.001);
    }

    #[test]
    fn test_reset() {
        let tracker = CostTracker::new();
        tracker.record_cost(0.1);
        assert_eq!(tracker.request_count(), 1);

        tracker.reset();
        assert_eq!(tracker.request_count(), 0);
        assert_eq!(tracker.total_cost(), 0.0);
    }
}
