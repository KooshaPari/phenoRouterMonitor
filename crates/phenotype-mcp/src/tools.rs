//! Tool definitions and types for MCP protocol

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// JSON Schema definition for tool input parameters
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolInputSchema {
    /// The type of the input (e.g., "object")
    #[serde(rename = "type")]
    pub type_: String,
    /// Properties of the input object
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub properties: HashMap<String, Value>,
    /// Required properties
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required: Vec<String>,
    /// Additional schema attributes
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

impl ToolInputSchema {
    /// Create a new tool input schema for an object type
    pub fn object() -> Self {
        Self {
            type_: "object".into(),
            properties: HashMap::new(),
            required: Vec::new(),
            extra: HashMap::new(),
        }
    }

    /// Add a property to the schema
    pub fn with_property(mut self, name: impl Into<String>, schema: Value) -> Self {
        self.properties.insert(name.into(), schema);
        self
    }

    /// Mark a property as required
    pub fn require(mut self, name: impl Into<String>) -> Self {
        self.required.push(name.into());
        self
    }
}

impl Default for ToolInputSchema {
    fn default() -> Self {
        Self::object()
    }
}

/// Tool input containing arguments passed to the tool
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolInput {
    /// The arguments passed to the tool
    pub arguments: HashMap<String, Value>,
}

impl ToolInput {
    /// Create new tool input
    pub fn new(arguments: HashMap<String, Value>) -> Self {
        Self { arguments }
    }

    /// Get a string argument
    pub fn get_string(&self, key: &str) -> Option<String> {
        self.arguments.get(key).and_then(|v| v.as_str().map(|s| s.to_string()))
    }

    /// Get an integer argument
    pub fn get_i64(&self, key: &str) -> Option<i64> {
        self.arguments.get(key).and_then(|v| v.as_i64())
    }

    /// Get a boolean argument
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.arguments.get(key).and_then(|v| v.as_bool())
    }

    /// Get a raw JSON value argument
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.arguments.get(key)
    }
}

impl Default for ToolInput {
    fn default() -> Self {
        Self {
            arguments: HashMap::new(),
        }
    }
}

/// Tool result containing the output of a tool execution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolResult {
    /// Whether the tool succeeded
    pub is_error: bool,
    /// The result content (text or structured data)
    pub content: Vec<ToolResultContent>,
}

impl ToolResult {
    /// Create a successful tool result with text content
    pub fn success(text: impl Into<String>) -> Self {
        Self {
            is_error: false,
            content: vec![ToolResultContent::text(text)],
        }
    }

    /// Create a successful tool result with structured JSON content
    pub fn success_json(value: Value) -> Self {
        Self {
            is_error: false,
            content: vec![ToolResultContent::Text {
                text: value.to_string(),
            }],
        }
    }

    /// Create an error tool result
    pub fn error(text: impl Into<String>) -> Self {
        Self {
            is_error: true,
            content: vec![ToolResultContent::text(text)],
        }
    }

    /// Add content to the result
    pub fn with_content(mut self, content: ToolResultContent) -> Self {
        self.content.push(content);
        self
    }
}

impl Default for ToolResult {
    fn default() -> Self {
        Self {
            is_error: false,
            content: Vec::new(),
        }
    }
}

/// Tool result content types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ToolResultContent {
    /// Text content
    Text { text: String },
    /// Image content (MIME type and base64 data)
    Image {
        #[serde(rename = "mimeType")]
        mime_type: String,
        data: String,
    },
}

impl ToolResultContent {
    /// Create text content
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text {
            text: text.into(),
        }
    }

    /// Create image content
    pub fn image(mime_type: impl Into<String>, data: impl Into<String>) -> Self {
        Self::Image {
            mime_type: mime_type.into(),
            data: data.into(),
        }
    }
}

/// Tool definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Tool {
    /// Tool name (unique identifier)
    pub name: String,
    /// Human-readable tool description
    pub description: String,
    /// JSON schema for the tool's input parameters
    #[serde(rename = "inputSchema")]
    pub input_schema: ToolInputSchema,
}

impl Tool {
    /// Create a new tool definition
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            input_schema: ToolInputSchema::default(),
        }
    }

    /// Set the input schema for the tool
    pub fn with_schema(mut self, schema: ToolInputSchema) -> Self {
        self.input_schema = schema;
        self
    }

    /// Add a property to the input schema
    pub fn with_property(mut self, name: impl Into<String>, schema: Value) -> Self {
        self.input_schema = self
            .input_schema
            .with_property(name, schema);
        self
    }

    /// Require a parameter in the input schema
    pub fn require_param(mut self, name: impl Into<String>) -> Self {
        self.input_schema = self.input_schema.require(name);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_creation() {
        let tool = Tool::new("test_tool", "A test tool");
        assert_eq!(tool.name, "test_tool");
        assert_eq!(tool.description, "A test tool");
    }

    #[test]
    fn test_tool_input_schema() {
        let mut schema = ToolInputSchema::object();
        schema.properties.insert(
            "name".into(),
            serde_json::json!({"type": "string"}),
        );
        schema.required.push("name".into());

        assert_eq!(schema.type_, "object");
        assert_eq!(schema.required.len(), 1);
    }

    #[test]
    fn test_tool_input_getters() {
        let mut args = HashMap::new();
        args.insert("text".into(), Value::String("hello".into()));
        args.insert("count".into(), Value::Number(42.into()));
        args.insert("active".into(), Value::Bool(true));

        let input = ToolInput::new(args);
        assert_eq!(input.get_string("text"), Some("hello".into()));
        assert_eq!(input.get_i64("count"), Some(42));
        assert_eq!(input.get_bool("active"), Some(true));
    }

    #[test]
    fn test_tool_result_success() {
        let result = ToolResult::success("Operation completed");
        assert!(!result.is_error);
        assert_eq!(result.content.len(), 1);
    }

    #[test]
    fn test_tool_result_error() {
        let result = ToolResult::error("Something went wrong");
        assert!(result.is_error);
        assert_eq!(result.content.len(), 1);
    }

    #[test]
    fn test_tool_result_content_text() {
        let content = ToolResultContent::text("test content");
        match content {
            ToolResultContent::Text { text } => assert_eq!(text, "test content"),
            _ => panic!("Expected text content"),
        }
    }

    #[test]
    fn test_tool_with_schema() {
        let schema = ToolInputSchema::object()
            .with_property("param1", serde_json::json!({"type": "string"}))
            .require("param1");

        let tool = Tool::new("my_tool", "A tool")
            .with_schema(schema.clone())
            .require_param("param2");

        assert_eq!(tool.input_schema.required.len(), 2);
    }
}
