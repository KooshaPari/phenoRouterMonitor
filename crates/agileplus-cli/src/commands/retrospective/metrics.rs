use chrono::{DateTime, Utc};

use agileplus_domain::domain::audit::AuditEntry;
use agileplus_domain::domain::metric::Metric;
use agileplus_domain::domain::work_package::WorkPackage;

/// Per-WP performance data.
#[derive(Debug, Clone)]
pub(crate) struct WpMetrics {
    pub(crate) sequence: i32,
    pub(crate) title: String,
    pub(crate) agent_runs: i32,
    pub(crate) review_cycles: i32,
    pub(crate) duration_ms: i64,
}

/// Aggregated feature metrics for the retrospective.
#[derive(Debug)]
pub(crate) struct FeatureMetrics {
    pub(crate) total_duration_ms: i64,
    pub(crate) wp_count: usize,
    pub(crate) total_agent_runs: i32,
    pub(crate) total_review_cycles: i32,
    pub(crate) avg_review_cycles_per_wp: f64,
    pub(crate) state_transition_durations: Vec<(String, i64)>,
    pub(crate) governance_exceptions: Vec<String>,
    pub(crate) high_review_wps: Vec<(i32, String, i32)>,
    pub(crate) wp_metrics: Vec<WpMetrics>,
}

pub(crate) fn collect_feature_metrics(
    feature_id: i64,
    created_at: &DateTime<Utc>,
    wps: &[WorkPackage],
    audit_trail: &[AuditEntry],
    metrics_data: &[Metric],
) -> FeatureMetrics {
    let (total_duration_ms, state_transition_durations) =
        compute_durations_from_audit(audit_trail, created_at);

    let governance_exceptions: Vec<String> = audit_trail
        .iter()
        .filter(|e| e.transition.contains("skipped") || e.transition.contains("exception"))
        .map(|e| format!("{}: {}", e.timestamp.format("%Y-%m-%d"), e.transition))
        .collect();

    let wp_metrics: Vec<WpMetrics> = wps
        .iter()
        .map(|wp| {
            let wp_metric = metrics_data.iter().find(|m| {
                m.feature_id == Some(feature_id)
                    && m.command.contains(&format!("WP{:02}", wp.sequence))
            });

            WpMetrics {
                sequence: wp.sequence,
                title: wp.title.clone(),
                agent_runs: wp_metric.map(|m| m.agent_runs).unwrap_or(0),
                review_cycles: wp_metric.map(|m| m.review_cycles).unwrap_or(0),
                duration_ms: wp_metric.map(|m| m.duration_ms).unwrap_or(0),
            }
        })
        .collect();

    let total_agent_runs: i32 = wp_metrics.iter().map(|w| w.agent_runs).sum::<i32>()
        + metrics_data.iter().map(|m| m.agent_runs).sum::<i32>();

    let total_review_cycles: i32 = wp_metrics.iter().map(|w| w.review_cycles).sum::<i32>()
        + metrics_data.iter().map(|m| m.review_cycles).sum::<i32>();

    let avg_review_cycles = if !wps.is_empty() {
        total_review_cycles as f64 / wps.len() as f64
    } else {
        0.0
    };

    let high_review_wps: Vec<(i32, String, i32)> = wp_metrics
        .iter()
        .filter(|w| w.review_cycles > 3)
        .map(|w| (w.sequence, w.title.clone(), w.review_cycles))
        .collect();

    FeatureMetrics {
        total_duration_ms,
        wp_count: wps.len(),
        total_agent_runs,
        total_review_cycles,
        avg_review_cycles_per_wp: avg_review_cycles,
        state_transition_durations,
        governance_exceptions,
        high_review_wps,
        wp_metrics,
    }
}

pub(crate) fn generate_insights(metrics: &FeatureMetrics) -> Vec<String> {
    let mut insights = Vec::new();

    if metrics.avg_review_cycles_per_wp > 3.0 {
        insights.push(format!(
            "High average review cycles ({:.1} per WP) suggest acceptance criteria may be unclear \
            or code areas are complex. Consider breaking future WPs into smaller units.",
            metrics.avg_review_cycles_per_wp
        ));
    }

    if !metrics.high_review_wps.is_empty() {
        let wp_list: Vec<String> = metrics
            .high_review_wps
            .iter()
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
        let implement_fraction = metrics
            .wp_metrics
            .iter()
            .map(|w| w.duration_ms)
            .sum::<i64>() as f64
            / metrics.total_duration_ms as f64;
        if implement_fraction > 0.5 {
            insights.push(format!(
                "{:.0}% of total time was spent in implementation/review phases. \
                Consider splitting WPs into smaller units to improve flow.",
                implement_fraction * 100.0
            ));
        }
    }

    if insights.is_empty() {
        insights
            .push("No significant issues detected. Development process is healthy.".to_string());
    }

    insights
}

pub(crate) fn generate_constitution_suggestions(metrics: &FeatureMetrics) -> Vec<String> {
    let mut suggestions = Vec::new();

    if metrics.avg_review_cycles_per_wp > 3.0 {
        suggestions.push(
            "Consider adding a pre-review self-check rule to the governance constitution:\n\
            ```toml\n\
            [[rules]]\n\
            name = \"pre-review-self-check\"\n\
            description = \"Agent must verify acceptance criteria before requesting review\"\n\
            trigger = \"doing -> review\"\n\
            ```"
            .to_string(),
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
            ```"
            .to_string(),
        );
    }

    if suggestions.is_empty() {
        suggestions.push(
            "No constitution amendments suggested. Current governance is appropriate.".to_string(),
        );
    }

    suggestions
}

pub(crate) fn format_duration(ms: i64) -> String {
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

pub(crate) fn compute_durations_from_audit(
    trail: &[AuditEntry],
    created_at: &DateTime<Utc>,
) -> (i64, Vec<(String, i64)>) {
    if trail.is_empty() {
        return (0, vec![]);
    }

    let last_ts = trail.last().map(|e| e.timestamp).unwrap_or_else(Utc::now);
    let total_ms = (last_ts - *created_at).num_milliseconds().max(0);

    let mut phase_durations = Vec::new();
    for window in trail.windows(2) {
        let from = &window[0];
        let to = &window[1];
        let dur_ms = (to.timestamp - from.timestamp).num_milliseconds().max(0);
        phase_durations.push((from.transition.clone(), dur_ms));
    }

    (total_ms, phase_durations)
}
