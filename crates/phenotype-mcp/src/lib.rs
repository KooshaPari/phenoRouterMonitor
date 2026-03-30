//! # Phenotype MCP
//!
//! MCP (Model Context Protocol) types and tool definitions for Phenotype.
//! Provides the data structures for registering tools, resources, and prompts.

pub mod tools;

use serde::{Deserialize, Serialize};

/// MCP tool definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDef {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

/// MCP tool result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub content: Vec<ContentBlock>,
    pub is_error: bool,
}

/// Content block in a tool result.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image { data: String, mime_type: String },
}

impl ToolResult {
    pub fn text(content: impl Into<String>) -> Self {
        Self {
            content: vec![ContentBlock::Text {
                text: content.into(),
            }],
            is_error: false,
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            content: vec![ContentBlock::Text {
                text: message.into(),
            }],
            is_error: true,
        }
    }
}

/// MCP server configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub name: String,
    pub version: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            name: "phenotype".into(),
            version: env!("CARGO_PKG_VERSION").into(),
        }
    }
}

/// Registry of available MCP tools.
#[derive(Debug, Default)]
pub struct ToolRegistry {
    tools: Vec<ToolDef>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, tool: ToolDef) {
        self.tools.push(tool);
    }

    pub fn list(&self) -> &[ToolDef] {
        &self.tools
    }

    pub fn find(&self, name: &str) -> Option<&ToolDef> {
        self.tools.iter().find(|t| t.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_result_text() {
        let r = ToolResult::text("hello");
        assert!(!r.is_error);
        assert_eq!(r.content.len(), 1);
    }

    #[test]
    fn tool_result_error() {
        let r = ToolResult::error("oops");
        assert!(r.is_error);
    }

    #[test]
    fn tool_registry() {
        let mut reg = ToolRegistry::new();
        reg.register(ToolDef {
            name: "test_tool".into(),
            description: "A test tool".into(),
            input_schema: serde_json::json!({}),
        });
        assert_eq!(reg.list().len(), 1);
        assert!(reg.find("test_tool").is_some());
        assert!(reg.find("missing").is_none());
    }
}
