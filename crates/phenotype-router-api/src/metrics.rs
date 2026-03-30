//! Metrics collection and export for the Router API.
//!
//! Provides Prometheus-style metrics and JSON export capabilities.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Metrics collector trait.
pub trait MetricsCollector: Send + Sync {
    /// Record a request.
    fn record_request(&self, path: &str, method: &str, status: u16);

    /// Record an error.
    fn record_error(&self, error_type: &str);

    /// Get current metrics.
    fn get_metrics(&self) -> Metrics;
}

/// Metrics snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metrics {
    /// Total requests handled.
    pub total_requests: u64,
    /// Total errors encountered.
    pub total_errors: u64,
    /// Request latency histogram (buckets in ms).
    pub latency_buckets: HashMap<String, u64>,
    /// Requests by status code.
    pub requests_by_status: HashMap<u16, u64>,
    /// Requests by path.
    pub requests_by_path: HashMap<String, u64>,
    /// Timestamp of this metrics snapshot.
    pub timestamp: DateTime<Utc>,
}

impl Metrics {
    /// Create a new metrics snapshot.
    pub fn new() -> Self {
        Self {
            total_requests: 0,
            total_errors: 0,
            latency_buckets: HashMap::new(),
            requests_by_status: HashMap::new(),
            requests_by_path: HashMap::new(),
            timestamp: Utc::now(),
        }
    }

    /// Get success rate as a percentage.
    pub fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            return 0.0;
        }
        let success = self.total_requests - self.total_errors;
        (success as f64 / self.total_requests as f64) * 100.0
    }

    /// Get average requests per second (approximate).
    pub fn requests_per_second(&self) -> f64 {
        if self.total_requests == 0 {
            return 0.0;
        }
        // Approximation: assumes metrics collection started at epoch
        let elapsed = Utc::now().timestamp_millis();
        if elapsed <= 0 {
            return 0.0;
        }
        self.total_requests as f64 / (elapsed as f64 / 1000.0)
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Default metrics collector implementation.
pub struct DefaultMetricsCollector {
    total_requests: Arc<AtomicU64>,
    total_errors: Arc<AtomicU64>,
    requests_by_status: Arc<parking_lot::RwLock<HashMap<u16, u64>>>,
    requests_by_path: Arc<parking_lot::RwLock<HashMap<String, u64>>>,
}

impl DefaultMetricsCollector {
    /// Create a new metrics collector.
    pub fn new() -> Self {
        Self {
            total_requests: Arc::new(AtomicU64::new(0)),
            total_errors: Arc::new(AtomicU64::new(0)),
            requests_by_status: Arc::new(parking_lot::RwLock::new(HashMap::new())),
            requests_by_path: Arc::new(parking_lot::RwLock::new(HashMap::new())),
        }
    }
}

impl Default for DefaultMetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsCollector for DefaultMetricsCollector {
    fn record_request(&self, path: &str, _method: &str, status: u16) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);

        let mut status_map = self.requests_by_status.write();
        *status_map.entry(status).or_insert(0) += 1;

        let mut path_map = self.requests_by_path.write();
        *path_map.entry(path.to_string()).or_insert(0) += 1;
    }

    fn record_error(&self, _error_type: &str) {
        self.total_errors.fetch_add(1, Ordering::Relaxed);
    }

    fn get_metrics(&self) -> Metrics {
        Metrics {
            total_requests: self.total_requests.load(Ordering::Relaxed),
            total_errors: self.total_errors.load(Ordering::Relaxed),
            latency_buckets: HashMap::new(),
            requests_by_status: self.requests_by_status.read().clone(),
            requests_by_path: self.requests_by_path.read().clone(),
            timestamp: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_new() {
        let metrics = Metrics::new();
        assert_eq!(metrics.total_requests, 0);
        assert_eq!(metrics.total_errors, 0);
    }

    #[test]
    fn test_metrics_success_rate_zero() {
        let metrics = Metrics::new();
        assert_eq!(metrics.success_rate(), 0.0);
    }

    #[test]
    fn test_metrics_success_rate_all_success() {
        let mut metrics = Metrics::new();
        metrics.total_requests = 100;
        metrics.total_errors = 0;
        assert_eq!(metrics.success_rate(), 100.0);
    }

    #[test]
    fn test_metrics_success_rate_half() {
        let mut metrics = Metrics::new();
        metrics.total_requests = 100;
        metrics.total_errors = 50;
        assert_eq!(metrics.success_rate(), 50.0);
    }

    #[test]
    fn test_default_collector_record_request() {
        let collector = DefaultMetricsCollector::new();
        collector.record_request("/health", "GET", 200);
        collector.record_request("/health", "GET", 200);
        collector.record_request("/api/agents", "GET", 201);

        let metrics = collector.get_metrics();
        assert_eq!(metrics.total_requests, 3);
        assert_eq!(metrics.total_errors, 0);
    }

    #[test]
    fn test_default_collector_record_error() {
        let collector = DefaultMetricsCollector::new();
        collector.record_request("/api/test", "GET", 500);
        collector.record_error("timeout");
        collector.record_error("network");

        let metrics = collector.get_metrics();
        assert_eq!(metrics.total_requests, 1);
        assert_eq!(metrics.total_errors, 2);
    }

    #[test]
    fn test_collector_requests_by_status() {
        let collector = DefaultMetricsCollector::new();
        collector.record_request("/health", "GET", 200);
        collector.record_request("/health", "GET", 200);
        collector.record_request("/api/agents", "GET", 404);

        let metrics = collector.get_metrics();
        assert_eq!(metrics.requests_by_status.get(&200), Some(&2));
        assert_eq!(metrics.requests_by_status.get(&404), Some(&1));
    }

    #[test]
    fn test_collector_requests_by_path() {
        let collector = DefaultMetricsCollector::new();
        collector.record_request("/health", "GET", 200);
        collector.record_request("/health", "GET", 200);
        collector.record_request("/api/agents", "GET", 200);

        let metrics = collector.get_metrics();
        assert_eq!(metrics.requests_by_path.get("/health"), Some(&2));
        assert_eq!(metrics.requests_by_path.get("/api/agents"), Some(&1));
    }

    #[test]
    fn test_metrics_requests_per_second() {
        let metrics = Metrics::new();
        let rate = metrics.requests_per_second();
        // Should be approximately 0 on startup
        assert!(rate >= 0.0);
    }
}
