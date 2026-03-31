use anyhow::{Context, Result, anyhow};

use crate::commands::cycle::args::RemoveArgs;
use crate::commands::cycle::find_cycle_by_name;
use agileplus_domain::ports::StoragePort;
use agileplus_plane::maybe_sync_feature_cycle_unassignment_from_env;

// ---------------------------------------------------------------------------
// T022: add + remove
// ---------------------------------------------------------------------------

pub(super) async fn cmd_remove<S: StoragePort>(args: RemoveArgs, storage: &S) -> Result<()> {
    let cycle = find_cycle_by_name(&args.cycle, storage).await?;

    let feature = storage
        .get_feature_by_slug(&args.feature)
        .await
        .context("looking up feature")?
        .ok_or_else(|| anyhow!("Feature '{}' not found.", args.feature))?;

    storage
        .remove_feature_from_cycle(cycle.id, feature.id)
        .await
        .context("removing feature from cycle")?;

    if let Err(err) =
        maybe_sync_feature_cycle_unassignment_from_env(storage, feature.id, cycle.id).await
    {
        tracing::warn!(
            feature_id = feature.id,
            cycle_id = cycle.id,
            error = %err,
            "Plane sync after cycle remove failed"
        );
    }

    println!(
        "Feature '{}' removed from cycle '{}'. Feature state is unchanged.",
        args.feature, args.cycle
    );

    Ok(())
}
