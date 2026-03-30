use chrono::Utc;

use agileplus_domain::domain::work_package::{WorkPackage, WpDependency};

use super::super::scheduler::ExecutionWave;

/// Slugify a string for use in file paths.
pub(crate) fn slugify(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join("-")
        .chars()
        .take(40)
        .collect()
}

/// Generate plan.md content.
pub(crate) fn generate_plan_md(
    slug: &str,
    wps: &[WorkPackage],
    deps: &[WpDependency],
    waves: &[ExecutionWave],
) -> String {
    let date = Utc::now().format("%Y-%m-%d").to_string();
    let mut lines = vec![
        format!("# Plan: {slug}"),
        format!("**Date**: {date} | **WPs**: {}", wps.len()),
        String::new(),
        "## Work Packages".to_string(),
        String::new(),
    ];

    for wp in wps {
        let dep_ids: Vec<String> = deps
            .iter()
            .filter(|d| d.wp_id == wp.id)
            .map(|d| d.depends_on.to_string())
            .collect();
        let dep_str = if dep_ids.is_empty() {
            "none".to_string()
        } else {
            dep_ids.join(", ")
        };
        lines.push(format!("### WP{:02}: {}", wp.sequence, wp.title));
        lines.push(format!("**ID**: {} | **Dependencies**: {}", wp.id, dep_str));
        lines.push(String::new());
        lines.push("**Acceptance Criteria:**".to_string());
        for crit in wp.acceptance_criteria.lines() {
            lines.push(format!("  {crit}"));
        }
        if !wp.file_scope.is_empty() {
            lines.push(String::new());
            lines.push("**File Scope:**".to_string());
            for file in &wp.file_scope {
                lines.push(format!("  - `{file}`"));
            }
        }
        lines.push(String::new());
    }

    lines.push("## Execution Waves".to_string());
    lines.push(String::new());
    for wave in waves {
        let ids: Vec<String> = wave.wp_ids.iter().map(|id| id.to_string()).collect();
        lines.push(format!(
            "- **Wave {}** (parallel): WPs [{}]",
            wave.wave_number,
            ids.join(", ")
        ));
    }
    lines.push(String::new());

    lines.join("\n")
}

/// Generate a WP prompt file.
pub(crate) fn generate_wp_prompt(wp: &WorkPackage, feature_name: &str, slug: &str) -> String {
    let date = Utc::now().format("%Y-%m-%d").to_string();
    let file_scope_str = if wp.file_scope.is_empty() {
        "(auto-detect from spec)".to_string()
    } else {
        wp.file_scope
            .iter()
            .map(|file| format!("- `{file}`"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    let mut lines = Vec::new();
    lines.push("---".to_string());
    lines.push(format!("work_package_id: WP{:02}", wp.sequence));
    lines.push(format!("title: {}", wp.title));
    lines.push(format!("feature: {feature_name}"));
    lines.push(format!("feature_slug: {slug}"));
    lines.push(format!("sequence: {}", wp.sequence));
    lines.push("state: planned".to_string());
    lines.push(format!("created_at: {date}T00:00:00Z"));
    lines.push("---".to_string());
    lines.push(String::new());
    lines.push(format!("# Work Package: {}", wp.title));
    lines.push(String::new());
    lines.push("## Feature".to_string());
    lines.push(format!("{feature_name} (`{slug}`)"));
    lines.push(String::new());
    lines.push("## Acceptance Criteria".to_string());
    lines.push(wp.acceptance_criteria.clone());
    lines.push(String::new());
    lines.push("## File Scope".to_string());
    lines.push(file_scope_str);
    lines.push(String::new());
    lines.push("## Instructions".to_string());
    lines.push(
        "Implement this work package according to the acceptance criteria above.".to_string(),
    );
    lines.push(format!(
        "Refer to `agileplus/{slug}/spec.md` for the full specification and"
    ));
    lines.push(format!(
        "`agileplus/{slug}/plan.md` for the implementation plan."
    ));
    lines.join("\n")
}
