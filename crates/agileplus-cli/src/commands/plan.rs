//! `agileplus plan` command implementation.
//!
//! Reads spec.md and research.md for a feature, generates work packages with
//! dependency ordering, creates a governance contract, writes plan.md, and
//! transitions the feature state to `planned`.
//! Traceability: FR-003, FR-038, FR-039 / WP12-T066, T067, T068

use anyhow::{Context, Result};
use chrono::Utc;

use agileplus_domain::domain::audit::{hash_entry, AuditEntry};
use agileplus_domain::domain::governance::{
    EvidenceRequirement, EvidenceType, GovernanceContract, GovernanceRule,
};
use agileplus_domain::domain::state_machine::FeatureState;
use agileplus_domain::domain::work_package::{DependencyType, WorkPackage, WpDependency};
use agileplus_domain::ports::{StoragePort, VcsPort};

use super::scope::{build_overlap_graph, detect_file_scope};
use super::scheduler::Scheduler;

/// Arguments for the `plan` subcommand.
#[derive(Debug, clap::Args)]
pub struct PlanArgs {
    /// Feature slug to plan.
    #[arg(long)]
    pub feature: String,

    /// Maximum number of work packages to generate.
    #[arg(long, default_value = "20")]
    pub max_wps: usize,

    /// Agent count per WP (for complexity estimation).
    #[arg(long, default_value = "1")]
    pub agents_per_wp: usize,
}

/// A parsed functional requirement.
#[derive(Debug, Clone)]
struct FunctionalRequirement {
    id: String,
    description: String,
}

/// Run the `plan` command.
pub async fn run_plan<S, V>(args: PlanArgs, storage: &S, vcs: &V) -> Result<()>
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
                slug,
                slug
            )
        })?;

    // Validate state: must be Researched (warn if not, but allow)
    if feature.state != FeatureState::Researched {
        if feature.state == FeatureState::Planned {
            anyhow::bail!(
                "Feature '{}' is already in 'Planned' state. No action needed.",
                slug
            );
        }
        eprintln!(
            "Warning: feature '{}' is in state '{}' (expected 'Researched'). Proceeding anyway.",
            slug, feature.state
        );
    }

    // Read spec and research artifacts
    let spec_content = vcs
        .read_artifact(slug, "spec.md")
        .await
        .unwrap_or_else(|_| String::new());
    let research_content = vcs
        .read_artifact(slug, "research.md")
        .await
        .unwrap_or_else(|_| String::new());

    // Parse FRs from spec
    let frs = parse_functional_requirements(&spec_content);
    tracing::debug!(fr_count = frs.len(), "parsed functional requirements");

    // Group FRs into WPs (3-7 FRs per WP, heuristic grouping)
    let wp_groups = group_frs_into_wps(&frs, args.max_wps);

    // Create WorkPackage domain objects (id=0 until persisted)
    let mut wps: Vec<WorkPackage> = wp_groups
        .iter()
        .enumerate()
        .map(|(i, group)| {
            let title = derive_wp_title(group, i + 1);
            let criteria = group
                .iter()
                .map(|fr| format!("- {} -- {}", fr.id, fr.description))
                .collect::<Vec<_>>()
                .join("\n");
            let description_for_scope = format!("{title}\n{criteria}\n{research_content}");
            let mut wp = WorkPackage::new(feature.id, &title, (i + 1) as i32, &criteria);
            wp.file_scope = detect_file_scope(&description_for_scope);
            wp
        })
        .collect();

    // Handle empty spec: create a placeholder WP
    if wps.is_empty() {
        let mut wp = WorkPackage::new(
            feature.id,
            "Initial Implementation",
            1,
            "- Implement the feature as specified.",
        );
        wp.file_scope = detect_file_scope(&spec_content);
        wps.push(wp);
    }

    // Persist WPs to get IDs
    let mut persisted_wps: Vec<WorkPackage> = Vec::with_capacity(wps.len());
    for mut wp in wps {
        let id = storage
            .create_work_package(&wp)
            .await
            .context("creating work package")?;
        wp.id = id;
        persisted_wps.push(wp);
    }

    // Build overlap graph and add file-overlap dependencies
    let overlap_graph = build_overlap_graph(&persisted_wps);
    let mut deps: Vec<WpDependency> = Vec::new();
    for (a_id, b_id, _) in &overlap_graph.edges {
        // Lower sequence depends on higher; higher depends on lower
        let a = persisted_wps.iter().find(|w| w.id == *a_id).unwrap();
        let b = persisted_wps.iter().find(|w| w.id == *b_id).unwrap();
        let (earlier, later) = if a.sequence <= b.sequence { (a, b) } else { (b, a) };
        let dep = WpDependency {
            wp_id: later.id,
            depends_on: earlier.id,
            dep_type: DependencyType::FileOverlap,
        };
        storage.add_wp_dependency(&dep).await.context("adding wp dependency")?;
        deps.push(dep);
    }

    // Build scheduler and compute execution plan
    let wp_states: std::collections::HashMap<i64, agileplus_domain::domain::work_package::WpState> =
        persisted_wps.iter().map(|w| (w.id, w.state)).collect();
    let scheduler = Scheduler::new(wp_states, deps.clone());
    let waves = scheduler.execution_plan().map_err(|e| anyhow::anyhow!("dependency cycle: {e}"))?;

    // Generate plan.md
    let plan_content = generate_plan_md(slug, &persisted_wps, &deps, &waves);
    vcs.write_artifact(slug, "plan.md", &plan_content)
        .await
        .context("writing plan.md")?;

    // Generate WP prompt files
    for wp in &persisted_wps {
        let prompt = generate_wp_prompt(wp, &feature.friendly_name, slug);
        let path = format!("tasks/WP{:02}-{}.md", wp.sequence, slugify(&wp.title));
        vcs.write_artifact(slug, &path, &prompt)
            .await
            .context("writing WP prompt file")?;
    }

    // Create governance contract
    let contract = build_governance_contract(feature.id, &persisted_wps);
    let contract_json = serde_json::to_string_pretty(&contract)
        .context("serializing governance contract")?;
    vcs.write_artifact(slug, "contracts/governance-v1.json", &contract_json)
        .await
        .context("writing governance contract artifact")?;
    storage
        .create_governance_contract(&contract)
        .await
        .context("persisting governance contract")?;

    // Transition feature state: Researched -> Planned
    storage
        .update_feature_state(feature.id, FeatureState::Planned)
        .await
        .context("transitioning feature to Planned")?;

    // Append audit entry
    let prev_hash = get_latest_hash(storage, feature.id).await;
    let mut audit = AuditEntry {
        id: 0,
        feature_id: feature.id,
        wp_id: None,
        timestamp: Utc::now(),
        actor: "user".into(),
        transition: "Researched -> Planned".into(),
        evidence_refs: vec![],
        prev_hash,
        hash: [0u8; 32],
            event_id: None,
            archived_to: None,
    };
    audit.hash = hash_entry(&audit);
    storage
        .append_audit_entry(&audit)
        .await
        .context("appending audit entry")?;

    let elapsed_ms = start.elapsed().as_millis();
    tracing::info!(command = "plan", slug = %slug, wp_count = persisted_wps.len(), elapsed_ms = %elapsed_ms, "plan completed");

    // Print summary table
    println!("Feature '{slug}' planned.");
    println!("  Generated {} work package(s):", persisted_wps.len());
    println!("  {:<6} {:<4} {:<40} {}", "WP ID", "Seq", "Title", "Dependencies");
    println!("  {}", "-".repeat(70));
    for wp in &persisted_wps {
        let dep_list: Vec<String> = deps
            .iter()
            .filter(|d| d.wp_id == wp.id)
            .map(|d| d.depends_on.to_string())
            .collect();
        let dep_str = if dep_list.is_empty() {
            "(none)".to_string()
        } else {
            dep_list.join(", ")
        };
        println!(
            "  {:<6} {:<4} {:<40} {}",
            wp.id,
            wp.sequence,
            &wp.title[..wp.title.len().min(40)],
            dep_str
        );
    }
    println!();
    println!("  Execution waves ({} total):", waves.len());
    for wave in &waves {
        let ids: Vec<String> = wave.wp_ids.iter().map(|id| id.to_string()).collect();
        println!("    Wave {}: [{}]", wave.wave_number, ids.join(", "));
    }
    println!();
    println!("  Plan written to: kitty-specs/{slug}/plan.md");
    println!("  Governance contract: kitty-specs/{slug}/contracts/governance-v1.json");
    println!("  State: Researched -> Planned");

    Ok(())
}

// --- helpers ---

/// Parse `FR-NNN: description` lines from spec content.
fn parse_functional_requirements(spec: &str) -> Vec<FunctionalRequirement> {
    let mut frs = Vec::new();
    for line in spec.lines() {
        // Match patterns like "- **FR-001**: description" or "FR-001: description"
        if let Some(pos) = line.find("FR-") {
            let rest = &line[pos..];
            // Find the ID
            let id_end = rest[3..]
                .find(|c: char| !c.is_ascii_digit())
                .map(|p| p + 3)
                .unwrap_or(rest.len());
            let id = &rest[..id_end];
            // Find description after colon
            let description = if let Some(colon) = rest.find(':') {
                rest[colon + 1..]
                    .trim()
                    .trim_matches('*')
                    .trim()
                    .to_string()
            } else {
                rest.to_string()
            };
            if !description.is_empty() && id.len() > 3 {
                frs.push(FunctionalRequirement {
                    id: id.to_string(),
                    description,
                });
            }
        }
    }
    frs.dedup_by_key(|fr| fr.id.clone());
    frs
}

/// Group FRs into logical WP batches (3-7 FRs per WP).
fn group_frs_into_wps(
    frs: &[FunctionalRequirement],
    max_wps: usize,
) -> Vec<Vec<FunctionalRequirement>> {
    if frs.is_empty() {
        return Vec::new();
    }
    let target_per_wp = ((frs.len() as f64) / (max_wps as f64).min(frs.len() as f64)).ceil() as usize;
    let per_wp = target_per_wp.max(3).min(7);
    frs.chunks(per_wp)
        .map(|chunk| chunk.to_vec())
        .collect()
}

/// Derive a human-readable WP title from a group of FRs.
fn derive_wp_title(frs: &[FunctionalRequirement], index: usize) -> String {
    if frs.is_empty() {
        return format!("Work Package {index:02}");
    }
    // Use the first FR description as a title hint, truncated
    let hint = &frs[0].description;
    let truncated: String = hint.chars().take(50).collect();
    format!("{truncated} (WP{index:02})")
}

/// Slugify a string for use in file paths.
fn slugify(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_alphanumeric() { c.to_ascii_lowercase() } else { '-' })
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
fn generate_plan_md(
    slug: &str,
    wps: &[WorkPackage],
    deps: &[WpDependency],
    waves: &[super::scheduler::ExecutionWave],
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
            for f in &wp.file_scope {
                lines.push(format!("  - `{f}`"));
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
fn generate_wp_prompt(wp: &WorkPackage, feature_name: &str, slug: &str) -> String {
    let date = Utc::now().format("%Y-%m-%d").to_string();
    let file_scope_str = if wp.file_scope.is_empty() {
        "(auto-detect from spec)".to_string()
    } else {
        wp.file_scope.iter().map(|f| format!("- `{f}`")).collect::<Vec<_>>().join("\n")
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
    lines.push("Implement this work package according to the acceptance criteria above.".to_string());
    lines.push(format!("Refer to `kitty-specs/{slug}/spec.md` for the full specification and"));
    lines.push(format!("`kitty-specs/{slug}/plan.md` for the implementation plan."));
    lines.join("\n")
}

/// Build a governance contract for a feature's work packages.
fn build_governance_contract(feature_id: i64, wps: &[WorkPackage]) -> GovernanceContract {
    let mut rules = Vec::new();

    // Rule: each WP must have CI pass before merge
    for wp in wps {
        rules.push(GovernanceRule {
            transition: format!("WP{:02}: Doing -> Review", wp.sequence),
            required_evidence: vec![EvidenceRequirement {
                fr_id: "FR-CI".to_string(),
                evidence_type: EvidenceType::CiOutput,
                threshold: None,
            }],
            policy_refs: vec!["policy:ci-required".to_string()],
        });
        rules.push(GovernanceRule {
            transition: format!("WP{:02}: Review -> Done", wp.sequence),
            required_evidence: vec![EvidenceRequirement {
                fr_id: "FR-REVIEW".to_string(),
                evidence_type: EvidenceType::ReviewApproval,
                threshold: None,
            }],
            policy_refs: vec!["policy:review-required".to_string()],
        });
    }

    GovernanceContract {
        id: 0,
        feature_id,
        version: 1,
        rules,
        bound_at: Utc::now(),
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
    fn parse_frs_basic() {
        let spec = "## Functional Requirements\n- **FR-001**: Login must work\n- **FR-002**: Logout must work\n";
        let frs = parse_functional_requirements(spec);
        assert_eq!(frs.len(), 2);
        assert_eq!(frs[0].id, "FR-001");
        assert!(frs[0].description.contains("Login"));
    }

    #[test]
    fn parse_frs_empty() {
        let frs = parse_functional_requirements("No FRs here");
        assert_eq!(frs.len(), 0);
    }

    #[test]
    fn group_frs_small() {
        let frs: Vec<FunctionalRequirement> = (1..=5)
            .map(|i| FunctionalRequirement { id: format!("FR-{i:03}"), description: format!("desc {i}") })
            .collect();
        let groups = group_frs_into_wps(&frs, 20);
        // All 5 frs should fit in some groups, each group 3-7
        let total: usize = groups.iter().map(|g| g.len()).sum();
        assert_eq!(total, 5);
    }

    #[test]
    fn group_frs_empty() {
        assert!(group_frs_into_wps(&[], 10).is_empty());
    }

    #[test]
    fn slugify_basic() {
        assert_eq!(slugify("Hello World WP01"), "hello-world-wp01");
    }

    #[test]
    fn generate_plan_md_contains_sections() {
        let wps = vec![WorkPackage::new(1, "Auth Module (WP01)", 1, "- FR-001 login")];
        let plan = generate_plan_md("my-feature", &wps, &[], &[]);
        assert!(plan.contains("# Plan: my-feature"));
        assert!(plan.contains("WP01: Auth Module"));
        assert!(plan.contains("Execution Waves"));
    }
}
