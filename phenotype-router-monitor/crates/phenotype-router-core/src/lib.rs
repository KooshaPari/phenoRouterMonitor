//! Core routing engine for phenotype-router-monitor
//!
//! This crate provides:
//! - Path-based routing with regex and wildcard patterns
//! - Round-robin load balancing across backend pools
//! - Request forwarding with full header/query preservation
//! - TOML configuration schema and validation

pub mod backend;
pub mod error;
pub mod loader;
pub mod patterns;
pub mod router;

// Pareto router modules (extracted from thegent)
pub mod pareto_audit;
pub mod pareto_executor;
pub mod pareto_hysteresis;
pub mod pareto_orchestrator;
pub mod pareto_risk;

pub use backend::{BackendAddress, BackendPool, LoadBalancingStrategy};
pub use error::{RouterError, Result};
pub use loader::ConfigLoader;
pub use patterns::PathPattern;
pub use router::Router;

// Re-exports: Pareto routing
pub use pareto_audit::{AuditLogger, AuditRecord};
pub use pareto_executor::{DispatchTarget, Dispatcher, ExecutionOutcome, RouteExecutor};
pub use pareto_hysteresis::HysteresisManager;
pub use pareto_orchestrator::{AgentRoutingState, ArbitrationPolicy, RouterStatus, RoutingOrchestrator};
pub use pareto_risk::{ComplexityLevel, RiskCalculator, RiskFactors};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_router_creation() {
        let router = Router::new();
        assert!(router.routes_count() == 0);
    }
}
