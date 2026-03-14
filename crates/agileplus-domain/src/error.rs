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
    // --- Module errors ---
    #[error("module not found: {0}")]
    ModuleNotFound(String),
    #[error("circular module reference: cannot set {child} as parent of {ancestor}")]
    CircularModuleRef { child: String, ancestor: String },
    #[error("module has dependents: {0}")]
    ModuleHasDependents(String),
    #[error(
        "feature not in module scope: feature {feature_slug} is not owned by or tagged to module {module_slug}"
    )]
    FeatureNotInModuleScope {
        feature_slug: String,
        module_slug: String,
    },
    // --- Cycle errors ---
    #[error("cycle not found: {0}")]
    CycleNotFound(String),
    #[error("cycle gate not met: {0}")]
    CycleGateNotMet(String),
}
