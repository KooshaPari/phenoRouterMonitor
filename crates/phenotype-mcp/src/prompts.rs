//! Prompt definitions for MCP protocol

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Prompt argument definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PromptArgument {
    /// Argument name
    pub name: String,
    /// Human-readable argument description
    pub description: String,
    /// Whether the argument is required
    #[serde(default)]
    pub required: bool,
}

impl PromptArgument {
    /// Create a new prompt argument
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            required: false,
        }
    }

    /// Mark this argument as required
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }
}

/// Prompt message definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PromptMessage {
    /// Message role ("user" or "assistant")
    pub role: String,
    /// Message content
    pub content: Value,
}

impl PromptMessage {
    /// Create a user message
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".into(),
            content: Value::String(content.into()),
        }
    }

    /// Create an assistant message
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".into(),
            content: Value::String(content.into()),
        }
    }

    /// Create a message with JSON content
    pub fn with_json(role: impl Into<String>, content: Value) -> Self {
        Self {
            role: role.into(),
            content,
        }
    }
}

/// Prompt definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Prompt {
    /// Prompt name (unique identifier)
    pub name: String,
    /// Human-readable prompt description
    pub description: String,
    /// Prompt arguments/parameters
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub arguments: Vec<PromptArgument>,
}

impl Prompt {
    /// Create a new prompt definition
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            arguments: Vec::new(),
        }
    }

    /// Add an argument to the prompt
    pub fn with_argument(mut self, arg: PromptArgument) -> Self {
        self.arguments.push(arg);
        self
    }

    /// Add a required argument to the prompt
    pub fn with_required_arg(self, name: impl Into<String>, description: impl Into<String>) -> Self {
        self.with_argument(PromptArgument::new(name, description).required())
    }

    /// Add an optional argument to the prompt
    pub fn with_optional_arg(self, name: impl Into<String>, description: impl Into<String>) -> Self {
        self.with_argument(PromptArgument::new(name, description))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_argument() {
        let arg = PromptArgument::new("context", "Additional context for the prompt");
        assert_eq!(arg.name, "context");
        assert!(!arg.required);

        let required_arg = arg.required();
        assert!(required_arg.required);
    }

    #[test]
    fn test_prompt_message_user() {
        let msg = PromptMessage::user("What should I do?");
        assert_eq!(msg.role, "user");
        assert_eq!(msg.content, Value::String("What should I do?".into()));
    }

    #[test]
    fn test_prompt_message_assistant() {
        let msg = PromptMessage::assistant("I can help you with that.");
        assert_eq!(msg.role, "assistant");
        assert_eq!(msg.content, Value::String("I can help you with that.".into()));
    }

    #[test]
    fn test_prompt_creation() {
        let prompt = Prompt::new("code_review", "Code review assistant")
            .with_required_arg("code", "Code to review")
            .with_optional_arg("language", "Programming language");

        assert_eq!(prompt.name, "code_review");
        assert_eq!(prompt.arguments.len(), 2);
        assert!(prompt.arguments[0].required);
        assert!(!prompt.arguments[1].required);
    }

    #[test]
    fn test_prompt_json_content() {
        let content = serde_json::json!({"type": "object", "data": "example"});
        let msg = PromptMessage::with_json("assistant", content.clone());
        assert_eq!(msg.role, "assistant");
        assert_eq!(msg.content, content);
    }
}
