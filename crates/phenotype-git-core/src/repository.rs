//! Git repository wrapper and operations.
//! Traces to: FR-PHENO-GIT-001

use crate::{git_ops::GitRepositoryOps, GitError, Result};

/// Wrapper around gix Repository.
pub struct GitRepository {
    inner: gix::Repository,
}

impl std::fmt::Debug for GitRepository {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GitRepository")
            .field("path", &self.inner.path())
            .field("bare", &self.inner.is_bare())
            .finish()
    }
}

impl GitRepository {
    /// Open a git repository at the given path.
    pub fn open<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        let inner = gix::open(path.as_ref())
            .map_err(|e| GitError::NotARepo(e.to_string()))?;
        Ok(Self { inner })
    }

    /// Check if the repository is bare.
    pub fn is_bare(&self) -> bool {
        self.inner.is_bare()
    }

    /// Get the HEAD commit if it exists.
    pub fn head_commit(&self) -> Result<Option<crate::GitCommit>> {
        match self.inner.head() {
            Ok(head) => match head.peel_to_commit_in_os() {
                Ok(commit) => {
                    let id = commit.id.to_string()[..8].to_string();
                    let message = commit
                        .message()
                        .ok()
                        .map(|m| m.lines().next().unwrap_or("").to_string())
                        .unwrap_or_default();
                    Ok(Some(crate::GitCommit::new(id, message)))
                }
                Err(_) => Ok(None),
            },
            Err(_) => Ok(None),
        }
    }
}

impl GitRepositoryOps for GitRepository {
    /// Get the commit hash of the current HEAD.
    /// Traces to: FR-PHENO-GIT-002
    fn commit_hash(&self) -> Result<String> {
        self.inner
            .head()
            .map_err(|e| GitError::Git { context: e.to_string() })?
            .peel_to_commit_in_os()
            .map(|commit| commit.id.to_string())
            .map_err(|e| GitError::Git { context: e.to_string() })
    }

    /// Get the current branch name.
    /// Traces to: FR-PHENO-GIT-003
    fn current_branch(&self) -> Result<Option<String>> {
        match self.inner.head() {
            Ok(head) => {
                if head.is_branch() {
                    Ok(head
                        .name()
                        .to_string()
                        .strip_prefix("refs/heads/")
                        .map(String::from))
                } else {
                    Ok(None)
                }
            }
            Err(_) => Ok(None),
        }
    }

    /// List all tags in the repository.
    /// Traces to: FR-PHENO-GIT-004
    fn list_tags(&self) -> Result<Vec<String>> {
        let mut tags = Vec::new();

        // Iterate through all references
        match self.inner.refs.all() {
            Ok(mut refs) => {
                while let Ok(Some(reference)) = refs.next() {
                    let name_str = reference.name().to_string();
                    if name_str.starts_with("refs/tags/") {
                        if let Some(tag_name) = name_str.strip_prefix("refs/tags/") {
                            tags.push(tag_name.to_string());
                        }
                    }
                }
                tags.sort();
                Ok(tags)
            }
            Err(e) => Err(GitError::Git { context: e.to_string() }),
        }
    }

    /// Get the count of commits from HEAD.
    /// Traces to: FR-PHENO-GIT-005
    fn commit_count(&self) -> Result<usize> {
        use gix::reference::Category;

        let mut revwalk = self
            .inner
            .revwalk(Category::LocalBranches)
            .map_err(|e| GitError::Git { context: e.to_string() })?;

        revwalk
            .push_head()
            .map_err(|e| GitError::Git { context: e.to_string() })?;

        let count = revwalk
            .by_ref()
            .take_while(|r| r.is_ok())
            .count();

        Ok(count)
    }

    /// Get the latest commit message.
    /// Traces to: FR-PHENO-GIT-006
    fn latest_message(&self) -> Result<Option<String>> {
        match self.inner.head() {
            Ok(head) => match head.peel_to_commit_in_os() {
                Ok(commit) => {
                    let msg = commit
                        .message()
                        .ok()
                        .map(|m| m.to_string());
                    Ok(msg)
                }
                Err(_) => Ok(None),
            },
            Err(_) => Ok(None),
        }
    }

    /// Check if working directory is clean.
    /// Traces to: FR-PHENO-GIT-007
    fn is_clean(&self) -> Result<bool> {
        match self.inner.status(gix::progress::Discard) {
            Ok(status) => Ok(status.is_empty()),
            Err(e) => Err(GitError::Git { context: e.to_string() }),
        }
    }

    /// Get absolute path to repository root.
    /// Traces to: FR-PHENO-GIT-008
    fn repo_path(&self) -> Result<std::path::PathBuf> {
        Ok(self.inner.path().to_path_buf())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn find_repo_root() -> std::path::PathBuf {
        let mut dir = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default());
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
    // Traces to: FR-PHENO-GIT-001
    fn test_git_repository_ops_implementation() {
        let root = find_repo_root();
        let repo = GitRepository::open(&root).expect("Failed to open repo");
        
        // Verify the repository opened successfully
        assert!(!repo.is_bare(), "Phenotype repo should not be bare");
    }

    #[test]
    // Traces to: FR-PHENO-GIT-002
    fn test_commit_hash_returns_full_sha() {
        let root = find_repo_root();
        let repo = GitRepository::open(&root).expect("Failed to open repo");
        let hash = repo.commit_hash().expect("Failed to get commit hash");
        
        // SHA-1 hashes are 40 hex characters
        assert_eq!(hash.len(), 40, "Commit hash should be 40 characters");
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()), "Hash should contain only hex digits");
    }

    #[test]
    // Traces to: FR-PHENO-GIT-003
    fn test_current_branch_returns_name() {
        let root = find_repo_root();
        let repo = GitRepository::open(&root).expect("Failed to open repo");
        let branch = repo.current_branch().expect("Failed to get current branch");
        
        // Should have a branch (not detached for this repo)
        assert!(branch.is_some(), "Repository should have a current branch");
    }

    #[test]
    // Traces to: FR-PHENO-GIT-004
    fn test_list_tags_returns_sorted() {
        let root = find_repo_root();
        let repo = GitRepository::open(&root).expect("Failed to open repo");
        let tags = repo.list_tags().expect("Failed to list tags");
        
        // Tags should be sorted
        let mut sorted_tags = tags.clone();
        sorted_tags.sort();
        assert_eq!(tags, sorted_tags, "Tags should be sorted alphabetically");
    }

    #[test]
    // Traces to: FR-PHENO-GIT-005
    fn test_commit_count_positive() {
        let root = find_repo_root();
        let repo = GitRepository::open(&root).expect("Failed to open repo");
        let count = repo.commit_count().expect("Failed to get commit count");
        
        // Repository should have at least one commit
        assert!(count > 0, "Repository should have at least one commit");
    }

    #[test]
    // Traces to: FR-PHENO-GIT-006
    fn test_latest_message_exists() {
        let root = find_repo_root();
        let repo = GitRepository::open(&root).expect("Failed to open repo");
        let msg = repo.latest_message().expect("Failed to get latest message");
        
        // Should have a commit message
        assert!(msg.is_some(), "HEAD should have a commit message");
        assert!(!msg.unwrap().is_empty(), "Commit message should not be empty");
    }

    #[test]
    // Traces to: FR-PHENO-GIT-007
    fn test_is_clean_works() {
        let root = find_repo_root();
        let repo = GitRepository::open(&root).expect("Failed to open repo");
        let clean = repo.is_clean().expect("Failed to check if clean");
        
        // Just verify it returns a boolean without error
        let _ = clean;
    }

    #[test]
    // Traces to: FR-PHENO-GIT-008
    fn test_repo_path_is_absolute() {
        let root = find_repo_root();
        let repo = GitRepository::open(&root).expect("Failed to open repo");
        let path = repo.repo_path().expect("Failed to get repo path");
        
        assert!(path.is_absolute(), "Repository path should be absolute");
        assert!(path.exists(), "Repository path should exist");
    }
}
