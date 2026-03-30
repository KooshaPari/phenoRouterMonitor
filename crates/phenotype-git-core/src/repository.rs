//! Git repository wrapper and operations.

use crate::commit::GitCommit;
use crate::{GitError, Result};
use git2::Repository as Git2Repository;
use std::path::Path;

/// Wrapper around libgit2 Repository.
///
/// Provides a simplified interface for common git operations.
pub struct GitRepository {
    inner: Git2Repository,
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
    ///
    /// # Errors
    ///
    /// Returns `GitError::NotARepo` if the path is not a git repository,
    /// or `GitError::Git` for other libgit2 errors.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let inner = Git2Repository::open(path)
            .map_err(|_e| GitError::NotARepo(path.display().to_string()))?;
        Ok(Self { inner })
    }

    /// Check if the repository is bare (no working directory).
    pub fn is_bare(&self) -> bool {
        self.inner.is_bare()
    }

    /// Get the HEAD commit if it exists.
    pub fn head_commit(&self) -> Result<Option<GitCommit>> {
        match self.inner.head() {
            Ok(head) => {
                match head.peel_to_commit() {
                    Ok(commit) => {
                        let id = commit.id().to_string()[..8].to_string();
                        let message = commit
                            .message()
                            .unwrap_or("")
                            .lines()
                            .next()
                            .unwrap_or("")
                            .to_string();
                        Ok(Some(GitCommit::new(id, message)))
                    }
                    Err(_) => Ok(None),
                }
            }
            Err(_) => Ok(None),
        }
    }
}
