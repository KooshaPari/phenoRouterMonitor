//! Core plugin traits for AgilePlus extensibility.
//!
//! These traits define the port interfaces that adapters must implement.
//! They follow the Hexagonal Architecture pattern where the core domain
//! defines the interfaces that adapters must satisfy.
//!
//! ## Dyn Compatibility
//!
//! These traits use `#[trait_variant]` to enable dynamic dispatch via `dyn Trait`.
//! This allows runtime plugin selection and swapping.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::PluginResult;

/// Configuration for a plugin adapter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    /// Plugin name
    pub name: String,
    /// Plugin version
    pub version: String,
    /// Adapter-specific configuration (JSON)
    #[serde(default)]
    pub adapter_config: serde_json::Value,
}

/// Base trait for all AgilePlus plugins.
///
/// All adapters must implement this trait to be registered in the system.
/// It provides metadata and lifecycle management for plugins.
///
/// ## Example
///
/// ```rust,ignore
/// struct GitAdapter { /* ... */ }
///
/// impl AdapterPlugin for GitAdapter {
///     fn name(&self) -> &str { "git" }
///     fn version(&self) -> &str { "0.1.0" }
///     fn initialize(&self, config: PluginConfig) -> PluginResult<()> {
///         // Initialize adapter
///         Ok(())
///     }
/// }
/// ```
pub trait AdapterPlugin: Send + Sync {
    /// Returns the plugin name (e.g., "git", "sqlite", "ollama").
    fn name(&self) -> &str;

    /// Returns the plugin version.
    fn version(&self) -> &str;

    /// Initializes the plugin with configuration.
    ///
    /// This is called once when the plugin is registered.
    fn initialize(&self, config: PluginConfig) -> PluginResult<()>;

    /// Returns the plugin health status.
    ///
    /// Returns `Ok(())` if healthy, or an error describing the issue.
    fn health_check(&self) -> PluginResult<()> {
        Ok(())
    }
}

// ============================================================================
// VCS Plugin Trait
// ============================================================================

/// Metadata about an active git worktree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeInfo {
    pub path: PathBuf,
    pub branch: String,
    pub feature_slug: String,
    pub wp_id: String,
}

/// Result of a merge operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeResult {
    pub success: bool,
    pub conflicts: Vec<ConflictInfo>,
    pub merged_commit: Option<String>,
}

/// Description of a merge conflict in a single file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictInfo {
    pub path: String,
    pub ours: Option<String>,
    pub theirs: Option<String>,
}

/// Collected feature artifacts discovered in the repository.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureArtifacts {
    pub meta_json: Option<String>,
    pub audit_chain: Option<String>,
    pub evidence_paths: Vec<String>,
}

/// VCS (Version Control System) plugin trait.
///
/// Abstracts git operations so tests can use in-memory mocks.
///
/// ## Dyn Compatibility
///
/// This trait uses `#[async_trait]` to support dynamic dispatch.
///
/// ## Implementations
///
/// - `agileplus-plugin-git`: Production git adapter using gitoxide
/// - Mock adapter: For testing without filesystem
#[async_trait::async_trait]
pub trait VcsPlugin: AdapterPlugin {
    // -- Worktree operations --

    /// Create a worktree for a feature work package.
    async fn create_worktree(
        &self,
        feature_slug: &str,
        wp_id: &str,
    ) -> PluginResult<PathBuf>;

    /// List all worktrees.
    async fn list_worktrees(&self) -> PluginResult<Vec<WorktreeInfo>>;

    /// Clean up (remove) a worktree.
    async fn cleanup_worktree(&self, worktree_path: &Path) -> PluginResult<()>;

    // -- Branch operations --

    /// Create a new branch.
    async fn create_branch(&self, branch_name: &str, base: &str) -> PluginResult<()>;

    /// Checkout a branch.
    async fn checkout_branch(&self, branch_name: &str) -> PluginResult<()>;

    // -- Merge operations --

    /// Merge source branch into target.
    async fn merge_to_target(
        &self,
        source: &str,
        target: &str,
    ) -> PluginResult<MergeResult>;

    /// Detect conflicts between branches.
    async fn detect_conflicts(
        &self,
        source: &str,
        target: &str,
    ) -> PluginResult<Vec<ConflictInfo>>;

    // -- Artifact operations --

    /// Read an artifact file.
    async fn read_artifact(
        &self,
        feature_slug: &str,
        relative_path: &str,
    ) -> PluginResult<String>;

    /// Write an artifact file.
    async fn write_artifact(
        &self,
        feature_slug: &str,
        relative_path: &str,
        content: &str,
    ) -> PluginResult<()>;

    /// Check if an artifact exists.
    async fn artifact_exists(
        &self,
        feature_slug: &str,
        relative_path: &str,
    ) -> PluginResult<bool>;

    /// Scan and collect all artifacts for a feature.
    async fn scan_feature_artifacts(
        &self,
        feature_slug: &str,
    ) -> PluginResult<FeatureArtifacts>;
}

// ============================================================================
// Storage Plugin Trait
// ============================================================================

/// Storage plugin trait.
///
/// Abstracts database operations for persistence.
///
/// ## Dyn Compatibility
///
/// This trait uses `#[async_trait]` to support dynamic dispatch.
///
/// ## Implementations
///
/// - `agileplus-plugin-sqlite`: SQLite adapter (rusqlite)
/// - `agileplus-plugin-postgres`: PostgreSQL adapter (sqlx) [future]
#[async_trait::async_trait]
pub trait StoragePlugin: AdapterPlugin {
    // -- Feature operations --

    /// Create a new feature.
    async fn create_feature(
        &self,
        feature: &serde_json::Value,
    ) -> PluginResult<i64>;

    /// Get a feature by slug.
    async fn get_feature_by_slug(
        &self,
        slug: &str,
    ) -> PluginResult<Option<serde_json::Value>>;

    /// Get a feature by ID.
    async fn get_feature_by_id(
        &self,
        id: i64,
    ) -> PluginResult<Option<serde_json::Value>>;

    /// Update feature state.
    async fn update_feature_state(&self, id: i64, state: &str) -> PluginResult<()>;

    /// List all features.
    async fn list_all_features(&self) -> PluginResult<Vec<serde_json::Value>>;

    // -- Work package operations --

    /// Create a work package.
    async fn create_work_package(
        &self,
        wp: &serde_json::Value,
    ) -> PluginResult<i64>;

    /// Get a work package by ID.
    async fn get_work_package(
        &self,
        id: i64,
    ) -> PluginResult<Option<serde_json::Value>>;

    /// Update work package state.
    async fn update_wp_state(&self, id: i64, state: &str) -> PluginResult<()>;

    // -- Audit operations --

    /// Append an audit entry.
    async fn append_audit_entry(
        &self,
        entry: &serde_json::Value,
    ) -> PluginResult<i64>;

    /// Get audit trail for a feature.
    async fn get_audit_trail(
        &self,
        feature_id: i64,
    ) -> PluginResult<Vec<serde_json::Value>>;
}
