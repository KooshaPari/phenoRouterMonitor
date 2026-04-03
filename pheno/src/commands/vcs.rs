//! vcs command implementation - Version control via PhenoVCS

use anyhow::Result;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum VcsCommands {
    /// List worktrees
    List,
    /// Create worktree
    Create { branch: String, path: String },
    /// Remove worktree
    Remove { path: String },
}

pub async fn handle(cmd: VcsCommands) -> Result<()> {
    match cmd {
        VcsCommands::List => {
            println!("Listing worktrees via PhenoVCS...");
            Ok(())
        }
        VcsCommands::Create { branch, path } => {
            println!("Creating worktree for {} at {}", branch, path);
            Ok(())
        }
        VcsCommands::Remove { path } => {
            println!("Removing worktree at {}", path);
            Ok(())
        }
    }
}
