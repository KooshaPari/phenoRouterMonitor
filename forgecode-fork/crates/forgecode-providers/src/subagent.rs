//! Subagent Configuration System
//!
//! Zero-code YAML-based agent definition and discovery for ForgeCode.
//! Agents can be configured entirely through YAML without requiring Rust code changes.

use crate::error::{ForgeError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Trait defining the configuration interface for subagents
pub trait SubagentConfig: Send + Sync {
    /// Unique identifier for this agent
    fn id(&self) -> &str;

    /// Display name for this agent
    fn name(&self) -> &str;

    /// Agent version (e.g., "0.1.0")
    fn version(&self) -> &str;

    /// Agent type classification (e.g., "analyzer", "validator", "reporter")
    fn agent_type(&self) -> &str;

    /// Validate the agent configuration
    fn validate(&self) -> Result<()>;

    /// Serialize to YAML format
    fn to_yaml(&self) -> Result<String>;

    /// Deserialize from YAML format
    fn from_yaml(yaml: &str) -> Result<Self>
    where
        Self: Sized;
}

/// YAML-based agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub id: String,
    pub name: String,
    pub version: String,
    pub agent_type: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub metadata: HashMap<String, serde_yaml::Value>,
}

impl AgentConfig {
    /// Create a new agent configuration
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        version: impl Into<String>,
        agent_type: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            version: version.into(),
            agent_type: agent_type.into(),
            description: None,
            tags: Vec::new(),
            enabled: true,
            metadata: HashMap::new(),
        }
    }
}

impl SubagentConfig for AgentConfig {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn agent_type(&self) -> &str {
        &self.agent_type
    }

    fn validate(&self) -> Result<()> {
        if self.id.is_empty() {
            return Err(ForgeError::validation("Agent ID cannot be empty"));
        }
        if self.name.is_empty() {
            return Err(ForgeError::validation("Agent name cannot be empty"));
        }
        if self.version.is_empty() {
            return Err(ForgeError::validation("Agent version cannot be empty"));
        }
        if self.agent_type.is_empty() {
            return Err(ForgeError::validation("Agent type cannot be empty"));
        }
        Ok(())
    }

    fn to_yaml(&self) -> Result<String> {
        serde_yaml::to_string(self).map_err(|e| ForgeError::serialization(e.to_string()))
    }

    fn from_yaml(yaml: &str) -> Result<Self> {
        serde_yaml::from_str(yaml).map_err(|e| ForgeError::serialization(e.to_string()))
    }
}

/// Agent Discovery System
/// Automatically discovers and loads agents from the agents/ directory
pub struct AgentDiscovery {
    agents_dir: PathBuf,
    agents: Arc<RwLock<HashMap<String, AgentConfig>>>,
}

impl AgentDiscovery {
    /// Create a new agent discovery system
    pub async fn new(agents_dir: impl AsRef<Path>) -> Result<Self> {
        let agents_dir = agents_dir.as_ref().to_path_buf();

        // Ensure directory exists
        if !agents_dir.exists() {
            tokio::fs::create_dir_all(&agents_dir)
                .await
                .map_err(|e| ForgeError::io(e.to_string()))?;
        }

        let discovery = Self {
            agents_dir,
            agents: Arc::new(RwLock::new(HashMap::new())),
        };

        // Load all existing agents
        discovery.reload().await?;

        Ok(discovery)
    }

    /// Reload all agents from disk
    pub async fn reload(&self) -> Result<()> {
        let mut agents = HashMap::new();

        if self.agents_dir.exists() {
            let mut entries = tokio::fs::read_dir(&self.agents_dir)
                .await
                .map_err(|e| ForgeError::io(e.to_string()))?;

            while let Some(entry) = entries
                .next_entry()
                .await
                .map_err(|e| ForgeError::io(e.to_string()))?
            {
                let path = entry.path();
                if path.extension().map(|e| e == "yaml" || e == "yml").unwrap_or(false) {
                    if let Ok(content) = tokio::fs::read_to_string(&path).await {
                        if let Ok(config) = AgentConfig::from_yaml(&content) {
                            if config.validate().is_ok() {
                                agents.insert(config.id.clone(), config);
                            }
                        }
                    }
                }
            }
        }

        *self.agents.write().await = agents;
        Ok(())
    }

    /// Get an agent by ID
    pub async fn get_agent(&self, id: &str) -> Option<AgentConfig> {
        self.agents.read().await.get(id).cloned()
    }

    /// List all agents
    pub async fn list_agents(&self) -> Vec<AgentConfig> {
        self.agents.read().await.values().cloned().collect()
    }

    /// Find agents by type
    pub async fn find_by_type(&self, agent_type: &str) -> Vec<AgentConfig> {
        self.agents
            .read()
            .await
            .values()
            .filter(|a| a.agent_type == agent_type)
            .cloned()
            .collect()
    }

    /// Find agents by tag
    pub async fn find_by_tag(&self, tag: &str) -> Vec<AgentConfig> {
        self.agents
            .read()
            .await
            .values()
            .filter(|a| a.tags.contains(&tag.to_string()))
            .cloned()
            .collect()
    }

    /// Get agents directory path
    pub fn agents_dir(&self) -> &Path {
        &self.agents_dir
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_config_creation() {
        let config = AgentConfig::new("test-1", "Test Agent", "0.1.0", "analyzer");
        assert_eq!(config.id(), "test-1");
        assert_eq!(config.name(), "Test Agent");
        assert_eq!(config.version(), "0.1.0");
        assert_eq!(config.agent_type(), "analyzer");
    }

    #[test]
    fn test_agent_config_validation() {
        let mut config = AgentConfig::new("test-1", "Test", "0.1.0", "analyzer");
        assert!(config.validate().is_ok());

        config.id = String::new();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_agent_config_yaml_serialization() {
        let config = AgentConfig::new("test-1", "Test Agent", "0.1.0", "analyzer");
        let yaml = config.to_yaml().unwrap();
        assert!(yaml.contains("id: test-1"));
        assert!(yaml.contains("name: Test Agent"));

        let deserialized = AgentConfig::from_yaml(&yaml).unwrap();
        assert_eq!(deserialized.id, config.id);
    }

    #[tokio::test]
    async fn test_agent_discovery_creation() {
        let tempdir = tempfile::tempdir().unwrap();
        let discovery = AgentDiscovery::new(tempdir.path()).await.unwrap();
        assert_eq!(discovery.list_agents().await.len(), 0);
    }

    #[tokio::test]
    async fn test_agent_discovery_find_by_type() {
        let tempdir = tempfile::tempdir().unwrap();
        let discovery = AgentDiscovery::new(tempdir.path()).await.unwrap();

        // Add agents directly for testing
        let agent = AgentConfig::new("test-1", "Test", "0.1.0", "analyzer");
        discovery
            .agents
            .write()
            .await
            .insert("test-1".to_string(), agent);

        let results = discovery.find_by_type("analyzer").await;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "test-1");
    }

    #[tokio::test]
    async fn test_agent_discovery_find_by_tag() {
        let tempdir = tempfile::tempdir().unwrap();
        let discovery = AgentDiscovery::new(tempdir.path()).await.unwrap();

        let mut agent = AgentConfig::new("test-1", "Test", "0.1.0", "analyzer");
        agent.tags.push("critical".to_string());
        discovery
            .agents
            .write()
            .await
            .insert("test-1".to_string(), agent);

        let results = discovery.find_by_tag("critical").await;
        assert_eq!(results.len(), 1);
    }
}
