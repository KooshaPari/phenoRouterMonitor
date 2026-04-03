//! VCS port — version control system abstraction.
//!
//! Traceability: FR-010, FR-014, FR-017 / WP05-T026

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
    ) -> impl std::future::Future<Output = Result<PathBuf, DomainError>> + Send;

    fn list_worktrees(
        &self,
    ) -> impl std::future::Future<Output = Result<Vec<WorktreeInfo>, DomainError>> + Send;

    fn cleanup_worktree(
        &self,
        worktree_path: &Path,
    ) -> impl std::future::Future<Output = Result<(), DomainError>> + Send;

    fn create_branch(
        &self,
        branch_name: &str,
        base: &str,
    ) -> impl std::future::Future<Output = Result<(), DomainError>> + Send;

    fn checkout_branch(
        &self,
        branch_name: &str,
    ) -> impl std::future::Future<Output = Result<(), DomainError>> + Send;

    fn merge_to_target(
        &self,
        source: &str,
        target: &str,
    ) -> impl std::future::Future<Output = Result<MergeResult, DomainError>> + Send;

    fn detect_conflicts(
        &self,
        source: &str,
        target: &str,
    ) -> impl std::future::Future<Output = Result<Vec<ConflictInfo>, DomainError>> + Send;

    fn read_artifact(
        &self,
        feature_slug: &str,
        relative_path: &str,
    ) -> impl std::future::Future<Output = Result<String, DomainError>> + Send;

    fn write_artifact(
        &self,
        feature_slug: &str,
        relative_path: &str,
        content: &str,
    ) -> impl std::future::Future<Output = Result<(), DomainError>> + Send;

    fn artifact_exists(
        &self,
        feature_slug: &str,
        relative_path: &str,
    ) -> impl std::future::Future<Output = Result<bool, DomainError>> + Send;

    fn scan_feature_artifacts(
        &self,
        feature_slug: &str,
    ) -> impl std::future::Future<Output = Result<FeatureArtifacts, DomainError>> + Send;
}
