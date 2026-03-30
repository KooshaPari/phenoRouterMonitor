//! # Phenotype Git Core
//!
//! Git operations via gitoxide (gix): repository info, branch, status, log.
//!
//! Migrated from libgit2 to gitoxide for better safety and performance.
//! - No C dependencies
//! - Pure Rust implementation
//! - Better error handling
//!
//! See: <https://github.com/GitoxideLabs/gitoxide>

mod commit;
mod repository;

pub use commit::GitCommit;
pub use repository::GitRepository;

use gix::reference::Category;
use gix::Repository;
use thiserror::Error;

/// Errors that can occur during git operations.
#[derive(Debug, Error)]
pub enum GitError {
    /// Git operation failed.
    #[error("git error: {0}")]
    Git(String),

    /// Repository not found at the given path.
    #[error("not a git repository: {0}")]
    NotARepo(String),
}

/// Convenient result type for git operations.
pub type Result<T> = std::result::Result<T, GitError>;

/// Summary of a git repository's current state.
#[derive(Debug, Clone)]
pub struct RepoInfo {
    pub head_branch: Option<String>,
    pub head_commit: Option<String>,
    pub is_dirty: bool,
    pub remote_url: Option<String>,
}

/// Open a repository and return summary info.
pub fn repo_info(path: &std::path::Path) -> Result<RepoInfo> {
    let repo = gix::open(path).map_err(|e| GitError::NotARepo(e.to_string()))?;

    let head_branch = repo.head().ok().and_then(|h| h.name().map(String::from));

    let head_commit = repo
        .head()
        .ok()
        .and_then(|h| h.peel_to_commit().ok())
        .map(|c| c.id.to_string()[..8].to_string());

    let is_dirty = repo.status().map(|s| !s.is_clean()).unwrap_or(false);

    let remote_url = repo
        .find_remote("origin")
        .ok()
        .and_then(|r| r.url().map(|u| u.to_string()));

    Ok(RepoInfo {
        head_branch,
        head_commit,
        is_dirty,
        remote_url,
    })
}

/// List changed files (staged + unstaged).
pub fn changed_files(path: &std::path::Path) -> Result<Vec<String>> {
    let repo = gix::open(path).map_err(|e| GitError::NotARepo(e.to_string()))?;
    let statuses = repo.status().map_err(|e| GitError::Git(e.to_string()))?;
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
        .and_then(|h| h.name().map(String::from)))
}

/// Get the latest N commit messages from HEAD.
pub fn recent_commits(path: &std::path::Path, count: usize) -> Result<Vec<(String, String)>> {
    let repo = gix::open(path).map_err(|e| GitError::NotARepo(e.to_string()))?;

    let mut revwalk = repo
        .revwalk(Category::LocalBranches)
        .map_err(|e| GitError::Git(e.to_string()))?;
    revwalk.push_head().map_err(|e| GitError::Git(e.to_string()))?;

    let mut commits = Vec::new();
    for oid in revwalk.take(count) {
        let oid = oid?;
        let commit = repo.find_commit(oid).map_err(|e| GitError::Git(e.to_string()))?;
        let short_id = oid.to_string()[..8].to_string();
        let message = commit.message().lines().next().unwrap_or("").to_string();
        commits.push((short_id, message));
    }
    Ok(commits)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Find the repo root by searching upward from CARGO_MANIFEST_DIR.
    fn find_repo_root() -> std::path::PathBuf {
        let mut dir =
            std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default());
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
        let branch = current_branch(&root);
        assert!(branch.is_ok());
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
