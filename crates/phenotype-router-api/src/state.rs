//! Router state management.
//!
//! Maintains runtime state for the router including agents, metrics, and configuration.

use crate::metrics::{DefaultMetricsCollector, MetricsCollector};
use crate::types::{Agent, RouterConfig};
use chrono::Utc;
use std::sync::Arc;
use std::time::Instant;

/// Shared router state.
pub struct RouterState {
    config: RouterConfig,
    agents: Arc<parking_lot::RwLock<Vec<Agent>>>,
    metrics: Arc<dyn MetricsCollector>,
    startup_time: Instant,
}

impl RouterState {
    /// Create a new router state.
    pub fn new(config: RouterConfig) -> Self {
        Self {
            config,
            agents: Arc::new(parking_lot::RwLock::new(Vec::new())),
            metrics: Arc::new(DefaultMetricsCollector::new()),
            startup_time: Instant::now(),
        }
    }

    /// Create a new router state with custom metrics collector.
    pub fn with_metrics(config: RouterConfig, metrics: Arc<dyn MetricsCollector>) -> Self {
        Self {
            config,
            agents: Arc::new(parking_lot::RwLock::new(Vec::new())),
            metrics,
            startup_time: Instant::now(),
        }
    }

    /// Get the router configuration.
    pub fn config(&self) -> &RouterConfig {
        &self.config
    }

    /// Get all agents.
    pub fn agents(&self) -> Vec<Agent> {
        self.agents.read().clone()
    }

    /// Add an agent.
    pub fn add_agent(&self, agent: Agent) -> Result<(), String> {
        let mut agents = self.agents.write();
        if agents.len() >= self.config.max_agents {
            return Err(format!("max agents ({}) reached", self.config.max_agents));
        }
        agents.push(agent);
        Ok(())
    }

    /// Get an agent by ID.
    pub fn get_agent(&self, id: &str) -> Option<Agent> {
        self.agents.read().iter().find(|a| a.id == id).cloned()
    }

    /// Remove an agent by ID.
    pub fn remove_agent(&self, id: &str) -> Option<Agent> {
        let mut agents = self.agents.write();
        agents.iter().position(|a| a.id == id).map(|idx| agents.remove(idx))
    }

    /// Update an agent.
    pub fn update_agent(&self, id: &str, agent: Agent) -> Result<(), String> {
        let mut agents = self.agents.write();
        if let Some(idx) = agents.iter().position(|a| a.id == id) {
            agents[idx] = agent;
            Ok(())
        } else {
            Err(format!("agent not found: {}", id))
        }
    }

    /// Get the number of active agents.
    pub fn active_agents_count(&self) -> usize {
        self.agents.read().iter().filter(|a| a.status == "active").count()
    }

    /// Get metrics.
    pub fn metrics(&self) -> crate::metrics::Metrics {
        self.metrics.get_metrics()
    }

    /// Record a request.
    pub fn record_request(&self, path: &str, method: &str, status: u16) {
        self.metrics.record_request(path, method, status);
    }

    /// Record an error.
    pub fn record_error(&self, error_type: &str) {
        self.metrics.record_error(error_type);
    }

    /// Get uptime in seconds.
    pub fn uptime_secs(&self) -> u64 {
        self.startup_time.elapsed().as_secs()
    }

    /// Refresh agents (re-register).
    pub fn refresh_agents(&self) -> usize {
        let mut agents = self.agents.write();
        for agent in agents.iter_mut() {
            agent.last_heartbeat = Some(Utc::now());
        }
        agents.len()
    }

    /// Get router status as a string.
    pub fn status(&self) -> String {
        let active = self.active_agents_count();
        let total = self.agents.read().len();
        format!("active_agents={}/{}", active, total)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_router_state_new() {
        let config = RouterConfig::default();
        let state = RouterState::new(config);
        assert_eq!(state.agents().len(), 0);
    }

    #[test]
    fn test_add_agent() {
        let config = RouterConfig::default();
        let state = RouterState::new(config);

        let agent = Agent::new("agent-1", "test-agent");
        assert!(state.add_agent(agent).is_ok());
        assert_eq!(state.agents().len(), 1);
    }

    #[test]
    fn test_add_agent_exceeds_max() {
        let config = RouterConfig::default().with_max_agents(1);
        let state = RouterState::new(config);

        let agent1 = Agent::new("agent-1", "test-1");
        let agent2 = Agent::new("agent-2", "test-2");

        assert!(state.add_agent(agent1).is_ok());
        assert!(state.add_agent(agent2).is_err());
    }

    #[test]
    fn test_get_agent() {
        let config = RouterConfig::default();
        let state = RouterState::new(config);

        let agent = Agent::new("agent-1", "test-agent");
        state.add_agent(agent.clone()).unwrap();

        let retrieved = state.get_agent("agent-1");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, "agent-1");
    }

    #[test]
    fn test_remove_agent() {
        let config = RouterConfig::default();
        let state = RouterState::new(config);

        let agent = Agent::new("agent-1", "test-agent");
        state.add_agent(agent).unwrap();
        assert_eq!(state.agents().len(), 1);

        let removed = state.remove_agent("agent-1");
        assert!(removed.is_some());
        assert_eq!(state.agents().len(), 0);
    }

    #[test]
    fn test_update_agent() {
        let config = RouterConfig::default();
        let state = RouterState::new(config);

        let agent = Agent::new("agent-1", "test-agent");
        state.add_agent(agent).unwrap();

        let updated = Agent::new("agent-1", "test-agent").mark_active();
        assert!(state.update_agent("agent-1", updated).is_ok());

        let retrieved = state.get_agent("agent-1").unwrap();
        assert_eq!(retrieved.status, "active");
    }

    #[test]
    fn test_active_agents_count() {
        let config = RouterConfig::default();
        let state = RouterState::new(config);

        let agent1 = Agent::new("agent-1", "test-1").mark_active();
        let agent2 = Agent::new("agent-2", "test-2").mark_active();
        let agent3 = Agent::new("agent-3", "test-3");

        state.add_agent(agent1).unwrap();
        state.add_agent(agent2).unwrap();
        state.add_agent(agent3).unwrap();

        assert_eq!(state.active_agents_count(), 2);
    }

    #[test]
    fn test_refresh_agents() {
        let config = RouterConfig::default();
        let state = RouterState::new(config);

        let agent = Agent::new("agent-1", "test-agent");
        state.add_agent(agent).unwrap();

        let count = state.refresh_agents();
        assert_eq!(count, 1);

        let refreshed = state.get_agent("agent-1").unwrap();
        assert!(refreshed.last_heartbeat.is_some());
    }

    #[test]
    fn test_uptime_secs() {
        let config = RouterConfig::default();
        let state = RouterState::new(config);
        assert_eq!(state.uptime_secs(), 0);
    }

    #[test]
    fn test_record_request() {
        let config = RouterConfig::default();
        let state = RouterState::new(config);

        state.record_request("/health", "GET", 200);
        let metrics = state.metrics();
        assert_eq!(metrics.total_requests, 1);
    }

    #[test]
    fn test_status() {
        let config = RouterConfig::default();
        let state = RouterState::new(config);

        let agent = Agent::new("agent-1", "test-agent").mark_active();
        state.add_agent(agent).unwrap();

        let status = state.status();
        assert!(status.contains("active_agents"));
    }
}
