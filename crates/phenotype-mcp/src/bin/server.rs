//! Phenotype MCP Server Binary
//!
//! Standalone MCP server built on FastMCP v3.0 GA.

use phenotype_mcp::PhenotypeMcpServer;
use tracing::{error, info};
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    info!("Starting Phenotype MCP Server v{}", env!("CARGO_PKG_VERSION"));
    info!("Built on FastMCP v3.0 GA");

    // Create and configure server
    let server = PhenotypeMcpServer::new();

    // Register tools
    if let Err(e) = server.register_tools().await {
        error!(error = %e, "Failed to register tools");
        return Err(e);
    }

    info!("MCP Server ready, listening on transport: {}", server.transport_url());

    // Run server
    if let Err(e) = server.run().await {
        error!(error = %e, "Server error");
        return Err(e);
    }

    Ok(())
}
