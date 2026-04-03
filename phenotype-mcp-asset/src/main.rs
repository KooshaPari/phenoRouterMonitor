//! MCP Server for Asset Building and Discovery
//!
//! This server provides tools for:
//! - Asset discovery and indexing
//! - Pack manifest creation and validation
//! - Content building and compilation
//! - Dependency resolution

use rmcp::handler::server::ServerHandler;
use rmcp::model::{CallToolRequest, CallToolResponse, Tool};
use rmcp::schemars;
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{info, error};

mod tools;
use tools::*;

/// Asset MCP Server
#[derive(Debug)]
pub struct AssetMcpServer {
    packs_dir: PathBuf,
    tools: Vec<Tool>,
}

impl AssetMcpServer {
    pub fn new(packs_dir: PathBuf) -> Self {
        let tools = vec![
            discover_assets_tool(),
            index_assets_tool(),
            query_assets_tool(),
            create_pack_tool(),
            validate_pack_tool(),
            build_pack_tool(),
        ];
        
        Self {
            packs_dir,
            tools,
        }
    }
}

#[rmcp::tool]
impl ServerHandler for AssetMcpServer {
    fn get_tools(&self) -> Vec<Tool> {
        self.tools.clone()
    }

    async fn call_tool(
        &self,
        request: CallToolRequest,
    ) -> Result<CallToolResponse, rmcp::Error> {
        let tool_name = request.name.as_str();
        let args = request.arguments.unwrap_or_default();
        
        info!("Calling tool: {}", tool_name);
        
        match tool_name {
            "discover_assets" => handle_discover_assets(&self.packs_dir, args).await,
            "index_assets" => handle_index_assets(&self.packs_dir, args).await,
            "query_assets" => handle_query_assets(&self.packs_dir, args).await,
            "create_pack" => handle_create_pack(&self.packs_dir, args).await,
            "validate_pack" => handle_validate_pack(&self.packs_dir, args).await,
            "build_pack" => handle_build_pack(&self.packs_dir, args).await,
            _ => Err(rmcp::Error::invalid_request(
                format!("Unknown tool: {}", tool_name),
                None,
            )),
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    // Get packs directory from args or use default
    let packs_dir = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("./packs"));
    
    info!("Starting phenotype-mcp-asset server");
    info!("Packs directory: {:?}", packs_dir);
    
    // Create and run server
    let server = AssetMcpServer::new(packs_dir);
    
    // Start with stdio transport
    let service = server.serve(rmcp::transport::stdio::StdioTransport).await?;
    
    service.waiting().await?;
    
    Ok(())
}
