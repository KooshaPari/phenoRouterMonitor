//! Metrics tracking for routing decisions and backend performance.

use serde::{Deserialize, Serialize};

/// Metrics for routing operations.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RoutingMetrics {
    pub total_decisions: usize,
    pub successful_routes: usize,
    pub failed_routes: usize,
    pub route_not_found: usize,
}

impl RoutingMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_success(&mut self) {
        self.total_decisions += 1;
        self.successful_routes += 1;
    }

    pub fn record_failure(&mut self) {
        self.total_decisions += 1;
        self.failed_routes += 1;
    }

    pub fn record_not_found(&mut self) {
        self.total_decisions += 1;
        self.route_not_found += 1;
    }

    pub fn success_rate(&self) -> f64 {
        if self.total_decisions == 0 {
            return 0.0;
        }
        (self.successful_routes as f64 / self.total_decisions as f64) * 100.0
    }

    pub fn failure_rate(&self) -> f64 {
        if self.total_decisions == 0 {
            return 0.0;
        }
        (self.failed_routes as f64 / self.total_decisions as f64) * 100.0
    }
}

/// Metrics for a specific backend.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BackendMetrics {
    pub id: String,
    pub total_requests: usize,
    pub successful_requests: usize,
    pub failed_requests: usize,
    pub total_errors: usize,
}

impl BackendMetrics {
    pub fn new(id: String) -> Self {
        Self {
            id,
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            total_errors: 0,
        }
    }

    pub fn record_request(&mut self) {
        self.total_requests += 1;
    }

    pub fn record_success(&mut self) {
        self.successful_requests += 1;
    }

    pub fn record_failure(&mut self) {
        self.failed_requests += 1;
    }

    pub fn record_error(&mut self) {
        self.total_errors += 1;
    }

    pub fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            return 0.0;
        }
        (self.successful_requests as f64 / self.total_requests as f64) * 100.0
    }

    pub fn error_rate(&self) -> f64 {
        if self.total_requests == 0 {
            return 0.0;
        }
        (self.total_errors as f64 / self.total_requests as f64) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_routing_metrics_creation() {
        let metrics = RoutingMetrics::new();
        assert_eq!(metrics.total_decisions, 0);
        assert_eq!(metrics.successful_routes, 0);
    }

    #[test]
    fn test_routing_metrics_record_success() {
        let mut metrics = RoutingMetrics::new();
        metrics.record_success();
        metrics.record_success();

        assert_eq!(metrics.total_decisions, 2);
        assert_eq!(metrics.successful_routes, 2);
        assert_eq!(metrics.failed_routes, 0);
    }

    #[test]
    fn test_routing_metrics_record_failure() {
        let mut metrics = RoutingMetrics::new();
        metrics.record_failure();
        metrics.record_failure();

        assert_eq!(metrics.total_decisions, 2);
        assert_eq!(metrics.failed_routes, 2);
        assert_eq!(metrics.successful_routes, 0);
    }

    #[test]
    fn test_routing_metrics_success_rate() {
        let mut metrics = RoutingMetrics::new();
        metrics.record_success();
        metrics.record_success();
        metrics.record_failure();

        assert!((metrics.success_rate() - 66.666_67).abs() < 0.1);
    }

    #[test]
    fn test_routing_metrics_failure_rate() {
        let mut metrics = RoutingMetrics::new();
        metrics.record_success();
        metrics.record_failure();
        metrics.record_failure();

        assert!((metrics.failure_rate() - 66.666_67).abs() < 0.1);
    }

    #[test]
    fn test_routing_metrics_zero_decisions() {
        let metrics = RoutingMetrics::new();
        assert_eq!(metrics.success_rate(), 0.0);
        assert_eq!(metrics.failure_rate(), 0.0);
    }

    #[test]
    fn test_backend_metrics_creation() {
        let metrics = BackendMetrics::new("backend-1".to_string());
        assert_eq!(metrics.id, "backend-1");
        assert_eq!(metrics.total_requests, 0);
    }

    #[test]
    fn test_backend_metrics_record_request() {
        let mut metrics = BackendMetrics::new("backend-1".to_string());
        metrics.record_request();
        metrics.record_request();

        assert_eq!(metrics.total_requests, 2);
    }

    #[test]
    fn test_backend_metrics_record_success() {
        let mut metrics = BackendMetrics::new("backend-1".to_string());
        metrics.record_request();
        metrics.record_success();

        assert_eq!(metrics.total_requests, 1);
        assert_eq!(metrics.successful_requests, 1);
    }

    #[test]
    fn test_backend_metrics_record_failure() {
        let mut metrics = BackendMetrics::new("backend-1".to_string());
        metrics.record_request();
        metrics.record_failure();

        assert_eq!(metrics.total_requests, 1);
        assert_eq!(metrics.failed_requests, 1);
    }

    #[test]
    fn test_backend_metrics_record_error() {
        let mut metrics = BackendMetrics::new("backend-1".to_string());
        metrics.record_error();
        metrics.record_error();
        metrics.record_error();

        assert_eq!(metrics.total_errors, 3);
    }

    #[test]
    fn test_backend_metrics_success_rate() {
        let mut metrics = BackendMetrics::new("backend-1".to_string());
        for _ in 0..10 {
            metrics.record_request();
            metrics.record_success();
        }

        assert_eq!(metrics.success_rate(), 100.0);
    }

    #[test]
    fn test_backend_metrics_mixed_results() {
        let mut metrics = BackendMetrics::new("backend-1".to_string());
        for _ in 0..8 {
            metrics.record_request();
            metrics.record_success();
        }
        for _ in 0..2 {
            metrics.record_request();
            metrics.record_failure();
        }

        assert_eq!(metrics.success_rate(), 80.0);
        assert_eq!(metrics.failed_requests, 2);
    }

    #[test]
    fn test_backend_metrics_error_rate() {
        let mut metrics = BackendMetrics::new("backend-1".to_string());
        for _ in 0..10 {
            metrics.record_request();
        }
        for _ in 0..2 {
            metrics.record_error();
        }

        assert_eq!(metrics.error_rate(), 20.0);
    }

    #[test]
    fn test_backend_metrics_zero_requests() {
        let metrics = BackendMetrics::new("backend-1".to_string());
        assert_eq!(metrics.success_rate(), 0.0);
        assert_eq!(metrics.error_rate(), 0.0);
    }

    #[test]
    fn test_routing_metrics_serialization() {
        let mut metrics = RoutingMetrics::new();
        metrics.record_success();
        metrics.record_failure();

        let json = serde_json::to_string(&metrics).unwrap();
        let deserialized: RoutingMetrics = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.total_decisions, 2);
        assert_eq!(deserialized.successful_routes, 1);
        assert_eq!(deserialized.failed_routes, 1);
    }

    #[test]
    fn test_backend_metrics_serialization() {
        let mut metrics = BackendMetrics::new("backend-1".to_string());
        metrics.record_request();
        metrics.record_success();
        metrics.record_error();

        let json = serde_json::to_string(&metrics).unwrap();
        let deserialized: BackendMetrics = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, "backend-1");
        assert_eq!(deserialized.total_requests, 1);
        assert_eq!(deserialized.successful_requests, 1);
        assert_eq!(deserialized.total_errors, 1);
    }
}
