//! Core types for the Router API.
//!
//! Defines configuration, routing, and agent types used throughout the API.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Router API configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterConfig {
    /// Router instance ID.
    pub id: String,
    /// Router version.
    pub version: String,
    /// Environment name (dev, staging, prod).
    pub environment: String,
    /// Maximum number of agents.
    pub max_agents: usize,
    /// Health check interval in seconds.
    pub health_check_interval_secs: u64,
    /// Route configuration.
    pub routes: Vec<Route>,
    /// Additional metadata.
    pub metadata: HashMap<String, String>,
}

impl Default for RouterConfig {
    fn default() -> Self {
        Self {
            id: format!("router-{}", Uuid::new_v4()),
            version: env!("CARGO_PKG_VERSION").to_string(),
            environment: "dev".to_string(),
            max_agents: 100,
            health_check_interval_secs: 30,
            routes: vec![],
            metadata: HashMap::new(),
        }
    }
}

impl RouterConfig {
    /// Create a new router configuration with custom ID.
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }

    /// Set the environment.
    pub fn with_environment(mut self, env: impl Into<String>) -> Self {
        self.environment = env.into();
        self
    }

    /// Set the maximum number of agents.
    pub fn with_max_agents(mut self, max: usize) -> Self {
        self.max_agents = max;
        self
    }

    /// Add a route to the configuration.
    pub fn with_route(mut self, route: Route) -> Self {
        self.routes.push(route);
        self
    }
}

/// Route information.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Route {
    /// Route path pattern.
    pub path: String,
    /// HTTP methods supported.
    pub methods: Vec<String>,
    /// Route description.
    pub description: String,
}

impl Route {
    /// Create a new route.
    pub fn new(path: impl Into<String>, methods: Vec<String>, description: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            methods,
            description: description.into(),
        }
    }
}

/// Agent information.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Agent {
    /// Unique agent ID.
    pub id: String,
    /// Agent name.
    pub name: String,
    /// Agent status (active, inactive, error).
    pub status: String,
    /// Last heartbeat timestamp.
    pub last_heartbeat: Option<DateTime<Utc>>,
    /// Agent capabilities.
    pub capabilities: Vec<String>,
    /// Metadata.
    pub metadata: HashMap<String, String>,
}

impl Agent {
    /// Create a new agent.
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            status: "inactive".to_string(),
            last_heartbeat: None,
            capabilities: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Mark agent as active.
    pub fn mark_active(mut self) -> Self {
        self.status = "active".to_string();
        self.last_heartbeat = Some(Utc::now());
        self
    }

    /// Add a capability.
    pub fn with_capability(mut self, capability: impl Into<String>) -> Self {
        self.capabilities.push(capability.into());
        self
    }

    /// Add metadata.
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Router information response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterInfo {
    /// Router ID.
    pub id: String,
    /// Router version.
    pub version: String,
    /// Environment.
    pub environment: String,
    /// Current timestamp.
    pub timestamp: DateTime<Utc>,
    /// Number of active agents.
    pub active_agents: usize,
    /// Total routes.
    pub total_routes: usize,
    /// Uptime in seconds.
    pub uptime_secs: u64,
    /// Additional metadata.
    pub metadata: HashMap<String, String>,
}

impl RouterInfo {
    /// Create router info from config and metrics.
    pub fn from_config(config: &RouterConfig, active_agents: usize, uptime_secs: u64) -> Self {
        Self {
            id: config.id.clone(),
            version: config.version.clone(),
            environment: config.environment.clone(),
            timestamp: Utc::now(),
            active_agents,
            total_routes: config.routes.len(),
            uptime_secs,
            metadata: config.metadata.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_router_config_default() {
        let config = RouterConfig::default();
        assert!(!config.id.is_empty());
        assert_eq!(config.max_agents, 100);
        assert_eq!(config.health_check_interval_secs, 30);
    }

    #[test]
    fn test_router_config_builder() {
        let config = RouterConfig::default()
            .with_id("my-router")
            .with_environment("prod")
            .with_max_agents(50);

        assert_eq!(config.id, "my-router");
        assert_eq!(config.environment, "prod");
        assert_eq!(config.max_agents, 50);
    }

    #[test]
    fn test_route_creation() {
        let route = Route::new("/api/users", vec!["GET".to_string()], "Get users");
        assert_eq!(route.path, "/api/users");
        assert_eq!(route.methods, vec!["GET"]);
    }

    #[test]
    fn test_agent_creation() {
        let agent = Agent::new("agent-1", "test-agent");
        assert_eq!(agent.id, "agent-1");
        assert_eq!(agent.name, "test-agent");
        assert_eq!(agent.status, "inactive");
    }

    #[test]
    fn test_agent_mark_active() {
        let agent = Agent::new("agent-1", "test-agent").mark_active();
        assert_eq!(agent.status, "active");
        assert!(agent.last_heartbeat.is_some());
    }

    #[test]
    fn test_agent_with_capability() {
        let agent = Agent::new("agent-1", "test-agent")
            .with_capability("read")
            .with_capability("write");
        assert_eq!(agent.capabilities, vec!["read", "write"]);
    }

    #[test]
    fn test_agent_with_metadata() {
        let agent = Agent::new("agent-1", "test-agent")
            .with_metadata("version", "1.0")
            .with_metadata("region", "us-east-1");

        assert_eq!(agent.metadata.get("version"), Some(&"1.0".to_string()));
        assert_eq!(agent.metadata.get("region"), Some(&"us-east-1".to_string()));
    }

    #[test]
    fn test_router_info_from_config() {
        let config = RouterConfig::default().with_id("router-1");
        let info = RouterInfo::from_config(&config, 5, 3600);

        assert_eq!(info.id, "router-1");
        assert_eq!(info.active_agents, 5);
        assert_eq!(info.uptime_secs, 3600);
    }
}
