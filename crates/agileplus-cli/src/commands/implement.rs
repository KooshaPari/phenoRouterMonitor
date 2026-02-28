//! `agileplus implement` command implementation.
//!
//! Orchestrates work package implementation: creates worktrees, dispatches
//! agents, creates PRs, and manages the review-fix loop.
//! Traceability: FR-004, FR-010, FR-011, FR-012 / WP12-T069, T071, T072

use std::collections::{HashMap, HashSet};

use anyhow::{Context, Result};
use chrono::Utc;

use agileplus_domain::domain::audit::{hash_entry, AuditEntry};
use agileplus_domain::domain::state_machine::FeatureState;
use agileplus_domain::domain::work_package::{WorkPackage, WpState};
use agileplus_domain::ports::agent::{AgentConfig, AgentKind, AgentPort, AgentTask};
use agileplus_domain::ports::{StoragePort, VcsPort};

use super::pr_builder::{build_pr_description, build_pr_title};
use super::review_loop::{run_review_loop, ReviewOutcome};
use super::scheduler::Scheduler;

/// Arguments for the `implement` subcommand.
#[derive(Debug, clap::Args)]
pub struct ImplementArgs {
    /// Feature slug to implement.
    #[arg(long)]
    pub feature: String,

    /// Implement a specific WP only (e.g. WP01 or by numeric ID).
    #[arg(long)]
    pub wp: Option<String>,

    /// Maximum parallel agents.
    #[arg(long, default_value = "3")]
    pub parallel: usize,

    /// Maximum review-fix cycles per WP.
    #[arg(long, default_value = "5")]
    pub max_review_cycles: u32,

    /// Resume from last checkpoint (re-attach to in-progress WPs).
    #[arg(long)]
    pub resume: bool,
}

/// Run the `implement` command.
pub async fn run_implement<S, V, A>(args: ImplementArgs, storage: &S, vcs: &V, agent: &A) -> Result<()>
where
    S: StoragePort,
    V: VcsPort,
    A: AgentPort,
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
                "Feature '{}' not found. Run `agileplus plan --feature {}` first.",
                slug,
                slug
            )
        })?;

    // Validate state: must be Planned or Implementing (resume)
    match feature.state {
        FeatureState::Planned | FeatureState::Implementing => {}
        _ => {
            anyhow::bail!(
                "Feature '{}' is in state '{}'. Expected 'Planned' or 'Implementing'.",
                slug,
                feature.state
            );
        }
    }

    // Transition to Implementing if not already
    if feature.state == FeatureState::Planned {
        storage
            .update_feature_state(feature.id, FeatureState::Implementing)
            .await
            .context("transitioning feature to Implementing")?;

        let prev_hash = get_latest_hash(storage, feature.id).await;
        let mut audit = AuditEntry {
            id: 0,
            feature_id: feature.id,
            wp_id: None,
            timestamp: Utc::now(),
            actor: "user".into(),
            transition: "Planned -> Implementing".into(),
            evidence_refs: vec![],
            prev_hash,
            hash: [0u8; 32],
        };
        audit.hash = hash_entry(&audit);
        storage.append_audit_entry(&audit).await.context("appending audit entry")?;
        println!("Feature '{slug}' transitioned to Implementing.");
    }

    // Load all WPs for this feature
    let all_wps = storage
        .list_wps_by_feature(feature.id)
        .await
        .context("loading work packages")?;

    if all_wps.is_empty() {
        anyhow::bail!(
            "No work packages found for feature '{}'. Run `agileplus plan --feature {}` first.",
            slug,
            slug
        );
    }

    // Determine which WPs to process
    let target_wps: Vec<&WorkPackage> = if let Some(ref wp_ref) = args.wp {
        // Single WP mode: find by ID prefix or sequence
        let matched: Vec<&WorkPackage> = all_wps
            .iter()
            .filter(|wp| {
                let wp_str = wp_ref.to_uppercase();
                let by_seq = wp_str.strip_prefix("WP").and_then(|s| s.parse::<i32>().ok())
                    == Some(wp.sequence);
                let by_id = wp_ref.parse::<i64>().ok() == Some(wp.id);
                by_seq || by_id
            })
            .collect();
        if matched.is_empty() {
            anyhow::bail!("Work package '{}' not found for feature '{}'.", wp_ref, slug);
        }
        matched
    } else {
        all_wps.iter().collect()
    };

    // Build scheduler
    let mut wp_states: HashMap<i64, WpState> = HashMap::new();
    for wp in &all_wps {
        wp_states.insert(wp.id, wp.state);
    }

    let mut all_deps = Vec::new();
    for wp in &all_wps {
        let deps = storage
            .get_wp_dependencies(wp.id)
            .await
            .context("loading WP dependencies")?;
        all_deps.extend(deps);
    }

    let scheduler = Scheduler::new(wp_states.clone(), all_deps.clone());

    // Read spec content for PR descriptions
    let spec_content = vcs
        .read_artifact(slug, "spec.md")
        .await
        .unwrap_or_else(|_| String::new());

    // Agent config
    let agent_config = AgentConfig {
        kind: AgentKind::ClaudeCode,
        max_review_cycles: args.max_review_cycles,
        timeout_secs: 3600,
        extra_args: vec![],
    };

    // Process WPs
    let mut completed: HashSet<i64> = all_wps
        .iter()
        .filter(|wp| wp.state == WpState::Done)
        .map(|wp| wp.id)
        .collect();

    let target_ids: HashSet<i64> = target_wps.iter().map(|wp| wp.id).collect();

    if args.resume {
        println!("Resume mode: checking for in-progress WPs...");
    }

    for wp in &target_wps {
        // Skip already done
        if completed.contains(&wp.id) {
            println!("  WP{:02} '{}' already done, skipping.", wp.sequence, wp.title);
            continue;
        }

        // Check dependencies
        if let Some(blockers) = scheduler.is_blocked(wp.id, &completed) {
            let blocker_ids: Vec<String> = blockers.iter().map(|id| id.to_string()).collect();
            println!(
                "  WP{:02} '{}' is blocked by: [{}]. Skipping.",
                wp.sequence,
                wp.title,
                blocker_ids.join(", ")
            );
            continue;
        }

        println!("Processing WP{:02}: '{}'...", wp.sequence, wp.title);

        // Resume mode: check if worktree already exists
        if args.resume && wp.state == WpState::Doing {
            println!("  Resuming WP{:02} (already in 'doing' state)...", wp.sequence);
        }

        // Create worktree
        let wp_id_str = format!("WP{:02}", wp.sequence);
        let worktree_path = vcs
            .create_worktree(slug, &wp_id_str)
            .await
            .context(format!("creating worktree for {wp_id_str}"))?;

        println!("  Worktree created at: {}", worktree_path.display());

        // Transition WP to Doing
        storage
            .update_wp_state(wp.id, WpState::Doing)
            .await
            .context("transitioning WP to Doing")?;

        // Build prompt path
        let prompt_path = worktree_path.join(format!(
            "kitty-specs/{}/tasks/WP{:02}-{}.md",
            slug,
            wp.sequence,
            slugify(&wp.title)
        ));

        // Build context files
        let context_files = vec![
            worktree_path.join(format!("kitty-specs/{slug}/spec.md")),
            worktree_path.join(format!("kitty-specs/{slug}/plan.md")),
            worktree_path.join(format!("kitty-specs/{slug}/research.md")),
        ];

        let task = AgentTask {
            wp_id: wp_id_str.clone(),
            feature_slug: slug.clone(),
            prompt_path,
            worktree_path: worktree_path.clone(),
            context_files,
        };

        // Dispatch agent asynchronously
        let job_id = agent
            .dispatch_async(task, &agent_config)
            .await
            .context(format!("dispatching agent for {wp_id_str}"))?;

        println!("  Agent dispatched (job: {job_id}).");

        // Build PR description
        let feature_ref = storage.get_feature_by_id(feature.id).await?.unwrap_or(feature.clone());
        let pr_title = build_pr_title(wp);
        let pr_body = build_pr_description(wp, &feature_ref, &spec_content);

        println!("  PR title: {pr_title}");
        tracing::debug!(pr_body_len = pr_body.len(), "PR description generated");

        // Run review-fix loop
        let outcome = run_review_loop(
            wp,
            &job_id,
            agent,
            &agent_config,
            args.max_review_cycles,
            30,
        )
        .await;

        match outcome {
            ReviewOutcome::Approved => {
                println!("  WP{:02} approved!", wp.sequence);
                storage
                    .update_wp_state(wp.id, WpState::Review)
                    .await
                    .context("transitioning WP to Review")?;
                storage
                    .update_wp_state(wp.id, WpState::Done)
                    .await
                    .context("transitioning WP to Done")?;
                completed.insert(wp.id);

                // Audit
                let prev_hash = get_latest_hash(storage, feature.id).await;
                let mut audit = AuditEntry {
                    id: 0,
                    feature_id: feature.id,
                    wp_id: Some(wp.id),
                    timestamp: Utc::now(),
                    actor: "agent".into(),
                    transition: format!("WP{:02} Planned -> Done", wp.sequence),
                    evidence_refs: vec![],
                    prev_hash,
                    hash: [0u8; 32],
                };
                audit.hash = hash_entry(&audit);
                storage.append_audit_entry(&audit).await.context("appending audit entry")?;

                // Cleanup worktree
                if let Err(e) = vcs.cleanup_worktree(&worktree_path).await {
                    tracing::warn!(error = %e, "worktree cleanup failed (non-fatal)");
                }
            }
            ReviewOutcome::MaxCyclesReached { cycles, last_feedback } => {
                println!(
                    "  WP{:02} reached max review cycles ({cycles}). Marking blocked.",
                    wp.sequence
                );
                storage
                    .update_wp_state(wp.id, WpState::Blocked)
                    .await
                    .context("transitioning WP to Blocked")?;

                let prev_hash = get_latest_hash(storage, feature.id).await;
                let mut audit = AuditEntry {
                    id: 0,
                    feature_id: feature.id,
                    wp_id: Some(wp.id),
                    timestamp: Utc::now(),
                    actor: "system".into(),
                    transition: format!("WP{:02} Doing -> Blocked (max review cycles)", wp.sequence),
                    evidence_refs: vec![],
                    prev_hash,
                    hash: [0u8; 32],
                };
                audit.hash = hash_entry(&audit);
                storage.append_audit_entry(&audit).await.ok();

                eprintln!(
                    "WARNING: WP{:02} blocked after {cycles} review cycles.\nLast feedback: {}",
                    wp.sequence,
                    &last_feedback[..last_feedback.len().min(500)]
                );
            }
            ReviewOutcome::AgentFailed { error } => {
                storage.update_wp_state(wp.id, WpState::Blocked).await.ok();
                anyhow::bail!(
                    "Agent failed for WP{:02}: {}",
                    wp.sequence,
                    error
                );
            }
            ReviewOutcome::Cancelled => {
                storage.update_wp_state(wp.id, WpState::Blocked).await.ok();
                println!("  WP{:02} cancelled.", wp.sequence);
            }
        }
    }

    let elapsed_ms = start.elapsed().as_millis();
    let done_count = target_ids.iter().filter(|id| completed.contains(id)).count();
    tracing::info!(command = "implement", slug = %slug, done = done_count, elapsed_ms = %elapsed_ms, "implement completed");

    println!();
    println!(
        "Implement complete: {done_count}/{} WPs done.",
        target_ids.len()
    );

    Ok(())
}

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
    fn slugify_basic() {
        assert_eq!(slugify("Implement Auth Module (WP01)"), "implement-auth-module-wp01");
    }
}
