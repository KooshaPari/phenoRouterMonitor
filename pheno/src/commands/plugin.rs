//! plugin command implementation - Plugin management via PhenoPlugins

use anyhow::Result;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum PluginCommands {
    /// List plugins
    List,
    /// Load plugin
    Load { name: String },
    /// Unload plugin
    Unload { name: String },
}

pub async fn handle(cmd: PluginCommands) -> Result<()> {
    match cmd {
        PluginCommands::List => {
            println!("Listing plugins via PhenoPlugins...");
            Ok(())
        }
        PluginCommands::Load { name } => {
            println!("Loading plugin: {}", name);
            Ok(())
        }
        PluginCommands::Unload { name } => {
            println!("Unloading plugin: {}", name);
            Ok(())
        }
    }
}
