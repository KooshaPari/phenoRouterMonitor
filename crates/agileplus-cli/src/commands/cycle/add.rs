use anyhow::{Context, Result, anyhow};

use agileplus_domain::domain::cycle::CycleFeature;
use agileplus_domain::error::DomainError;
use agileplus_domain::ports::StoragePort;
use agileplus_plane::{maybe_sync_cycle_from_env, maybe_sync_feature_cycle_assignment_from_env};

use crate::commands::cycle::args::AddArgs;
use crate::commands::cycle::find_cycle_by_name;

// ---------------------------------------------------------------------------
// T022: add + remove
// ---------------------------------------------------------------------------

pub(super) async fn cmd_add<S: StoragePort>(args: AddArgs, storage: &S) -> Result<()> {
    let cycle = find_cycle_by_name(&args.cycle, storage).await?;

    let feature = storage
        .get_feature_by_slug(&args.feature)
        .await
        .context("looking up feature")?
        .ok_or_else(|| {
            anyhow!(
                "Feature '{}' not found. Create it with `agileplus specify --feature {}`.",
                args.feature,
                args.feature
            )
        })?;

    let entry = CycleFeature::new(cycle.id, feature.id);
    match storage.add_feature_to_cycle(&entry).await {
        Ok(()) => {
            if let Err(err) = maybe_sync_cycle_from_env(storage, cycle.id).await {
                tracing::warn!(cycle_id = cycle.id, error = %err, "Plane sync before cycle assignment failed");
            }
            if let Err(err) =
                maybe_sync_feature_cycle_assignment_from_env(storage, feature.id, cycle.id).await
            {
                tracing::warn!(
                    feature_id = feature.id,
                    cycle_id = cycle.id,
                    error = %err,
                    "Plane sync after cycle assignment failed"
                );
            }
            println!(
                "Feature '{}' added to cycle '{}'.",
                args.feature, args.cycle
            );
        }
        Err(DomainError::FeatureNotInModuleScope {
            ref feature_slug,
            ref module_slug,
        }) => {
            anyhow::bail!(
                "Cannot add feature '{}' to cycle '{}': the cycle is scoped to module '{}'.\n\
                 Feature '{}' is not owned by or tagged to that module.\n\
                 Tag the feature first with `agileplus module tag --module {} --feature {}`.",
                feature_slug,
                args.cycle,
                module_slug,
                feature_slug,
                module_slug,
                feature_slug
            );
        }
        Err(e) => {
            return Err(anyhow::anyhow!(e).context("adding feature to cycle"));
        }
    }

    Ok(())
}
