//! # Phenotype Router Metrics
//!
//! Thread-safe metrics collection for HTTP routers with Prometheus-compatible export format.
//!
//! This crate provides a high-performance, concurrent metrics collector that tracks:
//! - Request counts by endpoint, method, and status code
//! - Request latency with percentile calculations (p50, p95, p99)
//! - Response status code distribution
//!
//! ## Features
//!
//! - **Thread-safe**: Uses `Arc` + `DashMap` for lock-free concurrent access
//! - **Prometheus-compatible**: Export metrics in standard Prometheus text format
//! - **Percentile calculations**: p50, p95, p99 latency statistics
//! - **Zero-copy observability**: Snapshot metrics without blocking writers
//!
//! ## Usage
//!
//! ```rust
//! use phenotype_router_metrics::RouterMetrics;
//! use std::sync::Arc;
//! use std::time::Duration;
//!
//! # #[tokio::main]
//! # async fn main() {
//! let metrics = Arc::new(RouterMetrics::new());
//!
//! // Record a request
//! metrics.record_request(
//!     "GET",
//!     "/api/users",
//!     200,
//!     Duration::from_millis(50),
//! );
//!
//! // Get a snapshot
//! let snapshot = metrics.snapshot();
//! println!("{}", metrics.prometheus_format());
//! # }
//! ```

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Key for identifying a unique request route.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct RouteKey {
    /// HTTP method (GET, POST, PUT, DELETE, etc.)
    pub method: String,
    /// Request path
    pub path: String,
}

impl RouteKey {
    /// Create a new route key.
    pub fn new(method: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            method: method.into(),
            path: path.into(),
        }
    }
}

/// Status code bucket for aggregating responses.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct StatusCodeBucket {
    /// Status code range (e.g., 2xx, 4xx, 5xx)
    pub code: u16,
}

impl StatusCodeBucket {
    /// Create a new status code bucket.
    pub const fn new(code: u16) -> Self {
        Self { code }
    }

    /// Get the bucket category for a status code.
    pub fn from_code(code: u16) -> Self {
        Self { code }
    }
}

/// Metrics for a single route endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteMetrics {
    /// Total number of requests
    pub total_requests: u64,
    /// Request count by status code
    pub status_code_counts: BTreeMap<u16, u64>,
    /// All recorded latencies (in milliseconds)
    pub latencies_ms: Vec<u64>,
    /// Total latency sum (for average calculation)
    pub total_latency_ms: u64,
}

impl RouteMetrics {
    /// Create new empty metrics.
    pub fn new() -> Self {
        Self {
            total_requests: 0,
            status_code_counts: BTreeMap::new(),
            latencies_ms: Vec::new(),
            total_latency_ms: 0,
        }
    }

    /// Calculate average latency in milliseconds.
    pub fn avg_latency_ms(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            self.total_latency_ms as f64 / self.total_requests as f64
        }
    }

    /// Calculate percentile latency.
    pub fn percentile(&self, p: f64) -> Option<u64> {
        if self.latencies_ms.is_empty() {
            return None;
        }

        let index = ((p / 100.0) * (self.latencies_ms.len() as f64 - 1.0)).round() as usize;
        self.latencies_ms.get(index).copied()
    }

    /// Calculate p50 (median) latency.
    pub fn p50(&self) -> Option<u64> {
        self.percentile(50.0)
    }

    /// Calculate p95 latency.
    pub fn p95(&self) -> Option<u64> {
        self.percentile(95.0)
    }

    /// Calculate p99 latency.
    pub fn p99(&self) -> Option<u64> {
        self.percentile(99.0)
    }
}

impl Default for RouteMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Snapshot of metrics at a point in time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    /// Timestamp when snapshot was taken (Unix seconds)
    pub timestamp_secs: u64,
    /// Metrics by route
    pub routes: BTreeMap<RouteKey, RouteMetrics>,
    /// Total requests across all routes
    pub total_requests: u64,
}

impl MetricsSnapshot {
    /// Create a new snapshot.
    pub fn new(routes: BTreeMap<RouteKey, RouteMetrics>) -> Self {
        let total_requests = routes.values().map(|r| r.total_requests).sum();
        Self {
            timestamp_secs: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            routes,
            total_requests,
        }
    }

    /// Get metrics for a specific route.
    pub fn get(&self, route: &RouteKey) -> Option<&RouteMetrics> {
        self.routes.get(route)
    }

    /// List all routes with requests.
    pub fn routes(&self) -> impl Iterator<Item = (&RouteKey, &RouteMetrics)> {
        self.routes.iter()
    }
}

/// Thread-safe router metrics collector.
///
/// Uses `DashMap` for lock-free concurrent access to metrics.
pub struct RouterMetrics {
    routes: DashMap<RouteKey, RouteMetrics>,
    total_requests: AtomicU64,
}

impl RouterMetrics {
    /// Create a new metrics collector.
    pub fn new() -> Self {
        Self {
            routes: DashMap::new(),
            total_requests: AtomicU64::new(0),
        }
    }

    /// Record a request with its details.
    pub fn record_request(
        &self,
        method: impl Into<String>,
        path: impl Into<String>,
        status_code: u16,
        latency: Duration,
    ) {
        let route = RouteKey::new(method, path);
        let latency_ms = latency.as_millis() as u64;

        self.routes
            .entry(route)
            .and_modify(|metrics| {
                metrics.total_requests += 1;
                *metrics
                    .status_code_counts
                    .entry(status_code)
                    .or_insert(0) += 1;
                metrics.latencies_ms.push(latency_ms);
                metrics.total_latency_ms += latency_ms;
            })
            .or_insert_with(|| {
                let mut metrics = RouteMetrics::new();
                metrics.total_requests = 1;
                metrics.status_code_counts.insert(status_code, 1);
                metrics.latencies_ms.push(latency_ms);
                metrics.total_latency_ms = latency_ms;
                metrics
            });

        self.total_requests.fetch_add(1, Ordering::SeqCst);
    }

    /// Get total request count.
    pub fn total_requests(&self) -> u64 {
        self.total_requests.load(Ordering::SeqCst)
    }

    /// Get metrics for a specific route.
    pub fn get(&self, route: &RouteKey) -> Option<RouteMetrics> {
        self.routes.get(route).map(|r| r.clone())
    }

    /// Get a snapshot of current metrics.
    pub fn snapshot(&self) -> MetricsSnapshot {
        let routes = self
            .routes
            .iter()
            .map(|ref_multi| (ref_multi.key().clone(), ref_multi.value().clone()))
            .collect::<BTreeMap<_, _>>();

        MetricsSnapshot::new(routes)
    }

    /// Export metrics in Prometheus text format.
    pub fn prometheus_format(&self) -> String {
        let snapshot = self.snapshot();
        let mut output = String::new();

        // Help section
        output.push_str("# HELP router_requests_total Total requests by route and status\n");
        output.push_str("# TYPE router_requests_total counter\n");

        // Total requests per route and status
        for (route, metrics) in snapshot.routes() {
            for (status, count) in &metrics.status_code_counts {
                output.push_str(&format!(
                    "router_requests_total{{method=\"{}\",path=\"{}\",status=\"{}\"}} {}\n",
                    route.method, route.path, status, count
                ));
            }
        }

        // Latency metrics
        output.push_str("# HELP router_request_duration_ms Request duration in milliseconds\n");
        output.push_str("# TYPE router_request_duration_ms histogram\n");

        for (route, metrics) in snapshot.routes() {
            if metrics.total_requests > 0 {
                output.push_str(&format!(
                    "router_request_duration_ms{{method=\"{}\",path=\"{}\",quantile=\"0.5\"}} {}\n",
                    route.method,
                    route.path,
                    metrics.p50().unwrap_or(0)
                ));
                output.push_str(&format!(
                    "router_request_duration_ms{{method=\"{}\",path=\"{}\",quantile=\"0.95\"}} {}\n",
                    route.method,
                    route.path,
                    metrics.p95().unwrap_or(0)
                ));
                output.push_str(&format!(
                    "router_request_duration_ms{{method=\"{}\",path=\"{}\",quantile=\"0.99\"}} {}\n",
                    route.method,
                    route.path,
                    metrics.p99().unwrap_or(0)
                ));
                output.push_str(&format!(
                    "router_request_duration_ms{{method=\"{}\",path=\"{}\",quantile=\"mean\"}} {:.2}\n",
                    route.method,
                    route.path,
                    metrics.avg_latency_ms()
                ));
            }
        }

        // Total count
        output.push_str("# HELP router_total_requests Total requests across all routes\n");
        output.push_str("# TYPE router_total_requests gauge\n");
        output.push_str(&format!(
            "router_total_requests {}\n",
            snapshot.total_requests
        ));

        output
    }

    /// Reset all metrics.
    pub fn reset(&self) {
        self.routes.clear();
        self.total_requests.store(0, Ordering::SeqCst);
    }

    /// Get count of tracked routes.
    pub fn routes_count(&self) -> usize {
        self.routes.len()
    }
}

impl Default for RouterMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::time::Duration;

    // Traces to: FR-METRICS-001 (Basic metrics recording)
    #[test]
    fn test_record_single_request() {
        let metrics = RouterMetrics::new();
        metrics.record_request("GET", "/users", 200, Duration::from_millis(50));

        assert_eq!(metrics.total_requests(), 1);
        let route = RouteKey::new("GET", "/users");
        let route_metrics = metrics.get(&route).expect("route should exist");
        assert_eq!(route_metrics.total_requests, 1);
    }

    // Traces to: FR-METRICS-001 (Multiple requests)
    #[test]
    fn test_record_multiple_requests() {
        let metrics = RouterMetrics::new();

        for i in 0..100 {
            metrics.record_request(
                "GET",
                "/users",
                200,
                Duration::from_millis(10 + i),
            );
        }

        assert_eq!(metrics.total_requests(), 100);
    }

    // Traces to: FR-METRICS-002 (Status code tracking)
    #[test]
    fn test_status_code_tracking() {
        let metrics = RouterMetrics::new();
        let route = RouteKey::new("GET", "/users");

        metrics.record_request("GET", "/users", 200, Duration::from_millis(50));
        metrics.record_request("GET", "/users", 404, Duration::from_millis(30));
        metrics.record_request("GET", "/users", 500, Duration::from_millis(100));

        let route_metrics = metrics.get(&route).expect("route should exist");
        assert_eq!(route_metrics.status_code_counts.get(&200), Some(&1));
        assert_eq!(route_metrics.status_code_counts.get(&404), Some(&1));
        assert_eq!(route_metrics.status_code_counts.get(&500), Some(&1));
    }

    // Traces to: FR-METRICS-003 (Latency recording)
    #[test]
    fn test_latency_recording() {
        let metrics = RouterMetrics::new();
        let route = RouteKey::new("POST", "/api/data");

        metrics.record_request("POST", "/api/data", 201, Duration::from_millis(25));
        metrics.record_request("POST", "/api/data", 201, Duration::from_millis(35));
        metrics.record_request("POST", "/api/data", 201, Duration::from_millis(45));

        let route_metrics = metrics.get(&route).expect("route should exist");
        assert_eq!(route_metrics.latencies_ms.len(), 3);
        assert!(route_metrics.latencies_ms.contains(&25));
        assert!(route_metrics.latencies_ms.contains(&35));
        assert!(route_metrics.latencies_ms.contains(&45));
    }

    // Traces to: FR-METRICS-003 (Average latency)
    #[test]
    fn test_average_latency() {
        let metrics = RouterMetrics::new();
        let route = RouteKey::new("GET", "/api");

        metrics.record_request("GET", "/api", 200, Duration::from_millis(10));
        metrics.record_request("GET", "/api", 200, Duration::from_millis(20));
        metrics.record_request("GET", "/api", 200, Duration::from_millis(30));

        let route_metrics = metrics.get(&route).expect("route should exist");
        assert!((route_metrics.avg_latency_ms() - 20.0).abs() < 0.01);
    }

    // Traces to: FR-METRICS-004 (Percentile calculations - p50)
    #[test]
    fn test_percentile_p50() {
        let metrics = RouterMetrics::new();
        let route = RouteKey::new("GET", "/search");

        for i in 1..=100 {
            metrics.record_request(
                "GET",
                "/search",
                200,
                Duration::from_millis(i),
            );
        }

        let route_metrics = metrics.get(&route).expect("route should exist");
        let p50 = route_metrics.p50().expect("p50 should be calculated");
        assert!(p50 >= 40 && p50 <= 60, "p50 should be near median");
    }

    // Traces to: FR-METRICS-004 (Percentile calculations - p95)
    #[test]
    fn test_percentile_p95() {
        let metrics = RouterMetrics::new();
        let route = RouteKey::new("GET", "/search");

        for i in 1..=100 {
            metrics.record_request(
                "GET",
                "/search",
                200,
                Duration::from_millis(i),
            );
        }

        let route_metrics = metrics.get(&route).expect("route should exist");
        let p95 = route_metrics.p95().expect("p95 should be calculated");
        assert!(p95 >= 85 && p95 <= 99, "p95 should be near 95th percentile");
    }

    // Traces to: FR-METRICS-004 (Percentile calculations - p99)
    #[test]
    fn test_percentile_p99() {
        let metrics = RouterMetrics::new();
        let route = RouteKey::new("GET", "/search");

        for i in 1..=100 {
            metrics.record_request(
                "GET",
                "/search",
                200,
                Duration::from_millis(i),
            );
        }

        let route_metrics = metrics.get(&route).expect("route should exist");
        let p99 = route_metrics.p99().expect("p99 should be calculated");
        assert!(p99 >= 95, "p99 should be 95 or higher");
    }

    // Traces to: FR-METRICS-005 (Multiple routes)
    #[test]
    fn test_multiple_routes() {
        let metrics = RouterMetrics::new();

        metrics.record_request("GET", "/users", 200, Duration::from_millis(50));
        metrics.record_request("GET", "/users/123", 200, Duration::from_millis(40));
        metrics.record_request("POST", "/users", 201, Duration::from_millis(100));

        assert_eq!(metrics.routes_count(), 3);
        assert_eq!(metrics.total_requests(), 3);
    }

    // Traces to: FR-METRICS-006 (Snapshot)
    #[test]
    fn test_snapshot() {
        let metrics = RouterMetrics::new();

        metrics.record_request("GET", "/api", 200, Duration::from_millis(50));
        metrics.record_request("POST", "/api", 201, Duration::from_millis(100));

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.total_requests, 2);
        assert_eq!(snapshot.routes.len(), 2);
    }

    // Traces to: FR-METRICS-007 (Prometheus export format)
    #[test]
    fn test_prometheus_format() {
        let metrics = RouterMetrics::new();
        metrics.record_request("GET", "/users", 200, Duration::from_millis(50));

        let output = metrics.prometheus_format();
        assert!(output.contains("router_requests_total"));
        assert!(output.contains("router_request_duration_ms"));
        assert!(output.contains("router_total_requests"));
        assert!(output.contains("method=\"GET\""));
        assert!(output.contains("path=\"/users\""));
        assert!(output.contains("status=\"200\""));
    }

    // Traces to: FR-METRICS-007 (Prometheus histogram quantiles)
    #[test]
    fn test_prometheus_histogram_quantiles() {
        let metrics = RouterMetrics::new();

        for i in 1..=50 {
            metrics.record_request(
                "GET",
                "/api",
                200,
                Duration::from_millis(i * 2),
            );
        }

        let output = metrics.prometheus_format();
        assert!(output.contains("quantile=\"0.5\""));
        assert!(output.contains("quantile=\"0.95\""));
        assert!(output.contains("quantile=\"0.99\""));
        assert!(output.contains("quantile=\"mean\""));
    }

    // Traces to: FR-METRICS-008 (Thread safety with Arc)
    #[tokio::test]
    async fn test_concurrent_recording() {
        let metrics = Arc::new(RouterMetrics::new());
        let mut handles = vec![];

        for thread_id in 0..10 {
            let metrics_clone = Arc::clone(&metrics);
            let handle = tokio::spawn(async move {
                for i in 0..100 {
                    metrics_clone.record_request(
                        "GET",
                        "/users",
                        200,
                        Duration::from_millis(10 + i),
                    );
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await.expect("task should complete");
        }

        assert_eq!(metrics.total_requests(), 1000);
    }

    // Traces to: FR-METRICS-008 (Concurrent snapshot access)
    #[tokio::test]
    async fn test_concurrent_snapshot_access() {
        let metrics = Arc::new(RouterMetrics::new());

        // Populate metrics
        for i in 0..100 {
            metrics.record_request(
                "GET",
                "/api",
                200,
                Duration::from_millis(i % 100 + 1),
            );
        }

        let mut handles = vec![];

        for _ in 0..5 {
            let metrics_clone = Arc::clone(&metrics);
            let handle = tokio::spawn(async move {
                let snapshot = metrics_clone.snapshot();
                assert_eq!(snapshot.total_requests, 100);
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await.expect("task should complete");
        }
    }

    // Traces to: FR-METRICS-009 (Reset functionality)
    #[test]
    fn test_reset_metrics() {
        let metrics = RouterMetrics::new();

        metrics.record_request("GET", "/users", 200, Duration::from_millis(50));
        assert_eq!(metrics.total_requests(), 1);

        metrics.reset();
        assert_eq!(metrics.total_requests(), 0);
        assert_eq!(metrics.routes_count(), 0);
    }

    // Traces to: FR-METRICS-010 (Different HTTP methods)
    #[test]
    fn test_different_http_methods() {
        let metrics = RouterMetrics::new();

        metrics.record_request("GET", "/api", 200, Duration::from_millis(50));
        metrics.record_request("POST", "/api", 201, Duration::from_millis(100));
        metrics.record_request("PUT", "/api", 200, Duration::from_millis(75));
        metrics.record_request("DELETE", "/api", 204, Duration::from_millis(60));

        assert_eq!(metrics.routes_count(), 4);

        let get_route = RouteKey::new("GET", "/api");
        let post_route = RouteKey::new("POST", "/api");
        let put_route = RouteKey::new("PUT", "/api");
        let delete_route = RouteKey::new("DELETE", "/api");

        assert!(metrics.get(&get_route).is_some());
        assert!(metrics.get(&post_route).is_some());
        assert!(metrics.get(&put_route).is_some());
        assert!(metrics.get(&delete_route).is_some());
    }

    // Traces to: FR-METRICS-011 (Complex path tracking)
    #[test]
    fn test_complex_path_tracking() {
        let metrics = RouterMetrics::new();

        metrics.record_request("GET", "/api/v1/users/123/profile", 200, Duration::from_millis(50));
        metrics.record_request(
            "GET",
            "/api/v1/users/456/profile",
            200,
            Duration::from_millis(60),
        );

        assert_eq!(metrics.routes_count(), 2);
    }

    // Traces to: FR-METRICS-012 (Error status codes)
    #[test]
    fn test_error_status_codes() {
        let metrics = RouterMetrics::new();
        let route = RouteKey::new("GET", "/notfound");

        metrics.record_request("GET", "/notfound", 404, Duration::from_millis(20));
        metrics.record_request("GET", "/notfound", 404, Duration::from_millis(22));
        metrics.record_request("POST", "/error", 500, Duration::from_millis(200));

        let get_metrics = metrics.get(&route).expect("route should exist");
        assert_eq!(get_metrics.status_code_counts.get(&404), Some(&2));
    }

    // Traces to: FR-METRICS-013 (Serialization - RouteKey)
    #[test]
    fn test_route_key_serialization() {
        let route = RouteKey::new("GET", "/users");
        let json = serde_json::to_string(&route).expect("serialization should work");
        let deserialized: RouteKey = serde_json::from_str(&json).expect("deserialization should work");

        assert_eq!(route, deserialized);
    }

    // Traces to: FR-METRICS-013 (Serialization - RouteMetrics)
    #[test]
    fn test_route_metrics_serialization() {
        let metrics = RouterMetrics::new();
        metrics.record_request("GET", "/api", 200, Duration::from_millis(50));
        metrics.record_request("GET", "/api", 200, Duration::from_millis(100));

        let snapshot = metrics.snapshot();
        let json = serde_json::to_string(&snapshot).expect("serialization should work");
        let deserialized: MetricsSnapshot =
            serde_json::from_str(&json).expect("deserialization should work");

        assert_eq!(snapshot.total_requests, deserialized.total_requests);
    }

    // Traces to: FR-METRICS-014 (Empty metrics behavior)
    #[test]
    fn test_empty_metrics() {
        let metrics = RouterMetrics::new();
        assert_eq!(metrics.total_requests(), 0);
        assert_eq!(metrics.routes_count(), 0);

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.total_requests, 0);
        assert!(snapshot.routes.is_empty());
    }

    // Traces to: FR-METRICS-014 (Percentile on empty)
    #[test]
    fn test_percentile_empty() {
        let route_metrics = RouteMetrics::new();
        assert!(route_metrics.p50().is_none());
        assert!(route_metrics.p95().is_none());
        assert!(route_metrics.p99().is_none());
    }

    // Traces to: FR-METRICS-015 (Mixed latencies)
    #[test]
    fn test_mixed_latencies() {
        let metrics = RouterMetrics::new();

        // Add varying latencies
        let latencies = vec![10, 20, 30, 40, 50, 100, 150, 200];
        for latency_ms in latencies {
            metrics.record_request(
                "GET",
                "/api",
                200,
                Duration::from_millis(latency_ms),
            );
        }

        let route_metrics = metrics.get(&RouteKey::new("GET", "/api")).unwrap();
        assert!(route_metrics.p50().is_some());
        assert!(route_metrics.p95().is_some());
        assert!(route_metrics.p99().is_some());

        // p50 should be lower than p99
        assert!(route_metrics.p50().unwrap() <= route_metrics.p99().unwrap());
    }

    // Traces to: FR-METRICS-016 (Route key ordering)
    #[test]
    fn test_route_key_ordering() {
        let route1 = RouteKey::new("GET", "/a");
        let route2 = RouteKey::new("GET", "/b");
        let route3 = RouteKey::new("POST", "/a");

        assert!(route1 < route2);
        assert!(route1 < route3 || route3 < route1); // Ordering is consistent
    }

    // Traces to: FR-METRICS-017 (Large dataset handling)
    #[test]
    fn test_large_dataset() {
        let metrics = RouterMetrics::new();

        for i in 0..10000 {
            metrics.record_request(
                "GET",
                "/api",
                200,
                Duration::from_millis((i % 1000) as u64),
            );
        }

        assert_eq!(metrics.total_requests(), 10000);

        let route_metrics = metrics.get(&RouteKey::new("GET", "/api")).unwrap();
        assert_eq!(route_metrics.total_requests, 10000);
        assert!(route_metrics.p50().is_some());
    }

    // Traces to: FR-METRICS-018 (Status code edge cases)
    #[test]
    fn test_status_code_edge_cases() {
        let metrics = RouterMetrics::new();

        metrics.record_request("GET", "/api", 100, Duration::from_millis(50)); // 1xx
        metrics.record_request("GET", "/api", 204, Duration::from_millis(50)); // 2xx
        metrics.record_request("GET", "/api", 301, Duration::from_millis(50)); // 3xx
        metrics.record_request("GET", "/api", 400, Duration::from_millis(50)); // 4xx
        metrics.record_request("GET", "/api", 503, Duration::from_millis(50)); // 5xx

        let route_metrics = metrics.get(&RouteKey::new("GET", "/api")).unwrap();
        assert_eq!(route_metrics.status_code_counts.len(), 5);
    }

    // Traces to: FR-METRICS-019 (Get non-existent route)
    #[test]
    fn test_get_nonexistent_route() {
        let metrics = RouterMetrics::new();
        let route = RouteKey::new("GET", "/nonexistent");
        assert!(metrics.get(&route).is_none());
    }

    // Traces to: FR-METRICS-020 (Snapshot isolation)
    #[test]
    fn test_snapshot_isolation() {
        let metrics = RouterMetrics::new();
        metrics.record_request("GET", "/api", 200, Duration::from_millis(50));

        let snapshot1 = metrics.snapshot();
        assert_eq!(snapshot1.total_requests, 1);

        // Record more requests
        metrics.record_request("GET", "/api", 200, Duration::from_millis(60));

        let snapshot2 = metrics.snapshot();
        assert_eq!(snapshot2.total_requests, 2);

        // Original snapshot should not be affected
        assert_eq!(snapshot1.total_requests, 1);
    }
}
