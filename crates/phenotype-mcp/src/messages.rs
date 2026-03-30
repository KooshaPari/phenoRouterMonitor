//! MCP protocol message types

use crate::{ClientInfo, ServerInfo};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Client message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "method")]
pub enum ClientMessage {
    /// Initialize connection
    #[serde(rename = "initialize")]
    Initialize(InitializeRequest),
}

/// Server message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "method")]
pub enum ServerMessage {
    /// Initialize response
    #[serde(rename = "initialize")]
    Initialize(InitializeResponse),
}

/// Initialize request from client
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InitializeRequest {
    /// Protocol version (e.g., "2024-11-05")
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    /// Client capabilities
    pub capabilities: ClientCapabilities,
    /// Client information
    pub clientInfo: ClientInfo,
}

impl InitializeRequest {
    /// Create a new initialize request
    pub fn new(client_info: ClientInfo) -> Self {
        Self {
            protocol_version: "2024-11-05".into(),
            capabilities: ClientCapabilities::default(),
            clientInfo: client_info,
        }
    }
}

/// Initialize response from server
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InitializeResponse {
    /// Protocol version (e.g., "2024-11-05")
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    /// Server capabilities
    pub capabilities: ServerCapabilities,
    /// Server information
    pub serverInfo: ServerInfo,
}

impl InitializeResponse {
    /// Create a new initialize response
    pub fn new(server_info: ServerInfo) -> Self {
        Self {
            protocol_version: "2024-11-05".into(),
            capabilities: ServerCapabilities::default(),
            serverInfo: server_info,
        }
    }
}

/// Client capabilities
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClientCapabilities {
    /// Whether client supports experimental features
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub experimental: Option<HashMap<String, Value>>,
}

impl Default for ClientCapabilities {
    fn default() -> Self {
        Self { experimental: None }
    }
}

/// Server capabilities
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServerCapabilities {
    /// Tool capabilities
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: Option<ToolCapability>,
    /// Resource capabilities
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resources: Option<ResourceCapability>,
    /// Prompt capabilities
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompts: Option<PromptCapability>,
    /// Whether experimental features are supported
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub experimental: Option<HashMap<String, Value>>,
}

impl Default for ServerCapabilities {
    fn default() -> Self {
        Self {
            tools: Some(ToolCapability::default()),
            resources: Some(ResourceCapability::default()),
            prompts: Some(PromptCapability::default()),
            experimental: None,
        }
    }
}

impl ServerCapabilities {
    /// Create a new server capabilities with all features enabled
    pub fn full() -> Self {
        Self::default()
    }

    /// Create server capabilities with only tools enabled
    pub fn tools_only() -> Self {
        Self {
            tools: Some(ToolCapability::default()),
            resources: None,
            prompts: None,
            experimental: None,
        }
    }

    /// Create server capabilities with only resources enabled
    pub fn resources_only() -> Self {
        Self {
            tools: None,
            resources: Some(ResourceCapability::default()),
            prompts: None,
            experimental: None,
        }
    }

    /// Create server capabilities with only prompts enabled
    pub fn prompts_only() -> Self {
        Self {
            tools: None,
            resources: None,
            prompts: Some(PromptCapability::default()),
            experimental: None,
        }
    }
}

/// Tool capability
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolCapability {
    /// Whether tool listing is supported
    #[serde(default)]
    pub list_changed: bool,
}

impl Default for ToolCapability {
    fn default() -> Self {
        Self { list_changed: true }
    }
}

/// Resource capability
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResourceCapability {
    /// Whether resource listing is supported
    #[serde(default)]
    pub subscribe: bool,
    /// Whether resource list changes are supported
    #[serde(default)]
    pub list_changed: bool,
}

impl Default for ResourceCapability {
    fn default() -> Self {
        Self {
            subscribe: true,
            list_changed: true,
        }
    }
}

/// Prompt capability
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PromptCapability {
    /// Whether prompt listing is supported
    #[serde(default)]
    pub list_changed: bool,
}

impl Default for PromptCapability {
    fn default() -> Self {
        Self { list_changed: true }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialize_request() {
        let client_info = ClientInfo {
            name: "test-client".into(),
            version: "1.0.0".into(),
        };
        let req = InitializeRequest::new(client_info.clone());
        assert_eq!(req.protocol_version, "2024-11-05");
        assert_eq!(req.clientInfo, client_info);
    }

    #[test]
    fn test_initialize_response() {
        let server_info = ServerInfo::new("phenotype", "0.1.0");
        let resp = InitializeResponse::new(server_info.clone());
        assert_eq!(resp.protocol_version, "2024-11-05");
        assert_eq!(resp.serverInfo, server_info);
    }

    #[test]
    fn test_server_capabilities_default() {
        let caps = ServerCapabilities::default();
        assert!(caps.tools.is_some());
        assert!(caps.resources.is_some());
        assert!(caps.prompts.is_some());
    }

    #[test]
    fn test_server_capabilities_tools_only() {
        let caps = ServerCapabilities::tools_only();
        assert!(caps.tools.is_some());
        assert!(caps.resources.is_none());
        assert!(caps.prompts.is_none());
    }

    #[test]
    fn test_tool_capability() {
        let cap = ToolCapability::default();
        assert!(cap.list_changed);
    }

    #[test]
    fn test_resource_capability() {
        let cap = ResourceCapability::default();
        assert!(cap.subscribe);
        assert!(cap.list_changed);
    }

    #[test]
    fn test_prompt_capability() {
        let cap = PromptCapability::default();
        assert!(cap.list_changed);
    }
}
