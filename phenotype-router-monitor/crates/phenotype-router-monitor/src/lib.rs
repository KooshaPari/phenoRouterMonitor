//! Phenotype Router Monitor — Monitoring and decision tracking for HTTP routing.
//!
//! This library builds on phenotype-router-core to provide:
//! - **Router Orchestration**: Coordinated routing with balancing and monitoring
//! - **Health Checks**: Backend health tracking and recovery
//! - **Decision Tracking**: Record and analyze all routing decisions
//! - **Metrics & Stats**: Comprehensive routing analytics

mod router;
mod health;
mod decision_tracker;
mod orchestrator;

pub use router::Router;
pub use health::{HealthChecker, HealthStatus};
pub use decision_tracker::DecisionTracker;
pub use orchestrator::RouterOrchestrator;
pub use phenotype_router_core::{
    Backend, LoadBalancingStrategy, MatcherStrategy, Route, RouteConfig, RoutingDecision,
    RoutingMetrics, BackendMetrics, RouteRegistry, RoundRobin, Random, LeastConnections,
    ExactMatcher, WildcardMatcher, RegexMatcher, RouterError, RouterResult,
};

#[cfg(test)]
mod tests {
    #[test]
    fn test_lib_compiles() {
        assert_eq!(1 + 1, 2);
    }
}
