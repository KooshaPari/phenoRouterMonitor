use anyhow::{Context, Result, anyhow};

use agileplus_domain::ports::StoragePort;

use crate::commands::cycle::CycleState;
use crate::commands::cycle::args::ListArgs;
use crate::commands::cycle::find_module_slug;

pub(super) async fn cmd_list<S: StoragePort>(args: ListArgs, storage: &S) -> Result<()> {
    let cycles = if let Some(ref state_str) = args.state {
        let state = state_str
            .parse::<CycleState>()
            .map_err(|e| anyhow!("{}", e))?;
        storage
            .list_cycles_by_state(state)
            .await
            .context("listing cycles by state")?
    } else {
        storage
            .list_all_cycles()
            .await
            .context("listing all cycles")?
    };

    if cycles.is_empty() {
        println!("No cycles found.");
        return Ok(());
    }

    // Print table header
    println!(
        "{:<30}  {:<10}  {:<12}  {:<12}  {:<10}",
        "NAME", "STATE", "START", "END", "SCOPE"
    );
    println!("{}", "-".repeat(80));

    for c in &cycles {
        let scope = if let Some(mid) = c.module_scope_id {
            find_module_slug(storage, mid)
                .await
                .unwrap_or_else(|| format!("id:{}", mid))
        } else {
            "-".to_string()
        };
        println!(
            "{:<30}  {:<10}  {:<12}  {:<12}  {:<10}",
            c.name,
            c.state.to_string(),
            c.start_date.to_string(),
            c.end_date.to_string(),
            scope
        );
    }

    Ok(())
}
