//! Git repository operations trait.
//! Traces to: FR-PHENO-GIT-001, FR-PHENO-GIT-002, FR-PHENO-GIT-003

use crate::{GitError, Result};

/// Core trait for git repository operations.
///
/// Traces to: FR-PHENO-GIT-001 (GitRepository trait SHALL support commit_hash, current_branch, list_tags)
pub trait GitRepositoryOps {
    /// Get the commit hash of the current HEAD.
    /// Returns full 40-character SHA-1 hash.
    /// Traces to: FR-PHENO-GIT-002 (commit_hash SHALL return full SHA-1 hash)
    fn commit_hash(&self) -> Result<String>;

    /// Get the current branch name.
    /// Returns None if in detached HEAD state.
    /// Traces to: FR-PHENO-GIT-003 (current_branch SHALL return branch name or None)
    fn current_branch(&self) -> Result<Option<String>>;

    /// List all tags in the repository.
    /// Tags are sorted alphabetically.
    /// Traces to: FR-PHENO-GIT-004 (list_tags SHALL return sorted tag names)
    fn list_tags(&self) -> Result<Vec<String>>;

    /// Get the count of commits from HEAD.
    /// Traces to: FR-PHENO-GIT-005 (commit_count SHALL return total commits reachable from HEAD)
    fn commit_count(&self) -> Result<usize>;

    /// Get the latest commit message.
    /// Traces to: FR-PHENO-GIT-006 (latest_message SHALL return HEAD commit message)
    fn latest_message(&self) -> Result<Option<String>>;

    /// Check if working directory is clean.
    /// Traces to: FR-PHENO-GIT-007 (is_clean SHALL detect uncommitted changes)
    fn is_clean(&self) -> Result<bool>;

    /// Get absolute path to repository root.
    /// Traces to: FR-PHENO-GIT-008 (repo_path SHALL return absolute path)
    fn repo_path(&self) -> Result<std::path::PathBuf>;
}
