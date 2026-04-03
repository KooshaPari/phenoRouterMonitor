//! MCP Server for Game Testing and Validation
//!
//! This server provides tools for:
//! - Game launching and management
//! - Automated testing scenarios
//! - Performance profiling
//! - State validation

use rmcp::handler::server::ServerHandler;
use rmcp::model::{CallToolRequest, CallToolResponse, Tool};
use rmcp::schemars;
use serde_json::json;
use std::path::PathBuf;
use tracing::{info, error};

mod tools;
use tools::*;

/// Testing MCP Server
#[derive(Debug)]
pub struct TestingMcpServer {
    workspace_dir: PathBuf,
    tools: Vec<Tool>,
}

impl TestingMcpServer {
    pub fn new(workspace_dir: PathBuf) -> Self {
        let tools = vec![
            launch_game_tool(),
            terminate_game_tool(),
            get_game_status_tool(),
            wait_for_world_tool(),
            run_test_scenario_tool(),
            validate_state_tool(),
            profile_performance_tool(),
        ];
        
        Self {
            workspace_dir,
            tools,
        }
    }
}

#[rmcp::tool]
impl ServerHandler for TestingMcpServer {
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
            "launch_game" => handle_launch_game(&self.workspace_dir, args).await,
            "terminate_game" => handle_terminate_game(&self.workspace_dir, args).await,
            "get_game_status" => handle_get_game_status(&self.workspace_dir, args).await,
            "wait_for_world" => handle_wait_for_world(&self.workspace_dir, args).await,
            "run_test_scenario" => handle_run_test_scenario(&self.workspace_dir, args).await,
            "validate_state" => handle_validate_state(&self.workspace_dir, args).await,
            "profile_performance" => handle_profile_performance(&self.workspace_dir, args).await,
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
    
    // Get workspace directory from args or use default
    let workspace_dir = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("./test-workspace"));
    
    info!("Starting phenotype-mcp-testing server");
    info!("Workspace directory: {:?}", workspace_dir);
    
    // Create and run server
    let server = TestingMcpServer::new(workspace_dir);
    
    // Start with stdio transport
    let service = server.serve(rmcp::transport::stdio::StdioTransport).await?;
    
    service.waiting().await?;
    
    Ok(())
}
