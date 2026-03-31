use anyhow::anyhow;
use anyhow::{Context, Result};

use agileplus_domain::domain::cycle::CycleState;
use agileplus_domain::error::DomainError;
use agileplus_domain::ports::StoragePort;
use agileplus_plane::maybe_sync_cycle_from_env;

use crate::commands::cycle::args::TransitionArgs;
use crate::commands::cycle::{find_cycle_by_name, prior_state_label};

// ---------------------------------------------------------------------------
// T023: transition (with Shipped gate enforcement)
// ---------------------------------------------------------------------------

pub(super) async fn cmd_transition<S: StoragePort>(
    args: TransitionArgs,
    storage: &S,
) -> Result<()> {
    let target: CycleState = args.to.parse().map_err(|e: DomainError| anyhow!("{}", e))?;

    let mut cycle = find_cycle_by_name(&args.cycle, storage).await?;

    // Validate state graph edge at domain level
    cycle.transition(target).map_err(|e| anyhow!("{}", e))?;

    // Shipped gate: all features must be Validated or Shipped
    if target == CycleState::Shipped {
        let cwf = storage
            .get_cycle_with_features(cycle.id)
            .await
            .context("loading cycle with features for shipped gate")?
            .ok_or_else(|| anyhow::anyhow!("Cycle '{}' disappeared unexpectedly.", args.cycle))?;

        if !cwf.is_shippable() {
            let blocking: Vec<String> = cwf
                .features
                .iter()
                .filter(|f| {
                    !matches!(
                        f.state,
                        agileplus_domain::domain::state_machine::FeatureState::Validated
                            | agileplus_domain::domain::state_machine::FeatureState::Shipped
                    )
                })
                .map(|f| format!("  {} (state: {})", f.slug, f.state))
                .collect();
            anyhow::bail!(
                "Cannot transition cycle '{}' to Shipped: {} feature(s) are not Validated or Shipped:\n{}\n\
                 Run `agileplus validate --feature <slug>` for each blocking feature.",
                args.cycle,
                blocking.len(),
                blocking.join("\n")
            );
        }
    }

    // Persist
    storage
        .update_cycle_state(cycle.id, target)
        .await
        .context("persisting cycle state transition")?;

    if let Err(err) = maybe_sync_cycle_from_env(storage, cycle.id).await {
        tracing::warn!(cycle_id = cycle.id, error = %err, "Plane sync after cycle transition failed");
    }

    println!(
        "Cycle '{}' transitioned: {} -> {}.",
        args.cycle,
        prior_state_label(target, &cycle),
        target
    );

    Ok(())
}
