//! cache command implementation - Cache management via Stashly

use anyhow::Result;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum CacheCommands {
    /// Get item
    Get { key: String },
    /// Set item
    Set { key: String, value: String },
    /// Clear cache
    Clear,
}

pub async fn handle(cmd: CacheCommands) -> Result<()> {
    match cmd {
        CacheCommands::Get { key } => {
            println!("Getting cache key: {}", key);
            Ok(())
        }
        CacheCommands::Set { key, value } => {
            println!("Setting cache key {} = {}", key, value);
            Ok(())
        }
        CacheCommands::Clear => {
            println!("Clearing cache via Stashly...");
            Ok(())
        }
    }
}
