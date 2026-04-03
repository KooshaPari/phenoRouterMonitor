use clap::{Args, Subcommand};

/// Manage modules (product-area groupings).
#[derive(Debug, Args)]
pub struct ModuleArgs {
    #[command(subcommand)]
    pub command: ModuleCommand,
}

/// Available sub-subcommands for `agileplus module`.
#[derive(Debug, Subcommand)]
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
#[derive(Debug, Args)]
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
#[derive(Debug, Args)]
pub struct ListArgs {
    /// Show modules as a recursive ASCII tree instead of a flat list.
    #[arg(long)]
    pub tree: bool,
}

/// Arguments for `agileplus module show`.
#[derive(Debug, Args)]
pub struct ShowArgs {
    /// Slug of the module to display.
    pub slug: String,
}

/// Arguments for `agileplus module assign`.
#[derive(Debug, Args)]
pub struct AssignArgs {
    /// Slug of the module to assign the feature to.
    #[arg(long)]
    pub module: String,

    /// Slug of the feature to assign.
    #[arg(long)]
    pub feature: String,
}

/// Arguments for `agileplus module tag`.
#[derive(Debug, Args)]
pub struct TagArgs {
    /// Slug of the module.
    #[arg(long)]
    pub module: String,

    /// Slug of the feature to tag.
    #[arg(long)]
    pub feature: String,
}

/// Arguments for `agileplus module untag`.
#[derive(Debug, Args)]
pub struct UntagArgs {
    /// Slug of the module.
    #[arg(long)]
    pub module: String,

    /// Slug of the feature to remove the tag from.
    #[arg(long)]
    pub feature: String,
}

/// Arguments for `agileplus module delete`.
#[derive(Debug, Args)]
pub struct DeleteArgs {
    /// Slug of the module to delete.
    pub slug: String,
}
