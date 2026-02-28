//! VCS port — version control system abstraction.
//!
//! Traceability: FR-010, FR-014, FR-017 / WP05-T026

use std::future::Future;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::DomainError;

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

/// Port for version control system operations.
///
/// Abstracts git so tests can use an in-memory mock.
/// The Git adapter (WP07) implements this with `git2`.
pub trait VcsPort: Send + Sync {
    // -- Worktree operations (FR-010) --

    /// Create a worktree for a feature work package, returning its absolute path.
    fn create_worktree(
        &self,
        feature_slug: &str,
        wp_id: &str,
    ) -> impl Future<Output = Result<PathBuf, DomainError>> + Send;

    /// List all active worktrees.
    fn list_worktrees(&self)
    -> impl Future<Output = Result<Vec<WorktreeInfo>, DomainError>> + Send;

    /// Remove a worktree at the given path.
    fn cleanup_worktree(
        &self,
        worktree_path: &Path,
    ) -> impl Future<Output = Result<(), DomainError>> + Send;

    // -- Branch operations --

    /// Create a new branch from a base ref.
    fn create_branch(
        &self,
        branch_name: &str,
        base: &str,
    ) -> impl Future<Output = Result<(), DomainError>> + Send;

    /// Check out an existing branch.
    fn checkout_branch(
        &self,
        branch_name: &str,
    ) -> impl Future<Output = Result<(), DomainError>> + Send;

    /// Merge source branch into target, returning the merge result.
    fn merge_to_target(
        &self,
        source: &str,
        target: &str,
    ) -> impl Future<Output = Result<MergeResult, DomainError>> + Send;

    /// Detect merge conflicts between two branches without performing the merge.
    fn detect_conflicts(
        &self,
        source: &str,
        target: &str,
    ) -> impl Future<Output = Result<Vec<ConflictInfo>, DomainError>> + Send;

    // -- Artifact operations (FR-014) --

    /// Read a text artifact relative to the feature directory.
    fn read_artifact(
        &self,
        feature_slug: &str,
        relative_path: &str,
    ) -> impl Future<Output = Result<String, DomainError>> + Send;

    /// Write a text artifact relative to the feature directory.
    fn write_artifact(
        &self,
        feature_slug: &str,
        relative_path: &str,
        content: &str,
    ) -> impl Future<Output = Result<(), DomainError>> + Send;

    /// Check whether an artifact exists.
    fn artifact_exists(
        &self,
        feature_slug: &str,
        relative_path: &str,
    ) -> impl Future<Output = Result<bool, DomainError>> + Send;

    // -- History scanning (FR-017) --

    /// Scan and collect all feature artifacts from the repository.
    fn scan_feature_artifacts(
        &self,
        feature_slug: &str,
    ) -> impl Future<Output = Result<FeatureArtifacts, DomainError>> + Send;
}
