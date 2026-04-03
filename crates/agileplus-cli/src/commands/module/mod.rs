//! `agileplus module` command group implementation.
//!
//! Provides CRUD and association management for Module entities.
//! Traces to: FR-M01, FR-M02, FR-M04, FR-M07 / WP03-T014..T018

use anyhow::Result;

use agileplus_domain::ports::StoragePort;

pub mod args;
pub mod assign;
pub mod create;
pub mod delete;
pub mod list;
pub mod show;
pub mod tag;
#[cfg(test)]
mod tests;
pub mod untag;

pub use args::{
    AssignArgs, CreateArgs, DeleteArgs, ListArgs, ModuleArgs, ModuleCommand, ShowArgs, TagArgs,
    UntagArgs,
};
use assign::run_assign;
use create::run_create;
use delete::run_delete;
use list::run_list;
use show::run_show;
use tag::run_tag;
use untag::run_untag;

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
