//! Core router implementation
//!
//! Provides request routing, forwarding, and HTTP handling

use crate::backend::{BackendAddress, BackendPool, LoadBalancingStrategy};
use crate::error::{Result, RouterError};
use crate::loader::{ConfigLoader, RouterConfig};
use crate::patterns::PathPattern;
use dashmap::DashMap;
use std::sync::Arc;

/// Route configuration with compiled patterns
#[derive(Debug, Clone)]
pub struct CompiledRoute {
    pub service: String,
    pub pattern: PathPattern,
    pub pool: Arc<BackendPool>,
    pub timeout_ms: u64,
}

/// Core router engine
#[derive(Debug, Clone)]
pub struct Router {
    routes: Vec<Arc<CompiledRoute>>,
    route_map: Arc<DashMap<String, usize>>,
    config: Arc<RouterConfig>,
}

impl Router {
    /// Create a new empty router
    pub fn new() -> Self {
        Self {
            routes: Vec::new(),
            route_map: Arc::new(DashMap::new()),
            config: Arc::new(RouterConfig {
                listen_addr: "127.0.0.1".to_string(),
                listen_port: 3030,
                routes: vec![],
                max_body_size: 10 * 1024 * 1024,
                timeout_ms: 30000,
            }),
        }
    }

    /// Load router from configuration
    pub fn from_config(config: RouterConfig) -> Result<Self> {
        let mut routes = Vec::new();
        let route_map = DashMap::new();

        for (idx, route_cfg) in config.routes.iter().enumerate() {
            let pattern = if route_cfg.path_pattern.starts_with('^') {
                PathPattern::regex(&route_cfg.path_pattern)?
            } else if route_cfg.path_pattern.contains('*') {
                PathPattern::wildcard(&route_cfg.path_pattern)
            } else {
                PathPattern::exact(&route_cfg.path_pattern)
            };

            let backends: Vec<BackendAddress> = route_cfg
                .backends
                .iter()
                .map(|url| BackendAddress::new(url))
                .collect();

            let strategy = match route_cfg.strategy.to_lowercase().as_str() {
                "random" => LoadBalancingStrategy::Random,
                "leastconnections" => LoadBalancingStrategy::LeastConnections,
                _ => LoadBalancingStrategy::RoundRobin,
            };

            let pool = Arc::new(BackendPool::new(backends, strategy));

            let compiled = Arc::new(CompiledRoute {
                service: route_cfg.service.clone(),
                pattern,
                pool,
                timeout_ms: route_cfg.timeout_ms,
            });

            route_map.insert(route_cfg.service.clone(), idx);
            routes.push(compiled);
        }

        Ok(Self {
            routes,
            route_map: Arc::new(route_map),
            config: Arc::new(config),
        })
    }

    /// Load router from TOML file
    pub fn from_file(path: impl AsRef<std::path::Path>) -> Result<Self> {
        let config = ConfigLoader::from_file(path)?;
        Self::from_config(config)
    }

    /// Load router from TOML string
    pub fn from_string(content: &str) -> Result<Self> {
        let config = ConfigLoader::from_string(content)?;
        Self::from_config(config)
    }

    /// Load router from environment
    pub fn from_env() -> Result<Self> {
        let config = ConfigLoader::from_env()?;
        Self::from_config(config)
    }

    /// Find route matching the given path
    pub fn find_route(&self, path: &str) -> Result<Arc<CompiledRoute>> {
        for route in &self.routes {
            if route.pattern.matches(path) {
                return Ok(route.clone());
            }
        }
        Err(RouterError::RouteNotFound {
            path: path.to_string(),
        })
    }

    /// Get route by service name
    pub fn get_route_by_service(&self, service: &str) -> Result<Arc<CompiledRoute>> {
        if let Some(entry) = self.route_map.get(service) {
            let idx = *entry;
            Ok(self.routes[idx].clone())
        } else {
            Err(RouterError::RouteNotFound {
                path: format!("/{}", service),
            })
        }
    }

    /// Get number of configured routes
    pub fn routes_count(&self) -> usize {
        self.routes.len()
    }

    /// Get configuration
    pub fn config(&self) -> &RouterConfig {
        &self.config
    }

    /// Get all routes
    pub fn routes(&self) -> &[Arc<CompiledRoute>] {
        &self.routes
    }

    /// Get listen socket address
    pub fn socket_addr(&self) -> String {
        self.config.socket_addr()
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_router_from_string() {
        let toml = r#"
listen_addr = "127.0.0.1"
listen_port = 3030

[[routes]]
service = "api"
path_pattern = "^/api/.*"
backends = ["http://localhost:3000"]
timeout_ms = 30000
strategy = "roundrobin"
"#;
        let router = Router::from_string(toml);
        assert!(router.is_ok());
        let r = router.unwrap();
        assert_eq!(r.routes_count(), 1);
    }

    #[test]
    fn test_router_find_route_by_path() {
        let toml = r#"
[[routes]]
service = "api"
path_pattern = "^/api/.*"
backends = ["http://localhost:3000"]
timeout_ms = 30000
strategy = "roundrobin"

[[routes]]
service = "web"
path_pattern = "^/web/.*"
backends = ["http://localhost:8080"]
timeout_ms = 30000
strategy = "roundrobin"
"#;
        let router = Router::from_string(toml).unwrap();

        let route1 = router.find_route("/api/users");
        assert!(route1.is_ok());
        assert_eq!(route1.unwrap().service, "api");

        let route2 = router.find_route("/web/home");
        assert!(route2.is_ok());
        assert_eq!(route2.unwrap().service, "web");

        let route3 = router.find_route("/unknown/path");
        assert!(route3.is_err());
    }

    #[test]
    fn test_router_get_route_by_service() {
        let toml = r#"
[[routes]]
service = "api"
path_pattern = "^/api/.*"
backends = ["http://localhost:3000"]
timeout_ms = 30000
strategy = "roundrobin"
"#;
        let router = Router::from_string(toml).unwrap();

        let route = router.get_route_by_service("api");
        assert!(route.is_ok());
        assert_eq!(route.unwrap().service, "api");

        let missing = router.get_route_by_service("unknown");
        assert!(missing.is_err());
    }

    #[test]
    fn test_router_multiple_backends() {
        let toml = r#"
[[routes]]
service = "api"
path_pattern = "^/api/.*"
backends = [
  "http://backend1:3000",
  "http://backend2:3000",
  "http://backend3:3000"
]
timeout_ms = 30000
strategy = "roundrobin"
"#;
        let router = Router::from_string(toml).unwrap();
        let route = router.find_route("/api/test").unwrap();
        assert_eq!(route.pool.len(), 3);
    }

    #[test]
    fn test_router_socket_addr() {
        let toml = r#"
listen_addr = "0.0.0.0"
listen_port = 8080

[[routes]]
service = "api"
path_pattern = "^/api/.*"
backends = ["http://localhost:3000"]
timeout_ms = 30000
strategy = "roundrobin"
"#;
        let router = Router::from_string(toml).unwrap();
        assert_eq!(router.socket_addr(), "0.0.0.0:8080");
    }

    #[test]
    fn test_router_load_balancing_strategy() {
        let toml = r#"
[[routes]]
service = "api"
path_pattern = "^/api/.*"
backends = ["http://localhost:3000"]
timeout_ms = 30000
strategy = "random"
"#;
        let router = Router::from_string(toml).unwrap();
        let route = router.find_route("/api/test").unwrap();
        assert!(route.pool.len() > 0);
    }
}
