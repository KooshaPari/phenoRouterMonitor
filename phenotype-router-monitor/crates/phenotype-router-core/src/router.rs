//! HTTP/Path-based router with load balancing
//!
//! This module provides path-based request routing with round-robin
//! load balancing across backend pools, complementing the Pareto
//! routing system for decision-making.

use crate::backend::BackendPool;
use crate::error::{RouterError, Result};
use crate::patterns::PathPattern;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;

/// HTTP request routing entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteEntry {
    /// Path pattern (regex or wildcard)
    pub pattern: PathPattern,
    /// Backend pool to route to
    pub pool_name: String,
}

/// Simple HTTP router for path-based routing with load balancing
pub struct Router {
    routes: Arc<RwLock<Vec<RouteEntry>>>,
    backends: Arc<RwLock<HashMap<String, BackendPool>>>,
}

impl Router {
    /// Create a new router
    pub fn new() -> Self {
        Self {
            routes: Arc::new(RwLock::new(Vec::new())),
            backends: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a route
    pub fn register_route(&self, pattern: PathPattern, pool_name: String) -> Result<()> {
        let mut routes = self.routes.write();
        routes.push(RouteEntry { pattern, pool_name });
        Ok(())
    }

    /// Register a backend pool
    pub fn register_backend(&self, name: String, pool: BackendPool) -> Result<()> {
        let mut backends = self.backends.write();
        backends.insert(name, pool);
        Ok(())
    }

    /// Get the number of registered routes
    pub fn routes_count(&self) -> usize {
        self.routes.read().len()
    }

    /// Find a route for the given path
    pub fn match_route(&self, path: &str) -> Option<String> {
        let routes = self.routes.read();
        for entry in routes.iter() {
            if entry.pattern.matches(path) {
                return Some(entry.pool_name.clone());
            }
        }
        None
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
    fn test_router_creation() {
        let router = Router::new();
        assert_eq!(router.routes_count(), 0);
    }

    #[test]
    fn test_register_route() {
        let router = Router::new();
        let pattern = PathPattern::Literal("/health".to_string());
        router
            .register_route(pattern, "default".to_string())
            .expect("register_route failed");
        assert_eq!(router.routes_count(), 1);
    }
}
