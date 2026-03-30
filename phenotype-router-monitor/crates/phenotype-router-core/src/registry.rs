//! Route registry for managing and querying routes.

use crate::error::{RouterError, RouterResult};
use crate::matcher::{ExactMatcher, RegexMatcher, WildcardMatcher, MatcherStrategy};
use crate::route::{Backend, Route, RouteConfig};
use std::collections::BTreeMap;
use std::sync::RwLock;

/// Registry for managing routes with priority-based lookup.
pub struct RouteRegistry {
    routes: RwLock<BTreeMap<usize, Vec<Route>>>,
}

impl RouteRegistry {
    pub fn new() -> Self {
        Self {
            routes: RwLock::new(BTreeMap::new()),
        }
    }

    /// Register a new route with configuration.
    pub fn register_route(
        &self,
        config: RouteConfig,
        backends: Vec<Backend>,
    ) -> RouterResult<()> {
        // Validate backends
        if backends.is_empty() {
            return Err(RouterError::EmptyBackendList(config.id.clone()));
        }

        // Create matcher based on type
        let matcher: Box<dyn MatcherStrategy> = match config.matcher_type.as_str() {
            "exact" => Box::new(ExactMatcher::new(config.pattern.clone())?),
            "wildcard" => Box::new(WildcardMatcher::new(config.pattern.clone())?),
            "regex" => Box::new(RegexMatcher::new(config.pattern.clone())?),
            _ => {
                return Err(RouterError::InvalidMatcher(format!(
                    "Unknown matcher type: {}",
                    config.matcher_type
                )))
            }
        };

        // Create route
        let route = Route::new(config.clone(), matcher, backends);

        // Add to registry by priority (higher priority first)
        let priority = route.priority();
        let mut routes = self.routes.write().unwrap();

        // Check for duplicates
        if let Some(routes_at_priority) = routes.get(&priority) {
            if routes_at_priority.iter().any(|r| r.id() == config.id) {
                return Err(RouterError::DuplicateRoute(config.id));
            }
        }

        routes
            .entry(priority)
            .or_default()
            .push(route);

        Ok(())
    }

    /// Find and return backends for a matching route.
    /// Routes are checked in priority order (highest first).
    pub fn find_route(&self, path: &str) -> RouterResult<(String, Vec<Backend>)> {
        let routes = self.routes.read().unwrap();

        // Iterate from highest priority to lowest
        for (_priority, routes_at_priority) in routes.iter().rev() {
            for route in routes_at_priority {
                if route.matcher().matches(path) {
                    let route_id = route.id().to_string();
                    let backends = route.backends().to_vec();
                    return Ok((route_id, backends));
                }
            }
        }

        Err(RouterError::RouteNotFound(path.to_string()))
    }

    /// Get all registered routes.
    pub fn all_routes(&self) -> Vec<(String, String)> {
        let routes = self.routes.read().unwrap();
        let mut result = vec![];

        for (_priority, routes_at_priority) in routes.iter().rev() {
            for route in routes_at_priority {
                result.push((route.id().to_string(), route.name().to_string()));
            }
        }

        result
    }

    /// Get a route by ID.
    pub fn get_route_by_id(&self, id: &str) -> RouterResult<()> {
        let routes = self.routes.read().unwrap();

        for (_priority, routes_at_priority) in routes.iter() {
            if routes_at_priority.iter().any(|r| r.id() == id) {
                return Ok(());
            }
        }

        Err(RouterError::RouteNotFound(id.to_string()))
    }

    /// Remove a route by ID.
    pub fn remove_route(&self, id: &str) -> RouterResult<()> {
        let mut routes = self.routes.write().unwrap();

        for routes_at_priority in routes.values_mut() {
            if let Some(pos) = routes_at_priority.iter().position(|r| r.id() == id) {
                routes_at_priority.remove(pos);
                return Ok(());
            }
        }

        Err(RouterError::RouteNotFound(id.to_string()))
    }

    /// Get the number of routes.
    pub fn route_count(&self) -> usize {
        let routes = self.routes.read().unwrap();
        routes.values().map(|v| v.len()).sum()
    }

    /// Clear all routes.
    pub fn clear(&self) {
        let mut routes = self.routes.write().unwrap();
        routes.clear();
    }
}

impl Default for RouteRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_route(id: &str) -> RouteConfig {
        RouteConfig {
            id: id.to_string(),
            name: format!("Route {}", id),
            matcher_type: "exact".to_string(),
            pattern: format!("/{}", id),
            balancer_type: "round-robin".to_string(),
            backends: vec!["backend-1".to_string()],
            priority: None,
        }
    }

    #[test]
    fn test_registry_creation() {
        let registry = RouteRegistry::new();
        assert_eq!(registry.route_count(), 0);
    }

    #[test]
    fn test_register_single_route() {
        let registry = RouteRegistry::new();
        let config = create_test_route("api");
        let backends = vec![Backend::new("backend-1".to_string(), "http://localhost:3000".to_string())];

        registry.register_route(config, backends).unwrap();
        assert_eq!(registry.route_count(), 1);
    }

    #[test]
    fn test_register_multiple_routes() {
        let registry = RouteRegistry::new();

        for i in 0..5 {
            let config = create_test_route(&format!("api{}", i));
            let backends = vec![Backend::new("backend-1".to_string(), "http://localhost:3000".to_string())];
            registry.register_route(config, backends).unwrap();
        }

        assert_eq!(registry.route_count(), 5);
    }

    #[test]
    fn test_find_exact_route() {
        let registry = RouteRegistry::new();
        let config = RouteConfig {
            id: "api".to_string(),
            name: "API".to_string(),
            matcher_type: "exact".to_string(),
            pattern: "/api/v1/users".to_string(),
            balancer_type: "round-robin".to_string(),
            backends: vec!["backend-1".to_string()],
            priority: None,
        };
        let backends = vec![Backend::new("backend-1".to_string(), "http://localhost:3000".to_string())];

        registry.register_route(config, backends).unwrap();
        let (route_id, _backends) = registry.find_route("/api/v1/users").unwrap();
        assert_eq!(route_id, "api");
    }

    #[test]
    fn test_find_wildcard_route() {
        let registry = RouteRegistry::new();
        let config = RouteConfig {
            id: "api".to_string(),
            name: "API".to_string(),
            matcher_type: "wildcard".to_string(),
            pattern: "/api/*".to_string(),
            balancer_type: "round-robin".to_string(),
            backends: vec!["backend-1".to_string()],
            priority: None,
        };
        let backends = vec![Backend::new("backend-1".to_string(), "http://localhost:3000".to_string())];

        registry.register_route(config, backends).unwrap();
        let (route_id, _backends) = registry.find_route("/api/v1/users").unwrap();
        assert_eq!(route_id, "api");
    }

    #[test]
    fn test_find_regex_route() {
        let registry = RouteRegistry::new();
        let config = RouteConfig {
            id: "api".to_string(),
            name: "API".to_string(),
            matcher_type: "regex".to_string(),
            pattern: "^/api/v[0-9]+/.*".to_string(),
            balancer_type: "round-robin".to_string(),
            backends: vec!["backend-1".to_string()],
            priority: None,
        };
        let backends = vec![Backend::new("backend-1".to_string(), "http://localhost:3000".to_string())];

        registry.register_route(config, backends).unwrap();
        let (route_id, _backends) = registry.find_route("/api/v1/users").unwrap();
        assert_eq!(route_id, "api");
    }

    #[test]
    fn test_find_route_not_found() {
        let registry = RouteRegistry::new();
        let config = create_test_route("api");
        let backends = vec![Backend::new("backend-1".to_string(), "http://localhost:3000".to_string())];

        registry.register_route(config, backends).unwrap();
        assert!(registry.find_route("/notfound").is_err());
    }

    #[test]
    fn test_priority_ordering() {
        let registry = RouteRegistry::new();

        // Register low priority route
        let low_priority = RouteConfig {
            id: "low".to_string(),
            name: "Low".to_string(),
            matcher_type: "wildcard".to_string(),
            pattern: "/api/*".to_string(),
            balancer_type: "round-robin".to_string(),
            backends: vec!["backend-1".to_string()],
            priority: Some(1),
        };

        // Register high priority route
        let high_priority = RouteConfig {
            id: "high".to_string(),
            name: "High".to_string(),
            matcher_type: "exact".to_string(),
            pattern: "/api/v1/special".to_string(),
            balancer_type: "round-robin".to_string(),
            backends: vec!["backend-2".to_string()],
            priority: Some(10),
        };

        let backends = vec![Backend::new("backend-1".to_string(), "http://localhost:3000".to_string())];
        let backends2 = vec![Backend::new("backend-2".to_string(), "http://localhost:3001".to_string())];

        registry.register_route(low_priority, backends).unwrap();
        registry.register_route(high_priority, backends2).unwrap();

        // Should match high priority route first
        let (route_id, _backends) = registry.find_route("/api/v1/special").unwrap();
        assert_eq!(route_id, "high");
    }

    #[test]
    fn test_duplicate_route_error() {
        let registry = RouteRegistry::new();
        let config = create_test_route("api");
        let backends = vec![Backend::new("backend-1".to_string(), "http://localhost:3000".to_string())];

        registry.register_route(config.clone(), backends.clone()).unwrap();
        assert!(registry.register_route(config, backends).is_err());
    }

    #[test]
    fn test_empty_backends_error() {
        let registry = RouteRegistry::new();
        let config = create_test_route("api");

        assert!(registry.register_route(config, vec![]).is_err());
    }

    #[test]
    fn test_remove_route() {
        let registry = RouteRegistry::new();
        let config = create_test_route("api");
        let backends = vec![Backend::new("backend-1".to_string(), "http://localhost:3000".to_string())];

        registry.register_route(config, backends).unwrap();
        assert_eq!(registry.route_count(), 1);

        registry.remove_route("api").unwrap();
        assert_eq!(registry.route_count(), 0);
    }

    #[test]
    fn test_remove_nonexistent_route() {
        let registry = RouteRegistry::new();
        assert!(registry.remove_route("nonexistent").is_err());
    }

    #[test]
    fn test_clear_routes() {
        let registry = RouteRegistry::new();

        for i in 0..5 {
            let config = create_test_route(&format!("api{}", i));
            let backends = vec![Backend::new("backend-1".to_string(), "http://localhost:3000".to_string())];
            registry.register_route(config, backends).unwrap();
        }

        assert_eq!(registry.route_count(), 5);
        registry.clear();
        assert_eq!(registry.route_count(), 0);
    }

    #[test]
    fn test_all_routes() {
        let registry = RouteRegistry::new();

        for i in 0..3 {
            let config = create_test_route(&format!("api{}", i));
            let backends = vec![Backend::new("backend-1".to_string(), "http://localhost:3000".to_string())];
            registry.register_route(config, backends).unwrap();
        }

        let all = registry.all_routes();
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn test_get_route_by_id() {
        let registry = RouteRegistry::new();
        let config = create_test_route("api");
        let backends = vec![Backend::new("backend-1".to_string(), "http://localhost:3000".to_string())];

        registry.register_route(config, backends).unwrap();
        assert!(registry.get_route_by_id("api").is_ok());
        assert!(registry.get_route_by_id("nonexistent").is_err());
    }

    #[test]
    fn test_default_registry() {
        let registry = RouteRegistry::default();
        assert_eq!(registry.route_count(), 0);
    }

    #[test]
    fn test_registry_thread_safe() {
        let registry = std::sync::Arc::new(RouteRegistry::new());
        let mut handles = vec![];

        for i in 0..5 {
            let reg = registry.clone();
            let handle = std::thread::spawn(move || {
                let config = RouteConfig {
                    id: format!("api{}", i),
                    name: format!("API {}", i),
                    matcher_type: "exact".to_string(),
                    pattern: format!("/api{}", i),
                    balancer_type: "round-robin".to_string(),
                    backends: vec![format!("backend-{}", i)],
                    priority: None,
                };
                let backends = vec![Backend::new(
                    format!("backend-{}", i),
                    format!("http://localhost:{}", 3000 + i),
                )];
                reg.register_route(config, backends).unwrap();
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(registry.route_count(), 5);
    }

    #[test]
    fn test_invalid_matcher_type() {
        let registry = RouteRegistry::new();
        let config = RouteConfig {
            id: "api".to_string(),
            name: "API".to_string(),
            matcher_type: "invalid".to_string(),
            pattern: "/api".to_string(),
            balancer_type: "round-robin".to_string(),
            backends: vec!["backend-1".to_string()],
            priority: None,
        };
        let backends = vec![Backend::new("backend-1".to_string(), "http://localhost:3000".to_string())];

        assert!(registry.register_route(config, backends).is_err());
    }
}
