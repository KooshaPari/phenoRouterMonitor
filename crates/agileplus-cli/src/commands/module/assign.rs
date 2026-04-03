use anyhow::{Context, Result, anyhow};

use agileplus_domain::domain::module::ModuleFeatureTag;
use agileplus_domain::ports::StoragePort;
use agileplus_plane::{maybe_sync_feature_module_assignment_from_env, maybe_sync_module_from_env};

use super::AssignArgs;

// ---------------------------------------------------------------------------
// T017: assign
// ---------------------------------------------------------------------------

pub async fn run_assign<S: StoragePort>(args: AssignArgs, storage: &S) -> Result<()> {
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

    // Primary ownership is recorded via the module_feature_tags join table
    // (the storage port does not yet expose a direct feature.module_id update).
    let tag = ModuleFeatureTag::new(module.id, feature.id);
    storage
        .tag_feature_to_module(&tag)
        .await
        .context("assigning feature to module")?;

    if let Err(err) = maybe_sync_module_from_env(storage, module.id).await {
        tracing::warn!(module_id = module.id, error = %err, "Plane sync before module assignment failed");
    }
    if let Err(err) =
        maybe_sync_feature_module_assignment_from_env(storage, feature.id, module.id).await
    {
        tracing::warn!(
            feature_id = feature.id,
            module_id = module.id,
            error = %err,
            "Plane sync after module assignment failed"
        );
    }

    println!(
        "Feature '{}' assigned to module '{}' (recorded as ownership tag).",
        args.feature, args.module
    );
    Ok(())
}
