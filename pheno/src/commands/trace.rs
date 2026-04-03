//! trace command implementation - Observability via Tracely

use anyhow::Result;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum TraceCommands {
    /// Start trace
    Start { name: String },
    /// End trace
    End { id: String },
    /// Show status
    Status,
}

pub async fn handle(cmd: TraceCommands) -> Result<()> {
    match cmd {
        TraceCommands::Start { name } => {
            println!("Starting trace: {}", name);
            Ok(())
        }
        TraceCommands::End { id } => {
            println!("Ending trace: {}", id);
            Ok(())
        }
        TraceCommands::Status => {
            println!("Trace status via Tracely...");
            Ok(())
        }
    }
}
