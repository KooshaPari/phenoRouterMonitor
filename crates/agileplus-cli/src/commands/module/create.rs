use anyhow::{Context, Result, anyhow};

use agileplus_domain::domain::module::Module;
use agileplus_domain::ports::StoragePort;
use agileplus_plane::maybe_sync_module_from_env;

use super::CreateArgs;

// ---------------------------------------------------------------------------
// T015: create
// ---------------------------------------------------------------------------

pub async fn run_create<S: StoragePort>(args: CreateArgs, storage: &S) -> Result<()> {
    // Resolve optional parent slug -> parent_module_id
    let parent_module_id: Option<i64> = match &args.parent {
        None => None,
        Some(parent_slug) => {
            let m = storage
                .get_module_by_slug(parent_slug)
                .await
                .context("looking up parent module")?
                .ok_or_else(|| {
                    anyhow!(
                        "parent module '{}' not found -- create it first with `agileplus module create --name <name>`",
                        parent_slug
                    )
                })?;
            Some(m.id)
        }
    };

    let mut module = Module::new(&args.name, parent_module_id);
    if let Some(desc) = args.description {
        module.description = Some(desc);
    }

    let id = storage
        .create_module(&module)
        .await
        .context("persisting new module")?;

    if let Err(err) = maybe_sync_module_from_env(storage, id).await {
        tracing::warn!(module_id = id, error = %err, "Plane sync after module create failed");
    }

    println!(
        "Module '{}' created (id={}, slug={}).",
        module.friendly_name, id, module.slug
    );
    if let Some(pid) = parent_module_id {
        println!("  Parent module id: {pid}");
    }
    Ok(())
}
