use super::metrics::{
    FeatureMetrics, format_duration, generate_constitution_suggestions, generate_insights,
};

pub(crate) fn generate_retro_markdown(
    feature_slug: &str,
    feature_name: &str,
    metrics: &FeatureMetrics,
    verbose: bool,
) -> String {
    let insights = generate_insights(metrics);
    let suggestions = generate_constitution_suggestions(metrics);

    let mut lines = vec![
        format!("# Retrospective: {feature_name}"),
        format!(
            "**Feature**: `{feature_slug}` | **Generated**: {}",
            chrono::Utc::now().format("%Y-%m-%d")
        ),
        String::new(),
        "## Summary".to_string(),
        String::new(),
        format!(
            "- **Total duration**: {}",
            format_duration(metrics.total_duration_ms)
        ),
        format!("- **Work packages**: {}", metrics.wp_count),
        format!(
            "- **Total agent invocations**: {}",
            metrics.total_agent_runs
        ),
        format!(
            "- **Total review cycles**: {} (avg {:.1} per WP)",
            metrics.total_review_cycles, metrics.avg_review_cycles_per_wp
        ),
        format!(
            "- **Governance exceptions**: {}",
            metrics.governance_exceptions.len()
        ),
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
