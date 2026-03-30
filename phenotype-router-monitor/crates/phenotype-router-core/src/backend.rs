//! Backend management and load balancing strategies
//!
//! Provides:
//! - Backend address management
//! - Round-robin load balancing
//! - Health status tracking
//! - Connection pooling support

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Supported load balancing strategies
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LoadBalancingStrategy {
    /// Round-robin distribution
    RoundRobin,

    /// Random selection
    Random,

    /// Least connections (requires tracking)
    LeastConnections,
}

impl Default for LoadBalancingStrategy {
    fn default() -> Self {
        LoadBalancingStrategy::RoundRobin
    }
}

/// Backend address
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackendAddress {
    /// Host and port (e.g., "http://localhost:3000")
    pub url: String,

    /// Optional weight for weighted round-robin
    #[serde(default)]
    pub weight: usize,

    /// Optional maximum concurrent connections
    #[serde(default)]
    pub max_connections: Option<usize>,
}

impl BackendAddress {
    /// Create a new backend address
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            weight: 1,
            max_connections: None,
        }
    }

    /// Set weight for load balancing
    pub fn with_weight(mut self, weight: usize) -> Self {
        self.weight = weight.max(1);
        self
    }

    /// Set maximum concurrent connections
    pub fn with_max_connections(mut self, max: usize) -> Self {
        self.max_connections = Some(max);
        self
    }
}

impl std::fmt::Display for BackendAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.url)
    }
}

/// Pool of backends with load balancing
#[derive(Debug, Clone)]
pub struct BackendPool {
    backends: Vec<BackendAddress>,
    strategy: LoadBalancingStrategy,
    current_index: Arc<AtomicUsize>,
}

impl BackendPool {
    /// Create a new backend pool
    pub fn new(backends: Vec<BackendAddress>, strategy: LoadBalancingStrategy) -> Self {
        assert!(!backends.is_empty(), "BackendPool requires at least one backend");
        Self {
            backends,
            strategy,
            current_index: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Get the next backend using configured strategy
    pub fn next(&self) -> &BackendAddress {
        match self.strategy {
            LoadBalancingStrategy::RoundRobin => self.next_round_robin(),
            LoadBalancingStrategy::Random => self.next_random(),
            LoadBalancingStrategy::LeastConnections => self.next_round_robin(), // TODO: implement
        }
    }

    /// Get next backend via round-robin
    fn next_round_robin(&self) -> &BackendAddress {
        let idx = self
            .current_index
            .fetch_add(1, Ordering::Relaxed)
            % self.backends.len();
        &self.backends[idx]
    }

    /// Get next backend randomly
    fn next_random(&self) -> &BackendAddress {
        use std::collections::hash_map::RandomState;
        use std::hash::{BuildHasher, Hash, Hasher};

        let mut hasher = RandomState::new().build_hasher();
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            .hash(&mut hasher);
        let idx = (hasher.finish() as usize) % self.backends.len();
        &self.backends[idx]
    }

    /// Get all backends
    pub fn backends(&self) -> &[BackendAddress] {
        &self.backends
    }

    /// Get number of backends
    pub fn len(&self) -> usize {
        self.backends.len()
    }

    /// Check if pool is empty
    pub fn is_empty(&self) -> bool {
        self.backends.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Traces to: FR-ROUTER-004 (Backend pool management)
    #[test]
    fn test_backend_address_creation() {
        let backend = BackendAddress::new("http://localhost:3000");
        assert_eq!(backend.url, "http://localhost:3000");
        assert_eq!(backend.weight, 1);
        assert!(backend.max_connections.is_none());
    }

    // Traces to: FR-ROUTER-004
    #[test]
    fn test_backend_address_with_weight() {
        let backend = BackendAddress::new("http://localhost:3000").with_weight(5);
        assert_eq!(backend.weight, 5);
    }

    // Traces to: FR-ROUTER-004
    #[test]
    fn test_backend_pool_round_robin() {
        let backends = vec![
            BackendAddress::new("http://backend1:3000"),
            BackendAddress::new("http://backend2:3000"),
            BackendAddress::new("http://backend3:3000"),
        ];
        let pool = BackendPool::new(backends, LoadBalancingStrategy::RoundRobin);

        // Should cycle through backends
        assert_eq!(pool.next().url, "http://backend1:3000");
        assert_eq!(pool.next().url, "http://backend2:3000");
        assert_eq!(pool.next().url, "http://backend3:3000");
        assert_eq!(pool.next().url, "http://backend1:3000");
    }

    // Traces to: FR-ROUTER-004
    #[test]
    fn test_backend_pool_single_backend() {
        let backends = vec![BackendAddress::new("http://backend1:3000")];
        let pool = BackendPool::new(backends, LoadBalancingStrategy::RoundRobin);

        assert_eq!(pool.next().url, "http://backend1:3000");
        assert_eq!(pool.next().url, "http://backend1:3000");
    }

    // Traces to: FR-ROUTER-004
    #[test]
    fn test_backend_pool_len() {
        let backends = vec![
            BackendAddress::new("http://backend1:3000"),
            BackendAddress::new("http://backend2:3000"),
        ];
        let pool = BackendPool::new(backends, LoadBalancingStrategy::RoundRobin);
        assert_eq!(pool.len(), 2);
        assert!(!pool.is_empty());
    }

    // Traces to: FR-ROUTER-004
    #[test]
    fn test_load_balancing_strategy_default() {
        let strategy: LoadBalancingStrategy = Default::default();
        assert_eq!(strategy, LoadBalancingStrategy::RoundRobin);
    }
}
