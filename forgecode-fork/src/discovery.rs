//! Agent discovery system for automatic detection and registration

use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::mpsc;

use crate::config::AgentConfig;
use crate::error::{ForgeError, Result};
use crate::parser::YamlParser;
use crate::registry::AgentRegistry;

/// Agent discovery system that automatically finds and registers agents
pub struct AgentDiscovery {
    discovery_path: PathBuf,
    registry: Arc<AgentRegistry>,
    watcher: Arc<RwLock<Option<RecommendedWatcher>>>,
}

impl AgentDiscovery {
    /// Create a new agent discovery system
    ///
    /// # Arguments
    /// * `discovery_path` - Path to directory containing agent YAML files
    /// * `registry` - Agent registry for storing discovered agents
    ///
    /// # Returns
    /// * `Ok(AgentDiscovery)` - Successfully initialized discovery system
    /// * `Err(ForgeError)` - If initialization fails
    pub fn new<P: AsRef<Path>>(discovery_path: P, registry: Arc<AgentRegistry>) -> Result<Self> {
        let discovery_path = discovery_path.as_ref().to_path_buf();

        if !discovery_path.exists() {
            std::fs::create_dir_all(&discovery_path).map_err(|e| {
                ForgeError::DiscoveryError(format!(
                    "Failed to create discovery directory {}: {}",
                    discovery_path.display(),
                    e
                ))
            })?;
        }

        Ok(AgentDiscovery {
            discovery_path,
            registry,
            watcher: Arc::new(RwLock::new(None)),
        })
    }

    /// Discover and register all agents in the discovery path
    ///
    /// # Returns
    /// * `Ok(Vec<AgentConfig>)` - Vector of discovered and registered agents
    /// * `Err(ForgeError)` - If discovery fails
    pub async fn discover(&self) -> Result<Vec<AgentConfig>> {
        tracing::info!(
            "Discovering agents in {}",
            self.discovery_path.display()
        );

        let configs = YamlParser::parse_directory(&self.discovery_path)?;

        for config in &configs {
            self.registry.register(config.clone()).await?;
            tracing::debug!("Registered agent: {}", config.id);
        }

        tracing::info!("Discovered and registered {} agents", configs.len());

        Ok(configs)
    }

    /// Initialize hot reload to watch for agent configuration changes
    ///
    /// This function starts a file watcher that automatically reloads agents
    /// when YAML files change, are created, or are deleted.
    ///
    /// # Returns
    /// * `Ok(())` - Successfully initialized hot reload
    /// * `Err(ForgeError)` - If initialization fails
    pub fn initialize_hot_reload(&self) -> Result<()> {
        let registry = Arc::clone(&self.registry);
        let discovery_path_clone = self.discovery_path.clone();

        let (tx, mut rx) = mpsc::channel(32);

        let watcher = notify::recommended_watcher(move |res| {
            if let Ok(event) = res {
                let _ = tx.blocking_send(event);
            }
        })
        .map_err(|e| {
            ForgeError::DiscoveryError(format!("Failed to initialize file watcher: {}", e))
        })?;

        let mut watcher_guard = self.watcher.write().map_err(|e| {
            ForgeError::DiscoveryError(format!("Failed to acquire watcher lock: {}", e))
        })?;

        *watcher_guard = Some(watcher);

        // Drop the lock before spawning the task
        drop(watcher_guard);

        // Spawn async task to handle file changes
        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                if let Err(e) = Self::handle_fs_event(&event, &registry, &discovery_path_clone).await {
                    tracing::error!("Error handling file system event: {}", e);
                }
            }
        });

        // Enable watching
        if let Ok(mut watcher_guard) = self.watcher.write() {
            if let Some(watcher) = watcher_guard.as_mut() {
                watcher
                    .watch(&self.discovery_path, RecursiveMode::Recursive)
                    .map_err(|e| {
                        ForgeError::DiscoveryError(format!("Failed to watch path: {}", e))
                    })?;
                tracing::info!("Hot reload enabled for {}", self.discovery_path.display());
            }
        }

        Ok(())
    }

    /// Handle a file system event
    async fn handle_fs_event(
        event: &notify::Event,
        registry: &AgentRegistry,
        _discovery_path: &Path,
    ) -> Result<()> {
        use notify::EventKind;

        match &event.kind {
            EventKind::Create(_) | EventKind::Modify(_) => {
                for path in &event.paths {
                    if Self::is_yaml_file(path) {
                        match YamlParser::parse_file(path) {
                            Ok(config) => {
                                registry.register(config.clone()).await?;
                                tracing::info!("Reloaded agent: {}", config.id);
                            }
                            Err(e) => {
                                tracing::warn!("Failed to reload agent from {}: {}", path.display(), e);
                            }
                        }
                    }
                }
            }
            EventKind::Remove(_) => {
                for path in &event.paths {
                    if Self::is_yaml_file(path) {
                        // Try to extract agent ID from filename
                        if let Some(stem) = path.file_stem() {
                            let agent_id = stem.to_string_lossy();
                            registry.unregister(&agent_id).await?;
                            tracing::info!("Unregistered removed agent: {}", agent_id);
                        }
                    }
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Check if a path points to a YAML file
    fn is_yaml_file(path: &Path) -> bool {
        matches!(
            path.extension().and_then(|s| s.to_str()),
            Some("yaml" | "yml")
        )
    }

    /// Get the discovery path
    pub fn discovery_path(&self) -> &Path {
        &self.discovery_path
    }

    /// Get reference to the registry
    pub fn registry(&self) -> &AgentRegistry {
        &self.registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_discovery_initialization() {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let registry = Arc::new(AgentRegistry::new());

        let discovery =
            AgentDiscovery::new(temp_dir.path(), registry).expect("Failed to create discovery");

        assert_eq!(discovery.discovery_path(), temp_dir.path());
    }

    #[tokio::test]
    async fn test_discover_agents() {
        let yaml1 = r#"
id: discover-agent-1
name: Discovery Agent 1
description: First discovery agent
instruction: Do something
input_schema: {}
output_schema: {}
"#;

        let yaml2 = r#"
id: discover-agent-2
name: Discovery Agent 2
description: Second discovery agent
instruction: Do something else
input_schema: {}
output_schema: {}
"#;

        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let file1 = temp_dir.path().join("agent1.yaml");
        let file2 = temp_dir.path().join("agent2.yaml");

        std::fs::write(&file1, yaml1).expect("Failed to write file 1");
        std::fs::write(&file2, yaml2).expect("Failed to write file 2");

        let registry = Arc::new(AgentRegistry::new());
        let discovery =
            AgentDiscovery::new(temp_dir.path(), registry).expect("Failed to create discovery");

        let agents = discovery.discover().await.expect("Failed to discover");

        assert_eq!(agents.len(), 2);
        assert_eq!(agents[0].id, "discover-agent-1");
        assert_eq!(agents[1].id, "discover-agent-2");

        // Verify agents are registered
        let agent1 = discovery
            .registry()
            .get("discover-agent-1")
            .await
            .expect("Failed to get agent")
            .expect("Agent 1 not found");
        assert_eq!(agent1.name, "Discovery Agent 1");
    }

    #[test]
    fn test_is_yaml_file() {
        assert!(AgentDiscovery::is_yaml_file(Path::new("agent.yaml")));
        assert!(AgentDiscovery::is_yaml_file(Path::new("agent.yml")));
        assert!(!AgentDiscovery::is_yaml_file(Path::new("agent.txt")));
        assert!(!AgentDiscovery::is_yaml_file(Path::new("agent.json")));
    }

    #[tokio::test]
    async fn test_discovery_with_subdirectories() {
        let yaml = r#"
id: nested-agent
name: Nested Agent
description: Agent in subdirectory
instruction: Do nested work
input_schema: {}
output_schema: {}
"#;

        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let subdir = temp_dir.path().join("agents");
        std::fs::create_dir(&subdir).expect("Failed to create subdir");

        let agent_file = subdir.join("agent.yaml");
        std::fs::write(&agent_file, yaml).expect("Failed to write file");

        let registry = Arc::new(AgentRegistry::new());
        let discovery =
            AgentDiscovery::new(temp_dir.path(), registry).expect("Failed to create discovery");

        let agents = discovery.discover().await.expect("Failed to discover");

        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0].id, "nested-agent");
    }
}
