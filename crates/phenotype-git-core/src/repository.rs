//! Git repository wrapper and operations.

use crate::{GitError, Result};

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
