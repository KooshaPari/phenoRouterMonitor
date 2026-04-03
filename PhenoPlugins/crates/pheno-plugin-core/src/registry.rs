//! Plugin registry for managing adapter registrations.
//!
//! The registry is the central component that holds all plugin instances.
//! It provides lookup by type and name.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::error::{PluginError, PluginResult};
use crate::traits::{StoragePlugin, VcsPlugin};

/// Thread-safe plugin registry.
/// Thread-safe plugin registry.
///
/// Manages registration and lookup of all adapter plugins.
/// Uses interior mutability for concurrent access.
pub struct PluginRegistry {
    vcs: RwLock<HashMap<String, Arc<dyn VcsPlugin>>>,
    storage: RwLock<HashMap<String, Arc<dyn StoragePlugin>>>,
    initialized: RwLock<bool>,
}
impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginRegistry {
    /// Creates a new empty registry.
    pub fn new() -> Self {
        Self {
            vcs: RwLock::new(HashMap::new()),
            storage: RwLock::new(HashMap::new()),
            initialized: RwLock::new(false),
        }
    }

    /// Mark registry as initialized.
    ///
    /// After initialization, no new plugins can be registered.
    pub fn finalize(&self) -> PluginResult<()> {
        let mut initialized = self.initialized.write().map_err(|_| {
            PluginError::Initialization("Poisoned lock".to_string())
        })?;
        *initialized = true;
        Ok(())
    }

    /// Check if registry is finalized.
    pub fn is_finalized(&self) -> bool {
        self.initialized
            .read()
            .map(|g| *g)
            .unwrap_or(false)
    }

    // -- VCS plugin management --

    /// Register a VCS adapter plugin.
    pub fn register_vcs(&self, plugin: Box<dyn VcsPlugin>) -> PluginResult<()> {
        if self.is_finalized() {
            return Err(PluginError::Initialization(
                "Registry is finalized, cannot register new plugins".to_string(),
            ));
        }

        let name = plugin.name().to_string();
        let mut vcs = self.vcs.write().map_err(|_| {
            PluginError::Initialization("Poisoned lock".to_string())
        })?;

        if vcs.contains_key(&name) {
            return Err(PluginError::AlreadyRegistered(format!(
                "VCS plugin '{}' already registered",
                name
            )));
        }

        vcs.insert(name, Arc::from(plugin));
        Ok(())
    }

    /// Get a VCS adapter by name.
    pub fn vcs(&self, name: &str) -> Option<Arc<dyn VcsPlugin>> {
        self.vcs
            .read()
            .ok()
            .and_then(|g| g.get(name).cloned())
    }

    /// Get all registered VCS adapter names.
    pub fn vcs_adapters(&self) -> Vec<String> {
        self.vcs.read().map(|g| g.keys().cloned().collect()).unwrap_or_default()
    }

    // -- Storage plugin management --

    /// Register a storage adapter plugin.
    pub fn register_storage(&self, plugin: Box<dyn StoragePlugin>) -> PluginResult<()> {
        if self.is_finalized() {
            return Err(PluginError::Initialization(
                "Registry is finalized, cannot register new plugins".to_string(),
            ));
        }

        let name = plugin.name().to_string();
        let mut storage = self.storage.write().map_err(|_| {
            PluginError::Initialization("Poisoned lock".to_string())
        })?;

        if storage.contains_key(&name) {
            return Err(PluginError::AlreadyRegistered(format!(
                "Storage plugin '{}' already registered",
                name
            )));
        }

        storage.insert(name, Arc::from(plugin));
        Ok(())
    }

    /// Get a storage adapter by name.
    pub fn storage(&self, name: &str) -> Option<Arc<dyn StoragePlugin>> {
        self.storage
            .read()
            .ok()
            .and_then(|g| g.get(name).cloned())
    }

    /// Get all registered storage adapter names.
    pub fn storage_adapters(&self) -> Vec<String> {
        self.storage.read().map(|g| g.keys().cloned().collect()).unwrap_or_default()
    }

    // -- Health checks --

    /// Check health of all registered plugins.
    pub async fn health_check(&self) -> PluginResult<()> {
        // Check VCS plugins
        for name in self.vcs_adapters() {
            if let Some(vcs) = self.vcs(&name) {
                vcs.health_check()?;
            }
        }

        // Check storage plugins
        for name in self.storage_adapters() {
            if let Some(storage) = self.storage(&name) {
                storage.health_check()?;
            }
        }

        Ok(())
    }

    /// Get registry statistics.
    pub fn stats(&self) -> RegistryStats {
        RegistryStats {
            vcs_count: self.vcs_adapters().len(),
            storage_count: self.storage_adapters().len(),
            finalized: self.is_finalized(),
        }
    }
}

/// Statistics about the plugin registry.
#[derive(Debug, Clone)]
pub struct RegistryStats {
    pub vcs_count: usize,
    pub storage_count: usize,
    pub finalized: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};
    use crate::traits::{
        AdapterPlugin, ConflictInfo, FeatureArtifacts, MergeResult, VcsPlugin, WorktreeInfo,
    };

    struct MockVcsPlugin;

    impl AdapterPlugin for MockVcsPlugin {
        fn name(&self) -> &str { "mock-vcs" }
        fn version(&self) -> &str { "0.1.0" }
        fn initialize(&self, _config: crate::traits::PluginConfig) -> PluginResult<()> {
            Ok(())
        }
    }

    #[async_trait::async_trait]
    impl VcsPlugin for MockVcsPlugin {
        async fn create_worktree(&self, _: &str, _: &str) -> PluginResult<PathBuf> {
            Ok(PathBuf::from("/tmp/test"))
        }
        async fn list_worktrees(&self) -> PluginResult<Vec<WorktreeInfo>> {
            Ok(vec![])
        }
        async fn cleanup_worktree(&self, _: &Path) -> PluginResult<()> {
            Ok(())
        }
        async fn create_branch(&self, _: &str, _: &str) -> PluginResult<()> {
            Ok(())
        }
        async fn checkout_branch(&self, _: &str) -> PluginResult<()> {
            Ok(())
        }
        async fn merge_to_target(&self, _: &str, _: &str) -> PluginResult<MergeResult> {
            Ok(MergeResult { success: true, conflicts: vec![], merged_commit: None })
        }
        async fn detect_conflicts(&self, _: &str, _: &str) -> PluginResult<Vec<ConflictInfo>> {
            Ok(vec![])
        }
        async fn read_artifact(&self, _: &str, _: &str) -> PluginResult<String> {
            Ok(String::new())
        }
        async fn write_artifact(&self, _: &str, _: &str, _: &str) -> PluginResult<()> {
            Ok(())
        }
        async fn artifact_exists(&self, _: &str, _: &str) -> PluginResult<bool> {
            Ok(false)
        }
        async fn scan_feature_artifacts(&self, _: &str) -> PluginResult<FeatureArtifacts> {
            Ok(FeatureArtifacts { meta_json: None, audit_chain: None, evidence_paths: vec![] })
        }
    }

    #[test]
    fn test_registry_creation() {
        let registry = PluginRegistry::new();
        assert!(!registry.is_finalized());
        assert_eq!(registry.stats().vcs_count, 0);
    }

    #[test]
    fn test_register_vcs_plugin() {
        let registry = PluginRegistry::new();
        let plugin = Box::new(MockVcsPlugin);

        registry.register_vcs(plugin).unwrap();

        assert!(registry.vcs("mock-vcs").is_some());
        assert_eq!(registry.stats().vcs_count, 1);
    }

    #[test]
    fn test_duplicate_registration() {
        let registry = PluginRegistry::new();
        let plugin = Box::new(MockVcsPlugin);

        registry.register_vcs(plugin).unwrap();
        let result = registry.register_vcs(Box::new(MockVcsPlugin));

        assert!(result.is_err());
    }

    #[test]
    fn test_finalize_registry() {
        let registry = PluginRegistry::new();
        registry.register_vcs(Box::new(MockVcsPlugin)).unwrap();

        registry.finalize().unwrap();
        assert!(registry.is_finalized());

        // Cannot register after finalization
        let result = registry.register_vcs(Box::new(MockVcsPlugin));
        assert!(result.is_err());
    }
}
