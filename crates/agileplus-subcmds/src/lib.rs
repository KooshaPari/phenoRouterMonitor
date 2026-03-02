//! AgilePlus hidden sub-commands registry.
//!
//! Defines ~25 sub-commands invocable via Claude Code's SlashCommand tool,
//! organized into 7 categories. Each invocation is logged to an append-only
//! JSONL audit trail.
//!
//! Traceability: FR-048, FR-049 / WP20

pub mod audit;
pub mod dashboard;
pub mod events;
pub mod platform;
pub mod registry;
pub mod sync;

pub use audit::AuditLog;
pub use dashboard::{
    run_dashboard, run_dashboard_open, run_dashboard_port,
    DashboardArgs, DashboardSubcommand, DashboardOpenArgs, DashboardPortArgs,
    dashboard_url, configured_port, api_reachable,
};
pub use events::{
    run_events, filter_events, parse_since, render_table, render_json, render_jsonl,
    EventsArgs, EventRecord, EventQueryResult, EventOutputFormat,
};
pub use platform::{
    run_platform, run_platform_up, run_platform_down, run_platform_status, run_platform_logs,
    PlatformArgs, PlatformSubcommand,
    PlatformUpArgs, PlatformDownArgs, PlatformStatusArgs, PlatformLogsArgs,
    ServiceHealth, ServiceStatus, PlatformHealth, OverallStatus,
};
pub use registry::{SubCommand, SubCommandCategory, SubCommandRegistry};
pub use sync::{
    run_sync, SyncArgs, SyncSubcommand,
    SyncPushArgs, SyncPullArgs, SyncAutoArgs, SyncStatusArgs, SyncResolveArgs,
    SyncConfig, SyncReport, SyncReportEntry, SyncStatusRow, SyncConflict,
    SyncDirection, SyncItemOutcome, ConflictResolution, AutoSyncAction,
};
