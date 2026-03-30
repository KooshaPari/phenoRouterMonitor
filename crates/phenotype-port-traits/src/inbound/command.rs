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
