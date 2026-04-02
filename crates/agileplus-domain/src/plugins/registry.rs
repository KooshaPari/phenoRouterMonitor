//! Plugin registry for managing VCS and storage adapters.
//!
//! Provides runtime discovery and management of plugins using the trait objects
//! from `agileplus-plugin-core`.
//!
//! # Example
//!
//! ```ignore
//! use agileplus_domain::plugins::PluginRegistry;
//! let registry = PluginRegistry::new();
//! registry.register_git_plugin("/path/to/repo")?;
//! let vcs = registry.get_vcs_adapter();
//! ```
//!
//! Traceability: WP11-T071

use std::sync::Arc;

use agileplus_plugin_core::{VcsPlugin, StoragePlugin, PluginRegistry};

/// Domain-level plugin registry that wraps the core plugin registry
/// and provides domain-specific integration.
pub struct DomainPluginRegistry {
    /// Core plugin registry for VCS operations.
    vcs_registry: PluginRegistry<Arc<dyn VcsPlugin>>,
    /// Core plugin registry for storage operations.
    storage_registry: PluginRegistry<Arc<dyn StoragePlugin>>,
}

impl DomainPluginRegistry {
    /// Create a new domain plugin registry.
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            vcs_registry: PluginRegistry::new("vcs"),
            storage_registry: PluginRegistry::new("storage"),
        }
    }

    /// Register a VCS plugin instance.
    #[allow(dead_code)]
    pub fn register_vcs_plugin(&mut self, name: &str, plugin: Arc<dyn VcsPlugin>) {
        self.vcs_registry.register(name, plugin);
    }

    /// Register a storage plugin instance.
    #[allow(dead_code)]
    pub fn register_storage_plugin(&mut self, name: &str, plugin: Arc<dyn StoragePlugin>) {
        self.storage_registry.register(name, plugin);
    }

    /// Get a VCS adapter by name.
    #[allow(dead_code)]
    pub fn get_vcs_adapter(&self, name: &str) -> Option<Arc<dyn VcsPlugin>> {
        self.vcs_registry.get(name)
    }

    /// Get a storage adapter by name.
    #[allow(dead_code)]
    pub fn get_storage_adapter(&self, name: &str) -> Option<Arc<dyn StoragePlugin>> {
        self.storage_registry.get(name)
    }

    /// List all registered VCS adapter names.
    #[allow(dead_code)]
    pub fn list_vcs_adapters(&self) -> Vec<String> {
        self.vcs_registry.list_plugins()
    }

    /// List all registered storage adapter names.
    #[allow(dead_code)]
    pub fn list_storage_adapters(&self) -> Vec<String> {
        self.storage_registry.list_plugins()
    }
}

impl Default for DomainPluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}
