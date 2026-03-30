//! Resource definitions for MCP protocol

use serde::{Deserialize, Serialize};

/// Resource content representation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ResourceContents {
    /// Text resource content
    Text { text: String },
    /// Binary resource content (base64 encoded)
    Binary {
        #[serde(rename = "mimeType")]
        mime_type: String,
        data: String,
    },
}

impl ResourceContents {
    /// Create text content
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text {
            text: text.into(),
        }
    }

    /// Create binary content
    pub fn binary(mime_type: impl Into<String>, data: impl Into<String>) -> Self {
        Self::Binary {
            mime_type: mime_type.into(),
            data: data.into(),
        }
    }
}

/// Resource definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Resource {
    /// Resource URI (unique identifier)
    pub uri: String,
    /// Human-readable resource name
    pub name: String,
    /// Human-readable resource description
    pub description: String,
    /// MIME type of the resource content
    #[serde(rename = "mimeType")]
    pub mime_type: String,
}

impl Resource {
    /// Create a new resource definition
    pub fn new(
        uri: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        mime_type: impl Into<String>,
    ) -> Self {
        Self {
            uri: uri.into(),
            name: name.into(),
            description: description.into(),
            mime_type: mime_type.into(),
        }
    }

    /// Create a text resource
    pub fn text(
        uri: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self::new(uri, name, description, "text/plain")
    }

    /// Create a JSON resource
    pub fn json(
        uri: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self::new(uri, name, description, "application/json")
    }

    /// Create a markdown resource
    pub fn markdown(
        uri: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self::new(uri, name, description, "text/markdown")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_creation() {
        let resource = Resource::new(
            "file:///path/to/resource",
            "My Resource",
            "A test resource",
            "text/plain",
        );
        assert_eq!(resource.uri, "file:///path/to/resource");
        assert_eq!(resource.name, "My Resource");
        assert_eq!(resource.mime_type, "text/plain");
    }

    #[test]
    fn test_resource_text() {
        let resource = Resource::text(
            "file:///test.txt",
            "Test File",
            "A text file",
        );
        assert_eq!(resource.mime_type, "text/plain");
    }

    #[test]
    fn test_resource_json() {
        let resource = Resource::json(
            "file:///test.json",
            "Test JSON",
            "A JSON file",
        );
        assert_eq!(resource.mime_type, "application/json");
    }

    #[test]
    fn test_resource_markdown() {
        let resource = Resource::markdown(
            "file:///test.md",
            "Test Markdown",
            "A markdown file",
        );
        assert_eq!(resource.mime_type, "text/markdown");
    }

    #[test]
    fn test_resource_contents_text() {
        let contents = ResourceContents::text("Hello, world!");
        match contents {
            ResourceContents::Text { text } => assert_eq!(text, "Hello, world!"),
            _ => panic!("Expected text content"),
        }
    }

    #[test]
    fn test_resource_contents_binary() {
        let contents = ResourceContents::binary("image/png", "iVBORw0KGgo...");
        match contents {
            ResourceContents::Binary { mime_type, data } => {
                assert_eq!(mime_type, "image/png");
                assert_eq!(data, "iVBORw0KGgo...");
            }
            _ => panic!("Expected binary content"),
        }
    }
}
