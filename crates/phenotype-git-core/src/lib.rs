//! # Phenotype Git Core
//!
//! Git operations via libgit2: repository info, branch, status, log.
//!
//! Provides high-level abstractions over libgit2 for working with git repositories:
//! - Repository information and state introspection
//! - Commit history navigation
//! - Branch and status queries
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

use git2::Repository;
use thiserror::Error;

/// Errors that can occur during git operations.
#[derive(Debug, Error)]
pub enum GitError {
    /// Wrapper around libgit2 errors.
    #[error("git error: {0}")]
    Git(#[from] git2::Error),

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
    let repo = Repository::open(path)?;

    let head_branch = repo
        .head()
        .ok()
        .and_then(|h| h.shorthand().map(String::from));

    let head_commit = repo
        .head()
        .ok()
        .and_then(|h| h.peel_to_commit().ok())
        .map(|c| c.id().to_string()[..8].to_string());

    let is_dirty = !repo.statuses(None)?.is_empty();

    let remote_url = repo
        .find_remote("origin")
        .ok()
        .and_then(|r| r.url().map(String::from));

    Ok(RepoInfo {
        head_branch,
        head_commit,
        is_dirty,
        remote_url,
    })
}

/// List changed files (staged + unstaged).
pub fn changed_files(path: &std::path::Path) -> Result<Vec<String>> {
    let repo = Repository::open(path)?;
    let statuses = repo.statuses(None)?;
    let files: Vec<String> = statuses
        .iter()
        .filter_map(|entry| entry.path().map(String::from))
        .collect();
    Ok(files)
}

/// Get the current branch name (or None if detached HEAD).
pub fn current_branch(path: &std::path::Path) -> Result<Option<String>> {
    let repo = Repository::open(path)?;
    Ok(repo
        .head()
        .ok()
        .filter(|h| h.is_branch())
        .and_then(|h| h.shorthand().map(String::from)))
}

/// Get the latest N commit messages from HEAD.
pub fn recent_commits(path: &std::path::Path, count: usize) -> Result<Vec<(String, String)>> {
    let repo = Repository::open(path)?;
    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;
    revwalk.set_sorting(git2::Sort::TIME)?;

    let mut commits = Vec::new();
    for oid in revwalk.take(count) {
        let oid = oid?;
        let commit = repo.find_commit(oid)?;
        let short_id = oid.to_string()[..8].to_string();
        let message = commit
            .message()
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
    use std::path::Path;

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
        let result = repo_info(Path::new("/nonexistent-path"));
        assert!(result.is_err());
    }
}
