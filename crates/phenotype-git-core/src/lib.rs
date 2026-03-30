//! # Phenotype Git Core
//!
//! Git operations via gitoxide (gix): repository info, branch, status, log.
//!
//! Migrated from libgit2 to gitoxide for better safety and performance:
//! - No C dependencies (pure Rust)
//! - Better memory safety guarantees
//! - Fixes RUSTSEC-2025-0140 (CVE-2024-24818)
//!
//! # Example
//!
//! ```rust,no_run
//! use phenotype_git_core::GitRepository;
//!
//! let repo = GitRepository::open(".")?;
//! if let Some(commit) = repo.head_commit()? {
//!     println!("HEAD: {} {}", commit.id, commit.message);
//! }
//! # Ok::<(), phenotype_git_core::GitError>(())
//! ```

mod commit;
mod repository;

pub use commit::GitCommit;
pub use repository::GitRepository;

use gix::reference::Category;
use thiserror::Error;

/// Errors that can occur during git operations.
#[derive(Debug, Error)]
pub enum GitError {
    /// Git operation failed.
    #[error("git error: {context}")]
    Git { context: String },

    /// Repository not found at the given path.
    #[error("not a git repository: {0}")]
    NotARepo(String),
}

/// Convenient result type for git operations.
pub type Result<T> = std::result::Result<T, GitError>;

/// Summary of a git repository's current state.
#[derive(Debug, Clone)]
pub struct RepoInfo {
    /// Current branch name (None if detached HEAD).
    pub head_branch: Option<String>,
    /// Short commit ID of HEAD.
    pub head_commit: Option<String>,
    /// Whether the working directory has uncommitted changes.
    pub is_dirty: bool,
    /// URL of the 'origin' remote.
    pub remote_url: Option<String>,
}

/// Open a repository and return summary info.
pub fn repo_info(path: &std::path::Path) -> Result<RepoInfo> {
    let repo = gix::open(path).map_err(|e| GitError::NotARepo(e.to_string()))?;

    let head_branch = repo
        .head()
        .ok()
        .and_then(|h| h.name().to_string().strip_prefix("refs/heads/").map(String::from));

    let head_commit = repo
        .head()
        .ok()
        .and_then(|h| h.peel_to_commit_in_os().ok())
        .map(|c| c.id.to_string()[..8].to_string());

    let is_dirty = repo
        .status(gix::progress::Discard)
        .map(|s| !s.is_empty())
        .unwrap_or(false);

    let remote_url = repo
        .find_remote("origin")
        .ok()
        .and_then(|r| r.url().map(|u| u.to_string()));

    Ok(RepoInfo { head_branch, head_commit, is_dirty, remote_url })
}

/// List changed files (staged + unstaged).
pub fn changed_files(path: &std::path::Path) -> Result<Vec<String>> {
    let repo = gix::open(path).map_err(|e| GitError::NotARepo(e.to_string()))?;
    let statuses = repo
        .status(gix::progress::Discard)
        .map_err(|e| GitError::Git { context: e.to_string() })?;
    let files: Vec<String> = statuses
        .index()
        .files()
        .filter_map(|entry| entry.path().map(String::from))
        .collect();
    Ok(files)
}

/// Get the current branch name (or None if detached HEAD).
pub fn current_branch(path: &std::path::Path) -> Result<Option<String>> {
    let repo = gix::open(path).map_err(|e| GitError::NotARepo(e.to_string()))?;
    Ok(repo
        .head()
        .ok()
        .filter(|h| h.is_branch())
        .and_then(|h| h.name().to_string().strip_prefix("refs/heads/").map(String::from)))
}

/// Get the latest N commit messages from HEAD.
pub fn recent_commits(path: &std::path::Path, count: usize) -> Result<Vec<(String, String)>> {
    let repo = gix::open(path).map_err(|e| GitError::NotARepo(e.to_string()))?;

    let mut revwalk = repo
        .revwalk(Category::LocalBranches)
        .map_err(|e| GitError::Git { context: e.to_string() })?;
    revwalk.push_head().map_err(|e| GitError::Git { context: e.to_string() })?;

    let mut commits = Vec::new();
    for oid_result in revwalk.take(count) {
        let oid = oid_result.map_err(|e: gix::revision::walk::Error| GitError::Git { context: e.to_string() })?;
        let commit = repo
            .find_commit(oid)
            .map_err(|e| GitError::Git { context: e.to_string() })?;
        let short_id = oid.to_string()[..8].to_string();
        let message: String = commit
            .message()
            .to_str()
            .unwrap_or("")
            .lines()
            .next()
            .unwrap_or("")
            .to_string();
        commits.push((short_id, message));
    }
    Ok(commits)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn find_repo_root() -> std::path::PathBuf {
        let mut dir = std::path::PathBuf::from(
            std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default(),
        );
        loop {
            if dir.join(".git").exists() {
                return dir;
            }
            if !dir.pop() {
                panic!("could not find git repo root");
            }
        }
    }

    #[test]
    fn repo_info_on_this_repo() {
        let root = find_repo_root();
        let info = repo_info(&root).unwrap();
        assert!(info.head_branch.is_some());
        assert!(info.head_commit.is_some());
    }

    #[test]
    fn current_branch_exists() {
        let root = find_repo_root();
        assert!(current_branch(&root).is_ok());
    }

    #[test]
    fn recent_commits_returns_some() {
        let root = find_repo_root();
        let commits = recent_commits(&root, 3).unwrap();
        assert!(!commits.is_empty());
        assert!(commits.len() <= 3);
    }

    #[test]
    fn not_a_repo() {
        let result = repo_info(std::path::Path::new("/nonexistent-path"));
        assert!(result.is_err());
    }
}
