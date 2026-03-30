//! Core router implementation with load balancing.

use phenotype_router_core::{
    Backend, LoadBalancingStrategy, RoundRobin, RouteRegistry, RouterResult,
    RoutingMetrics,
};
use std::sync::Arc;

/// Main router for HTTP request distribution.
pub struct Router {
    registry: Arc<RouteRegistry>,
    balancer: Box<dyn LoadBalancingStrategy>,
    metrics: Arc<std::sync::Mutex<RoutingMetrics>>,
}

impl Router {
    pub fn new(registry: Arc<RouteRegistry>) -> Self {
        Self {
            registry,
            balancer: Box::new(RoundRobin::new()),
            metrics: Arc::new(std::sync::Mutex::new(RoutingMetrics::new())),
        }
    }

    pub fn with_balancer(
        registry: Arc<RouteRegistry>,
        balancer: Box<dyn LoadBalancingStrategy>,
    ) -> Self {
        Self {
            registry,
            balancer,
            metrics: Arc::new(std::sync::Mutex::new(RoutingMetrics::new())),
        }
    }

    /// Route a request to a backend.
    pub fn route(&self, path: &str) -> RouterResult<Backend> {
        // Find matching route and get backends
        let (_route_id, backends) = self.registry.find_route(path)?;

        // Select backend using load balancer
        let backend = self.balancer.select(&backends)?;

        // Record metrics
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.record_success();
        }

        Ok(backend.clone())
    }

    /// Get current metrics.
    pub fn metrics(&self) -> RoutingMetrics {
        self.metrics
            .lock()
            .map(|m| m.clone())
            .unwrap_or_default()
    }

    /// Get the route registry.
    pub fn registry(&self) -> &Arc<RouteRegistry> {
        &self.registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use phenotype_router_core::{RouteConfig, Random};

    #[test]
    fn test_router_creation() {
        let registry = Arc::new(RouteRegistry::new());
        let router = Router::new(registry);
        assert_eq!(router.metrics().total_decisions, 0);
    }

    #[test]
    fn test_router_with_custom_balancer() {
        let registry = Arc::new(RouteRegistry::new());
        let balancer = Box::new(Random::new());
        let router = Router::with_balancer(registry, balancer);
        assert_eq!(router.metrics().total_decisions, 0);
    }

    #[test]
    fn test_router_no_matching_route() {
        let registry = Arc::new(RouteRegistry::new());
        let router = Router::new(registry);

        let result = router.route("/api/users");
        assert!(result.is_err());
    }

    #[test]
    fn test_router_successful_route() {
        let registry = Arc::new(RouteRegistry::new());
        let config = RouteConfig {
            id: "api".to_string(),
            name: "API".to_string(),
            matcher_type: "exact".to_string(),
            pattern: "/api/users".to_string(),
            balancer_type: "round-robin".to_string(),
            backends: vec!["backend-1".to_string()],
            priority: None,
        };
        let backends = vec![Backend::new(
            "backend-1".to_string(),
            "http://localhost:3000".to_string(),
        )];

        registry.register_route(config, backends).unwrap();
        let router = Router::new(registry);

        let result = router.route("/api/users");
        assert!(result.is_ok());

        let backend = result.unwrap();
        assert_eq!(backend.id(), "backend-1");
    }

    #[test]
    fn test_router_metrics_recording() {
        let registry = Arc::new(RouteRegistry::new());
        let config = RouteConfig {
            id: "api".to_string(),
            name: "API".to_string(),
            matcher_type: "exact".to_string(),
            pattern: "/api/users".to_string(),
            balancer_type: "round-robin".to_string(),
            backends: vec!["backend-1".to_string()],
            priority: None,
        };
        let backends = vec![Backend::new(
            "backend-1".to_string(),
            "http://localhost:3000".to_string(),
        )];

        registry.register_route(config, backends).unwrap();
        let router = Router::new(registry);

        for _ in 0..5 {
            router.route("/api/users").ok();
        }

        let metrics = router.metrics();
        assert_eq!(metrics.total_decisions, 5);
        assert_eq!(metrics.successful_routes, 5);
    }
}
