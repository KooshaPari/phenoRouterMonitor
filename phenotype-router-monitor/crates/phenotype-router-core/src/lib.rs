//! Core routing engine for phenotype-router-monitor
//!
//! This crate provides:
//! - Path-based routing with regex and wildcard patterns
//! - Round-robin load balancing across backend pools
//! - Request forwarding with full header/query preservation
//! - TOML configuration schema and validation
//! - Pareto risk-aware routing with hysteresis
//! - SHA-256 audit logging with hash-chain integrity
//! - Task execution orchestration across multiple agents

// Core infrastructure modules
pub mod backend;
pub mod error;
pub mod loader;
pub mod patterns;
pub mod router;

// Pareto router modules (extracted from thegent)
pub mod audit;
pub mod executor;
pub mod hysteresis;
pub mod orchestrator;
pub mod risk;

// Re-exports: infrastructure
pub use backend::{BackendAddress, BackendPool, LoadBalancingStrategy};
pub use error::{RouterError, Result};
pub use loader::ConfigLoader;
pub use patterns::PathPattern;
pub use router::Router;

// Re-exports: Pareto routing
pub use audit::{AuditLogger, AuditRecord};
pub use executor::{DispatchTarget, Dispatcher, ExecutionOutcome, RouteExecutor};
pub use hysteresis::HysteresisManager;
pub use orchestrator::{AgentRoutingState, ArbitrationPolicy, RouterStatus, RoutingOrchestrator};
pub use risk::{ComplexityLevel, RiskCalculator, RiskFactors};

#[cfg(test)]
mod tests {
    use super::*;

    // Traces to: FR-ROUTER-001 (Core router functionality)
    #[test]
    fn test_router_creation() {
        let router = Router::new();
        assert!(router.routes_count() == 0);
    }
}
