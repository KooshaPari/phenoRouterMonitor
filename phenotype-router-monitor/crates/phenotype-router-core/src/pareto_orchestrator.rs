//! Routing Orchestrator: coordinates routing decisions across multiple concurrent agents.
//!
//! Manages multiple RouteExecutor instances and provides:
//! - Route arbitration quorum (majority-wins or most-restrictive-wins)
//! - Per-agent routing state
//! - `thegent router status` output via RouterStatus

use crate::audit::AuditLogger;
use crate::executor::Dispatcher;
use crate::risk::RiskFactors;
use crate::router::{ParetoRouter, RouterConfig, RoutingDecision, RoutingMode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Arbitration policy applied when multiple agents request conflicting routing decisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ArbitrationPolicy {
    /// The routing mode with more votes wins; TheGent wins ties.
    #[default]
    MajorityWins,
    /// The more restrictive (higher-quality) mode always wins.
    /// TheGent > Lifecycle.
    MostRestrictiveWins,
}

/// Current routing state for a single agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRoutingState {
    /// Agent identifier.
    pub agent_id: String,
    /// Current routing mode for this agent.
    pub current_mode: RoutingMode,
    /// Total decisions made for this agent.
    pub total_decisions: usize,
    /// Decisions routed to Lifecycle.
    pub lifecycle_decisions: usize,
    /// Decisions routed to TheGent.
    pub thegent_decisions: usize,
    /// Last routing decision rationale.
    pub last_rationale: String,
}

/// Aggregated status snapshot for `thegent router status`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterStatus {
    /// All active agent routing states.
    pub agents: Vec<AgentRoutingState>,
    /// Total decisions across all agents.
    pub total_decisions: usize,
    /// Arbitration policy in use.
    pub policy: ArbitrationPolicy,
    /// Current quorum decision (when multiple agents have conflicting decisions).
    pub quorum_decision: Option<RoutingMode>,
    /// Lifecycle percentage across all agents.
    pub lifecycle_pct: f64,
    /// TheGent percentage across all agents.
    pub thegent_pct: f64,
}

impl RouterStatus {
    /// Format as a human-readable status string for CLI display.
    pub fn display(&self) -> String {
        let mut lines = vec![
            format!("Router Status ({} agents)", self.agents.len()),
            format!("Policy: {:?}", self.policy),
            format!("Total decisions: {}", self.total_decisions),
            format!(
                "Lifecycle: {:.1}% | TheGent: {:.1}%",
                self.lifecycle_pct, self.thegent_pct
            ),
        ];
        if let Some(quorum) = self.quorum_decision {
            lines.push(format!("Quorum: {:?}", quorum));
        }
        lines.push(String::new());
        lines.push("Agent states:".to_string());
        for agent in &self.agents {
            lines.push(format!(
                "  {} → {:?} (lc={}, tg={}) | {}",
                agent.agent_id,
                agent.current_mode,
                agent.lifecycle_decisions,
                agent.thegent_decisions,
                agent.last_rationale,
            ));
        }
        lines.join("\n")
    }
}

/// Internal per-agent state tracked by the orchestrator.
struct AgentEntry {
    state: AgentRoutingState,
    last_decision: Option<RoutingDecision>,
}

/// Routing Orchestrator.
///
/// Manages routing across multiple concurrent agents. Each agent gets its own
/// `RouteExecutor`. When multiple agents request conflicting routes for the same
/// task, the configured `ArbitrationPolicy` resolves conflicts.
pub struct RoutingOrchestrator {
    router: Arc<ParetoRouter>,
    /// Held for API compatibility; audit records are written by RouteExecutor callers.
    _audit: Arc<AuditLogger>,
    policy: ArbitrationPolicy,
    agents: Mutex<HashMap<String, AgentEntry>>,
    /// Optional shared dispatcher for all executors.
    dispatcher: Option<Arc<dyn Dispatcher>>,
}

impl RoutingOrchestrator {
    /// Create a new orchestrator with default configuration.
    pub fn new(audit: Arc<AuditLogger>) -> Self {
        Self {
            router: Arc::new(ParetoRouter::new()),
            _audit: audit,
            policy: ArbitrationPolicy::default(),
            agents: Mutex::new(HashMap::new()),
            dispatcher: None,
        }
    }

    /// Create an orchestrator with custom router config and policy.
    pub fn with_config(
        config: RouterConfig,
        policy: ArbitrationPolicy,
        audit: Arc<AuditLogger>,
    ) -> Self {
        Self {
            router: Arc::new(ParetoRouter::with_config(config)),
            _audit: audit,
            policy,
            agents: Mutex::new(HashMap::new()),
            dispatcher: None,
        }
    }

    /// Set a custom dispatcher shared across all agent executors.
    pub fn with_dispatcher(mut self, dispatcher: Arc<dyn Dispatcher>) -> Self {
        self.dispatcher = Some(dispatcher);
        self
    }

    /// Register an agent with the orchestrator.
    ///
    /// Idempotent: re-registering an existing agent is a no-op.
    pub fn register_agent(&self, agent_id: &str) {
        let mut agents = self.agents.lock().unwrap();
        if agents.contains_key(agent_id) {
            return;
        }
        agents.insert(
            agent_id.to_string(),
            AgentEntry {
                state: AgentRoutingState {
                    agent_id: agent_id.to_string(),
                    current_mode: RoutingMode::Lifecycle,
                    total_decisions: 0,
                    lifecycle_decisions: 0,
                    thegent_decisions: 0,
                    last_rationale: String::new(),
                },
                last_decision: None,
            },
        );
    }

    /// Route a task for a specific agent using session-aware hysteresis.
    ///
    /// Updates the agent's routing state and returns the decision.
    pub fn route_for_agent(&self, agent_id: &str, factors: &RiskFactors) -> RoutingDecision {
        self.register_agent(agent_id);

        let decision = self.router.route_with_session(agent_id, factors);

        let mut agents = self.agents.lock().unwrap();
        if let Some(entry) = agents.get_mut(agent_id) {
            entry.state.total_decisions += 1;
            entry.state.current_mode = decision.mode;
            entry.state.last_rationale = decision.rationale.clone();
            match decision.mode {
                RoutingMode::Lifecycle => entry.state.lifecycle_decisions += 1,
                RoutingMode::TheGent => entry.state.thegent_decisions += 1,
            }
            entry.last_decision = Some(decision.clone());
        }

        decision
    }

    /// Apply quorum arbitration across all currently registered agents.
    ///
    /// Called when N agents have conflicting routing decisions.
    /// Returns the arbitrated RoutingMode.
    ///
    /// - `MajorityWins`: count votes; TheGent wins ties.
    /// - `MostRestrictiveWins`: any TheGent vote → TheGent.
    pub fn arbitrate(&self) -> RoutingMode {
        let agents = self.agents.lock().unwrap();
        let decisions: Vec<RoutingMode> = agents
            .values()
            .filter_map(|e| e.last_decision.as_ref().map(|d| d.mode))
            .collect();

        if decisions.is_empty() {
            return RoutingMode::Lifecycle;
        }

        match self.policy {
            ArbitrationPolicy::MostRestrictiveWins => {
                if decisions.contains(&RoutingMode::TheGent) {
                    RoutingMode::TheGent
                } else {
                    RoutingMode::Lifecycle
                }
            }
            ArbitrationPolicy::MajorityWins => {
                let thegent_votes = decisions
                    .iter()
                    .filter(|&&m| m == RoutingMode::TheGent)
                    .count();
                let lifecycle_votes = decisions.len() - thegent_votes;
                // TheGent wins ties.
                if thegent_votes >= lifecycle_votes {
                    RoutingMode::TheGent
                } else {
                    RoutingMode::Lifecycle
                }
            }
        }
    }

    /// Get the current routing status snapshot.
    ///
    /// Used by `thegent router status` CLI command.
    pub fn status(&self) -> RouterStatus {
        let agents = self.agents.lock().unwrap();
        let agent_states: Vec<AgentRoutingState> =
            agents.values().map(|e| e.state.clone()).collect();

        let total = agent_states
            .iter()
            .map(|a| a.total_decisions)
            .sum::<usize>();
        let lc_total = agent_states
            .iter()
            .map(|a| a.lifecycle_decisions)
            .sum::<usize>();
        let tg_total = agent_states
            .iter()
            .map(|a| a.thegent_decisions)
            .sum::<usize>();

        let (lifecycle_pct, thegent_pct) = if total == 0 {
            (0.0, 0.0)
        } else {
            (
                (lc_total as f64 / total as f64) * 100.0,
                (tg_total as f64 / total as f64) * 100.0,
            )
        };

        // Quorum: only meaningful when 2+ agents have decisions.
        let quorum_decision = if agents.values().any(|e| e.last_decision.is_some()) {
            drop(agents);
            Some(self.arbitrate())
        } else {
            None
        };

        RouterStatus {
            agents: agent_states,
            total_decisions: total,
            policy: self.policy,
            quorum_decision,
            lifecycle_pct,
            thegent_pct,
        }
    }

    /// List all registered agent IDs.
    pub fn agent_ids(&self) -> Vec<String> {
        self.agents.lock().unwrap().keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::risk::{ComplexityLevel, RiskFactors};
    use tempfile::tempdir;

    fn make_orchestrator() -> RoutingOrchestrator {
        let dir = tempdir().unwrap();
        let audit = Arc::new(AuditLogger::new(dir.path().join("audit.jsonl")));
        RoutingOrchestrator::new(audit)
    }

    fn make_orchestrator_with_policy(policy: ArbitrationPolicy) -> RoutingOrchestrator {
        let dir = tempdir().unwrap();
        let audit = Arc::new(AuditLogger::new(dir.path().join("audit.jsonl")));
        RoutingOrchestrator::with_config(RouterConfig::default(), policy, audit)
    }

    #[test]
    fn test_orchestrator_register_agent() {
        let orch = make_orchestrator();
        orch.register_agent("agent-1");
        assert!(orch.agent_ids().contains(&"agent-1".to_string()));
    }

    #[test]
    fn test_orchestrator_register_idempotent() {
        let orch = make_orchestrator();
        orch.register_agent("agent-1");
        orch.register_agent("agent-1"); // should not panic or duplicate
        assert_eq!(orch.agent_ids().len(), 1);
    }

    #[test]
    fn test_orchestrator_route_for_agent_simple() {
        let orch = make_orchestrator();
        let factors = RiskFactors::new(ComplexityLevel::Simple);
        let decision = orch.route_for_agent("agent-1", &factors);
        assert_eq!(decision.mode, RoutingMode::Lifecycle);
    }

    #[test]
    fn test_orchestrator_route_for_agent_complex() {
        let orch = make_orchestrator();
        let factors = RiskFactors {
            complexity: ComplexityLevel::VeryComplex,
            cost_cents: 8000,
            dependency_count: 8,
            security_sensitive: true,
            max_cost_cents: 10_000,
        };
        let decision = orch.route_for_agent("agent-1", &factors);
        assert_eq!(decision.mode, RoutingMode::TheGent);
    }

    #[test]
    fn test_orchestrator_tracks_agent_state() {
        let orch = make_orchestrator();
        let simple = RiskFactors::new(ComplexityLevel::Simple);
        orch.route_for_agent("agent-1", &simple);
        orch.route_for_agent("agent-1", &simple);

        let status = orch.status();
        let agent = status
            .agents
            .iter()
            .find(|a| a.agent_id == "agent-1")
            .unwrap();
        assert_eq!(agent.total_decisions, 2);
        assert_eq!(agent.lifecycle_decisions, 2);
    }

    #[test]
    fn test_arbitration_majority_wins_lifecycle() {
        let orch = make_orchestrator_with_policy(ArbitrationPolicy::MajorityWins);
        // 2 agents → Lifecycle, 1 agent → TheGent.
        let simple = RiskFactors::new(ComplexityLevel::Simple);
        let complex = RiskFactors {
            complexity: ComplexityLevel::VeryComplex,
            cost_cents: 9000,
            dependency_count: 9,
            security_sensitive: true,
            max_cost_cents: 10_000,
        };
        orch.route_for_agent("agent-1", &simple);
        orch.route_for_agent("agent-2", &simple);
        orch.route_for_agent("agent-3", &complex);

        let result = orch.arbitrate();
        assert_eq!(result, RoutingMode::Lifecycle);
    }

    #[test]
    fn test_arbitration_majority_wins_thegent_tie() {
        let orch = make_orchestrator_with_policy(ArbitrationPolicy::MajorityWins);
        // Equal votes → TheGent wins tie.
        let simple = RiskFactors::new(ComplexityLevel::Simple);
        let complex = RiskFactors {
            complexity: ComplexityLevel::VeryComplex,
            cost_cents: 9000,
            dependency_count: 9,
            security_sensitive: true,
            max_cost_cents: 10_000,
        };
        orch.route_for_agent("agent-1", &simple);
        orch.route_for_agent("agent-2", &complex);

        let result = orch.arbitrate();
        assert_eq!(result, RoutingMode::TheGent);
    }

    #[test]
    fn test_arbitration_most_restrictive() {
        let orch = make_orchestrator_with_policy(ArbitrationPolicy::MostRestrictiveWins);
        // 1 TheGent vote → TheGent wins regardless.
        let simple = RiskFactors::new(ComplexityLevel::Simple);
        let complex = RiskFactors {
            complexity: ComplexityLevel::VeryComplex,
            cost_cents: 9000,
            dependency_count: 9,
            security_sensitive: true,
            max_cost_cents: 10_000,
        };
        orch.route_for_agent("agent-1", &simple);
        orch.route_for_agent("agent-2", &simple);
        orch.route_for_agent("agent-3", &complex);

        let result = orch.arbitrate();
        assert_eq!(result, RoutingMode::TheGent);
    }

    #[test]
    fn test_orchestrator_status_lifecycle_pct() {
        let orch = make_orchestrator();
        let simple = RiskFactors::new(ComplexityLevel::Simple);
        orch.route_for_agent("agent-1", &simple);
        orch.route_for_agent("agent-1", &simple);

        let status = orch.status();
        assert!((status.lifecycle_pct - 100.0).abs() < 0.1);
    }

    #[test]
    fn test_orchestrator_status_display() {
        let orch = make_orchestrator();
        let factors = RiskFactors::new(ComplexityLevel::Simple);
        orch.route_for_agent("agent-1", &factors);

        let status = orch.status();
        let display = status.display();
        assert!(display.contains("agent-1"));
        assert!(display.contains("Router Status"));
    }

    #[test]
    fn test_orchestrator_multi_agent_isolation() {
        let orch = make_orchestrator();
        let simple = RiskFactors::new(ComplexityLevel::Simple);
        let complex = RiskFactors {
            complexity: ComplexityLevel::VeryComplex,
            cost_cents: 9000,
            dependency_count: 9,
            security_sensitive: true,
            max_cost_cents: 10_000,
        };

        orch.route_for_agent("agent-1", &simple);
        orch.route_for_agent("agent-2", &complex);

        let status = orch.status();
        let a1 = status
            .agents
            .iter()
            .find(|a| a.agent_id == "agent-1")
            .unwrap();
        let a2 = status
            .agents
            .iter()
            .find(|a| a.agent_id == "agent-2")
            .unwrap();
        assert_eq!(a1.current_mode, RoutingMode::Lifecycle);
        assert_eq!(a2.current_mode, RoutingMode::TheGent);
    }

    #[test]
    fn test_arbitrate_empty_returns_lifecycle() {
        let orch = make_orchestrator();
        // No agents registered yet.
        let result = orch.arbitrate();
        assert_eq!(result, RoutingMode::Lifecycle);
    }

    #[test]
    fn test_orchestrator_thread_safe() {
        use std::sync::Arc;
        use std::thread;

        let dir = tempdir().unwrap();
        let audit = Arc::new(AuditLogger::new(dir.path().join("audit.jsonl")));
        let orch = Arc::new(RoutingOrchestrator::new(audit));
        let mut handles = vec![];

        for i in 0..4 {
            let o = Arc::clone(&orch);
            let handle = thread::spawn(move || {
                let agent_id = format!("agent-{}", i);
                let factors = RiskFactors::new(ComplexityLevel::Moderate);
                for _ in 0..10 {
                    o.route_for_agent(&agent_id, &factors);
                }
            });
            handles.push(handle);
        }
        for h in handles {
            h.join().unwrap();
        }

        let status = orch.status();
        assert_eq!(status.total_decisions, 40);
    }
}
