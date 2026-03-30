//! Git commit representation.

/// Represents a single git commit.
#[derive(Debug, Clone)]
pub struct GitCommit {
    /// Short commit ID (8 characters).
    pub id: String,
    /// First line of the commit message.
    pub message: String,
}

impl GitCommit {
    /// Create a new GitCommit.
    pub fn new(id: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            message: message.into(),
        }
    }
}
