//! `agileplus research` command implementation.
//!
//! Supports pre-specify (codebase scan) and post-specify (feasibility analysis) modes.
//! Traceability: FR-002 / WP11-T062

use anyhow::{Context, Result};
use chrono::Utc;
use clap::ValueEnum;

use agileplus_domain::domain::audit::{hash_entry, AuditEntry};
use agileplus_domain::domain::state_machine::FeatureState;
use agileplus_domain::ports::{StoragePort, VcsPort};

use super::governance::{enforce_governance, load_constitution, validate_spec_consistency};

/// Research mode override values.
#[derive(Debug, Clone, ValueEnum)]
pub enum ResearchMode {
    /// Scan the codebase (pre-specify).
    Scan,
    /// Feasibility analysis against existing spec (post-specify).
    Feasibility,
}

/// Arguments for the `research` subcommand.
#[derive(Debug, clap::Args)]
pub struct ResearchArgs {
    /// Feature slug to research.
    #[arg(long)]
    pub feature: String,

    /// Research mode override (auto-detected if omitted).
    #[arg(long, value_enum)]
    pub mode: Option<ResearchMode>,
}

/// Run the `research` command.
pub async fn run_research<S, V>(args: ResearchArgs, storage: &S, vcs: &V) -> Result<()>
where
    S: StoragePort,
    V: VcsPort,
{
    let start = std::time::Instant::now();
    let slug = &args.feature;

    // Look up feature
    let existing = storage
        .get_feature_by_slug(slug)
        .await
        .context("looking up feature")?;

    // Determine mode
    let mode = match &args.mode {
        Some(m) => m.clone(),
        None => {
            if let Some(ref f) = existing {
                if f.state == FeatureState::Specified
                    || f.state.ordinal() > FeatureState::Specified.ordinal()
                {
                    ResearchMode::Feasibility
                } else {
                    ResearchMode::Scan
                }
            } else {
                ResearchMode::Scan
            }
        }
    };

    match mode {
        ResearchMode::Scan => {
            research_pre_specify(slug, vcs).await?;
        }
        ResearchMode::Feasibility => {
            let feature = existing.ok_or_else(|| {
                anyhow::anyhow!(
                    "Feature '{}' not found. Run `agileplus specify --feature {}` first.",
                    slug,
                    slug
                )
            })?;

            if feature.state.ordinal() < FeatureState::Specified.ordinal() {
                anyhow::bail!(
                    "Feature '{}' is in state '{}'. It must be in 'Specified' state for feasibility research.",
                    slug,
                    feature.state
                );
            }

            // Governance checks before transition
            let constitution = load_constitution(vcs).await;
            if let Some(ref c) = constitution {
                if let Ok(spec) = vcs.read_artifact(slug, "spec.md").await {
                    let violations = validate_spec_consistency(&spec, c);
                    enforce_governance(&violations)?;
                }
            }

            research_post_specify(feature, slug, storage, vcs).await?;
        }
    }

    let elapsed_ms = start.elapsed().as_millis();
    tracing::info!(command = "research", slug = %slug, elapsed_ms = %elapsed_ms, "research completed");
    Ok(())
}

/// Pre-specify research: scan the repository for context.
async fn research_pre_specify<V: VcsPort>(slug: &str, vcs: &V) -> Result<()> {
    let date = Utc::now().format("%Y-%m-%d").to_string();

    // Scan top-level directory via VCS artifacts
    let dir_structure = scan_directory_structure(vcs).await;
    let technologies = detect_technologies(vcs).await;
    let existing_specs = list_existing_specs(vcs).await;

    let content = format!(
        r"# Research: {slug} (Pre-Specify)
**Date**: {date} | **Mode**: codebase-scan

## Repository Overview
{dir_structure}

## Detected Technologies
{technologies}

## Existing Specifications
{existing_specs}

## Recommended Investigation Areas
- Review existing specifications for patterns and conventions
- Check for related features that may share components
- Identify integration points with current codebase
",
        slug = slug,
        date = date,
        dir_structure = dir_structure,
        technologies = technologies,
        existing_specs = existing_specs,
    );

    vcs.write_artifact(slug, "research.md", &content)
        .await
        .context("writing research.md")?;

    println!("Pre-specify research complete for '{slug}'.");
    println!("  Research saved to: kitty-specs/{slug}/research.md");
    Ok(())
}

/// Post-specify research: feasibility analysis against the spec.
async fn research_post_specify<S: StoragePort, V: VcsPort>(
    feature: agileplus_domain::domain::feature::Feature,
    slug: &str,
    storage: &S,
    vcs: &V,
) -> Result<()> {
    let date = Utc::now().format("%Y-%m-%d").to_string();

    // Read existing spec
    let spec_content = vcs
        .read_artifact(slug, "spec.md")
        .await
        .unwrap_or_else(|_| "(spec not found)".to_string());

    let fr_count = count_pattern(&spec_content, "**FR-");
    let nfr_count = count_pattern(&spec_content, "**NFR-");

    let existing_code_analysis = analyze_existing_code(vcs, slug).await;

    let content = format!(
        r"# Research: {slug} (Post-Specify)
**Date**: {date} | **Mode**: feasibility

## Spec Summary
{fr_count} functional requirements, {nfr_count} non-functional

## Existing Code Analysis
{existing_code_analysis}

## Feasibility Assessment
- Scope estimate: {scope_estimate} based on {fr_count} FRs
- Risk areas: Review the existing code analysis above for potential conflicts

## Recommended Approach
- Create work packages for each functional requirement group
- Identify shared components that can be reused
- Plan for incremental delivery of functional requirements
",
        slug = slug,
        date = date,
        fr_count = fr_count,
        nfr_count = nfr_count,
        existing_code_analysis = existing_code_analysis,
        scope_estimate = estimate_scope(fr_count),
    );

    vcs.write_artifact(slug, "research.md", &content)
        .await
        .context("writing research.md")?;

    // Transition state: Specified -> Researched
    storage
        .update_feature_state(feature.id, FeatureState::Researched)
        .await
        .context("transitioning feature to Researched")?;

    // Append audit entry
    let prev_hash = get_latest_hash(storage, feature.id).await;
    let mut audit = AuditEntry {
        id: 0,
        feature_id: feature.id,
        wp_id: None,
        timestamp: Utc::now(),
        actor: "user".into(),
        transition: "Specified -> Researched".into(),
        evidence_refs: vec![],
        prev_hash,
        hash: [0u8; 32],
    };
    audit.hash = hash_entry(&audit);
    storage
        .append_audit_entry(&audit)
        .await
        .context("appending audit entry")?;

    println!("Post-specify research complete for '{slug}'.");
    println!("  Research saved to: kitty-specs/{slug}/research.md");
    println!("  State: Specified -> Researched");
    Ok(())
}

/// Scan the top-level directory structure for overview.
async fn scan_directory_structure<V: VcsPort>(vcs: &V) -> String {
    // We use artifact_exists to probe for common top-level items
    let common_dirs = [
        "src", "crates", "packages", "lib", "tests", "docs", "scripts", "kitty-specs",
    ];
    let mut found = Vec::new();
    for dir in &common_dirs {
        // We check for a README or common file inside
        if vcs.artifact_exists("", &format!("../../{dir}", dir = dir)).await.unwrap_or(false) {
            found.push(format!("- `{dir}/`"));
        }
    }
    if found.is_empty() {
        "(directory structure not available via VCS scan)".to_string()
    } else {
        found.join("\n")
    }
}

/// Detect technologies based on common config files.
async fn detect_technologies<V: VcsPort>(_vcs: &V) -> String {
    // In a real implementation we'd probe for Cargo.toml, package.json, etc.
    // For now, return a static placeholder since we'd need filesystem access.
    "(technology detection requires filesystem access — run from project root)".to_string()
}

/// List existing spec files.
async fn list_existing_specs<V: VcsPort>(vcs: &V) -> String {
    match vcs.scan_feature_artifacts("").await {
        Ok(artifacts) => {
            if let Some(meta) = artifacts.meta_json {
                format!("- meta.json found: {}", &meta[..meta.len().min(100)])
            } else {
                "(no existing specs found)".to_string()
            }
        }
        Err(_) => "(could not scan for existing specs)".to_string(),
    }
}

/// Analyze existing code for potential conflicts.
async fn analyze_existing_code<V: VcsPort>(_vcs: &V, _slug: &str) -> String {
    "(static analysis not yet implemented — manual review recommended)".to_string()
}

/// Count occurrences of a pattern in content.
fn count_pattern(content: &str, pattern: &str) -> usize {
    let mut count = 0;
    let mut start = 0;
    while let Some(pos) = content[start..].find(pattern) {
        count += 1;
        start += pos + pattern.len();
    }
    count
}

/// Estimate scope label based on FR count.
fn estimate_scope(fr_count: usize) -> &'static str {
    match fr_count {
        0..=2 => "small",
        3..=5 => "medium",
        6..=10 => "large",
        _ => "extra-large",
    }
}

/// Get the prev_hash for audit chain (last entry's hash, or zeroes if none).
async fn get_latest_hash<S: StoragePort>(storage: &S, feature_id: i64) -> [u8; 32] {
    match storage.get_latest_audit_entry(feature_id).await {
        Ok(Some(entry)) => entry.hash,
        _ => [0u8; 32],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn count_pattern_basic() {
        let text = "- **FR-1**: foo\n- **FR-2**: bar\n- **NFR-1**: baz\n";
        assert_eq!(count_pattern(text, "**FR-"), 2);
        assert_eq!(count_pattern(text, "**NFR-"), 1);
    }

    #[test]
    fn estimate_scope_thresholds() {
        assert_eq!(estimate_scope(0), "small");
        assert_eq!(estimate_scope(2), "small");
        assert_eq!(estimate_scope(3), "medium");
        assert_eq!(estimate_scope(6), "large");
        assert_eq!(estimate_scope(11), "extra-large");
    }
}
