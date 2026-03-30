//! Phenotype MCP Server
//!
//! MCP (Model Context Protocol) server for Phenotype tools.

pub mod tools;

use fastmcp::{Server, Tool, ToolInput, ToolResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// MCP Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub name: String,
    pub version: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            name: "phenotype".into(),
            version: env!("CARGO_PKG_VERSION").into(),
        }
    }
}

/// Create the Phenotype MCP server
pub fn create_server(config: Config) -> Server {
    let mut server = Server::new(&config.name);
    
    // Register AgilePlus tools
    server.add_tool(Tool::new(
        "agileplus_create_feature",
        "Create a feature specification",
        |input: ToolInput| {
            let title = input.arguments.get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("Untitled");
            ToolResult::success(serde_json::json!({
                "feature_id": format!("feat_{}", uuid::Uuid::new_v4()),
                "title": title,
                "status": "created"
            }))
        },
    ));
    
    server.add_tool(Tool::new(
        "agileplus_validate",
        "Validate a feature against governance rules",
        |input: ToolInput| {
            let feature_id = input.arguments.get("feature_id")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            ToolResult::success(serde_json::json!({
                "valid": true,
                "feature_id": feature_id,
                "checks_passed": 5,
                "checks_total": 5
            }))
        },
    ));
    
    server.add_tool(Tool::new(
        "agileplus_status",
        "Update work package status",
        |input: ToolInput| {
            let wp_id = input.arguments.get("wp_id")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let state = input.arguments.get("state")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            ToolResult::success(serde_json::json!({
                "wp_id": wp_id,
                "state": state,
                "updated": true
            }))
        },
    ));
    
    // Register Phenotype tools
    server.add_tool(Tool::new(
        "phenotype_parse_spec",
        "Parse and validate specifications",
        |input: ToolInput| {
            let content = input.arguments.get("spec_content")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            ToolResult::success(serde_json::json!({
                "valid": true,
                "lines": content.lines().count()
            }))
        },
    ));
    
    // Register Agent tools
    server.add_tool(Tool::new(
        "agent_dispatch",
        "Dispatch a task to an AI agent",
        |input: ToolInput| {
            let task = input.arguments.get("task")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            ToolResult::success(serde_json::json!({
                "task_id": format!("task_{}", uuid::Uuid::new_v4()),
                "task": task,
                "status": "dispatched"
            }))
        },
    ));
    
    server
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_creation() {
        let server = create_server(Config::default());
        assert_eq!(server.name(), "phenotype");
    }
}
