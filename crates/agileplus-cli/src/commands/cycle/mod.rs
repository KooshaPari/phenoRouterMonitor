//! `agileplus cycle` subcommand group.
//!
//! Provides create, list, show, add, remove, and transition operations for Cycles.
//! Traceability: FR-C01, FR-C02, FR-C03, FR-C04, FR-C05, FR-C07 / WP04-T019..T023

use anyhow::{Context, Result};

use agileplus_domain::domain::cycle::{Cycle, CycleState};
use agileplus_domain::ports::StoragePort;

mod add;
mod args;
mod create;
mod list;
mod remove;
mod show;
mod transition;

use add::cmd_add;
pub use args::{
    AddArgs, CreateArgs, CycleArgs, CycleCommand, ListArgs, RemoveArgs, ShowArgs, TransitionArgs,
};
use create::cmd_create;
use list::cmd_list;
use remove::cmd_remove;
use show::cmd_show;
use transition::cmd_transition;

/// Dispatch the `cycle` subcommand group.
pub async fn run<S: StoragePort>(args: CycleArgs, storage: &S) -> Result<()> {
    match args.command {
        CycleCommand::Create(a) => cmd_create(a, storage).await,
        CycleCommand::List(a) => cmd_list(a, storage).await,
        CycleCommand::Show(a) => cmd_show(a, storage).await,
        CycleCommand::Add(a) => cmd_add(a, storage).await,
        CycleCommand::Remove(a) => cmd_remove(a, storage).await,
        CycleCommand::Transition(a) => cmd_transition(a, storage).await,
    }
}

/// Find a cycle by its name, scanning all states.
pub(super) async fn find_cycle_by_name<S: StoragePort>(name: &str, storage: &S) -> Result<Cycle> {
    let all = storage.list_all_cycles().await.context("listing cycles")?;
    all.into_iter()
        .find(|c| c.name == name)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Cycle '{}' not found. Create it with `agileplus cycle create --name {} --start YYYY-MM-DD --end YYYY-MM-DD`.",
                name,
                name
            )
        })
}

/// Resolve module slug for display output.
pub(super) async fn find_module_slug<S: StoragePort>(
    storage: &S,
    module_id: i64,
) -> Option<String> {
    storage
        .get_module(module_id)
        .await
        .ok()
        .flatten()
        .map(|module| module.slug)
}

/// Return a human-readable prior-state string for the transition output line.
/// After `cycle.transition(target)`, cycle.state is already `target`.
/// We derive the prior state from the allowed graph (best effort for display only).
pub(super) fn prior_state_label(target: CycleState, cycle: &Cycle) -> String {
    // cycle.state was already mutated to `target` in the Cycle::transition call above.
    // We use the `updated_at` timestamp; since we can't recover the old state from
    // the mutable reference easily, we just report the target as the "now" value and
    // indicate the transition with a generic "previous" label.
    let _ = cycle;
    let prior = match target {
        CycleState::Active => "Draft or Review",
        CycleState::Draft => "Active",
        CycleState::Review => "Active",
        CycleState::Shipped => "Review",
        CycleState::Archived => "Shipped",
    };
    prior.to_string()
}

#[cfg(test)]
mod tests;
