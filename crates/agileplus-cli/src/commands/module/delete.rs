use anyhow::{Context, Result, anyhow};

use agileplus_domain::error::DomainError;
use agileplus_domain::ports::StoragePort;
use agileplus_plane::maybe_delete_module_from_env;

use super::DeleteArgs;

// ---------------------------------------------------------------------------
// T015: delete
// ---------------------------------------------------------------------------

pub async fn run_delete<S: StoragePort>(args: DeleteArgs, storage: &S) -> Result<()> {
    let module = storage
        .get_module_by_slug(&args.slug)
        .await
        .context("looking up module")?
        .ok_or_else(|| anyhow::anyhow!("module '{}' not found", args.slug))?;

    storage
        .delete_module(module.id)
        .await
        .map_err(|e| match e {
            DomainError::ModuleHasDependents(msg) => anyhow!(
                "cannot delete module '{}': it still has children or owned features.\n  Detail: {}\n  Reassign or delete dependents first.",
                args.slug,
                msg
            ),
            other => anyhow!("deleting module '{}': {}", args.slug, other),
        })?;

    if let Err(err) = maybe_delete_module_from_env(storage, module.id).await {
        tracing::warn!(module_id = module.id, error = %err, "Plane sync after module delete failed");
    }

    println!("Module '{}' deleted.", args.slug);
    Ok(())
}
