//! Port traits: abstractions over external dependencies (VCS, review system).
//!
//! All real implementations live in adapter crates. Test doubles live in
//! `#[cfg(test)]` modules alongside the code under test.

use crate::types::{CiStatus, DomainError, ReviewComment, ReviewOutcome};
use async_trait::async_trait;
use std::path::PathBuf;
use std::time::Duration;

// ─── VcsPort ──────────────────────────────────────────────────────────────────

/// Abstraction over git worktree management.
#[async_trait]
pub trait VcsPort: Send + Sync {
    /// Create a new git worktree for the given feature / WP and return its
    /// absolute path.
    async fn create_worktree(
        &self,
        feature_slug: &str,
        wp_id: &str,
    ) -> Result<PathBuf, DomainError>;

    /// Remove the worktree once the job is complete (optional cleanup).
    async fn remove_worktree(&self, worktree_path: &PathBuf) -> Result<(), DomainError>;

    /// List commit SHAs added to `worktree_path` since `since_sha` (exclusive).
    async fn new_commits_since(
        &self,
        worktree_path: &PathBuf,
        since_sha: &str,
    ) -> Result<Vec<String>, DomainError>;
}

// ─── ReviewPort ───────────────────────────────────────────────────────────────

/// Abstraction over code-review tooling (GitHub Reviews + Coderabbit).
#[async_trait]
pub trait ReviewPort: Send + Sync {
    /// Block until a review decision arrives or `timeout` elapses.
    async fn await_review(
        &self,
        pr_url: &str,
        timeout: Duration,
    ) -> Result<ReviewOutcome, DomainError>;

    /// Return all actionable (critical / major) comments on the PR.
    async fn get_actionable_comments(
        &self,
        pr_url: &str,
    ) -> Result<Vec<ReviewComment>, DomainError>;

    /// Block until CI finishes or `timeout` elapses.
    async fn await_ci(
        &self,
        pr_url: &str,
        timeout: Duration,
    ) -> Result<CiStatus, DomainError>;
}
