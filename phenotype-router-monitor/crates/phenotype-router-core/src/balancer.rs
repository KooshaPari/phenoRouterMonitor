//! Load balancing strategies for distributing requests across backends.

use crate::route::Backend;
use crate::error::{RouterError, RouterResult};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Trait for load balancing strategies.
pub trait LoadBalancingStrategy: Send + Sync {
    /// Select a backend from the available list.
    fn select<'a>(&self, backends: &'a [Backend]) -> RouterResult<&'a Backend>;

    /// Get the strategy name.
    fn strategy_name(&self) -> &str;

    /// Reset internal state (used in tests).
    fn reset(&self) {}
}

/// Round-robin load balancing.
/// Distributes requests evenly across backends in order.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoundRobin {
    #[serde(skip)]
    counter: Arc<AtomicUsize>,
}

impl RoundRobin {
    pub fn new() -> Self {
        Self {
            counter: Arc::new(AtomicUsize::new(0)),
        }
    }
}

impl Default for RoundRobin {
    fn default() -> Self {
        Self::new()
    }
}

impl LoadBalancingStrategy for RoundRobin {
    fn select<'a>(&self, backends: &'a [Backend]) -> RouterResult<&'a Backend> {
        if backends.is_empty() {
            return Err(RouterError::EmptyBackendList("Cannot select from empty backend list".to_string()));
        }

        let idx = self.counter.fetch_add(1, Ordering::Relaxed) % backends.len();
        Ok(&backends[idx])
    }

    fn strategy_name(&self) -> &str {
        "round-robin"
    }

    fn reset(&self) {
        self.counter.store(0, Ordering::Relaxed);
    }
}

/// Random load balancing.
/// Randomly selects a backend from the available list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Random;

impl Random {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Random {
    fn default() -> Self {
        Self::new()
    }
}

impl LoadBalancingStrategy for Random {
    fn select<'a>(&self, backends: &'a [Backend]) -> RouterResult<&'a Backend> {
        if backends.is_empty() {
            return Err(RouterError::EmptyBackendList("Cannot select from empty backend list".to_string()));
        }

        use std::collections::hash_map::RandomState;
        use std::hash::{BuildHasher, Hasher};

        let mut hasher = RandomState::new().build_hasher();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos();
        hasher.write_u32(nanos);
        let idx = (hasher.finish() as usize) % backends.len();
        Ok(&backends[idx])
    }

    fn strategy_name(&self) -> &str {
        "random"
    }
}

/// Least connections load balancing.
/// Selects the backend with the fewest active connections.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeastConnections;

impl LeastConnections {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LeastConnections {
    fn default() -> Self {
        Self::new()
    }
}

impl LoadBalancingStrategy for LeastConnections {
    fn select<'a>(&self, backends: &'a [Backend]) -> RouterResult<&'a Backend> {
        if backends.is_empty() {
            return Err(RouterError::EmptyBackendList("Cannot select from empty backend list".to_string()));
        }

        backends
            .iter()
            .min_by_key(|b| b.active_connections())
            .ok_or_else(|| RouterError::NoHealthyBackends("No backends available".to_string()))
    }

    fn strategy_name(&self) -> &str {
        "least-connections"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_backends() -> Vec<Backend> {
        vec![
            Backend::new("backend-1".to_string(), "http://localhost:3001".to_string()),
            Backend::new("backend-2".to_string(), "http://localhost:3002".to_string()),
            Backend::new("backend-3".to_string(), "http://localhost:3003".to_string()),
        ]
    }

    #[test]
    fn test_round_robin_distribution() {
        let rr = RoundRobin::new();
        let backends = create_test_backends();

        let b1 = rr.select(&backends).unwrap();
        let b2 = rr.select(&backends).unwrap();
        let b3 = rr.select(&backends).unwrap();
        let b4 = rr.select(&backends).unwrap();

        assert_eq!(b1.id(), "backend-1");
        assert_eq!(b2.id(), "backend-2");
        assert_eq!(b3.id(), "backend-3");
        assert_eq!(b4.id(), "backend-1");
    }

    #[test]
    fn test_round_robin_reset() {
        let rr = RoundRobin::new();
        let backends = create_test_backends();

        let b1 = rr.select(&backends).unwrap();
        assert_eq!(b1.id(), "backend-1");

        rr.reset();
        let b2 = rr.select(&backends).unwrap();
        assert_eq!(b2.id(), "backend-1");
    }

    #[test]
    fn test_round_robin_single_backend() {
        let rr = RoundRobin::new();
        let backends = vec![Backend::new("single".to_string(), "http://localhost:3001".to_string())];

        for _ in 0..5 {
            let b = rr.select(&backends).unwrap();
            assert_eq!(b.id(), "single");
        }
    }

    #[test]
    fn test_round_robin_empty_backends() {
        let rr = RoundRobin::new();
        let backends: Vec<Backend> = vec![];
        assert!(rr.select(&backends).is_err());
    }

    #[test]
    fn test_random_selection() {
        let random = Random::new();
        let backends = create_test_backends();

        for _ in 0..20 {
            let b = random.select(&backends).unwrap();
            assert!(vec!["backend-1", "backend-2", "backend-3"].contains(&b.id()));
        }
    }

    #[test]
    fn test_random_single_backend() {
        let random = Random::new();
        let backends = vec![Backend::new("single".to_string(), "http://localhost:3001".to_string())];

        for _ in 0..5 {
            let b = random.select(&backends).unwrap();
            assert_eq!(b.id(), "single");
        }
    }

    #[test]
    fn test_random_empty_backends() {
        let random = Random::new();
        let backends: Vec<Backend> = vec![];
        assert!(random.select(&backends).is_err());
    }

    #[test]
    fn test_least_connections_selection() {
        let lc = LeastConnections::new();
        let backends = create_test_backends();

        // All start with 0 connections, should select first
        let b1 = lc.select(&backends).unwrap();
        assert_eq!(b1.id(), "backend-1");

        // Add connections to first backend
        backends[0].add_connection();
        backends[0].add_connection();

        // Should now select second (has fewer)
        let b2 = lc.select(&backends).unwrap();
        assert_eq!(b2.id(), "backend-2");

        // Add connections to second
        backends[1].add_connection();

        // Should select third
        let b3 = lc.select(&backends).unwrap();
        assert_eq!(b3.id(), "backend-3");
    }

    #[test]
    fn test_least_connections_empty_backends() {
        let lc = LeastConnections::new();
        let backends: Vec<Backend> = vec![];
        assert!(lc.select(&backends).is_err());
    }

    #[test]
    fn test_strategy_names() {
        let rr = RoundRobin::new();
        let random = Random::new();
        let lc = LeastConnections::new();

        assert_eq!(rr.strategy_name(), "round-robin");
        assert_eq!(random.strategy_name(), "random");
        assert_eq!(lc.strategy_name(), "least-connections");
    }

    #[test]
    fn test_load_balancing_strategy_trait_object() {
        let strategy: Box<dyn LoadBalancingStrategy> = Box::new(RoundRobin::new());
        let backends = create_test_backends();

        let b1 = strategy.select(&backends).unwrap();
        let b2 = strategy.select(&backends).unwrap();

        assert_ne!(b1.id(), b2.id());
    }

    #[test]
    fn test_round_robin_large_count() {
        let rr = RoundRobin::new();
        let backends = create_test_backends();

        for i in 0..300 {
            let b = rr.select(&backends).unwrap();
            let expected_idx = i % 3;
            let expected_id = match expected_idx {
                0 => "backend-1",
                1 => "backend-2",
                _ => "backend-3",
            };
            assert_eq!(b.id(), expected_id);
        }
    }

    #[test]
    fn test_least_connections_all_equal() {
        let lc = LeastConnections::new();
        let backends = create_test_backends();

        // Add equal connections to all
        for b in &backends {
            b.add_connection();
            b.add_connection();
        }

        // Should select first one
        let b = lc.select(&backends).unwrap();
        assert_eq!(b.id(), "backend-1");
    }

    #[test]
    fn test_random_distribution_roughly_equal() {
        let random = Random::new();
        let backends = create_test_backends();

        let mut counts = [0, 0, 0];
        for _ in 0..300 {
            let b = random.select(&backends).unwrap();
            match b.id() {
                "backend-1" => counts[0] += 1,
                "backend-2" => counts[1] += 1,
                "backend-3" => counts[2] += 1,
                _ => {}
            }
        }

        // Each should have roughly 100 (allow 20% variance)
        for count in &counts {
            assert!(*count > 80 && *count < 120, "Distribution not roughly equal: {:?}", counts);
        }
    }
}
