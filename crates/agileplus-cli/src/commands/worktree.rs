//! `agileplus worktree` command group.
//!
//! Provides worktree add, remove, and list operations.

use std::path::PathBuf;

use anyhow::{Context, Result, bail};

use agileplus_domain::ports::{VcsPort, WorktreeInfo};

#[derive(Debug, clap::Args)]
pub struct WorktreeArgs {
    #[command(subcommand)]
    pub command: WorktreeCommand,
}

#[derive(Debug, clap::Subcommand)]
pub enum WorktreeCommand {
    /// Create a new worktree for a feature/WP pair.
    Add {
        /// Feature slug.
        #[arg(long)]
        feature_slug: String,
        /// Work package ID.
        #[arg(long)]
        wp_id: String,
        /// Optional expected path for the new worktree.
        #[arg(long)]
        path: Option<PathBuf>,
    },
    /// Delete a worktree.
    Remove {
        /// Worktree path.
        #[arg(long)]
        path: PathBuf,
    },
    /// List active worktrees.
    List {
        /// Output format: table (default) or json.
        #[arg(long, default_value = "table")]
        output: String,
    },
}

pub async fn run<V: VcsPort>(args: WorktreeArgs, vcs: &V) -> Result<()> {
    match args.command {
        WorktreeCommand::Add {
            feature_slug,
            wp_id,
            path,
        } => {
            let created = vcs
                .create_worktree(&feature_slug, &wp_id)
                .await
                .with_context(|| format!("creating worktree for {feature_slug}/{wp_id}"))?;
            if let Some(expected) = path {
                let expected = expected.canonicalize().unwrap_or_else(|_| expected.clone());
                if expected != created {
                    bail!(
                        "created worktree at '{}' but caller expected '{}'",
                        created.display(),
                        expected.display()
                    );
                }
            }
            println!("Created worktree at {}", created.display());
        }
        WorktreeCommand::Remove { path } => {
            vcs.cleanup_worktree(&path)
                .await
                .with_context(|| format!("removing worktree '{}'", path.display()))?;
            println!("Removed worktree {}", path.display());
        }
        WorktreeCommand::List { output } => {
            let worktrees = vcs.list_worktrees().await.context("listing worktrees")?;
            print_worktrees(&worktrees, &output)?;
        }
    }

    Ok(())
}

fn print_worktrees(worktrees: &[WorktreeInfo], output: &str) -> Result<()> {
    if output == "json" {
        println!("{}", serde_json::to_string_pretty(worktrees)?);
        return Ok(());
    }

    if worktrees.is_empty() {
        println!("No worktrees found");
        return Ok(());
    }

    for worktree in worktrees {
        println!(
            "{}  {}  {}",
            worktree.branch,
            worktree.feature_slug,
            worktree.path.display()
        );
    }

    Ok(())
}
