use anyhow::{Context, Result, anyhow};
use chrono::NaiveDate;

use agileplus_domain::domain::cycle::Cycle;
use agileplus_domain::ports::StoragePort;
use agileplus_plane::maybe_sync_cycle_from_env;

use crate::commands::cycle::args::CreateArgs;

// ---------------------------------------------------------------------------
// T020: create + list
// ---------------------------------------------------------------------------

pub(super) async fn cmd_create<S: StoragePort>(args: CreateArgs, storage: &S) -> Result<()> {
    let start_date = NaiveDate::parse_from_str(&args.start, "%Y-%m-%d")
        .with_context(|| format!("invalid start date '{}'; expected YYYY-MM-DD", args.start))?;
    let end_date = NaiveDate::parse_from_str(&args.end, "%Y-%m-%d")
        .with_context(|| format!("invalid end date '{}'; expected YYYY-MM-DD", args.end))?;

    // Resolve optional module scope
    let module_scope_id = if let Some(ref module_slug) = args.module {
        let m = storage
            .get_module_by_slug(module_slug)
            .await
            .context("looking up module by slug")?
            .ok_or_else(|| anyhow!("Module '{}' not found.", module_slug))?;
        Some(m.id)
    } else {
        None
    };

    let mut cycle = Cycle::new(&args.name, start_date, end_date, module_scope_id)
        .map_err(|e| anyhow!("{}", e))?;
    if let Some(desc) = args.description {
        cycle.description = Some(desc);
    }

    let id = storage
        .create_cycle(&cycle)
        .await
        .context("creating cycle")?;

    if let Err(err) = maybe_sync_cycle_from_env(storage, id).await {
        tracing::warn!(cycle_id = id, error = %err, "Plane sync after cycle create failed");
    }

    println!("Cycle '{}' created (id={}).", cycle.name, id);
    println!("  State:      Draft");
    println!("  Start:      {}", cycle.start_date);
    println!("  End:        {}", cycle.end_date);
    if let Some(mid) = cycle.module_scope_id {
        println!("  Module id:  {}", mid);
    }

    Ok(())
}
