//! Orchestrator for coordinating multiple routing decisions.

use phenotype_router_core::{Backend, RouteRegistry, RouterError, RouterResult};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Strategy for route orchestration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArbitrationPolicy {
    /// Use the first matching route
    FirstMatch,
    /// Use the highest priority matching route
    HighestPriority,
    /// Randomly select from matching routes
    Random,
}

/// Status of routing orchestration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterStatus {
    pub total_routes: usize,
    pub active_routes: usize,
    pub decisions_made: usize,
    pub last_decision: Option<String>,
}

impl RouterStatus {
    pub fn new() -> Self {
        Self {
            total_routes: 0,
            active_routes: 0,
            decisions_made: 0,
            last_decision: None,
        }
    }
}

impl Default for RouterStatus {
    fn default() -> Self {
        Self::new()
    }
}

/// State tracking for agent routing.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct AgentRoutingState {
    pub agent_id: String,
    pub current_route: Option<String>,
    pub active: bool,
    pub decision_count: usize,
}

/// Orchestrates complex routing decisions.
pub struct RouterOrchestrator {
    registry: Arc<RouteRegistry>,
    policy: ArbitrationPolicy,
    status: Arc<std::sync::Mutex<RouterStatus>>,
}

impl RouterOrchestrator {
    pub fn new(registry: Arc<RouteRegistry>, policy: ArbitrationPolicy) -> Self {
        Self {
            registry,
            policy,
            status: Arc::new(std::sync::Mutex::new(RouterStatus::new())),
        }
    }

    pub fn with_first_match(registry: Arc<RouteRegistry>) -> Self {
        Self::new(registry, ArbitrationPolicy::FirstMatch)
    }

    pub fn with_highest_priority(registry: Arc<RouteRegistry>) -> Self {
        Self::new(registry, ArbitrationPolicy::HighestPriority)
    }

    /// Get the current status.
    pub fn status(&self) -> RouterStatus {
        self.status
            .lock()
            .map(|s| s.clone())
            .unwrap_or_default()
    }

    /// Make a routing decision based on policy.
    pub fn decide(&self, path: &str) -> RouterResult<Backend> {
        // Find matching route and backends
        let (route_id, backends) = self.registry.find_route(path)?;

        // Update status
        if let Ok(mut status) = self.status.lock() {
            status.decisions_made += 1;
            status.last_decision = Some(route_id.clone());
        }

        // Select a backend (simplified - just take first)
        backends
            .first()
            .cloned()
            .ok_or(RouterError::NoHealthyBackends(route_id))
    }
}

impl Clone for RouterOrchestrator {
    fn clone(&self) -> Self {
        Self {
            registry: self.registry.clone(),
            policy: self.policy,
            status: self.status.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use phenotype_router_core::{RouteConfig, Backend};

    fn setup_registry() -> Arc<RouteRegistry> {
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
        registry.register_route(config, backends).ok();
        registry
    }

    #[test]
    fn test_orchestrator_creation() {
        let registry = setup_registry();
        let orchestrator = RouterOrchestrator::with_first_match(registry);
        let status = orchestrator.status();
        assert_eq!(status.decisions_made, 0);
    }

    #[test]
    fn test_orchestrator_with_policy() {
        let registry = setup_registry();
        let orchestrator =
            RouterOrchestrator::new(registry, ArbitrationPolicy::HighestPriority);
        assert_eq!(orchestrator.policy, ArbitrationPolicy::HighestPriority);
    }

    #[test]
    fn test_orchestrator_decision() {
        let registry = setup_registry();
        let orchestrator = RouterOrchestrator::with_first_match(registry);

        let result = orchestrator.decide("/api/users");
        assert!(result.is_ok());

        let status = orchestrator.status();
        assert_eq!(status.decisions_made, 1);
    }

    #[test]
    fn test_orchestrator_failed_decision() {
        let registry = setup_registry();
        let orchestrator = RouterOrchestrator::with_first_match(registry);

        let result = orchestrator.decide("/notfound");
        assert!(result.is_err());
    }

    #[test]
    fn test_router_status_default() {
        let status = RouterStatus::default();
        assert_eq!(status.total_routes, 0);
        assert_eq!(status.decisions_made, 0);
    }

    #[test]
    fn test_agent_routing_state() {
        let state = AgentRoutingState {
            agent_id: "agent-1".to_string(),
            current_route: Some("api".to_string()),
            active: true,
            decision_count: 5,
        };

        assert_eq!(state.agent_id, "agent-1");
        assert!(state.active);
    }

    #[test]
    fn test_arbitration_policy_equality() {
        assert_eq!(ArbitrationPolicy::FirstMatch, ArbitrationPolicy::FirstMatch);
        assert_ne!(ArbitrationPolicy::FirstMatch, ArbitrationPolicy::Random);
    }

    #[test]
    fn test_orchestrator_clone() {
        let registry = setup_registry();
        let orchestrator1 = RouterOrchestrator::with_first_match(registry);
        orchestrator1.decide("/api/users").ok();

        let orchestrator2 = orchestrator1.clone();

        let status1 = orchestrator1.status();
        let status2 = orchestrator2.status();

        assert_eq!(status1.decisions_made, status2.decisions_made);
    }

    #[test]
    fn test_multiple_policies() {
        let registry = setup_registry();

        let _first_match = RouterOrchestrator::with_first_match(registry.clone());
        let _highest_priority =
            RouterOrchestrator::with_highest_priority(registry);

        assert_eq!(_first_match.policy, ArbitrationPolicy::FirstMatch);
        assert_eq!(
            _highest_priority.policy,
            ArbitrationPolicy::HighestPriority
        );
    }
}
