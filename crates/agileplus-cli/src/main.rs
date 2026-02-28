//! AgilePlus CLI entry point.
//!
//! Parses CLI arguments, initialises adapters, and routes to command handlers.
//! Traceability: WP11-T060, T065

use std::path::PathBuf;
use std::process;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

use agileplus_cli::commands::{research::ResearchArgs, specify::SpecifyArgs};
use agileplus_git::GitVcsAdapter;
use agileplus_sqlite::SqliteStorageAdapter;

/// Spec-driven development engine.
#[derive(Parser)]
#[command(name = "agileplus", version, about = "Spec-driven development engine")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Increase verbosity (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    verbose: u8,

    /// Path to SQLite database
    #[arg(long, global = true, default_value = ".agileplus/agileplus.db")]
    db: PathBuf,

    /// Path to git repository root (defaults to current directory)
    #[arg(long, global = true)]
    repo: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Create or revise a feature specification.
    Specify(SpecifyArgs),
    /// Research a feature (pre-specify codebase scan or post-specify feasibility).
    Research(ResearchArgs),
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Configure logging based on verbosity
    let log_level = match cli.verbose {
        0 => tracing::Level::INFO,
        1 => tracing::Level::DEBUG,
        _ => tracing::Level::TRACE,
    };
    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_target(false)
        .compact()
        .init();

    if let Err(e) = run(cli).await {
        eprintln!("Error: {e:#}");
        process::exit(1);
    }
}

async fn run(cli: Cli) -> Result<()> {
    // Initialise storage adapter (create DB directory if needed)
    if let Some(parent) = cli.db.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("creating directory {}", parent.display()))?;
        }
    }

    let storage = SqliteStorageAdapter::new(&cli.db)
        .with_context(|| format!("opening database at {}", cli.db.display()))?;

    // Initialise VCS adapter
    let vcs = match cli.repo {
        Some(ref path) => GitVcsAdapter::new(path.clone())
            .context("opening git repository at specified path")?,
        None => GitVcsAdapter::from_current_dir()
            .context("Not inside a git repository. Run agileplus from your project root.")?,
    };

    match cli.command {
        Commands::Specify(args) => {
            agileplus_cli::commands::specify::run_specify(args, &storage, &vcs).await?;
        }
        Commands::Research(args) => {
            agileplus_cli::commands::research::run_research(args, &storage, &vcs).await?;
        }
    }

    Ok(())
}
