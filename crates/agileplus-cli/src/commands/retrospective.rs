//! `agileplus retrospective` command implementation.
//!
//! Analyzes the completed feature's development history and generates
//! a retrospective report with metrics and constitution amendment suggestions.
//! Transitions to Retrospected state.
//! Traceability: FR-007 / WP13-T076, T077

use std::path::PathBuf;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};

use agileplus_domain::domain::audit::{hash_entry, AuditEntry};
use agileplus_domain::domain::state_machine::FeatureState;
use agileplus_domain::ports::{StoragePort, VcsPort};

/// Arguments for the `retrospective` subcommand.
#[derive(Debug, clap::Args)]
pub struct RetrospectiveArgs {
    /// Feature slug to retrospect.
    #[arg(long)]
    pub feature: String,

    /// Output file path (default: kitty-specs/<feature>/retrospective.md).
    #[arg(long)]
    pub output: Option<PathBuf>,

    /// Include raw metric data in report.
    #[arg(long)]
    pub verbose: bool,
}

/// Per-WP performance data.
#[derive(Debug, Clone)]
struct WpMetrics {
    sequence: i32,
    title: String,
    agent_runs: i32,
    review_cycles: i32,
    duration_ms: i64,
}

/// Aggregated feature metrics for the retrospective.
#[derive(Debug)]
struct FeatureMetrics {
    total_duration_ms: i64,
    wp_count: usize,
    total_agent_runs: i32,
    total_review_cycles: i32,
    avg_review_cycles_per_wp: f64,
    state_transition_durations: Vec<(String, i64)>,
    governance_exceptions: Vec<String>,
    high_review_wps: Vec<(i32, String, i32)>,  // (sequence, title, review_cycles)
    wp_metrics: Vec<WpMetrics>,
}

/// Generate insights from metrics.
fn generate_insights(metrics: &FeatureMetrics) -> Vec<String> {
    let mut insights = Vec::new();

    if metrics.avg_review_cycles_per_wp > 3.0 {
        insights.push(format!(
            "High average review cycles ({:.1} per WP) suggest acceptance criteria may be unclear \
            or code areas are complex. Consider breaking future WPs into smaller units.",
            metrics.avg_review_cycles_per_wp
        ));
    }

    if !metrics.high_review_wps.is_empty() {
        let wp_list: Vec<String> = metrics.high_review_wps.iter()
            .map(|(seq, title, cycles)| format!("WP{:02} '{}' ({} cycles)", seq, title, cycles))
            .collect();
        insights.push(format!(
            "WPs with >3 review cycles (potential bottlenecks): {}",
            wp_list.join(", ")
        ));
    }

    if metrics.wp_count > 0 {
        let avg_agent_per_wp = metrics.total_agent_runs as f64 / metrics.wp_count as f64;
        if avg_agent_per_wp > 5.0 {
            insights.push(format!(
                "High agent invocation rate ({:.1} per WP) suggests agent failures or restarts. \
                Consider improving prompts or adding pre-flight checks.",
                avg_agent_per_wp
            ));
        }
    }

    if !metrics.governance_exceptions.is_empty() {
        insights.push(format!(
            "{} governance exception(s) occurred. Review whether contracts are too strict \
            or process is unclear.",
            metrics.governance_exceptions.len()
        ));
    }

    if metrics.total_duration_ms > 0 {
        let implement_fraction = metrics.wp_metrics.iter()
            .map(|w| w.duration_ms)
            .sum::<i64>() as f64 / metrics.total_duration_ms as f64;
        if implement_fraction > 0.5 {
            insights.push(format!(
                "{:.0}% of total time was spent in implementation/review phases. \
                Consider splitting WPs into smaller units to improve flow.",
                implement_fraction * 100.0
            ));
        }
    }

    if insights.is_empty() {
        insights.push("No significant issues detected. Development process is healthy.".to_string());
    }

    insights
}

/// Generate constitution amendment suggestions.
fn generate_constitution_suggestions(metrics: &FeatureMetrics) -> Vec<String> {
    let mut suggestions = Vec::new();

    if metrics.avg_review_cycles_per_wp > 3.0 {
        suggestions.push(
            "Consider adding a pre-review self-check rule to the governance constitution:\n\
            ```toml\n\
            [[rules]]\n\
            name = \"pre-review-self-check\"\n\
            description = \"Agent must verify acceptance criteria before requesting review\"\n\
            trigger = \"doing -> review\"\n\
            ```".to_string()
        );
    }

    if !metrics.governance_exceptions.is_empty() {
        suggestions.push(
            "Governance exceptions occurred. Consider adding a fast-track path:\n\
            ```toml\n\
            [[rules]]\n\
            name = \"fast-track\"\n\
            description = \"Allow expedited transitions with documented rationale\"\n\
            allow_skip = true\n\
            require_justification = true\n\
            ```".to_string()
        );
    }

    if suggestions.is_empty() {
        suggestions.push("No constitution amendments suggested. Current governance is appropriate.".to_string());
    }

    suggestions
}

/// Format duration in a human-readable way.
fn format_duration(ms: i64) -> String {
    if ms < 0 {
        return "N/A".to_string();
    }
    let secs = ms / 1000;
    let minutes = secs / 60;
    let hours = minutes / 60;
    let days = hours / 24;

    if days > 0 {
        format!("{}d {}h", days, hours % 24)
    } else if hours > 0 {
        format!("{}h {}m", hours, minutes % 60)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, secs % 60)
    } else {
        format!("{}s", secs)
    }
}

/// Generate the retrospective markdown report.
fn generate_retro_markdown(
    feature_slug: &str,
    feature_name: &str,
    metrics: &FeatureMetrics,
    verbose: bool,
) -> String {
    let insights = generate_insights(metrics);
    let suggestions = generate_constitution_suggestions(metrics);

    let mut lines = vec![
        format!("# Retrospective: {feature_name}"),
        format!("**Feature**: `{feature_slug}` | **Generated**: {}", Utc::now().format("%Y-%m-%d")),
        String::new(),
        "## Summary".to_string(),
        String::new(),
        format!("- **Total duration**: {}", format_duration(metrics.total_duration_ms)),
        format!("- **Work packages**: {}", metrics.wp_count),
        format!("- **Total agent invocations**: {}", metrics.total_agent_runs),
        format!("- **Total review cycles**: {} (avg {:.1} per WP)",
            metrics.total_review_cycles, metrics.avg_review_cycles_per_wp),
        format!("- **Governance exceptions**: {}", metrics.governance_exceptions.len()),
        String::new(),
    ];

    if !metrics.state_transition_durations.is_empty() {
        lines.push("## Phase Breakdown".to_string());
        lines.push(String::new());
        lines.push("| Phase | Duration |".to_string());
        lines.push("|-------|----------|".to_string());
        for (phase, dur_ms) in &metrics.state_transition_durations {
            lines.push(format!("| {} | {} |", phase, format_duration(*dur_ms)));
        }
        lines.push(String::new());
    }

    if !metrics.wp_metrics.is_empty() {
        lines.push("## WP Performance".to_string());
        lines.push(String::new());
        lines.push("| WP | Title | Agent Runs | Review Cycles | Duration |".to_string());
        lines.push("|----|-------|------------|---------------|----------|".to_string());
        for wp in &metrics.wp_metrics {
            lines.push(format!(
                "| WP{:02} | {} | {} | {} | {} |",
                wp.sequence,
                &wp.title[..wp.title.len().min(40)],
                wp.agent_runs,
                wp.review_cycles,
                format_duration(wp.duration_ms),
            ));
        }
        lines.push(String::new());
    }

    lines.push("## Insights".to_string());
    lines.push(String::new());
    for insight in &insights {
        lines.push(format!("- {insight}"));
    }
    lines.push(String::new());

    lines.push("## Suggested Constitution Amendments".to_string());
    lines.push(String::new());
    for suggestion in &suggestions {
        lines.push(suggestion.clone());
        lines.push(String::new());
    }

    if verbose && !metrics.governance_exceptions.is_empty() {
        lines.push("## Governance Exceptions".to_string());
        lines.push(String::new());
        for exc in &metrics.governance_exceptions {
            lines.push(format!("- {exc}"));
        }
        lines.push(String::new());
    }

    lines.join("\n")
}

/// Run the `retrospective` command.
pub async fn run_retrospective<S, V>(args: RetrospectiveArgs, storage: &S, vcs: &V) -> Result<()>
where
    S: StoragePort,
    V: VcsPort,
{
    let start = std::time::Instant::now();
    let slug = &args.feature;

    // Look up feature
    let feature = storage
        .get_feature_by_slug(slug)
        .await
        .context("looking up feature")?
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Feature '{}' not found. Run `agileplus specify --feature {}` first.",
                slug, slug
            )
        })?;

    // State enforcement
    if feature.state != FeatureState::Shipped {
        anyhow::bail!(
            "Feature '{}' is in state '{}'. Expected 'Shipped'. \
            Run `agileplus ship --feature {}` first.",
            slug, feature.state, slug
        );
    }

    // Load audit trail
    let audit_trail = storage
        .get_audit_trail(feature.id)
        .await
        .context("loading audit trail")?;

    // Load work packages
    let wps = storage
        .list_wps_by_feature(feature.id)
        .await
        .context("loading work packages")?;

    // Load metrics
    let metrics_data = storage
        .get_metrics_by_feature(feature.id)
        .await
        .context("loading metrics")?;

    // Compute total duration from audit trail
    let (total_duration_ms, state_transition_durations) =
        compute_durations_from_audit(&audit_trail, &feature.created_at);

    // Extract governance exceptions from audit trail
    let governance_exceptions: Vec<String> = audit_trail.iter()
        .filter(|e| e.transition.contains("skipped") || e.transition.contains("exception"))
        .map(|e| format!("{}: {}", e.timestamp.format("%Y-%m-%d"), e.transition))
        .collect();

    // Compute per-WP metrics from stored metrics
    let wp_metrics: Vec<WpMetrics> = wps.iter().map(|wp| {
        // Find metrics for this WP
        let wp_metric = metrics_data.iter()
            .find(|m| m.feature_id == Some(feature.id) && m.command.contains(&format!("WP{:02}", wp.sequence)));

        WpMetrics {
            sequence: wp.sequence,
            title: wp.title.clone(),
            agent_runs: wp_metric.map(|m| m.agent_runs).unwrap_or(0),
            review_cycles: wp_metric.map(|m| m.review_cycles).unwrap_or(0),
            duration_ms: wp_metric.map(|m| m.duration_ms).unwrap_or(0),
        }
    }).collect();

    let total_agent_runs: i32 = wp_metrics.iter().map(|w| w.agent_runs).sum::<i32>()
        + metrics_data.iter().map(|m| m.agent_runs).sum::<i32>();

    let total_review_cycles: i32 = wp_metrics.iter().map(|w| w.review_cycles).sum::<i32>()
        + metrics_data.iter().map(|m| m.review_cycles).sum::<i32>();

    let avg_review_cycles = if !wps.is_empty() {
        total_review_cycles as f64 / wps.len() as f64
    } else {
        0.0
    };

    let high_review_wps: Vec<(i32, String, i32)> = wp_metrics.iter()
        .filter(|w| w.review_cycles > 3)
        .map(|w| (w.sequence, w.title.clone(), w.review_cycles))
        .collect();

    let feature_metrics = FeatureMetrics {
        total_duration_ms,
        wp_count: wps.len(),
        total_agent_runs,
        total_review_cycles,
        avg_review_cycles_per_wp: avg_review_cycles,
        state_transition_durations,
        governance_exceptions,
        high_review_wps,
        wp_metrics,
    };

    // Generate report
    let report_content = generate_retro_markdown(
        slug,
        &feature.friendly_name,
        &feature_metrics,
        args.verbose,
    );

    // Write to output path
    let output_path = args.output.clone().unwrap_or_else(|| {
        PathBuf::from(format!("kitty-specs/{slug}/retrospective.md"))
    });

    // Write to file if we have a path that's not the VCS path
    if args.output.is_some() {
        std::fs::write(&output_path, &report_content)
            .with_context(|| format!("writing retro to {}", output_path.display()))?;
        println!("Retrospective written to: {}", output_path.display());
    }

    // Write as VCS artifact
    vcs.write_artifact(slug, "retrospective.md", &report_content)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!("Failed to write retrospective.md artifact: {e}");
        });

    // Transition feature state to Retrospected
    storage
        .update_feature_state(feature.id, FeatureState::Retrospected)
        .await
        .context("transitioning feature to Retrospected")?;

    // Append audit entry
    let prev_hash = get_latest_hash(storage, feature.id).await;
    let mut audit = AuditEntry {
        id: 0,
        feature_id: feature.id,
        wp_id: None,
        timestamp: Utc::now(),
        actor: "user".into(),
        transition: "Shipped -> Retrospected".into(),
        evidence_refs: vec![],
        prev_hash,
        hash: [0u8; 32],
    };
    audit.hash = hash_entry(&audit);
    storage
        .append_audit_entry(&audit)
        .await
        .context("appending audit entry")?;

    let elapsed_ms = start.elapsed().as_millis();
    tracing::info!(command = "retrospective", slug = %slug, audit_entries = audit_trail.len(), elapsed_ms = %elapsed_ms, "retrospective completed");

    println!("Retrospective for '{}' completed.", slug);
    println!("  WPs analyzed: {}", wps.len());
    println!("  Audit entries: {}", audit_trail.len());
    println!("  Report: kitty-specs/{slug}/retrospective.md");
    println!("  State: Shipped -> Retrospected");

    Ok(())
}

/// Compute duration metrics from the audit trail.
fn compute_durations_from_audit(
    trail: &[AuditEntry],
    created_at: &DateTime<Utc>,
) -> (i64, Vec<(String, i64)>) {
    if trail.is_empty() {
        return (0, vec![]);
    }

    // Total: from feature creation to last audit entry
    let last_ts = trail.last().map(|e| e.timestamp).unwrap_or_else(Utc::now);
    let total_ms = (last_ts - *created_at).num_milliseconds().max(0);

    // State transition durations
    let mut phase_durations = Vec::new();
    for window in trail.windows(2) {
        let from = &window[0];
        let to = &window[1];
        let dur_ms = (to.timestamp - from.timestamp).num_milliseconds().max(0);
        phase_durations.push((from.transition.clone(), dur_ms));
    }

    (total_ms, phase_durations)
}

async fn get_latest_hash<S: StoragePort>(storage: &S, feature_id: i64) -> [u8; 32] {
    match storage.get_latest_audit_entry(feature_id).await {
        Ok(Some(entry)) => entry.hash,
        _ => [0u8; 32],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agileplus_domain::domain::audit::AuditEntry;
    use chrono::{Duration, Utc};

    fn make_audit(transition: &str, ts: DateTime<Utc>) -> AuditEntry {
        let mut e = AuditEntry {
            id: 0,
            feature_id: 1,
            wp_id: None,
            timestamp: ts,
            actor: "user".into(),
            transition: transition.into(),
            evidence_refs: vec![],
            prev_hash: [0u8; 32],
            hash: [0u8; 32],
        };
        e.hash = hash_entry(&e);
        e
    }

    #[test]
    fn format_duration_secs() {
        assert_eq!(format_duration(5000), "5s");
    }

    #[test]
    fn format_duration_minutes() {
        assert_eq!(format_duration(65000), "1m 5s");
    }

    #[test]
    fn format_duration_hours() {
        assert_eq!(format_duration(3660000), "1h 1m");
    }

    #[test]
    fn format_duration_days() {
        let ms = (2 * 24 * 3600 + 3 * 3600) * 1000i64;
        let result = format_duration(ms);
        assert!(result.starts_with("2d"));
    }

    #[test]
    fn format_duration_negative() {
        assert_eq!(format_duration(-100), "N/A");
    }

    #[test]
    fn compute_durations_empty_trail() {
        let now = Utc::now();
        let (total, phases) = compute_durations_from_audit(&[], &now);
        assert_eq!(total, 0);
        assert!(phases.is_empty());
    }

    #[test]
    fn compute_durations_single_entry() {
        let created = Utc::now() - Duration::hours(2);
        let e = make_audit("Created -> Specified", Utc::now());
        let (total, phases) = compute_durations_from_audit(&[e], &created);
        assert!(total > 0);
        assert!(phases.is_empty()); // only 1 entry -> no windows
    }

    #[test]
    fn compute_durations_two_entries() {
        let base = Utc::now();
        let t0 = base - Duration::hours(3);
        let t1 = base - Duration::hours(1);
        let e0 = make_audit("Created -> Specified", t0);
        let e1 = make_audit("Specified -> Researched", t1);
        let (total, phases) = compute_durations_from_audit(&[e0, e1], &t0);
        assert!(total > 0);
        assert_eq!(phases.len(), 1);
        assert!(phases[0].1 > 0);
    }

    #[test]
    fn generate_insights_healthy() {
        let metrics = FeatureMetrics {
            total_duration_ms: 3600000,
            wp_count: 5,
            total_agent_runs: 10,
            total_review_cycles: 5,
            avg_review_cycles_per_wp: 1.0,
            state_transition_durations: vec![],
            governance_exceptions: vec![],
            high_review_wps: vec![],
            wp_metrics: vec![],
        };
        let insights = generate_insights(&metrics);
        assert!(!insights.is_empty());
        // Should report healthy process
        assert!(insights[0].contains("healthy") || insights[0].contains("No significant"));
    }

    #[test]
    fn generate_insights_high_review_cycles() {
        let metrics = FeatureMetrics {
            total_duration_ms: 3600000,
            wp_count: 3,
            total_agent_runs: 20,
            total_review_cycles: 15,
            avg_review_cycles_per_wp: 5.0,
            state_transition_durations: vec![],
            governance_exceptions: vec![],
            high_review_wps: vec![(1, "Auth Module".to_string(), 5)],
            wp_metrics: vec![],
        };
        let insights = generate_insights(&metrics);
        let combined = insights.join(" ");
        assert!(combined.contains("review cycles") || combined.contains("bottleneck"));
    }

    #[test]
    fn generate_retro_markdown_contains_summary() {
        let metrics = FeatureMetrics {
            total_duration_ms: 86400000,
            wp_count: 3,
            total_agent_runs: 6,
            total_review_cycles: 4,
            avg_review_cycles_per_wp: 1.3,
            state_transition_durations: vec![("Created -> Specified".to_string(), 3600000)],
            governance_exceptions: vec![],
            high_review_wps: vec![],
            wp_metrics: vec![
                WpMetrics {
                    sequence: 1,
                    title: "Auth Module".to_string(),
                    agent_runs: 2,
                    review_cycles: 2,
                    duration_ms: 28800000,
                },
            ],
        };
        let report = generate_retro_markdown("my-feat", "My Feature", &metrics, false);
        assert!(report.contains("# Retrospective: My Feature"));
        assert!(report.contains("## Summary"));
        assert!(report.contains("## WP Performance"));
        assert!(report.contains("## Insights"));
        assert!(report.contains("## Suggested Constitution Amendments"));
    }
}
