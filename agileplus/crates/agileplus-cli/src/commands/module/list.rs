use anyhow::{Context, Result};

use agileplus_domain::domain::module::Module;
use agileplus_domain::ports::StoragePort;

use super::ListArgs;

// ---------------------------------------------------------------------------
// T016: list
// ---------------------------------------------------------------------------

pub async fn run_list<S: StoragePort>(args: ListArgs, storage: &S) -> Result<()> {
    let roots = storage
        .list_root_modules()
        .await
        .context("listing root modules")?;

    if roots.is_empty() {
        println!("No modules found. Create one with `agileplus module create --name <name>`.");
        return Ok(());
    }

    if args.tree {
        // Recursive ASCII tree
        for root in &roots {
            print_module_tree(storage, root, 0).await?;
        }
    } else {
        // Flat list
        for m in &roots {
            println!("{} (slug: {})", m.friendly_name, m.slug);
        }
        // Also list all children by enumerating recursively in flat mode
        for root in &roots {
            print_children_flat(storage, root.id).await?;
        }
    }

    Ok(())
}

/// Recursively print a module tree with ASCII connectors.
fn print_module_tree<'a, S: StoragePort>(
    storage: &'a S,
    module: &'a Module,
    depth: usize,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
    Box::pin(async move {
        let indent = "  ".repeat(depth);
        let connector = if depth == 0 { "" } else { "+-- " };
        println!(
            "{indent}{connector}{} (slug: {})",
            module.friendly_name, module.slug
        );

        let children = storage
            .list_child_modules(module.id)
            .await
            .context("listing child modules")?;

        for child in &children {
            print_module_tree(storage, child, depth + 1).await?;
        }
        Ok(())
    })
}

/// Print children of `parent_id` in flat mode (recursive).
fn print_children_flat<'a, S: StoragePort>(
    storage: &'a S,
    parent_id: i64,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
    Box::pin(async move {
        let children = storage
            .list_child_modules(parent_id)
            .await
            .context("listing child modules")?;

        for child in &children {
            println!(
                "  {} (slug: {}, parent_id: {})",
                child.friendly_name, child.slug, parent_id
            );
            print_children_flat(storage, child.id).await?;
        }
        Ok(())
    })
}
