use anyhow::{Context, Result, anyhow};

use agileplus_domain::domain::module::ModuleFeatureTag;
use agileplus_domain::ports::StoragePort;
use agileplus_plane::{maybe_sync_feature_module_assignment_from_env, maybe_sync_module_from_env};

use super::TagArgs;

// ---------------------------------------------------------------------------
// T017: tag
// ---------------------------------------------------------------------------

pub async fn run_tag<S: StoragePort>(args: TagArgs, storage: &S) -> Result<()> {
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

    let tag = ModuleFeatureTag::new(module.id, feature.id);
    storage
        .tag_feature_to_module(&tag)
        .await
        .context("tagging feature to module")?;

    if let Err(err) = maybe_sync_module_from_env(storage, module.id).await {
        tracing::warn!(module_id = module.id, error = %err, "Plane sync before module tag failed");
    }
    if let Err(err) =
        maybe_sync_feature_module_assignment_from_env(storage, feature.id, module.id).await
    {
        tracing::warn!(
            feature_id = feature.id,
            module_id = module.id,
            error = %err,
            "Plane sync after module tag failed"
        );
    }

    println!(
        "Feature '{}' tagged to module '{}'.",
        args.feature, args.module
    );
    Ok(())
}
