//! Port trait re-exports and application context.
//!
//! Traceability: WP05-T030

pub mod agent;
pub mod observability;
pub mod review;
pub mod storage;
pub mod vcs;

// -- Trait re-exports --
pub use agent::AgentPort;
pub use observability::ObservabilityPort;
pub use review::ReviewPort;
pub use storage::StoragePort;
pub use vcs::VcsPort;

// -- Supporting type re-exports --
pub use agent::{AgentConfig, AgentKind, AgentResult, AgentStatus, AgentTask};
pub use observability::{LogEntry, LogLevel, MetricValue, SpanContext};
pub use review::{CiStatus, CommentSeverity, PrInfo, ReviewComment, ReviewStatus};
pub use vcs::{ConflictInfo, FeatureArtifacts, MergeResult, WorktreeInfo};

/// Application context bundling all ports for dependency injection.
///
/// Uses generics rather than trait objects because native async traits
/// in Rust 2024 edition are not dyn-compatible.
pub struct AppContext<S, V, A, R, T>
where
    S: StoragePort,
    V: VcsPort,
    A: AgentPort,
    R: ReviewPort,
    T: ObservabilityPort,
{
    pub storage: S,
    pub vcs: V,
    pub agent: A,
    pub review: R,
    pub telemetry: T,
}
