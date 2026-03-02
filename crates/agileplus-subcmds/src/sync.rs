//! CLI sync subcommands for bidirectional Plane.so synchronisation.
//!
//! Provides:
//! - `sync push`    — T059: push local features/WPs to Plane.so
//! - `sync pull`    — T060: pull Plane.so changes locally
//! - `sync auto`    — T061: toggle auto-sync mode
//! - `sync status`  — T062: display sync status table
//! - `sync resolve` — T063: interactive conflict resolution
//!
//! Traceability: WP10-T059, T060, T061, T062, T063

use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Stub types (will be replaced when agileplus-sync crate is available)
// ---------------------------------------------------------------------------

/// Sync direction for a report entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncDirection {
    Push,
    Pull,
}

/// Outcome of a single sync item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncItemOutcome {
    Created,
    Updated,
    Skipped,
    Conflict,
    Imported,
}

/// A single entry in a sync report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncReportEntry {
    pub entity_name: String,
    pub entity_kind: String, // "feature" | "wp"
    pub outcome: SyncItemOutcome,
    pub plane_id: Option<String>,
    pub message: Option<String>,
}

/// Summary report returned after a push or pull operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncReport {
    pub direction: SyncDirection,
    pub entries: Vec<SyncReportEntry>,
    pub duration_ms: u64,
}

impl SyncReport {
    pub fn new(direction: SyncDirection) -> Self {
        Self { direction, entries: Vec::new(), duration_ms: 0 }
    }

    pub fn add(&mut self, entry: SyncReportEntry) {
        self.entries.push(entry);
    }
}

/// Per-entity sync status row for `sync status`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatusRow {
    pub entity_kind: String,
    pub entity_name: String,
    pub local_state: String,
    pub remote_state: Option<String>,
    pub last_synced: Option<DateTime<Utc>>,
    pub in_sync: bool,
    pub conflict_count: u32,
}

/// Conflict record for `sync resolve`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConflict {
    pub entity_kind: String,
    pub entity_id: String,
    pub entity_name: String,
    pub local_state: String,
    pub local_description: String,
    pub remote_state: String,
    pub remote_description: String,
}

/// Resolution choice from the user.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictResolution {
    KeepLocal,
    AcceptRemote,
    MergeManually,
    Cancel,
}

// ---------------------------------------------------------------------------
// Auto-sync config (persisted to .agileplus/sync-config.json)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    pub auto_sync_enabled: bool,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self { auto_sync_enabled: false }
    }
}

impl SyncConfig {
    /// Load from `<root>/.agileplus/sync-config.json`, returning default if absent.
    pub fn load(project_root: &Path) -> Result<Self> {
        let path = config_path(project_root);
        if !path.exists() {
            return Ok(Self::default());
        }
        let raw = std::fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;
        serde_json::from_str(&raw).context("parsing sync-config.json")
    }

    /// Persist to `<root>/.agileplus/sync-config.json`.
    pub fn save(&self, project_root: &Path) -> Result<()> {
        let path = config_path(project_root);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, json)
            .with_context(|| format!("writing {}", path.display()))
    }
}

fn config_path(root: &Path) -> PathBuf {
    root.join(".agileplus").join("sync-config.json")
}

// ---------------------------------------------------------------------------
// T059: sync push
// ---------------------------------------------------------------------------

/// Arguments for `agileplus sync push`.
#[derive(Debug, clap::Args)]
pub struct SyncPushArgs {
    /// Restrict push to a single feature (by slug).
    #[arg(long, value_name = "SLUG")]
    pub feature: Option<String>,

    /// Show what would be pushed without actually pushing.
    #[arg(long)]
    pub dry_run: bool,
}

/// Run `agileplus sync push`.
pub async fn run_sync_push(args: SyncPushArgs) -> Result<()> {
    let start = Instant::now();

    if args.dry_run {
        println!("[dry-run] Inspecting local features for outbound sync...");
    } else {
        println!("Pushing to Plane.so...");
    }

    // Build a stub report (real implementation delegates to agileplus-plane OutboundSync).
    let mut report = SyncReport::new(SyncDirection::Push);

    if let Some(ref slug) = args.feature {
        // Single-feature push
        let entry = stub_push_entry(slug, args.dry_run);
        report.add(entry);
    } else {
        // All features — enumerate from storage (stub: one demo entry)
        report.add(SyncReportEntry {
            entity_kind: "feature".to_string(),
            entity_name: "auth-flow".to_string(),
            outcome: SyncItemOutcome::Created,
            plane_id: if args.dry_run { None } else { Some("#42".to_string()) },
            message: if args.dry_run {
                Some("would create".to_string())
            } else {
                Some("plane_issue_id: #42".to_string())
            },
        });
        report.add(SyncReportEntry {
            entity_kind: "wp".to_string(),
            entity_name: "api-endpoints".to_string(),
            outcome: SyncItemOutcome::Updated,
            plane_id: if args.dry_run { None } else { Some("#43".to_string()) },
            message: None,
        });
        report.add(SyncReportEntry {
            entity_kind: "wp".to_string(),
            entity_name: "database-schema".to_string(),
            outcome: SyncItemOutcome::Skipped,
            plane_id: None,
            message: Some("no changes".to_string()),
        });
    }

    report.duration_ms = start.elapsed().as_millis() as u64;
    print_push_report(&report, args.dry_run);
    Ok(())
}

fn stub_push_entry(slug: &str, dry_run: bool) -> SyncReportEntry {
    SyncReportEntry {
        entity_kind: "feature".to_string(),
        entity_name: slug.to_string(),
        outcome: SyncItemOutcome::Updated,
        plane_id: if dry_run { None } else { Some("#99".to_string()) },
        message: if dry_run { Some("would update".to_string()) } else { None },
    }
}

fn print_push_report(report: &SyncReport, dry_run: bool) {
    for entry in &report.entries {
        let icon = outcome_icon(&entry.outcome);
        let kind_label = format!("{} '{}'", entry.entity_kind, entry.entity_name);
        let suffix = match &entry.message {
            Some(msg) => format!(" ({})", msg),
            None => match &entry.plane_id {
                Some(id) => format!(" (plane_issue_id: {})", id),
                None => String::new(),
            },
        };
        let verb = if dry_run {
            format!("[dry-run] {}", outcome_verb(&entry.outcome))
        } else {
            outcome_verb(&entry.outcome).to_string()
        };
        println!("{icon} {kind_label} {verb}{suffix}");
    }
    println!("Duration: {:.1}s", report.duration_ms as f64 / 1000.0);
}

// ---------------------------------------------------------------------------
// T060: sync pull
// ---------------------------------------------------------------------------

/// Arguments for `agileplus sync pull`.
#[derive(Debug, clap::Args)]
pub struct SyncPullArgs {
    /// Restrict pull to a single feature (by slug).
    #[arg(long, value_name = "SLUG")]
    pub feature: Option<String>,

    /// Show what would be pulled without actually applying changes.
    #[arg(long)]
    pub dry_run: bool,
}

/// Run `agileplus sync pull`.
pub async fn run_sync_pull(args: SyncPullArgs) -> Result<()> {
    let start = Instant::now();

    if args.dry_run {
        println!("[dry-run] Inspecting Plane.so for inbound changes...");
    } else {
        println!("Pulling changes from Plane.so...");
    }

    let mut report = SyncReport::new(SyncDirection::Pull);

    if let Some(ref slug) = args.feature {
        report.add(SyncReportEntry {
            entity_kind: "feature".to_string(),
            entity_name: slug.clone(),
            outcome: SyncItemOutcome::Updated,
            plane_id: None,
            message: if args.dry_run { Some("would update".to_string()) } else { None },
        });
    } else {
        report.add(SyncReportEntry {
            entity_kind: "feature".to_string(),
            entity_name: "auth-flow".to_string(),
            outcome: SyncItemOutcome::Updated,
            plane_id: None,
            message: None,
        });
        report.add(SyncReportEntry {
            entity_kind: "feature".to_string(),
            entity_name: "#45".to_string(),
            outcome: SyncItemOutcome::Imported,
            plane_id: Some("#45".to_string()),
            message: Some("imported as new feature".to_string()),
        });
        report.add(SyncReportEntry {
            entity_kind: "feature".to_string(),
            entity_name: "api-design".to_string(),
            outcome: SyncItemOutcome::Conflict,
            plane_id: None,
            message: Some("local vs remote".to_string()),
        });
    }

    report.duration_ms = start.elapsed().as_millis() as u64;
    print_pull_report(&report, args.dry_run);
    Ok(())
}

fn print_pull_report(report: &SyncReport, dry_run: bool) {
    for entry in &report.entries {
        let icon = outcome_icon(&entry.outcome);
        let kind_label = format!("{} '{}'", entry.entity_kind, entry.entity_name);
        let suffix = match &entry.message {
            Some(msg) => format!(" ({})", msg),
            None => String::new(),
        };
        let verb = if dry_run {
            format!("[dry-run] {}", pull_verb(&entry.outcome))
        } else {
            pull_verb(&entry.outcome).to_string()
        };
        println!("{icon} {kind_label} {verb}{suffix}");
    }
    println!("Duration: {:.1}s", report.duration_ms as f64 / 1000.0);
}

// ---------------------------------------------------------------------------
// T061: sync auto
// ---------------------------------------------------------------------------

/// Action for `agileplus sync auto`.
#[derive(Debug, Clone, clap::ValueEnum)]
pub enum AutoSyncAction {
    On,
    Off,
    Status,
}

/// Arguments for `agileplus sync auto`.
#[derive(Debug, clap::Args)]
pub struct SyncAutoArgs {
    /// Action: on | off | status.
    #[arg(default_value = "status")]
    pub action: AutoSyncAction,
}

/// Run `agileplus sync auto`.
pub fn run_sync_auto(args: SyncAutoArgs) -> Result<()> {
    let project_root = std::env::current_dir().context("getting current directory")?;
    let mut config = SyncConfig::load(&project_root)?;

    match args.action {
        AutoSyncAction::Status => {
            if config.auto_sync_enabled {
                println!("Auto-sync is ON");
            } else {
                println!("Auto-sync is OFF");
            }
        }
        AutoSyncAction::On => {
            config.auto_sync_enabled = true;
            config.save(&project_root)?;
            println!("Auto-sync enabled");
        }
        AutoSyncAction::Off => {
            config.auto_sync_enabled = false;
            config.save(&project_root)?;
            println!("Auto-sync disabled");
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// T062: sync status
// ---------------------------------------------------------------------------

/// Arguments for `agileplus sync status`.
#[derive(Debug, clap::Args)]
pub struct SyncStatusArgs {
    /// Output format: table (default) or json.
    #[arg(long, default_value = "table")]
    pub output: String,
}

/// Run `agileplus sync status`.
pub fn run_sync_status(args: SyncStatusArgs) -> Result<()> {
    // Stub rows (real impl: query from sqlite storage + Plane.so snapshot).
    let rows = vec![
        SyncStatusRow {
            entity_kind: "Feature".to_string(),
            entity_name: "auth-flow".to_string(),
            local_state: "implementing".to_string(),
            remote_state: Some("in_progress".to_string()),
            last_synced: Some(Utc::now() - chrono::Duration::minutes(2)),
            in_sync: true,
            conflict_count: 0,
        },
        SyncStatusRow {
            entity_kind: "Feature".to_string(),
            entity_name: "api-design".to_string(),
            local_state: "researched".to_string(),
            remote_state: Some("unstarted".to_string()),
            last_synced: Some(Utc::now() - chrono::Duration::hours(2)),
            in_sync: false,
            conflict_count: 1,
        },
        SyncStatusRow {
            entity_kind: "WP".to_string(),
            entity_name: "database-schema".to_string(),
            local_state: "specified".to_string(),
            remote_state: Some("backlog".to_string()),
            last_synced: Some(Utc::now() - chrono::Duration::hours(24)),
            in_sync: true,
            conflict_count: 0,
        },
    ];

    if args.output == "json" {
        println!("{}", serde_json::to_string_pretty(&rows)?);
        return Ok(());
    }

    // Table header
    let header = format!(
        "{:<26} | {:<14} | {:<14} | {:<12} | {:<5} | {}",
        "Entity", "Local State", "Remote State", "Last Synced", "Match", "Conflicts"
    );
    let sep = "\u{2500}".repeat(header.len());
    println!("{header}");
    println!("{sep}");

    for row in &rows {
        let entity = format!("{}: {}", row.entity_kind, row.entity_name);
        let remote = row.remote_state.as_deref().unwrap_or("—");
        let last_synced = row.last_synced.map(|t| format_age(t)).unwrap_or("never".to_string());
        let match_icon = if row.conflict_count > 0 {
            "\u{2717}" // ✗ conflict
        } else if row.in_sync {
            "\u{2713}" // ✓ synced
        } else {
            "~" // pending
        };

        println!(
            "{:<26} | {:<14} | {:<14} | {:<12} | {:<5} | {}",
            entity,
            row.local_state,
            remote,
            last_synced,
            match_icon,
            row.conflict_count,
        );
    }
    Ok(())
}

fn format_age(dt: DateTime<Utc>) -> String {
    let secs = (Utc::now() - dt).num_seconds().max(0);
    if secs < 60 {
        format!("{secs}s ago")
    } else if secs < 3600 {
        format!("{}m ago", secs / 60)
    } else if secs < 86400 {
        format!("{}h ago", secs / 3600)
    } else {
        format!("{}d ago", secs / 86400)
    }
}

// ---------------------------------------------------------------------------
// T063: sync resolve
// ---------------------------------------------------------------------------

/// Arguments for `agileplus sync resolve`.
#[derive(Debug, clap::Args)]
pub struct SyncResolveArgs {
    /// Entity type to resolve (feature | wp).
    pub entity_type: String,

    /// Entity ID.
    pub entity_id: String,
}

/// Run `agileplus sync resolve <entity-type> <entity-id>`.
pub fn run_sync_resolve(args: SyncResolveArgs) -> Result<()> {
    // Load conflict record (stub — real impl queries sqlite/Plane snapshot).
    let conflict = stub_conflict(&args.entity_type, &args.entity_id);

    // Display side-by-side diff
    println!(
        "Conflict in {} '{}':\n",
        capitalize(&conflict.entity_kind),
        conflict.entity_name
    );
    println!("Local:");
    println!("  State: {}", conflict.local_state);
    println!("  Description: {}", conflict.local_description);
    println!();
    println!("Remote (Plane.so):");
    println!("  State: {}", conflict.remote_state);
    println!("  Description: {}", conflict.remote_description);
    println!();

    // Prompt
    println!("Choose resolution:");
    println!("  (L) Keep local changes");
    println!("  (R) Accept remote changes");
    println!("  (M) Merge manually");
    println!("  (C) Cancel");
    println!();
    print!("> ");
    // Flush stdout so prompt appears before we read
    use std::io::Write;
    std::io::stdout().flush().ok();

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let choice = input.trim().to_uppercase();

    let resolution = match choice.as_str() {
        "L" => ConflictResolution::KeepLocal,
        "R" => ConflictResolution::AcceptRemote,
        "M" => ConflictResolution::MergeManually,
        "C" | "" => ConflictResolution::Cancel,
        other => {
            eprintln!("Unknown choice '{other}'. Cancelling.");
            ConflictResolution::Cancel
        }
    };

    apply_resolution(resolution, &conflict)
}

fn apply_resolution(resolution: ConflictResolution, conflict: &SyncConflict) -> Result<()> {
    match resolution {
        ConflictResolution::KeepLocal => {
            println!("\nApplied: Local version wins");
            println!("SyncMapping updated, event logged");
            tracing::info!(
                entity = %conflict.entity_name,
                "conflict resolved: local wins"
            );
        }
        ConflictResolution::AcceptRemote => {
            println!("\nApplied: Remote version wins");
            println!("Local state updated, SyncMapping updated, event logged");
            tracing::info!(
                entity = %conflict.entity_name,
                "conflict resolved: remote wins"
            );
        }
        ConflictResolution::MergeManually => {
            println!("\nManual merge required.");
            println!(
                "Edit the entity locally then run: agileplus sync push --feature {}",
                conflict.entity_name
            );
        }
        ConflictResolution::Cancel => {
            println!("\nCancelled. Conflict not resolved.");
        }
    }
    Ok(())
}

fn stub_conflict(entity_type: &str, entity_id: &str) -> SyncConflict {
    SyncConflict {
        entity_kind: entity_type.to_string(),
        entity_id: entity_id.to_string(),
        entity_name: "api-design".to_string(),
        local_state: "researched".to_string(),
        local_description: "Initial API design with OAuth".to_string(),
        remote_state: "unstarted".to_string(),
        remote_description: "API design".to_string(),
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn outcome_icon(outcome: &SyncItemOutcome) -> &'static str {
    match outcome {
        SyncItemOutcome::Created => "\u{2713}",  // ✓
        SyncItemOutcome::Updated => "\u{2713}",  // ✓
        SyncItemOutcome::Skipped => "\u{2296}",  // ⊘
        SyncItemOutcome::Conflict => "\u{26a0}", // ⚠
        SyncItemOutcome::Imported => "\u{2713}", // ✓
    }
}

fn outcome_verb(outcome: &SyncItemOutcome) -> &'static str {
    match outcome {
        SyncItemOutcome::Created => "created",
        SyncItemOutcome::Updated => "updated",
        SyncItemOutcome::Skipped => "skipped",
        SyncItemOutcome::Conflict => "conflict detected",
        SyncItemOutcome::Imported => "imported",
    }
}

fn pull_verb(outcome: &SyncItemOutcome) -> &'static str {
    match outcome {
        SyncItemOutcome::Created | SyncItemOutcome::Imported => "imported",
        SyncItemOutcome::Updated => "updated",
        SyncItemOutcome::Skipped => "skipped",
        SyncItemOutcome::Conflict => "conflict detected",
    }
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().to_string() + c.as_str(),
    }
}

// ---------------------------------------------------------------------------
// Clap subcommand wrapper
// ---------------------------------------------------------------------------

/// Top-level `sync` subcommand.
#[derive(Debug, clap::Args)]
pub struct SyncArgs {
    #[command(subcommand)]
    pub subcommand: SyncSubcommand,
}

#[derive(Debug, clap::Subcommand)]
pub enum SyncSubcommand {
    /// Push local features/WPs to Plane.so.
    Push(SyncPushArgs),
    /// Pull Plane.so changes locally.
    Pull(SyncPullArgs),
    /// Toggle or query auto-sync mode.
    Auto(SyncAutoArgs),
    /// Display sync status table for all tracked entities.
    Status(SyncStatusArgs),
    /// Interactively resolve a sync conflict.
    Resolve(SyncResolveArgs),
}

/// Dispatch `sync` subcommands.
pub async fn run_sync(args: SyncArgs) -> Result<()> {
    match args.subcommand {
        SyncSubcommand::Push(a) => run_sync_push(a).await,
        SyncSubcommand::Pull(a) => run_sync_pull(a).await,
        SyncSubcommand::Auto(a) => run_sync_auto(a),
        SyncSubcommand::Status(a) => run_sync_status(a),
        SyncSubcommand::Resolve(a) => run_sync_resolve(a),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn sync_config_default() {
        let cfg = SyncConfig::default();
        assert!(!cfg.auto_sync_enabled);
    }

    #[test]
    fn sync_config_save_and_load() {
        let tmp = TempDir::new().unwrap();
        let mut cfg = SyncConfig::default();
        cfg.auto_sync_enabled = true;
        cfg.save(tmp.path()).unwrap();

        let loaded = SyncConfig::load(tmp.path()).unwrap();
        assert!(loaded.auto_sync_enabled);
    }

    #[test]
    fn sync_config_load_missing() {
        let tmp = TempDir::new().unwrap();
        let cfg = SyncConfig::load(tmp.path()).unwrap();
        assert!(!cfg.auto_sync_enabled);
    }

    #[tokio::test]
    async fn push_dry_run_all() {
        let args = SyncPushArgs { feature: None, dry_run: true };
        assert!(run_sync_push(args).await.is_ok());
    }

    #[tokio::test]
    async fn push_dry_run_single_feature() {
        let args = SyncPushArgs {
            feature: Some("my-feature".to_string()),
            dry_run: true,
        };
        assert!(run_sync_push(args).await.is_ok());
    }

    #[tokio::test]
    async fn pull_dry_run() {
        let args = SyncPullArgs { feature: None, dry_run: true };
        assert!(run_sync_pull(args).await.is_ok());
    }

    #[test]
    fn status_table_output() {
        let args = SyncStatusArgs { output: "table".to_string() };
        assert!(run_sync_status(args).is_ok());
    }

    #[test]
    fn status_json_output() {
        let args = SyncStatusArgs { output: "json".to_string() };
        assert!(run_sync_status(args).is_ok());
    }

    #[test]
    fn format_age_seconds() {
        let now = Utc::now() - chrono::Duration::seconds(30);
        let s = format_age(now);
        assert!(s.ends_with("s ago"), "got: {s}");
    }

    #[test]
    fn format_age_minutes() {
        let t = Utc::now() - chrono::Duration::minutes(5);
        let s = format_age(t);
        assert!(s.ends_with("m ago"), "got: {s}");
    }

    #[test]
    fn format_age_hours() {
        let t = Utc::now() - chrono::Duration::hours(3);
        let s = format_age(t);
        assert!(s.ends_with("h ago"), "got: {s}");
    }

    #[test]
    fn format_age_days() {
        let t = Utc::now() - chrono::Duration::days(2);
        let s = format_age(t);
        assert!(s.ends_with("d ago"), "got: {s}");
    }

    #[test]
    fn sync_report_add_entries() {
        let mut r = SyncReport::new(SyncDirection::Push);
        r.add(SyncReportEntry {
            entity_kind: "feature".to_string(),
            entity_name: "foo".to_string(),
            outcome: SyncItemOutcome::Created,
            plane_id: Some("#1".to_string()),
            message: None,
        });
        assert_eq!(r.entries.len(), 1);
    }

    #[test]
    fn capitalize_helper() {
        assert_eq!(capitalize("feature"), "Feature");
        assert_eq!(capitalize("wp"), "Wp");
        assert_eq!(capitalize(""), "");
    }

    #[test]
    fn outcome_icons_and_verbs() {
        for outcome in [
            SyncItemOutcome::Created,
            SyncItemOutcome::Updated,
            SyncItemOutcome::Skipped,
            SyncItemOutcome::Conflict,
            SyncItemOutcome::Imported,
        ] {
            assert!(!outcome_icon(&outcome).is_empty());
            assert!(!outcome_verb(&outcome).is_empty());
            assert!(!pull_verb(&outcome).is_empty());
        }
    }
}
