use anyhow::{Context, Result};
use chrono::Utc;
use std::collections::HashSet;

use agileplus_domain::domain::audit::{AuditEntry, hash_entry};
use agileplus_domain::domain::feature::Feature;
use agileplus_domain::domain::work_package::{WorkPackage, WpState};
use agileplus_domain::ports::agent::{AgentConfig, AgentPort, AgentTask};
use agileplus_domain::ports::{StoragePort, VcsPort};

use super::super::pr_builder::{build_pr_description, build_pr_title};
use super::super::review_loop::{ReviewOutcome, run_review_loop};
use super::Scheduler;

pub(crate) struct ImplementContext<'a, S, V, A> {
    pub storage: &'a S,
    pub vcs: &'a V,
    pub agent: &'a A,
    pub agent_config: &'a AgentConfig,
    pub feature: &'a Feature,
    pub feature_ref: &'a Feature,
    pub slug: &'a str,
    pub spec_content: &'a str,
    pub scheduler: &'a Scheduler,
    pub completed: &'a mut HashSet<i64>,
}

pub(crate) async fn process_target_wps<S, V, A>(
    ctx: &mut ImplementContext<'_, S, V, A>,
    target_wps: &[&WorkPackage],
    resume: bool,
    max_review_cycles: u32,
) -> Result<()>
where
    S: StoragePort,
    V: VcsPort,
    A: AgentPort,
{
    for wp in target_wps {
        if ctx.completed.contains(&wp.id) {
            println!(
                "  WP{:02} '{}' already done, skipping.",
                wp.sequence, wp.title
            );
            continue;
        }

        if let Some(blockers) = ctx.scheduler.is_blocked(wp.id, ctx.completed) {
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

        if resume && wp.state == WpState::Doing {
            println!(
                "  Resuming WP{:02} (already in 'doing' state)...",
                wp.sequence
            );
        }

        let wp_id_str = format!("WP{:02}", wp.sequence);
        let worktree_path = ctx
            .vcs
            .create_worktree(ctx.slug, &wp_id_str)
            .await
            .context(format!("creating worktree for {wp_id_str}"))?;

        println!("  Worktree created at: {}", worktree_path.display());

        ctx.storage
            .update_wp_state(wp.id, WpState::Doing)
            .await
            .context("transitioning WP to Doing")?;

        let prompt_path = worktree_path.join(format!(
            "agileplus/{}/tasks/WP{:02}-{}.md",
            ctx.slug,
            wp.sequence,
            slugify(&wp.title)
        ));

        let context_files = vec![
            worktree_path.join(format!("agileplus/{}/spec.md", ctx.slug)),
            worktree_path.join(format!("agileplus/{}/plan.md", ctx.slug)),
            worktree_path.join(format!("agileplus/{}/research.md", ctx.slug)),
        ];

        let task = AgentTask {
            wp_id: wp_id_str.clone(),
            feature_slug: ctx.slug.to_string(),
            prompt_path,
            worktree_path: worktree_path.clone(),
            context_files,
        };

        let job_id = ctx
            .agent
            .dispatch_async(task, ctx.agent_config)
            .await
            .context(format!("dispatching agent for {wp_id_str}"))?;

        println!("  Agent dispatched (job: {job_id}).");

        let pr_title = build_pr_title(wp);
        let pr_body = build_pr_description(wp, ctx.feature_ref, ctx.spec_content);

        println!("  PR title: {pr_title}");
        tracing::debug!(pr_body_len = pr_body.len(), "PR description generated");

        let outcome = run_review_loop(
            wp,
            &job_id,
            ctx.agent,
            ctx.agent_config,
            max_review_cycles,
            30,
        )
        .await;

        match outcome {
            ReviewOutcome::Approved => {
                println!("  WP{:02} approved!", wp.sequence);
                ctx.storage
                    .update_wp_state(wp.id, WpState::Review)
                    .await
                    .context("transitioning WP to Review")?;
                ctx.storage
                    .update_wp_state(wp.id, WpState::Done)
                    .await
                    .context("transitioning WP to Done")?;
                ctx.completed.insert(wp.id);

                let prev_hash = get_latest_hash(ctx.storage, ctx.feature.id).await;
                let mut audit = AuditEntry {
                    id: 0,
                    feature_id: ctx.feature.id,
                    wp_id: Some(wp.id),
                    timestamp: Utc::now(),
                    actor: "agent".into(),
                    transition: format!("WP{:02} Planned -> Done", wp.sequence),
                    evidence_refs: vec![],
                    prev_hash,
                    hash: [0u8; 32],
                    event_id: None,
                    archived_to: None,
                };
                audit.hash = hash_entry(&audit);
                ctx.storage
                    .append_audit_entry(&audit)
                    .await
                    .context("appending audit entry")?;

                if let Err(e) = ctx.vcs.cleanup_worktree(&worktree_path).await {
                    tracing::warn!(error = %e, "worktree cleanup failed (non-fatal)");
                }
            }
            ReviewOutcome::MaxCyclesReached {
                cycles,
                last_feedback,
            } => {
                println!(
                    "  WP{:02} reached max review cycles ({cycles}). Marking blocked.",
                    wp.sequence
                );
                ctx.storage
                    .update_wp_state(wp.id, WpState::Blocked)
                    .await
                    .context("transitioning WP to Blocked")?;

                let prev_hash = get_latest_hash(ctx.storage, ctx.feature.id).await;
                let mut audit = AuditEntry {
                    id: 0,
                    feature_id: ctx.feature.id,
                    wp_id: Some(wp.id),
                    timestamp: Utc::now(),
                    actor: "system".into(),
                    transition: format!(
                        "WP{:02} Doing -> Blocked (max review cycles)",
                        wp.sequence
                    ),
                    evidence_refs: vec![],
                    prev_hash,
                    hash: [0u8; 32],
                    event_id: None,
                    archived_to: None,
                };
                audit.hash = hash_entry(&audit);
                ctx.storage.append_audit_entry(&audit).await.ok();

                eprintln!(
                    "WARNING: WP{:02} blocked after {cycles} review cycles.\nLast feedback: {}",
                    wp.sequence,
                    &last_feedback[..last_feedback.len().min(500)]
                );
            }
            ReviewOutcome::AgentFailed { error } => {
                ctx.storage
                    .update_wp_state(wp.id, WpState::Blocked)
                    .await
                    .ok();
                anyhow::bail!("Agent failed for WP{:02}: {}", wp.sequence, error);
            }
            ReviewOutcome::Cancelled => {
                ctx.storage
                    .update_wp_state(wp.id, WpState::Blocked)
                    .await
                    .ok();
                println!("  WP{:02} cancelled.", wp.sequence);
            }
        }
    }

    Ok(())
}

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

pub(crate) async fn get_latest_hash<S: StoragePort>(storage: &S, feature_id: i64) -> [u8; 32] {
    match storage.get_latest_audit_entry(feature_id).await {
        Ok(Some(entry)) => entry.hash,
        _ => [0u8; 32],
    }
}
