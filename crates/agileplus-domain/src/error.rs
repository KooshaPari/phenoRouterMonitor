//! Domain error types.

#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("not implemented")]
    NotImplemented,
    #[error("invalid transition from {from} to {to}: {reason}")]
    InvalidTransition {
        from: String,
        to: String,
        reason: String,
    },
    #[error("no-op transition: already in state {0}")]
    NoOpTransition(String),
    #[error("entity not found: {0}")]
    NotFound(String),
    #[error("storage error: {0}")]
    Storage(String),
    #[error("vcs error: {0}")]
    Vcs(String),
    #[error("agent error: {0}")]
    Agent(String),
    #[error("review error: {0}")]
    Review(String),
    #[error("timeout after {0} seconds")]
    Timeout(u64),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("{0}")]
    Other(String),
}
