//! `agileplus ship` command implementation.
//!
//! Merges all WP branches to the target branch, cleans up worktrees,
//! archives the feature, and transitions to Shipped.
//! Traceability: FR-006 / WP13-T075, T077

use anyhow::{Context, Result};
use chrono::Utc;

use agileplus_domain::domain::audit::{hash_entry, AuditEntry};
use agileplus_domain::domain::state_machine::FeatureState;
use agileplus_domain::domain::work_package::WpState;
use agileplus_domain::ports::{StoragePort, VcsPort};

/// Arguments for the `ship` subcommand.
#[derive(Debug, clap::Args)]
pub struct ShipArgs {
    /// Feature slug to ship.
    #[arg(long)]
    pub feature: String,

    /// Target branch override (default: feature.target_branch).
    #[arg(long)]
    pub target: Option<String>,

    /// Skip validation check (dangerous).
    #[arg(long)]
    pub skip_validate: bool,

    /// Dry run: show what would be merged without doing it.
    #[arg(long)]
    pub dry_run: bool,
}

/// Run the `ship` command.
pub async fn run_ship<S, V>(args: ShipArgs, storage: &S, vcs: &V) -> Result<()>
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
                "Feature '{}' not found. Run `agileplus plan --feature {}` first.",
                slug, slug
            )
        })?;

    // State enforcement
    if feature.state != FeatureState::Validated {
        if args.skip_validate {
            eprintln!(
                "Warning: --skip-validate used. Feature '{}' is in state '{}' (expected 'Validated').",
                slug, feature.state
            );
        } else {
            anyhow::bail!(
                "Feature '{}' is in state '{}'. Expected 'Validated'. \
                Run `agileplus validate --feature {}` first, or use --skip-validate.",
                slug, feature.state, slug
            );
        }
    }

    // Determine target branch
    let target_branch = args.target.clone().unwrap_or_else(|| feature.target_branch.clone());
    tracing::debug!(target_branch = %target_branch, "shipping to target branch");

    // Load all WPs for the feature
    let all_wps = storage
        .list_wps_by_feature(feature.id)
        .await
        .context("listing work packages")?;

    // Check all WPs are done
    let incomplete: Vec<_> = all_wps.iter()
        .filter(|wp| wp.state != WpState::Done)
        .collect();

    if !incomplete.is_empty() {
        let names: Vec<String> = incomplete.iter()
            .map(|wp| format!("WP{:02} '{}' (state: {:?})", wp.sequence, wp.title, wp.state))
            .collect();
        anyhow::bail!(
            "Feature '{}' has incomplete work packages:\n  {}\nFinish all WPs before shipping.",
            slug,
            names.join("\n  ")
        );
    }

    // Collect WPs that have a worktree path or PR (in sequence order)
    let mut sorted_wps = all_wps.clone();
    sorted_wps.sort_by_key(|wp| wp.sequence);

    // Derive branch name for each WP from worktree_path or a convention
    // Convention: feature/{slug}/wp{sequence:02}
    let wp_branches: Vec<(String, String)> = sorted_wps.iter()
        .map(|wp| {
            // Derive branch from worktree_path if available, otherwise use convention
            let branch = wp.worktree_path.as_deref()
                .and_then(|p| {
                    // Extract branch name from path like ".worktrees/slug-WP01" -> "slug/wp01"
                    std::path::Path::new(p)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .map(|n| n.to_string())
                })
                .unwrap_or_else(|| format!("feature/{slug}/wp{:02}", wp.sequence));
            (format!("WP{:02}", wp.sequence), branch)
        })
        .collect();

    // Dry-run: just show the merge plan
    if args.dry_run {
        println!("Dry-run: merge plan for feature '{slug}'");
        println!("  Target branch: {target_branch}");
        println!("  WPs to merge ({} total):", sorted_wps.len());
        for (wp, (wp_label, branch)) in sorted_wps.iter().zip(wp_branches.iter()) {
            println!("    {} '{}' <- branch: {}", wp_label, wp.title, branch);
        }
        if sorted_wps.is_empty() {
            println!("    (no WPs to merge)");
        }
        return Ok(());
    }

    // Perform merges in order
    let mut merged_branches: Vec<String> = Vec::new();
    for (wp, (wp_label, branch)) in sorted_wps.iter().zip(wp_branches.iter()) {
        tracing::info!(wp_seq = wp.sequence, branch = %branch, target = %target_branch, "merging WP branch");

        let merge_result = match vcs.merge_to_target(branch, &target_branch).await {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!(branch = %branch, error = %e, "merge failed (branch may not exist, skipping)");
                continue;
            }
        };

        if !merge_result.success {
            let conflicts: Vec<String> = merge_result.conflicts.iter()
                .map(|c| c.path.clone())
                .collect();
            anyhow::bail!(
                "Merge conflict when merging {} branch '{}' into '{}'.\n\
                Conflicting files:\n  {}\n\
                Resolve conflicts manually and re-run `agileplus ship`.",
                wp_label,
                branch,
                target_branch,
                conflicts.join("\n  ")
            );
        }

        merged_branches.push(branch.clone());
        tracing::info!(branch = %branch, commit = ?merge_result.merged_commit, "merged successfully");
    }

    // Clean up worktrees
    let worktrees = vcs.list_worktrees().await.unwrap_or_default();
    for worktree in &worktrees {
        if worktree.feature_slug == *slug {
            match vcs.cleanup_worktree(&worktree.path).await {
                Ok(()) => tracing::info!(path = ?worktree.path, "cleaned up worktree"),
                Err(e) => tracing::warn!(path = ?worktree.path, error = %e, "failed to clean worktree (skipping)"),
            }
        }
    }

    // Write final meta artifact
    let meta = serde_json::json!({
        "feature_slug": slug,
        "state": "shipped",
        "target_branch": target_branch,
        "shipped_at": Utc::now().to_rfc3339(),
        "merged_branches": merged_branches,
        "wp_count": all_wps.len(),
    });
    let meta_json = serde_json::to_string_pretty(&meta).unwrap_or_default();
    vcs.write_artifact(slug, "meta.json", &meta_json)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!("Failed to write meta.json: {e}");
        });

    // Transition feature state to Shipped
    storage
        .update_feature_state(feature.id, FeatureState::Shipped)
        .await
        .context("transitioning feature to Shipped")?;

    // Append audit entry
    let prev_hash = get_latest_hash(storage, feature.id).await;
    let mut audit = AuditEntry {
        id: 0,
        feature_id: feature.id,
        wp_id: None,
        timestamp: Utc::now(),
        actor: "user".into(),
        transition: "Validated -> Shipped".into(),
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
    tracing::info!(command = "ship", slug = %slug, merged = merged_branches.len(), elapsed_ms = %elapsed_ms, "ship completed");

    println!("Feature '{}' shipped.", slug);
    println!("  Target branch: {target_branch}");
    println!("  WPs merged: {}", merged_branches.len());
    println!("  State: Validated -> Shipped");

    Ok(())
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

    #[test]
    fn ship_args_defaults() {
        // Verify struct can be constructed
        let args = ShipArgs {
            feature: "my-feature".to_string(),
            target: None,
            skip_validate: false,
            dry_run: false,
        };
        assert_eq!(args.feature, "my-feature");
        assert!(!args.dry_run);
    }

    #[test]
    fn ship_args_with_target() {
        let args = ShipArgs {
            feature: "feat".to_string(),
            target: Some("release/v1.0".to_string()),
            skip_validate: false,
            dry_run: true,
        };
        assert_eq!(args.target, Some("release/v1.0".to_string()));
        assert!(args.dry_run);
    }

    #[test]
    fn audit_hash_is_nonzero() {
        let mut entry = AuditEntry {
            id: 0,
            feature_id: 42,
            wp_id: None,
            timestamp: Utc::now(),
            actor: "user".into(),
            transition: "Validated -> Shipped".into(),
            evidence_refs: vec![],
            prev_hash: [0u8; 32],
            hash: [0u8; 32],
            event_id: None,
            archived_to: None,
        };
        entry.hash = hash_entry(&entry);
        assert_ne!(entry.hash, [0u8; 32]);
    }
}
