//! Agent registry for centralized agent management and lookup

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::config::{AgentConfig, SubagentConfig};
use crate::error::{ForgeError, Result};

/// Central registry for managing and looking up agents
#[derive(Debug)]
pub struct AgentRegistry {
    agents: Arc<RwLock<HashMap<String, AgentConfig>>>,
}

impl AgentRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        AgentRegistry {
            agents: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register an agent in the registry
    ///
    /// # Arguments
    /// * `agent` - The agent configuration to register
    ///
    /// # Returns
    /// * `Ok(())` - Successfully registered
    /// * `Err(ForgeError)` - If registration fails (e.g., validation error)
    pub async fn register(&self, agent: AgentConfig) -> Result<()> {
        agent.validate()?;

        let mut agents = self.agents.write().await;
        let agent_id = agent.id.clone();
        agents.insert(agent_id.clone(), agent);

        tracing::debug!("Registered agent: {}", agent_id);

        Ok(())
    }

    /// Unregister an agent from the registry
    ///
    /// # Arguments
    /// * `agent_id` - The ID of the agent to unregister
    ///
    /// # Returns
    /// * `Ok(())` - Successfully unregistered (or agent didn't exist)
    /// * `Err(ForgeError)` - If operation fails
    pub async fn unregister(&self, agent_id: &str) -> Result<()> {
        let mut agents = self.agents.write().await;

        if agents.remove(agent_id).is_some() {
            tracing::debug!("Unregistered agent: {}", agent_id);
        }

        Ok(())
    }

    /// Get an agent by ID
    ///
    /// # Arguments
    /// * `agent_id` - The ID of the agent
    ///
    /// # Returns
    /// * `Ok(Some(AgentConfig))` - Agent found
    /// * `Ok(None)` - Agent not found
    /// * `Err(ForgeError)` - If operation fails
    pub async fn get(&self, agent_id: &str) -> Result<Option<AgentConfig>> {
        let agents = self.agents.read().await;
        Ok(agents.get(agent_id).cloned())
    }

    /// Get an agent by ID, returning an error if not found
    ///
    /// # Arguments
    /// * `agent_id` - The ID of the agent
    ///
    /// # Returns
    /// * `Ok(AgentConfig)` - Agent found
    /// * `Err(ForgeError::AgentNotFound)` - Agent not found
    /// * `Err(ForgeError)` - If operation fails
    pub async fn get_required(&self, agent_id: &str) -> Result<AgentConfig> {
        self.get(agent_id)
            .await?
            .ok_or_else(|| ForgeError::AgentNotFound(agent_id.to_string()))
    }

    /// List all registered agents
    ///
    /// # Returns
    /// * `Ok(Vec<AgentConfig>)` - All registered agents
    /// * `Err(ForgeError)` - If operation fails
    pub async fn list_all(&self) -> Result<Vec<AgentConfig>> {
        let agents = self.agents.read().await;
        Ok(agents.values().cloned().collect())
    }

    /// List agents by tag
    ///
    /// # Arguments
    /// * `tag` - The tag to filter by
    ///
    /// # Returns
    /// * `Ok(Vec<AgentConfig>)` - All agents with the specified tag
    /// * `Err(ForgeError)` - If operation fails
    pub async fn list_by_tag(&self, tag: &str) -> Result<Vec<AgentConfig>> {
        let agents = self.agents.read().await;

        let filtered: Vec<_> = agents
            .values()
            .filter(|agent| agent.tags.contains(&tag.to_string()))
            .cloned()
            .collect();

        Ok(filtered)
    }

    /// List enabled agents
    ///
    /// # Returns
    /// * `Ok(Vec<AgentConfig>)` - All enabled agents
    /// * `Err(ForgeError)` - If operation fails
    pub async fn list_enabled(&self) -> Result<Vec<AgentConfig>> {
        let agents = self.agents.read().await;

        let enabled: Vec<_> = agents
            .values()
            .filter(|agent| agent.enabled)
            .cloned()
            .collect();

        Ok(enabled)
    }

    /// Check if an agent is registered
    ///
    /// # Arguments
    /// * `agent_id` - The ID of the agent
    ///
    /// # Returns
    /// * `Ok(bool)` - Whether the agent is registered
    /// * `Err(ForgeError)` - If operation fails
    pub async fn exists(&self, agent_id: &str) -> Result<bool> {
        let agents = self.agents.read().await;
        Ok(agents.contains_key(agent_id))
    }

    /// Get the count of registered agents
    ///
    /// # Returns
    /// * `Ok(usize)` - Number of registered agents
    /// * `Err(ForgeError)` - If operation fails
    pub async fn count(&self) -> Result<usize> {
        let agents = self.agents.read().await;
        Ok(agents.len())
    }

    /// Clear all agents from the registry
    ///
    /// # Returns
    /// * `Ok(())` - Successfully cleared
    /// * `Err(ForgeError)` - If operation fails
    pub async fn clear(&self) -> Result<()> {
        let mut agents = self.agents.write().await;

        let count = agents.len();
        agents.clear();

        tracing::debug!("Cleared {} agents from registry", count);

        Ok(())
    }
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_agent(id: &str) -> AgentConfig {
        AgentConfig {
            id: id.to_string(),
            name: format!("Test Agent {}", id),
            description: "A test agent".to_string(),
            instruction: "Do something".to_string(),
            input_schema: serde_json::json!({}),
            output_schema: serde_json::json!({}),
            metadata: HashMap::new(),
            tags: vec![],
            enabled: true,
            version: "1.0.0".to_string(),
            extra: serde_json::json!({}),
        }
    }

    #[tokio::test]
    async fn test_register_agent() {
        let registry = AgentRegistry::new();
        let agent = create_test_agent("test-1");

        registry.register(agent.clone()).await.expect("Failed to register");

        let retrieved = registry
            .get("test-1")
            .await
            .expect("Failed to get agent")
            .expect("Agent not found");

        assert_eq!(retrieved.id, "test-1");
        assert_eq!(retrieved.name, "Test Agent test-1");
    }

    #[tokio::test]
    async fn test_unregister_agent() {
        let registry = AgentRegistry::new();
        let agent = create_test_agent("test-1");

        registry.register(agent).await.expect("Failed to register");
        assert!(registry
            .exists("test-1")
            .await
            .expect("Failed to check existence"));

        registry
            .unregister("test-1")
            .await
            .expect("Failed to unregister");

        assert!(!registry
            .exists("test-1")
            .await
            .expect("Failed to check existence"));
    }

    #[tokio::test]
    async fn test_list_all() {
        let registry = AgentRegistry::new();

        for i in 1..=3 {
            let agent = create_test_agent(&format!("agent-{}", i));
            registry.register(agent).await.expect("Failed to register");
        }

        let agents = registry.list_all().await.expect("Failed to list");
        assert_eq!(agents.len(), 3);
    }

    #[tokio::test]
    async fn test_list_by_tag() {
        let registry = AgentRegistry::new();

        let mut agent1 = create_test_agent("agent-1");
        agent1.tags = vec!["analyzer".to_string()];

        let mut agent2 = create_test_agent("agent-2");
        agent2.tags = vec!["validator".to_string()];

        let mut agent3 = create_test_agent("agent-3");
        agent3.tags = vec!["analyzer".to_string()];

        registry.register(agent1).await.expect("Failed");
        registry.register(agent2).await.expect("Failed");
        registry.register(agent3).await.expect("Failed");

        let analyzers = registry
            .list_by_tag("analyzer")
            .await
            .expect("Failed to list by tag");

        assert_eq!(analyzers.len(), 2);
    }

    #[tokio::test]
    async fn test_list_enabled() {
        let registry = AgentRegistry::new();

        let mut agent1 = create_test_agent("agent-1");
        agent1.enabled = true;

        let mut agent2 = create_test_agent("agent-2");
        agent2.enabled = false;

        let mut agent3 = create_test_agent("agent-3");
        agent3.enabled = true;

        registry.register(agent1).await.expect("Failed");
        registry.register(agent2).await.expect("Failed");
        registry.register(agent3).await.expect("Failed");

        let enabled = registry.list_enabled().await.expect("Failed");
        assert_eq!(enabled.len(), 2);
    }

    #[tokio::test]
    async fn test_get_required() {
        let registry = AgentRegistry::new();
        let agent = create_test_agent("test-1");

        registry.register(agent).await.expect("Failed");

        let retrieved = registry
            .get_required("test-1")
            .await
            .expect("Failed to get required");

        assert_eq!(retrieved.id, "test-1");
    }

    #[tokio::test]
    async fn test_get_required_not_found() {
        let registry = AgentRegistry::new();
        let result = registry.get_required("nonexistent").await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ForgeError::AgentNotFound(_)));
    }

    #[tokio::test]
    async fn test_count() {
        let registry = AgentRegistry::new();

        assert_eq!(
            registry.count().await.expect("Failed to count"),
            0,
            "Initial count should be 0"
        );

        for i in 1..=5 {
            let agent = create_test_agent(&format!("agent-{}", i));
            registry.register(agent).await.expect("Failed");
        }

        assert_eq!(
            registry.count().await.expect("Failed to count"),
            5,
            "Count should be 5 after registration"
        );
    }

    #[tokio::test]
    async fn test_clear() {
        let registry = AgentRegistry::new();

        for i in 1..=3 {
            let agent = create_test_agent(&format!("agent-{}", i));
            registry.register(agent).await.expect("Failed");
        }

        assert_eq!(registry.count().await.expect("Failed"), 3);

        registry.clear().await.expect("Failed to clear");

        assert_eq!(registry.count().await.expect("Failed"), 0);
    }

    #[tokio::test]
    async fn test_exists() {
        let registry = AgentRegistry::new();
        let agent = create_test_agent("test-1");

        assert!(!registry
            .exists("test-1")
            .await
            .expect("Failed to check"));

        registry.register(agent).await.expect("Failed");

        assert!(registry
            .exists("test-1")
            .await
            .expect("Failed to check"));
    }
}
