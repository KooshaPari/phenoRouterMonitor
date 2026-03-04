//! `agileplus module` command group implementation.
//!
//! Provides CRUD and association management for Module entities.
//! Traces to: FR-M01, FR-M02, FR-M04, FR-M07 / WP03-T014..T018

use anyhow::{anyhow, Context, Result};

use agileplus_domain::domain::module::{Module, ModuleFeatureTag};
use agileplus_domain::error::DomainError;
use agileplus_domain::ports::StoragePort;

// ---------------------------------------------------------------------------
// Clap arg structs
// ---------------------------------------------------------------------------

/// Manage modules (product-area groupings).
#[derive(Debug, clap::Args)]
pub struct ModuleArgs {
    #[command(subcommand)]
    pub command: ModuleCommand,
}

/// Available sub-subcommands for `agileplus module`.
#[derive(Debug, clap::Subcommand)]
pub enum ModuleCommand {
    /// Create a new module.
    Create(CreateArgs),
    /// List all modules.
    List(ListArgs),
    /// Show details for a single module.
    Show(ShowArgs),
    /// Assign a feature to a module (sets primary ownership via tag).
    Assign(AssignArgs),
    /// Tag a feature to a module (many-to-many).
    Tag(TagArgs),
    /// Remove a tag between a feature and a module.
    Untag(UntagArgs),
    /// Delete a module (fails if it has children or owned features).
    Delete(DeleteArgs),
}

/// Arguments for `agileplus module create`.
#[derive(Debug, clap::Args)]
pub struct CreateArgs {
    /// Human-readable name for the module (slug is derived automatically).
    #[arg(long)]
    pub name: String,

    /// Optional description.
    #[arg(long)]
    pub description: Option<String>,

    /// Slug of the parent module (omit for a root module).
    #[arg(long)]
    pub parent: Option<String>,
}

/// Arguments for `agileplus module list`.
#[derive(Debug, clap::Args)]
pub struct ListArgs {
    /// Show modules as a recursive ASCII tree instead of a flat list.
    #[arg(long)]
    pub tree: bool,
}

/// Arguments for `agileplus module show`.
#[derive(Debug, clap::Args)]
pub struct ShowArgs {
    /// Slug of the module to display.
    pub slug: String,
}

/// Arguments for `agileplus module assign`.
#[derive(Debug, clap::Args)]
pub struct AssignArgs {
    /// Slug of the module to assign the feature to.
    #[arg(long)]
    pub module: String,

    /// Slug of the feature to assign.
    #[arg(long)]
    pub feature: String,
}

/// Arguments for `agileplus module tag`.
#[derive(Debug, clap::Args)]
pub struct TagArgs {
    /// Slug of the module.
    #[arg(long)]
    pub module: String,

    /// Slug of the feature to tag.
    #[arg(long)]
    pub feature: String,
}

/// Arguments for `agileplus module untag`.
#[derive(Debug, clap::Args)]
pub struct UntagArgs {
    /// Slug of the module.
    #[arg(long)]
    pub module: String,

    /// Slug of the feature to remove the tag from.
    #[arg(long)]
    pub feature: String,
}

/// Arguments for `agileplus module delete`.
#[derive(Debug, clap::Args)]
pub struct DeleteArgs {
    /// Slug of the module to delete.
    pub slug: String,
}

// ---------------------------------------------------------------------------
// Dispatch
// ---------------------------------------------------------------------------

/// Entry point for the `module` subcommand group.
pub async fn run<S: StoragePort>(args: ModuleArgs, storage: &S) -> Result<()> {
    match args.command {
        ModuleCommand::Create(a) => run_create(a, storage).await,
        ModuleCommand::List(a) => run_list(a, storage).await,
        ModuleCommand::Show(a) => run_show(a, storage).await,
        ModuleCommand::Assign(a) => run_assign(a, storage).await,
        ModuleCommand::Tag(a) => run_tag(a, storage).await,
        ModuleCommand::Untag(a) => run_untag(a, storage).await,
        ModuleCommand::Delete(a) => run_delete(a, storage).await,
    }
}

// ---------------------------------------------------------------------------
// T015: create & delete
// ---------------------------------------------------------------------------

async fn run_create<S: StoragePort>(args: CreateArgs, storage: &S) -> Result<()> {
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

    println!("Module '{}' created (id={}, slug={}).", module.friendly_name, id, module.slug);
    if let Some(pid) = parent_module_id {
        println!("  Parent module id: {pid}");
    }
    Ok(())
}

async fn run_delete<S: StoragePort>(args: DeleteArgs, storage: &S) -> Result<()> {
    let module = storage
        .get_module_by_slug(&args.slug)
        .await
        .context("looking up module")?
        .ok_or_else(|| anyhow!("module '{}' not found", args.slug))?;

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

    println!("Module '{}' deleted.", args.slug);
    Ok(())
}

// ---------------------------------------------------------------------------
// T016: list & show
// ---------------------------------------------------------------------------

async fn run_list<S: StoragePort>(args: ListArgs, storage: &S) -> Result<()> {
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
        println!("{indent}{connector}{} (slug: {})", module.friendly_name, module.slug);

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
            println!("  {} (slug: {}, parent_id: {})", child.friendly_name, child.slug, parent_id);
            print_children_flat(storage, child.id).await?;
        }
        Ok(())
    })
}

async fn run_show<S: StoragePort>(args: ShowArgs, storage: &S) -> Result<()> {
    let module = storage
        .get_module_by_slug(&args.slug)
        .await
        .context("looking up module")?
        .ok_or_else(|| anyhow!("module '{}' not found", args.slug))?;

    let details = storage
        .get_module_with_features(module.id)
        .await
        .context("loading module details")?
        .ok_or_else(|| anyhow!("module '{}' disappeared during load", args.slug))?;

    println!("Module: {} (slug: {})", details.module.friendly_name, details.module.slug);
    if let Some(ref desc) = details.module.description {
        println!("  Description: {desc}");
    }
    if let Some(pid) = details.module.parent_module_id {
        println!("  Parent module id: {pid}");
    }
    println!("  Created: {}", details.module.created_at.format("%Y-%m-%d %H:%M UTC"));
    println!("  Updated: {}", details.module.updated_at.format("%Y-%m-%d %H:%M UTC"));

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

// ---------------------------------------------------------------------------
// T017: assign, tag, untag
// ---------------------------------------------------------------------------

async fn run_assign<S: StoragePort>(args: AssignArgs, storage: &S) -> Result<()> {
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

    println!(
        "Feature '{}' assigned to module '{}' (recorded as ownership tag).",
        args.feature, args.module
    );
    Ok(())
}

async fn run_tag<S: StoragePort>(args: TagArgs, storage: &S) -> Result<()> {
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

    println!("Feature '{}' tagged to module '{}'.", args.feature, args.module);
    Ok(())
}

async fn run_untag<S: StoragePort>(args: UntagArgs, storage: &S) -> Result<()> {
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

    println!("Feature '{}' untagged from module '{}'.", args.feature, args.module);
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests (T018 unit tests for clap parsing)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    /// Wrap ModuleArgs so we can parse it from a top-level binary name.
    #[derive(Debug, clap::Parser)]
    struct TestCli {
        #[command(subcommand)]
        command: ModuleCommand,
    }

    fn parse(args: &[&str]) -> ModuleCommand {
        TestCli::parse_from(args).command
    }

    #[test]
    fn parse_create_minimal() {
        let cmd = parse(&["cli", "create", "--name", "Auth"]);
        match cmd {
            ModuleCommand::Create(a) => {
                assert_eq!(a.name, "Auth");
                assert!(a.description.is_none());
                assert!(a.parent.is_none());
            }
            _ => panic!("expected Create"),
        }
    }

    #[test]
    fn parse_create_full() {
        let cmd = parse(&[
            "cli", "create", "--name", "Auth", "--description", "Authentication module",
            "--parent", "platform",
        ]);
        match cmd {
            ModuleCommand::Create(a) => {
                assert_eq!(a.name, "Auth");
                assert_eq!(a.description.as_deref(), Some("Authentication module"));
                assert_eq!(a.parent.as_deref(), Some("platform"));
            }
            _ => panic!("expected Create"),
        }
    }

    #[test]
    fn parse_list_flat() {
        let cmd = parse(&["cli", "list"]);
        match cmd {
            ModuleCommand::List(a) => assert!(!a.tree),
            _ => panic!("expected List"),
        }
    }

    #[test]
    fn parse_list_tree() {
        let cmd = parse(&["cli", "list", "--tree"]);
        match cmd {
            ModuleCommand::List(a) => assert!(a.tree),
            _ => panic!("expected List"),
        }
    }

    #[test]
    fn parse_show() {
        let cmd = parse(&["cli", "show", "my-module"]);
        match cmd {
            ModuleCommand::Show(a) => assert_eq!(a.slug, "my-module"),
            _ => panic!("expected Show"),
        }
    }

    #[test]
    fn parse_assign() {
        let cmd = parse(&["cli", "assign", "--module", "platform", "--feature", "auth"]);
        match cmd {
            ModuleCommand::Assign(a) => {
                assert_eq!(a.module, "platform");
                assert_eq!(a.feature, "auth");
            }
            _ => panic!("expected Assign"),
        }
    }

    #[test]
    fn parse_tag() {
        let cmd = parse(&["cli", "tag", "--module", "platform", "--feature", "auth"]);
        match cmd {
            ModuleCommand::Tag(a) => {
                assert_eq!(a.module, "platform");
                assert_eq!(a.feature, "auth");
            }
            _ => panic!("expected Tag"),
        }
    }

    #[test]
    fn parse_untag() {
        let cmd = parse(&["cli", "untag", "--module", "platform", "--feature", "auth"]);
        match cmd {
            ModuleCommand::Untag(a) => {
                assert_eq!(a.module, "platform");
                assert_eq!(a.feature, "auth");
            }
            _ => panic!("expected Untag"),
        }
    }

    #[test]
    fn parse_delete() {
        let cmd = parse(&["cli", "delete", "old-module"]);
        match cmd {
            ModuleCommand::Delete(a) => assert_eq!(a.slug, "old-module"),
            _ => panic!("expected Delete"),
        }
    }
}
