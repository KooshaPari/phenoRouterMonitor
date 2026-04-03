//! pheno - Unified CLI for the Phenotype ecosystem
//!
//! Orchestrates all Phenotype domain registries:
//! - PhenoProc: Process management
//! - PhenoVCS: Version control
//! - PhenoPlugins: Plugin system
//! - Tracely: Observability
//! - Stashly: Caching
//! - HexaKit: Templates

use anyhow::Result;
use clap::{Parser, Subcommand};

mod commands;

#[derive(Parser)]
#[command(name = "pheno")]
#[command(about = "Unified CLI for the Phenotype ecosystem")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Process management via PhenoProc
    #[command(alias = "p")]
    Proc {
        #[command(subcommand)]
        cmd: commands::proc::ProcCommands,
    },
    /// Version control via PhenoVCS
    #[command(alias = "v")]
    Vcs {
        #[command(subcommand)]
        cmd: commands::vcs::VcsCommands,
    },
    /// Plugin management via PhenoPlugins
    #[command(alias = "pl")]
    Plugin {
        #[command(subcommand)]
        cmd: commands::plugin::PluginCommands,
    },
    /// Observability via Tracely
    #[command(alias = "t")]
    Trace {
        #[command(subcommand)]
        cmd: commands::trace::TraceCommands,
    },
    /// Cache management via Stashly
    #[command(alias = "c")]
    Cache {
        #[command(subcommand)]
        cmd: commands::cache::CacheCommands,
    },
    /// Registry management
    #[command(alias = "r")]
    Registry {
        #[command(subcommand)]
        cmd: commands::registry::RegistryCommands,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Proc { cmd } => commands::proc::handle(cmd).await,
        Commands::Vcs { cmd } => commands::vcs::handle(cmd).await,
        Commands::Plugin { cmd } => commands::plugin::handle(cmd).await,
        Commands::Trace { cmd } => commands::trace::handle(cmd).await,
        Commands::Cache { cmd } => commands::cache::handle(cmd).await,
        Commands::Registry { cmd } => commands::registry::handle(cmd).await,
    }
}
