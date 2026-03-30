//! Phenotype Router Core — HTTP routing engine with path matching and load balancing.
//!
//! This library provides:
//! - **Path Matching**: Exact, wildcard, and regex-based route matching
//! - **Load Balancing**: Round-robin, random, and least-connections strategies
//! - **Route Registry**: Dynamic route management and updates
//! - **Metrics**: Request tracking and statistics per route/backend

mod error;
mod matcher;
mod balancer;
mod route;
mod registry;
mod metrics;

pub use error::{RouterError, RouterResult};
pub use matcher::{MatcherStrategy, ExactMatcher, WildcardMatcher, RegexMatcher};
pub use balancer::{LoadBalancingStrategy, RoundRobin, Random, LeastConnections};
pub use route::{Route, Backend, RouteConfig};
pub use registry::RouteRegistry;
pub use metrics::{RoutingMetrics, BackendMetrics};

/// A complete routing decision with selected backend and metadata.
#[derive(Debug, Clone)]
pub struct RoutingDecision {
    pub backend: Backend,
    pub route_id: String,
    pub matched_path: String,
    pub strategy_used: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lib_compiles() {
        assert_eq!(1 + 1, 2);
    }
}
