//! Command handler port for CQRS command processing.

use async_trait::async_trait;

/// Marker trait for command types.
pub trait Command: Send + Sync + Sized {}

/// Command handler port for processing write operations (CQRS).
#[async_trait]
pub trait CommandHandler<C: Command>: Send + Sync {
    /// Handle the given command.
    async fn handle(&self, command: C) -> Result<CommandResult, CommandError>;
}

/// Result of command execution, optionally containing the ID of the affected entity.
#[derive(Debug)]
pub struct CommandResult {
    pub entity_id: Option<String>,
    pub message: Option<String>,
}

impl CommandResult {
    pub fn with_id(id: impl Into<String>) -> Self {
        Self {
            entity_id: Some(id.into()),
            message: None,
        }
    }

    pub fn with_message(msg: impl Into<String>) -> Self {
        Self {
            entity_id: None,
            message: Some(msg.into()),
        }
    }
}

/// Errors that can occur during command handling.
#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error("validation failed: {0}")]
    Validation(String),

    #[error("not found: {entity} {id}")]
    NotFound { entity: String, id: String },

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("not permitted: {0}")]
    NotPermitted(String),

    #[error("internal error: {0}")]
    Internal(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_result_with_id() {
        let result = CommandResult::with_id("entity-123");
        assert_eq!(result.entity_id.as_deref(), Some("entity-123"));
        assert!(result.message.is_none());
    }

    #[test]
    fn command_result_with_message() {
        let result = CommandResult::with_message("created successfully");
        assert!(result.entity_id.is_none());
        assert_eq!(result.message.as_deref(), Some("created successfully"));
    }

    #[test]
    fn command_result_debug() {
        let result = CommandResult::with_id("abc");
        let debug = format!("{:?}", result);
        assert!(debug.contains("abc"));
    }

    #[test]
    fn command_error_validation_display() {
        let err = CommandError::Validation("name required".into());
        assert_eq!(err.to_string(), "validation failed: name required");
    }

    #[test]
    fn command_error_not_found_display() {
        let err = CommandError::NotFound {
            entity: "User".into(),
            id: "42".into(),
        };
        assert_eq!(err.to_string(), "not found: User 42");
    }

    #[test]
    fn command_error_conflict_display() {
        let err = CommandError::Conflict("duplicate email".into());
        assert_eq!(err.to_string(), "conflict: duplicate email");
    }

    #[test]
    fn command_error_not_permitted_display() {
        let err = CommandError::NotPermitted("admin only".into());
        assert_eq!(err.to_string(), "not permitted: admin only");
    }

    #[test]
    fn command_error_internal_display() {
        let err = CommandError::Internal("db down".into());
        assert_eq!(err.to_string(), "internal error: db down");
    }
}
