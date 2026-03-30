//! Route and Backend definitions.

use crate::matcher::MatcherStrategy;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Configuration for a route.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteConfig {
    /// Unique identifier for the route.
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Matcher strategy type: "exact", "wildcard", or "regex".
    pub matcher_type: String,
    /// Pattern for the matcher.
    pub pattern: String,
    /// Load balancing strategy: "round-robin", "random", or "least-connections".
    pub balancer_type: String,
    /// List of backend identifiers.
    pub backends: Vec<String>,
    /// Optional priority (higher priority routes are checked first).
    pub priority: Option<usize>,
}

/// A backend server configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Backend {
    id: String,
    url: String,
    #[serde(skip)]
    active_connections: Arc<AtomicUsize>,
    #[serde(skip)]
    total_requests: Arc<AtomicUsize>,
    #[serde(skip)]
    total_errors: Arc<AtomicUsize>,
    healthy: Arc<std::sync::Mutex<bool>>,
}

impl Backend {
    /// Create a new backend.
    pub fn new(id: String, url: String) -> Self {
        Self {
            id,
            url,
            active_connections: Arc::new(AtomicUsize::new(0)),
            total_requests: Arc::new(AtomicUsize::new(0)),
            total_errors: Arc::new(AtomicUsize::new(0)),
            healthy: Arc::new(std::sync::Mutex::new(true)),
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    /// Increment active connection count.
    pub fn add_connection(&self) {
        self.active_connections.fetch_add(1, Ordering::Relaxed);
    }

    /// Decrement active connection count.
    pub fn remove_connection(&self) {
        if self.active_connections.load(Ordering::Relaxed) > 0 {
            self.active_connections.fetch_sub(1, Ordering::Relaxed);
        }
    }

    /// Get current active connection count.
    pub fn active_connections(&self) -> usize {
        self.active_connections.load(Ordering::Relaxed)
    }

    /// Increment total request count.
    pub fn record_request(&self) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment error count.
    pub fn record_error(&self) {
        self.total_errors.fetch_add(1, Ordering::Relaxed);
    }

    /// Get total requests served.
    pub fn total_requests(&self) -> usize {
        self.total_requests.load(Ordering::Relaxed)
    }

    /// Get total errors.
    pub fn total_errors(&self) -> usize {
        self.total_errors.load(Ordering::Relaxed)
    }

    /// Mark backend as healthy.
    pub fn set_healthy(&self, healthy: bool) {
        if let Ok(mut h) = self.healthy.lock() {
            *h = healthy;
        }
    }

    /// Check if backend is healthy.
    pub fn is_healthy(&self) -> bool {
        self.healthy.lock().map(|h| *h).unwrap_or(true)
    }

    /// Reset all counters (useful for testing).
    pub fn reset_counters(&self) {
        self.active_connections.store(0, Ordering::Relaxed);
        self.total_requests.store(0, Ordering::Relaxed);
        self.total_errors.store(0, Ordering::Relaxed);
    }
}

/// A route with matcher and backends.
pub struct Route {
    config: RouteConfig,
    matcher: Box<dyn MatcherStrategy>,
    backends: Vec<Backend>,
}

impl Route {
    pub fn new(
        config: RouteConfig,
        matcher: Box<dyn MatcherStrategy>,
        backends: Vec<Backend>,
    ) -> Self {
        Self {
            config,
            matcher,
            backends,
        }
    }

    pub fn id(&self) -> &str {
        &self.config.id
    }

    pub fn name(&self) -> &str {
        &self.config.name
    }

    pub fn priority(&self) -> usize {
        self.config.priority.unwrap_or(0)
    }

    pub fn matcher(&self) -> &dyn MatcherStrategy {
        self.matcher.as_ref()
    }

    pub fn backends(&self) -> &[Backend] {
        &self.backends
    }

    pub fn config(&self) -> &RouteConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_creation() {
        let backend = Backend::new("api-1".to_string(), "http://localhost:3000".to_string());
        assert_eq!(backend.id(), "api-1");
        assert_eq!(backend.url(), "http://localhost:3000");
        assert!(backend.is_healthy());
    }

    #[test]
    fn test_backend_connection_tracking() {
        let backend = Backend::new("api-1".to_string(), "http://localhost:3000".to_string());
        assert_eq!(backend.active_connections(), 0);

        backend.add_connection();
        assert_eq!(backend.active_connections(), 1);

        backend.add_connection();
        assert_eq!(backend.active_connections(), 2);

        backend.remove_connection();
        assert_eq!(backend.active_connections(), 1);
    }

    #[test]
    fn test_backend_request_tracking() {
        let backend = Backend::new("api-1".to_string(), "http://localhost:3000".to_string());
        assert_eq!(backend.total_requests(), 0);

        backend.record_request();
        assert_eq!(backend.total_requests(), 1);

        backend.record_request();
        backend.record_request();
        assert_eq!(backend.total_requests(), 3);
    }

    #[test]
    fn test_backend_error_tracking() {
        let backend = Backend::new("api-1".to_string(), "http://localhost:3000".to_string());
        assert_eq!(backend.total_errors(), 0);

        backend.record_error();
        assert_eq!(backend.total_errors(), 1);

        backend.record_error();
        assert_eq!(backend.total_errors(), 2);
    }

    #[test]
    fn test_backend_health_status() {
        let backend = Backend::new("api-1".to_string(), "http://localhost:3000".to_string());
        assert!(backend.is_healthy());

        backend.set_healthy(false);
        assert!(!backend.is_healthy());

        backend.set_healthy(true);
        assert!(backend.is_healthy());
    }

    #[test]
    fn test_backend_reset_counters() {
        let backend = Backend::new("api-1".to_string(), "http://localhost:3000".to_string());

        backend.add_connection();
        backend.add_connection();
        backend.record_request();
        backend.record_request();
        backend.record_request();
        backend.record_error();

        assert_eq!(backend.active_connections(), 2);
        assert_eq!(backend.total_requests(), 3);
        assert_eq!(backend.total_errors(), 1);

        backend.reset_counters();

        assert_eq!(backend.active_connections(), 0);
        assert_eq!(backend.total_requests(), 0);
        assert_eq!(backend.total_errors(), 0);
    }

    #[test]
    fn test_backend_connection_underflow() {
        let backend = Backend::new("api-1".to_string(), "http://localhost:3000".to_string());

        // Try to remove when empty
        backend.remove_connection();
        assert_eq!(backend.active_connections(), 0);
    }

    #[test]
    fn test_backend_clone() {
        let backend1 = Backend::new("api-1".to_string(), "http://localhost:3000".to_string());
        backend1.add_connection();
        backend1.record_request();

        let backend2 = backend1.clone();

        // Should share counters
        assert_eq!(backend2.active_connections(), 1);
        assert_eq!(backend2.total_requests(), 1);

        backend2.add_connection();
        assert_eq!(backend1.active_connections(), 2);
    }

    #[test]
    fn test_route_config_creation() {
        let config = RouteConfig {
            id: "api-route".to_string(),
            name: "API Routes".to_string(),
            matcher_type: "prefix".to_string(),
            pattern: "/api/*".to_string(),
            balancer_type: "round-robin".to_string(),
            backends: vec!["backend-1".to_string(), "backend-2".to_string()],
            priority: Some(10),
        };

        assert_eq!(config.id, "api-route");
        assert_eq!(config.name, "API Routes");
        assert_eq!(config.priority, Some(10));
    }

    #[test]
    fn test_backend_metrics() {
        let backend = Backend::new("api-1".to_string(), "http://localhost:3000".to_string());

        for _ in 0..100 {
            backend.record_request();
        }
        for _ in 0..5 {
            backend.record_error();
        }

        assert_eq!(backend.total_requests(), 100);
        assert_eq!(backend.total_errors(), 5);
    }

    #[test]
    fn test_backend_thread_safe_counters() {
        let backend = std::sync::Arc::new(Backend::new(
            "api-1".to_string(),
            "http://localhost:3000".to_string(),
        ));

        let mut handles = vec![];
        for _ in 0..10 {
            let b = backend.clone();
            let handle = std::thread::spawn(move || {
                for _ in 0..10 {
                    b.add_connection();
                    b.record_request();
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(backend.active_connections(), 100);
        assert_eq!(backend.total_requests(), 100);
    }
}
