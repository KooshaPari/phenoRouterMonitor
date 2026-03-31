//! # Phenotype Git Core
//!
//! Git operations via gitoxide (gix): repository info, branch, status, log.

use std::path::Path;
use thiserror::Error;

/// Errors that can occur during git operations.
#[derive(Debug, Error)]
pub enum GitError {
    #[error("git error: {context}")]
    Git { context: String },

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
pub fn repo_info(path: &Path) -> Result<RepoInfo> {
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

/// Get the current branch name (or None if detached HEAD).
pub fn current_branch(path: &Path) -> Result<Option<String>> {
    let repo = gix::open(path).map_err(|e| GitError::NotARepo(e.to_string()))?;
    Ok(repo
        .head()
        .ok()
        .filter(|h| h.is_branch())
        .and_then(|h| h.name().to_string().strip_prefix("refs/heads/").map(String::from)))
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
    }

    #[test]
    fn current_branch_exists() {
        let root = find_repo_root();
        assert!(current_branch(&root).is_ok());
    }
}
