use anyhow::{Context, Result, anyhow};

use agileplus_domain::ports::StoragePort;
use agileplus_plane::maybe_sync_feature_module_unassignment_from_env;

use super::UntagArgs;

// ---------------------------------------------------------------------------
// T017: untag
// ---------------------------------------------------------------------------

pub async fn run_untag<S: StoragePort>(args: UntagArgs, storage: &S) -> Result<()> {
    let module = storage
        .get_module_by_slug(&args.module)
        .await
        .context("looking up module")?
        .ok_or_else(|| anyhow!("module '{}' not found", args.module))?;

    let feature = storage
        .get_feature_by_slug(&args.feature)
        .await
        .context("looking up feature")?
        .ok_or_else(|| anyhow!("feature '{}' not found", args.feature))?;

    storage
        .untag_feature_from_module(module.id, feature.id)
        .await
        .context("removing feature tag from module")?;

    if let Err(err) =
        maybe_sync_feature_module_unassignment_from_env(storage, feature.id, module.id).await
    {
        tracing::warn!(
            feature_id = feature.id,
            module_id = module.id,
            error = %err,
            "Plane sync after module untag failed"
        );
    }

    println!(
        "Feature '{}' untagged from module '{}'.",
        args.feature, args.module
    );
    Ok(())
}
