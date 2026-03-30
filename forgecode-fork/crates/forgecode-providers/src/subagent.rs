//! Subagent YAML-based configuration system
//!
//! Provides framework for discovering, loading, and configuring subagents via YAML files.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::error::{Error, Result};

/// Trait that all subagent configurations must implement
pub trait SubagentConfig: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn agent_type(&self) -> &str;
    fn validate(&self) -> Result<()>;
    fn to_yaml(&self) -> Result<String>;
    fn from_yaml(yaml: &str) -> Result<Self> where Self: Sized;
}

/// Concrete subagent configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub version: String,
    pub agent_type: String,
    pub description: Option<String>,
    pub provider: AgentProviderConfig,
    pub parameters: HashMap<String, serde_yaml::Value>,
    pub tags: Vec<String>,
    pub enabled: bool,
    pub timeout_ms: Option<u64>,
    pub max_retries: Option<u32>,
    pub metadata: Option<HashMap<String, String>>,
}

/// Provider configuration for a subagent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentProviderConfig {
    pub provider_type: String,
    pub model: String,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub top_p: Option<f32>,
}

impl SubagentConfig for Agent {
    fn id(&self) -> &str { &self.id }
    fn name(&self) -> &str { &self.name }
    fn version(&self) -> &str { &self.version }
    fn agent_type(&self) -> &str { &self.agent_type }

    fn validate(&self) -> Result<()> {
        if self.id.is_empty() {
            return Err(Error::InvalidConfig { field: "id".to_string(), reason: "id cannot be empty".to_string() });
        }
        if self.name.is_empty() {
            return Err(Error::InvalidConfig { field: "name".to_string(), reason: "name cannot be empty".to_string() });
        }
        if self.version.is_empty() {
            return Err(Error::InvalidConfig { field: "version".to_string(), reason: "version cannot be empty".to_string() });
        }
        if self.agent_type.is_empty() {
            return Err(Error::InvalidConfig { field: "agent_type".to_string(), reason: "agent_type cannot be empty".to_string() });
        }
        if self.provider.provider_type.is_empty() {
            return Err(Error::InvalidConfig { field: "provider.provider_type".to_string(), reason: "provider_type cannot be empty".to_string() });
        }
        if self.provider.model.is_empty() {
            return Err(Error::InvalidConfig { field: "provider.model".to_string(), reason: "provider model cannot be empty".to_string() });
        }
        if let Some(temp) = self.provider.temperature {
            if !(0.0..=2.0).contains(&temp) {
                return Err(Error::InvalidConfig { field: "provider.temperature".to_string(), reason: "temperature must be between 0.0 and 2.0".to_string() });
            }
        }
        if let Some(top_p) = self.provider.top_p {
            if !(0.0..=1.0).contains(&top_p) {
                return Err(Error::InvalidConfig { field: "provider.top_p".to_string(), reason: "top_p must be between 0.0 and 1.0".to_string() });
            }
        }
        Ok(())
    }

    fn to_yaml(&self) -> Result<String> {
        serde_yaml::to_string(self).map_err(|e| Error::Serialization(e.to_string()))
    }

    fn from_yaml(yaml: &str) -> Result<Self> {
        serde_yaml::from_str(yaml).map_err(|e| Error::Serialization(e.to_string()))
    }
}

/// Agent discovery and management system
pub struct AgentDiscovery {
    agents_dir: PathBuf,
    agents: Arc<RwLock<HashMap<String, Agent>>>,
    #[allow(dead_code)]
    watch_enabled: bool,
}

impl AgentDiscovery {
    pub async fn new(agents_dir: impl AsRef<Path>) -> Result<Self> {
        let agents_dir = agents_dir.as_ref().to_path_buf();
        if !agents_dir.exists() {
            fs::create_dir_all(&agents_dir).map_err(|e| Error::InvalidConfig { field: "agents_dir".to_string(), reason: format!("failed to create agents directory: {}", e) })?;
        }
        Ok(Self { agents_dir, agents: Arc::new(RwLock::new(HashMap::new())), watch_enabled: false })
    }

    pub async fn load_all(&self) -> Result<Vec<Agent>> {
        let mut agents = HashMap::new();
        for entry in fs::read_dir(&self.agents_dir).map_err(|e| Error::InvalidConfig { field: "agents_dir".to_string(), reason: format!("failed to read agents directory: {}", e) })? {
            let entry = entry.map_err(|e| Error::InvalidConfig { field: "agents_dir".to_string(), reason: format!("failed to read directory entry: {}", e) })?;
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "yaml" || ext == "yml") {
                match self.load_agent(&path).await {
                    Ok(agent) => { agents.insert(agent.id.clone(), agent); }
                    Err(e) => { eprintln!("Failed to load agent from {:?}: {}", path, e); }
                }
            }
        }
        *self.agents.write().await = agents.clone();
        Ok(agents.into_values().collect())
    }

    pub async fn load_agent(&self, path: impl AsRef<Path>) -> Result<Agent> {
        let path = path.as_ref();
        let content = fs::read_to_string(path).map_err(|e| Error::InvalidConfig { field: "file".to_string(), reason: format!("failed to read agent file {:?}: {}", path, e) })?;
        let agent: Agent = Agent::from_yaml(&content)?;
        agent.validate()?;
        Ok(agent)
    }

    pub async fn get_agent(&self, id: &str) -> Result<Option<Agent>> {
        let agents = self.agents.read().await;
        Ok(agents.get(id).cloned())
    }

    pub async fn list_agents(&self) -> Result<Vec<Agent>> {
        let agents = self.agents.read().await;
        Ok(agents.values().cloned().collect())
    }

    pub async fn reload(&self) -> Result<Vec<Agent>> {
        self.load_all().await
    }

    pub async fn find_by_type(&self, agent_type: &str) -> Result<Vec<Agent>> {
        let agents = self.agents.read().await;
        Ok(agents.values().filter(|a| a.agent_type == agent_type).cloned().collect())
    }

    pub async fn find_by_tag(&self, tag: &str) -> Result<Vec<Agent>> {
        let agents = self.agents.read().await;
        Ok(agents.values().filter(|a| a.tags.contains(&tag.to_string())).cloned().collect())
    }

    pub fn agents_dir(&self) -> &Path { &self.agents_dir }
    pub async fn exists(&self, id: &str) -> bool { self.agents.read().await.contains_key(id) }
    pub async fn count(&self) -> usize { self.agents.read().await.len() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_agent() -> Agent {
        Agent {
            id: "test-analyzer".to_string(),
            name: "Test Analyzer".to_string(),
            version: "1.0.0".to_string(),
            agent_type: "analyzer".to_string(),
            description: Some("A test analyzer agent".to_string()),
            provider: AgentProviderConfig {
                provider_type: "openrouter".to_string(),
                model: "gpt-4".to_string(),
                temperature: Some(0.7),
                max_tokens: Some(2048),
                top_p: Some(0.9),
            },
            parameters: HashMap::new(),
            tags: vec!["test".to_string(), "analyzer".to_string()],
            enabled: true,
            timeout_ms: Some(5000),
            max_retries: Some(3),
            metadata: None,
        }
    }

    #[test]
    fn test_agent_creation() {
        let agent = create_test_agent();
        assert_eq!(agent.id, "test-analyzer");
        assert_eq!(agent.name, "Test Analyzer");
        assert_eq!(agent.version, "1.0.0");
        assert_eq!(agent.agent_type, "analyzer");
    }

    #[test]
    fn test_agent_validate_success() {
        let agent = create_test_agent();
        assert!(agent.validate().is_ok());
    }

    #[test]
    fn test_agent_validate_empty_id() {
        let mut agent = create_test_agent();
        agent.id = String::new();
        assert!(agent.validate().is_err());
    }

    #[test]
    fn test_agent_validate_empty_name() {
        let mut agent = create_test_agent();
        agent.name = String::new();
        assert!(agent.validate().is_err());
    }

    #[test]
    fn test_agent_validate_empty_version() {
        let mut agent = create_test_agent();
        agent.version = String::new();
        assert!(agent.validate().is_err());
    }

    #[test]
    fn test_agent_validate_empty_agent_type() {
        let mut agent = create_test_agent();
        agent.agent_type = String::new();
        assert!(agent.validate().is_err());
    }

    #[test]
    fn test_agent_validate_invalid_temperature() {
        let mut agent = create_test_agent();
        agent.provider.temperature = Some(3.0);
        assert!(agent.validate().is_err());
    }

    #[test]
    fn test_agent_validate_invalid_top_p() {
        let mut agent = create_test_agent();
        agent.provider.top_p = Some(1.5);
        assert!(agent.validate().is_err());
    }

    #[test]
    fn test_agent_validate_empty_provider_type() {
        let mut agent = create_test_agent();
        agent.provider.provider_type = String::new();
        assert!(agent.validate().is_err());
    }

    #[test]
    fn test_agent_validate_empty_model() {
        let mut agent = create_test_agent();
        agent.provider.model = String::new();
        assert!(agent.validate().is_err());
    }

    #[test]
    fn test_agent_subagent_config_trait() {
        let agent = create_test_agent();
        assert_eq!(agent.id(), "test-analyzer");
        assert_eq!(agent.name(), "Test Analyzer");
        assert_eq!(agent.version(), "1.0.0");
        assert_eq!(agent.agent_type(), "analyzer");
    }

    #[test]
    fn test_agent_to_yaml() {
        let agent = create_test_agent();
        let yaml = agent.to_yaml();
        assert!(yaml.is_ok());
        let yaml_str = yaml.unwrap();
        assert!(yaml_str.contains("test-analyzer"));
        assert!(yaml_str.contains("Test Analyzer"));
    }

    #[test]
    fn test_agent_from_yaml() {
        let agent = create_test_agent();
        let yaml_str = agent.to_yaml().unwrap();
        let loaded = Agent::from_yaml(&yaml_str);
        assert!(loaded.is_ok());
        let loaded_agent = loaded.unwrap();
        assert_eq!(loaded_agent.id, "test-analyzer");
        assert_eq!(loaded_agent.name, "Test Analyzer");
    }

    #[test]
    fn test_agent_from_yaml_invalid() {
        let invalid_yaml = "invalid: [yaml: structure:";
        let result = Agent::from_yaml(invalid_yaml);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_agent_discovery_creation() {
        let temp_dir = "/tmp/test_agents_discovery";
        let _ = fs::remove_dir_all(temp_dir);
        let discovery = AgentDiscovery::new(temp_dir).await;
        assert!(discovery.is_ok());
    }

    #[tokio::test]
    async fn test_agent_discovery_load_all_empty() {
        let temp_dir = "/tmp/test_agents_empty";
        let _ = fs::remove_dir_all(temp_dir);
        let discovery = AgentDiscovery::new(temp_dir).await.unwrap();
        let agents = discovery.load_all().await.unwrap();
        assert_eq!(agents.len(), 0);
    }

    #[tokio::test]
    async fn test_agent_discovery_load_agent() {
        let temp_dir = "/tmp/test_agents_load";
        let _ = fs::remove_dir_all(temp_dir);
        fs::create_dir_all(temp_dir).unwrap();
        let agent = create_test_agent();
        let yaml = agent.to_yaml().unwrap();
        let agent_file = format!("{}/test-analyzer.yaml", temp_dir);
        fs::write(&agent_file, yaml).unwrap();
        let discovery = AgentDiscovery::new(temp_dir).await.unwrap();
        let loaded = discovery.load_agent(&agent_file).await.unwrap();
        assert_eq!(loaded.id, "test-analyzer");
    }

    #[tokio::test]
    async fn test_agent_discovery_get_agent() {
        let temp_dir = "/tmp/test_agents_get";
        let _ = fs::remove_dir_all(temp_dir);
        fs::create_dir_all(temp_dir).unwrap();
        let agent = create_test_agent();
        let yaml = agent.to_yaml().unwrap();
        fs::write(format!("{}/test-analyzer.yaml", temp_dir), yaml).unwrap();
        let discovery = AgentDiscovery::new(temp_dir).await.unwrap();
        discovery.load_all().await.unwrap();
        let found = discovery.get_agent("test-analyzer").await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, "test-analyzer");
    }

    #[tokio::test]
    async fn test_agent_discovery_get_agent_not_found() {
        let temp_dir = "/tmp/test_agents_not_found";
        let _ = fs::remove_dir_all(temp_dir);
        let discovery = AgentDiscovery::new(temp_dir).await.unwrap();
        discovery.load_all().await.unwrap();
        let found = discovery.get_agent("nonexistent").await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_agent_discovery_list_agents() {
        let temp_dir = "/tmp/test_agents_list";
        let _ = fs::remove_dir_all(temp_dir);
        fs::create_dir_all(temp_dir).unwrap();
        for i in 0..3 {
            let mut agent = create_test_agent();
            agent.id = format!("agent-{}", i);
            agent.name = format!("Agent {}", i);
            let yaml = agent.to_yaml().unwrap();
            fs::write(format!("{}/agent-{}.yaml", temp_dir, i), yaml).unwrap();
        }
        let discovery = AgentDiscovery::new(temp_dir).await.unwrap();
        let agents = discovery.list_agents().await.unwrap();
        assert_eq!(agents.len(), 0);
        discovery.load_all().await.unwrap();
        let agents = discovery.list_agents().await.unwrap();
        assert_eq!(agents.len(), 3);
    }

    #[tokio::test]
    async fn test_agent_discovery_reload() {
        let temp_dir = "/tmp/test_agents_reload";
        let _ = fs::remove_dir_all(temp_dir);
        fs::create_dir_all(temp_dir).unwrap();
        let agent = create_test_agent();
        let yaml = agent.to_yaml().unwrap();
        fs::write(format!("{}/test-analyzer.yaml", temp_dir), yaml).unwrap();
        let discovery = AgentDiscovery::new(temp_dir).await.unwrap();
        discovery.load_all().await.unwrap();
        assert_eq!(discovery.count().await, 1);
        let agents = discovery.reload().await.unwrap();
        assert_eq!(agents.len(), 1);
    }

    #[tokio::test]
    async fn test_agent_discovery_find_by_type() {
        let temp_dir = "/tmp/test_agents_type";
        let _ = fs::remove_dir_all(temp_dir);
        fs::create_dir_all(temp_dir).unwrap();
        let mut agent1 = create_test_agent();
        agent1.id = "analyzer-1".to_string();
        agent1.agent_type = "analyzer".to_string();
        fs::write(format!("{}/analyzer-1.yaml", temp_dir), agent1.to_yaml().unwrap()).unwrap();
        let mut agent2 = create_test_agent();
        agent2.id = "validator-1".to_string();
        agent2.agent_type = "validator".to_string();
        fs::write(format!("{}/validator-1.yaml", temp_dir), agent2.to_yaml().unwrap()).unwrap();
        let discovery = AgentDiscovery::new(temp_dir).await.unwrap();
        discovery.load_all().await.unwrap();
        let analyzers = discovery.find_by_type("analyzer").await.unwrap();
        assert_eq!(analyzers.len(), 1);
        assert_eq!(analyzers[0].agent_type, "analyzer");
    }

    #[tokio::test]
    async fn test_agent_discovery_find_by_tag() {
        let temp_dir = "/tmp/test_agents_tag";
        let _ = fs::remove_dir_all(temp_dir);
        fs::create_dir_all(temp_dir).unwrap();
        let mut agent = create_test_agent();
        agent.tags = vec!["critical".to_string(), "production".to_string()];
        fs::write(format!("{}/critical-agent.yaml", temp_dir), agent.to_yaml().unwrap()).unwrap();
        let discovery = AgentDiscovery::new(temp_dir).await.unwrap();
        discovery.load_all().await.unwrap();
        let critical_agents = discovery.find_by_tag("critical").await.unwrap();
        assert_eq!(critical_agents.len(), 1);
        assert!(critical_agents[0].tags.contains(&"critical".to_string()));
    }

    #[tokio::test]
    async fn test_agent_discovery_exists() {
        let temp_dir = "/tmp/test_agents_exists";
        let _ = fs::remove_dir_all(temp_dir);
        fs::create_dir_all(temp_dir).unwrap();
        let agent = create_test_agent();
        fs::write(format!("{}/test-analyzer.yaml", temp_dir), agent.to_yaml().unwrap()).unwrap();
        let discovery = AgentDiscovery::new(temp_dir).await.unwrap();
        discovery.load_all().await.unwrap();
        assert!(discovery.exists("test-analyzer").await);
        assert!(!discovery.exists("nonexistent").await);
    }

    #[tokio::test]
    async fn test_agent_discovery_count() {
        let temp_dir = "/tmp/test_agents_count";
        let _ = fs::remove_dir_all(temp_dir);
        fs::create_dir_all(temp_dir).unwrap();
        for i in 0..5 {
            let mut agent = create_test_agent();
            agent.id = format!("agent-{}", i);
            fs::write(format!("{}/agent-{}.yaml", temp_dir, i), agent.to_yaml().unwrap()).unwrap();
        }
        let discovery = AgentDiscovery::new(temp_dir).await.unwrap();
        discovery.load_all().await.unwrap();
        assert_eq!(discovery.count().await, 5);
    }

    #[test]
    fn test_agent_provider_config_validation_bounds() {
        let mut agent = create_test_agent();
        agent.provider.temperature = Some(0.0);
        assert!(agent.validate().is_ok());
        agent.provider.temperature = Some(2.0);
        assert!(agent.validate().is_ok());
        agent.provider.top_p = Some(0.0);
        assert!(agent.validate().is_ok());
        agent.provider.top_p = Some(1.0);
        assert!(agent.validate().is_ok());
    }

    #[tokio::test]
    async fn test_agent_discovery_mixed_yaml_extensions() {
        let temp_dir = "/tmp/test_agents_mixed";
        let _ = fs::remove_dir_all(temp_dir);
        fs::create_dir_all(temp_dir).unwrap();
        let mut agent1 = create_test_agent();
        agent1.id = "yaml-agent".to_string();
        fs::write(format!("{}/agent-yaml.yaml", temp_dir), agent1.to_yaml().unwrap()).unwrap();
        let mut agent2 = create_test_agent();
        agent2.id = "yml-agent".to_string();
        fs::write(format!("{}/agent-yml.yml", temp_dir), agent2.to_yaml().unwrap()).unwrap();
        fs::write(format!("{}/readme.txt", temp_dir), "This should be ignored").unwrap();
        let discovery = AgentDiscovery::new(temp_dir).await.unwrap();
        let agents = discovery.load_all().await.unwrap();
        assert_eq!(agents.len(), 2);
    }
}
