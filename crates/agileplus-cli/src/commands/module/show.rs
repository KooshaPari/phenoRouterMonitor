use anyhow::{Context, Result};

use agileplus_domain::ports::StoragePort;

use super::ShowArgs;

// ---------------------------------------------------------------------------
// T016: show
// ---------------------------------------------------------------------------

pub async fn run_show<S: StoragePort>(args: ShowArgs, storage: &S) -> Result<()> {
    let module = storage
        .get_module_by_slug(&args.slug)
        .await
        .context("looking up module")?
        .ok_or_else(|| anyhow::anyhow!("module '{}' not found", args.slug))?;

    let details = storage
        .get_module_with_features(module.id)
        .await
        .context("loading module details")?
        .ok_or_else(|| anyhow::anyhow!("module '{}' disappeared during load", args.slug))?;

    println!(
        "Module: {} (slug: {})",
        details.module.friendly_name, details.module.slug
    );
    if let Some(ref desc) = details.module.description {
        println!("  Description: {desc}");
    }
    if let Some(pid) = details.module.parent_module_id {
        println!("  Parent module id: {pid}");
    }
    println!(
        "  Created: {}",
        details.module.created_at.format("%Y-%m-%d %H:%M UTC")
    );
    println!(
        "  Updated: {}",
        details.module.updated_at.format("%Y-%m-%d %H:%M UTC")
    );

    println!();
    println!("Owned features ({}):", details.owned_features.len());
    if details.owned_features.is_empty() {
        println!("  (none)");
    } else {
        for f in &details.owned_features {
            println!("  - {} (slug: {})", f.friendly_name, f.slug);
        }
    }

    println!();
    println!("Tagged features ({}):", details.tagged_features.len());
    if details.tagged_features.is_empty() {
        println!("  (none)");
    } else {
        for f in &details.tagged_features {
            println!("  - {} (slug: {})", f.friendly_name, f.slug);
        }
    }

    println!();
    println!("Child modules ({}):", details.child_modules.len());
    if details.child_modules.is_empty() {
        println!("  (none)");
    } else {
        for c in &details.child_modules {
            println!("  +-- {} (slug: {})", c.friendly_name, c.slug);
        }
    }

    Ok(())
}
