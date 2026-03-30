use clap::{Args, Subcommand};

// ---------------------------------------------------------------------------
// Clap structs -- T019
// ---------------------------------------------------------------------------

/// Manage cycles (time-boxed delivery units).
#[derive(Debug, Args)]
pub struct CycleArgs {
    #[command(subcommand)]
    pub command: CycleCommand,
}

#[derive(Debug, Subcommand)]
pub enum CycleCommand {
    /// Create a new cycle.
    Create(CreateArgs),
    /// List cycles, optionally filtered by state.
    List(ListArgs),
    /// Show full detail for a cycle.
    Show(ShowArgs),
    /// Add a feature to a cycle.
    Add(AddArgs),
    /// Remove a feature from a cycle.
    Remove(RemoveArgs),
    /// Transition a cycle to a new state.
    Transition(TransitionArgs),
}

/// Arguments for `cycle create`.
#[derive(Debug, Args)]
pub struct CreateArgs {
    /// Cycle name (must be unique).
    #[arg(long)]
    pub name: String,

    /// Start date in YYYY-MM-DD format.
    #[arg(long)]
    pub start: String,

    /// End date in YYYY-MM-DD format.
    #[arg(long)]
    pub end: String,

    /// Optional description.
    #[arg(long)]
    pub description: Option<String>,

    /// Scope this cycle to a module slug.
    #[arg(long)]
    pub module: Option<String>,
}

/// Arguments for `cycle list`.
#[derive(Debug, Args)]
pub struct ListArgs {
    /// Filter by state (Draft, Active, Review, Shipped, Archived).
    #[arg(long)]
    pub state: Option<String>,
}

/// Arguments for `cycle show`.
#[derive(Debug, Args)]
pub struct ShowArgs {
    /// Cycle name.
    pub name: String,
}

/// Arguments for `cycle add`.
#[derive(Debug, Args)]
pub struct AddArgs {
    /// Cycle name.
    #[arg(long)]
    pub cycle: String,

    /// Feature slug to add.
    #[arg(long)]
    pub feature: String,
}

/// Arguments for `cycle remove`.
#[derive(Debug, Args)]
pub struct RemoveArgs {
    /// Cycle name.
    #[arg(long)]
    pub cycle: String,

    /// Feature slug to remove.
    #[arg(long)]
    pub feature: String,
}

/// Arguments for `cycle transition`.
#[derive(Debug, Args)]
pub struct TransitionArgs {
    /// Cycle name.
    #[arg(long)]
    pub cycle: String,

    /// Target state (Draft, Active, Review, Shipped, Archived).
    #[arg(long)]
    pub to: String,
}
