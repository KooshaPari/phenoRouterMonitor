//! Metrics collection and aggregation
//!
//! Thread-safe metrics collector with:
//! - Request count tracking
//! - Latency histogram (p50, p95, p99)
//! - Status code counters
//! - In-flight request gauge

use crate::metrics::RequestMetrics;
use dashmap::DashMap;
use parking_lot::RwLock;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Latency histogram with percentile tracking
#[derive(Debug, Clone)]
pub struct LatencyHistogram {
    pub min_ms: u64,
    pub max_ms: u64,
    pub avg_ms: f64,
    pub p50_ms: u64,
    pub p95_ms: u64,
    pub p99_ms: u64,
    pub count: u64,
    pub sum_ms: u64,
}

impl LatencyHistogram {
    /// Create empty histogram
    fn new() -> Self {
        Self {
            min_ms: u64::MAX,
            max_ms: 0,
            avg_ms: 0.0,
            p50_ms: 0,
            p95_ms: 0,
            p99_ms: 0,
            count: 0,
            sum_ms: 0,
        }
    }

    /// Update histogram with new latency sample
    fn update(&mut self, latency_ms: u64, samples: &[u64]) {
        self.min_ms = self.min_ms.min(latency_ms);
        self.max_ms = self.max_ms.max(latency_ms);
        self.count += 1;
        self.sum_ms += latency_ms;
        self.avg_ms = self.sum_ms as f64 / self.count as f64;

        if !samples.is_empty() {
            let mut sorted = samples.to_vec();
            sorted.sort_unstable();

            let p50_idx = (sorted.len() as f64 * 0.5) as usize;
            let p95_idx = (sorted.len() as f64 * 0.95) as usize;
            let p99_idx = (sorted.len() as f64 * 0.99) as usize;

            self.p50_ms = sorted.get(p50_idx.min(sorted.len() - 1)).copied().unwrap_or(0);
            self.p95_ms = sorted.get(p95_idx.min(sorted.len() - 1)).copied().unwrap_or(0);
            self.p99_ms = sorted.get(p99_idx.min(sorted.len() - 1)).copied().unwrap_or(0);
        }
    }
}

/// Status code counters
#[derive(Debug, Clone, Copy, Default)]
pub struct StatusCounters {
    pub total: u64,
    pub success_2xx: u64,
    pub client_error_4xx: u64,
    pub server_error_5xx: u64,
}

/// Metrics collector (thread-safe)
#[derive(Debug, Clone)]
pub struct MetricsCollector {
    request_count: Arc<AtomicU64>,
    in_flight: Arc<AtomicU64>,
    latency_samples: Arc<RwLock<Vec<u64>>>,
    status_counters: Arc<RwLock<StatusCounters>>,
    service_counters: Arc<DashMap<String, u64>>,
    latency_histogram: Arc<RwLock<LatencyHistogram>>,
}

impl MetricsCollector {
    /// Create new metrics collector
    pub fn new() -> Self {
        Self {
            request_count: Arc::new(AtomicU64::new(0)),
            in_flight: Arc::new(AtomicU64::new(0)),
            latency_samples: Arc::new(RwLock::new(Vec::with_capacity(10000))),
            status_counters: Arc::new(RwLock::new(StatusCounters::default())),
            service_counters: Arc::new(DashMap::new()),
            latency_histogram: Arc::new(RwLock::new(LatencyHistogram::new())),
        }
    }

    /// Record a request start
    pub fn record_request_start(&self) {
        self.in_flight.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a request completion
    pub fn record_request_end(&self, metrics: &RequestMetrics) {
        self.request_count.fetch_add(1, Ordering::Relaxed);
        self.in_flight.fetch_sub(1, Ordering::Relaxed);

        // Update status counters
        {
            let mut counters = self.status_counters.write();
            counters.total += 1;
            if metrics.is_success() {
                counters.success_2xx += 1;
            } else if metrics.is_client_error() {
                counters.client_error_4xx += 1;
            } else if metrics.is_server_error() {
                counters.server_error_5xx += 1;
            }
        }

        // Update service counters
        *self
            .service_counters
            .entry(metrics.service.clone())
            .or_insert(0) += 1;

        // Update latency histogram
        {
            let mut samples = self.latency_samples.write();
            samples.push(metrics.latency_ms);

            // Keep only last 10000 samples for percentile calculation
            if samples.len() > 10000 {
                samples.drain(0..samples.len() - 10000);
            }

            let mut histogram = self.latency_histogram.write();
            histogram.update(metrics.latency_ms, &samples);
        }
    }

    /// Get total request count
    pub fn get_request_count(&self) -> u64 {
        self.request_count.load(Ordering::Relaxed)
    }

    /// Get in-flight request count
    pub fn get_in_flight_count(&self) -> u64 {
        self.in_flight.load(Ordering::Relaxed)
    }

    /// Get status counters
    pub fn get_status_counters(&self) -> StatusCounters {
        *self.status_counters.read()
    }

    /// Get latency histogram
    pub fn get_latency_histogram(&self) -> LatencyHistogram {
        self.latency_histogram.read().clone()
    }

    /// Get request count for service
    pub fn get_service_count(&self, service: &str) -> u64 {
        self.service_counters
            .get(service)
            .map(|r| *r)
            .unwrap_or(0)
    }

    /// Get all service counts
    pub fn get_all_service_counts(&self) -> Vec<(String, u64)> {
        self.service_counters
            .iter()
            .map(|entry| (entry.key().clone(), *entry.value()))
            .collect()
    }

    /// Reset all metrics
    pub fn reset(&self) {
        self.request_count.store(0, Ordering::Relaxed);
        self.in_flight.store(0, Ordering::Relaxed);
        self.latency_samples.write().clear();
        *self.status_counters.write() = StatusCounters::default();
        self.service_counters.clear();
        *self.latency_histogram.write() = LatencyHistogram::new();
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Traces to: FR-ROUTER-010 (Metrics collection)
    #[test]
    fn test_record_request() {
        let collector = MetricsCollector::new();
        let metrics =
            RequestMetrics::new("/api/users", 200, 150, "api", "http://localhost:3000");

        collector.record_request_start();
        assert_eq!(collector.get_in_flight_count(), 1);

        collector.record_request_end(&metrics);
        assert_eq!(collector.get_request_count(), 1);
        assert_eq!(collector.get_in_flight_count(), 0);
    }

    // Traces to: FR-ROUTER-010
    #[test]
    fn test_status_counters() {
        let collector = MetricsCollector::new();

        collector.record_request_start();
        collector.record_request_end(&RequestMetrics::new(
            "/api/users",
            200,
            150,
            "api",
            "http://localhost:3000",
        ));

        collector.record_request_start();
        collector.record_request_end(&RequestMetrics::new(
            "/api/users",
            404,
            100,
            "api",
            "http://localhost:3000",
        ));

        collector.record_request_start();
        collector.record_request_end(&RequestMetrics::new(
            "/api/users",
            500,
            200,
            "api",
            "http://localhost:3000",
        ));

        let counters = collector.get_status_counters();
        assert_eq!(counters.total, 3);
        assert_eq!(counters.success_2xx, 1);
        assert_eq!(counters.client_error_4xx, 1);
        assert_eq!(counters.server_error_5xx, 1);
    }

    // Traces to: FR-ROUTER-010
    #[test]
    fn test_service_counters() {
        let collector = MetricsCollector::new();

        collector.record_request_start();
        collector.record_request_end(&RequestMetrics::new(
            "/api/users",
            200,
            150,
            "api",
            "http://localhost:3000",
        ));

        collector.record_request_start();
        collector.record_request_end(&RequestMetrics::new(
            "/web/home",
            200,
            100,
            "web",
            "http://localhost:8080",
        ));

        assert_eq!(collector.get_service_count("api"), 1);
        assert_eq!(collector.get_service_count("web"), 1);
        assert_eq!(collector.get_service_count("unknown"), 0);
    }

    // Traces to: FR-ROUTER-010
    #[test]
    fn test_latency_histogram() {
        let collector = MetricsCollector::new();

        for latency in [50, 100, 150, 200, 250].iter() {
            collector.record_request_start();
            collector.record_request_end(&RequestMetrics::new(
                "/api/test",
                200,
                *latency,
                "api",
                "http://localhost:3000",
            ));
        }

        let histogram = collector.get_latency_histogram();
        assert_eq!(histogram.count, 5);
        assert_eq!(histogram.min_ms, 50);
        assert_eq!(histogram.max_ms, 250);
        assert!(histogram.avg_ms > 140.0 && histogram.avg_ms < 160.0);
    }

    // Traces to: FR-ROUTER-010
    #[test]
    fn test_reset_metrics() {
        let collector = MetricsCollector::new();

        collector.record_request_start();
        collector.record_request_end(&RequestMetrics::new(
            "/api/users",
            200,
            150,
            "api",
            "http://localhost:3000",
        ));

        assert_eq!(collector.get_request_count(), 1);

        collector.reset();
        assert_eq!(collector.get_request_count(), 0);
        assert_eq!(collector.get_in_flight_count(), 0);
        assert_eq!(collector.get_status_counters().total, 0);
    }
}
