//! phenotype-git-core
//!
//! # Deprecated
//! This crate is deprecated. Use `phenotype-shared-config` for git operations.
//! See ADR-015.

/// A git repository handle.
#[deprecated(note = "Use phenotype-shared-config for git operations. See ADR-015.")]
#[derive(Debug, Clone)]
pub struct GitRepository {
    path: std::path::PathBuf,
}

#[allow(deprecated)]
impl GitRepository {
    /// Open a git repository at the given path.
    pub fn open(path: &std::path::Path) -> Result<Self, String> {
        Ok(Self {
            path: path.to_path_buf(),
        })
    }

    /// Returns the path of this repository.
    pub fn path(&self) -> &std::path::Path {
        &self.path
    }

    /// Returns whether this repository is bare.
    pub fn is_bare(&self) -> bool {
        false
    }

    /// Returns the head commit, if any.
    pub fn head_commit(&self) -> Result<Option<GitCommit>, String> {
        Ok(None)
    }
}

/// A git commit reference.
#[deprecated(note = "Use phenotype-shared-config for git operations. See ADR-015.")]
#[derive(Debug, Clone)]
pub struct GitCommit {
    /// The abbreviated commit ID.
    pub id: String,
    /// The commit message.
    pub message: String,
}

#[allow(deprecated)]
impl GitCommit {
    /// Create a new `GitCommit` with the given id and message.
    pub fn new(id: &str, message: &str) -> Self {
        Self {
            id: id.to_string(),
            message: message.to_string(),
        }
    }
}
