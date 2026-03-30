//! Phenotype MCP Server
//!
//! MCP (Model Context Protocol) server for Phenotype tools.

pub mod tools;

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

/// MCP Server state
pub struct Server {
    config: Config,
    tools: HashMap<String, ToolDef>,
}

/// Tool definition
#[derive(Debug, Clone)]
pub struct ToolDef {
    pub name: String,
    pub description: String,
}

impl Server {
    /// Create a new server
    pub fn new() -> Self {
        let mut server = Self {
            config: Config::default(),
            tools: HashMap::new(),
        };
        server.register_default_tools();
        server
    }
    
    /// Register default tools
    fn register_default_tools(&mut self) {
        self.tools.insert("agileplus_create_feature".into(), ToolDef {
            name: "agileplus_create_feature".into(),
            description: "Create a feature specification".into(),
        });
        self.tools.insert("agileplus_validate".into(), ToolDef {
            name: "agileplus_validate".into(),
            description: "Validate a feature against governance rules".into(),
        });
        self.tools.insert("agileplus_status".into(), ToolDef {
            name: "agileplus_status".into(),
            description: "Update work package status".into(),
        });
        self.tools.insert("phenotype_parse_spec".into(), ToolDef {
            name: "phenotype_parse_spec".into(),
            description: "Parse and validate specifications".into(),
        });
        self.tools.insert("agent_dispatch".into(), ToolDef {
            name: "agent_dispatch".into(),
            description: "Dispatch a task to an AI agent".into(),
        });
    }
    
    /// Get server info
    pub fn info(&self) -> ServerInfo {
        ServerInfo {
            name: self.config.name.clone(),
            version: self.config.version.clone(),
            tool_count: self.tools.len(),
        }
    }
    
    /// List all tools
    pub fn list_tools(&self) -> Vec<ToolInfo> {
        self.tools.values().map(|t| ToolInfo {
            name: t.name.clone(),
            description: t.description.clone(),
        }).collect()
    }
}

/// Server information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
    pub tool_count: usize,
}

/// Tool information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_creation() {
        let server = Server::new();
        let info = server.info();
        assert_eq!(info.name, "phenotype");
        assert_eq!(info.tool_count, 5);
    }
}
