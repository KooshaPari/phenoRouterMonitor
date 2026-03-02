//! AgilePlus hidden sub-commands registry.
//!
//! Defines ~25 sub-commands invocable via Claude Code's SlashCommand tool,
//! organized into 7 categories. Each invocation is logged to an append-only
//! JSONL audit trail.
//!
//! Traceability: FR-048, FR-049 / WP20

pub mod audit;
pub mod registry;
pub mod sync;

pub use audit::AuditLog;
pub use registry::{SubCommand, SubCommandCategory, SubCommandRegistry};
pub use sync::{
    run_sync, SyncArgs, SyncSubcommand,
    SyncPushArgs, SyncPullArgs, SyncAutoArgs, SyncStatusArgs, SyncResolveArgs,
    SyncConfig, SyncReport, SyncReportEntry, SyncStatusRow, SyncConflict,
    SyncDirection, SyncItemOutcome, ConflictResolution, AutoSyncAction,
};
