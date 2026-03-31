use anyhow::{Context, Result};

use crate::commands::cycle::args::ShowArgs;
use crate::commands::cycle::{find_cycle_by_name, find_module_slug};
use agileplus_domain::ports::StoragePort;

// ---------------------------------------------------------------------------
// T021: show
// ---------------------------------------------------------------------------

#[allow(unknown_lints, clippy::manual_checked_ops)]
pub(super) async fn cmd_show<S: StoragePort>(args: ShowArgs, storage: &S) -> Result<()> {
    let cycle = find_cycle_by_name(&args.name, storage).await?;

    let cwf = storage
        .get_cycle_with_features(cycle.id)
        .await
        .context("loading cycle with features")?
        .ok_or_else(|| anyhow::anyhow!("Cycle '{}' disappeared unexpectedly.", args.name))?;

    let days = (cwf.cycle.end_date - cwf.cycle.start_date).num_days();

    println!("Cycle:        {}", cwf.cycle.name);
    if let Some(ref desc) = cwf.cycle.description {
        println!("Description:  {}", desc);
    }
    println!("State:        {}", cwf.cycle.state);
    println!("Start:        {}", cwf.cycle.start_date);
    println!("End:          {}", cwf.cycle.end_date);
    println!("Duration:     {} days", days);

    if let Some(mid) = cwf.cycle.module_scope_id {
        let scope_label = find_module_slug(storage, mid)
            .await
            .unwrap_or_else(|| format!("id:{}", mid));
        println!("Module scope: {}", scope_label);
    }

    // WP progress
    let wp = &cwf.wp_progress;
    let total = wp.total;
    if total > 0 {
        let done_pct = wp.done * 100 / total;
        let in_progress_pct = wp.in_progress * 100 / total;
        let planned_pct = wp.planned * 100 / total;
        let blocked_pct = wp.blocked * 100 / total;
        println!();
        println!("WP Progress ({} total):", total);
        println!("  Done:        {} ({}%)", wp.done, done_pct);
        println!("  In Progress: {} ({}%)", wp.in_progress, in_progress_pct);
        println!("  Planned:     {} ({}%)", wp.planned, planned_pct);
        println!("  Blocked:     {} ({}%)", wp.blocked, blocked_pct);
    } else {
        println!();
        println!("WP Progress:  no work packages tracked");
    }

    // Features sorted by slug
    if cwf.features.is_empty() {
        println!();
        println!("Features:     (none assigned)");
    } else {
        let mut features = cwf.features.clone();
        features.sort_by(|a, b| a.slug.cmp(&b.slug));
        println!();
        println!("Features ({}):", features.len());
        for f in &features {
            println!("  {} -- {}", f.slug, f.state);
        }
    }

    Ok(())
}
