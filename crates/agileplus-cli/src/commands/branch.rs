//! `agileplus branch` command group.
//!
//! Provides branch create, checkout, delete, list, and sync operations.

use anyhow::{Context, Result};

use agileplus_domain::ports::{BranchInfo, VcsPort};

#[derive(Debug, clap::Args)]
pub struct BranchArgs {
    #[command(subcommand)]
    pub command: BranchCommand,
}

#[derive(Debug, clap::Subcommand)]
pub enum BranchCommand {
    /// Create a new branch from a base ref.
    Create {
        /// Branch name to create.
        #[arg(long)]
        name: String,
        /// Base ref to branch from.
        #[arg(long, default_value = "main")]
        base: String,
    },
    /// Check out an existing local branch.
    Checkout {
        /// Branch name to check out.
        #[arg(long)]
        name: String,
    },
    /// Delete a branch locally or remotely.
    Delete {
        /// Branch name to delete.
        #[arg(long)]
        name: String,
        /// Force deletion even if not merged.
        #[arg(long)]
        force: bool,
        /// Remote name to delete from (for example, origin).
        #[arg(long)]
        remote: Option<String>,
    },
    /// List branches, optionally filtering by pattern.
    List {
        /// Shell-style pattern (for example, feat/*).
        #[arg(long)]
        pattern: Option<String>,
        /// List remote branches instead of local branches.
        #[arg(long)]
        remote: bool,
        /// Output format: table (default) or json.
        #[arg(long, default_value = "table")]
        output: String,
    },
    /// Sync one branch into another using the normal merge engine.
    Sync {
        /// Source branch to merge from.
        #[arg(long, default_value = "main")]
        source: String,
        /// Target branch to merge into.
        #[arg(long, default_value = "canary")]
        target: String,
        /// Output format: table (default) or json.
        #[arg(long, default_value = "table")]
        output: String,
    },
}

pub async fn run<V: VcsPort>(args: BranchArgs, vcs: &V) -> Result<()> {
    match args.command {
        BranchCommand::Create { name, base } => {
            vcs.create_branch(&name, &base)
                .await
                .with_context(|| format!("creating branch '{name}' from '{base}'"))?;
            println!("Created branch {name} from {base}");
        }
        BranchCommand::Checkout { name } => {
            vcs.checkout_branch(&name)
                .await
                .with_context(|| format!("checking out branch '{name}'"))?;
            println!("Checked out branch {name}");
        }
        BranchCommand::Delete {
            name,
            force,
            remote,
        } => {
            vcs.delete_branch(&name, force, remote.as_deref())
                .await
                .with_context(|| format!("deleting branch '{name}'"))?;
            if let Some(remote) = remote {
                println!("Deleted remote branch {remote}/{name}");
            } else {
                println!("Deleted branch {name}");
            }
        }
        BranchCommand::List {
            pattern,
            remote,
            output,
        } => {
            let branches = vcs
                .list_branches(pattern.as_deref(), remote)
                .await
                .context("listing branches")?;
            print_branches(&branches, &output)?;
        }
        BranchCommand::Sync {
            source,
            target,
            output,
        } => {
            let result = vcs
                .merge_to_target(&source, &target)
                .await
                .with_context(|| format!("syncing '{source}' into '{target}'"))?;
            print_sync_result(
                &source,
                &target,
                result.success,
                &output,
                result.merged_commit,
            )?;
        }
    }

    Ok(())
}

fn print_branches(branches: &[BranchInfo], output: &str) -> Result<()> {
    if output == "json" {
        println!("{}", serde_json::to_string_pretty(branches)?);
        return Ok(());
    }

    if branches.is_empty() {
        println!("No branches found");
        return Ok(());
    }

    for branch in branches {
        let remote = if branch.is_remote { "remote" } else { "local" };
        println!("{:<8} {}", remote, branch.name);
    }

    Ok(())
}

fn print_sync_result(
    source: &str,
    target: &str,
    success: bool,
    output: &str,
    merged_commit: Option<String>,
) -> Result<()> {
    if output == "json" {
        let payload = serde_json::json!({
            "source": source,
            "target": target,
            "success": success,
            "merged_commit": merged_commit,
        });
        println!("{}", serde_json::to_string_pretty(&payload)?);
        return Ok(());
    }

    if success {
        if let Some(commit) = merged_commit {
            println!("Synced {source} -> {target} at {commit}");
        } else {
            println!("Synced {source} -> {target}");
        }
    } else {
        println!("Sync {source} -> {target} reported conflicts");
    }

    Ok(())
}
