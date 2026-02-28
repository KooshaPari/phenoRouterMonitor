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
}
