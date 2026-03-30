//! Outbound ports for hexagonal architecture.

use async_trait::async_trait;
use std::time::Duration;

pub type PortResult<T> = std::result::Result<T, PortError>;

#[derive(Debug, Clone)]
pub enum PortError {
    NotFound(String),
    StorageError(String),
    SerializationError(String),
    CacheError(String),
    VaultError(String),
    EventPublicationError(String),
    Timeout(String),
    InvalidConfiguration(String),
    Internal(String),
}

impl std::fmt::Display for PortError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(msg) => write!(f, "not found: {msg}"),
            Self::StorageError(msg) => write!(f, "storage error: {msg}"),
            Self::SerializationError(msg) => write!(f, "serialization error: {msg}"),
            Self::CacheError(msg) => write!(f, "cache error: {msg}"),
            Self::VaultError(msg) => write!(f, "vault error: {msg}"),
            Self::EventPublicationError(msg) => write!(f, "event publication error: {msg}"),
            Self::Timeout(msg) => write!(f, "timeout: {msg}"),
            Self::InvalidConfiguration(msg) => write!(f, "invalid configuration: {msg}"),
            Self::Internal(msg) => write!(f, "internal error: {msg}"),
        }
    }
}

impl std::error::Error for PortError {}

#[async_trait]
pub trait Repository: Send + Sync {
    type Aggregate: Send + Sync;
    type Id: Clone + Send + Sync;
    async fn save(&self, aggregate: Self::Aggregate) -> PortResult<()>;
    async fn find(&self, id: &Self::Id) -> PortResult<Self::Aggregate>;
    async fn delete(&self, id: &Self::Id) -> PortResult<()>;
    async fn exists(&self, id: &Self::Id) -> PortResult<bool>;
}

#[async_trait]
pub trait Cache: Send + Sync {
    type Key: Clone + Send + Sync;
    type Value: Clone + Send + Sync;
    async fn get(&self, key: &Self::Key) -> PortResult<Option<Self::Value>>;
    async fn set(&self, key: Self::Key, value: Self::Value) -> PortResult<()>;
    async fn set_with_ttl(
        &self,
        key: Self::Key,
        value: Self::Value,
        ttl: Duration,
    ) -> PortResult<()>;
    async fn remove(&self, key: &Self::Key) -> PortResult<()>;
    async fn clear(&self) -> PortResult<()>;
}

#[async_trait]
pub trait SecretVault: Send + Sync {
    async fn get_secret(&self, key: &str) -> PortResult<String>;
    async fn set_secret(&self, key: &str, value: &str) -> PortResult<()>;
    async fn delete_secret(&self, key: &str) -> PortResult<()>;
    async fn has_secret(&self, key: &str) -> PortResult<bool>;
}

#[async_trait]
pub trait EventBus: Send + Sync {
    type Event: Send + Sync;
    async fn publish(&self, event: Self::Event) -> PortResult<()>;
    async fn publish_batch(&self, events: Vec<Self::Event>) -> PortResult<()>;
}

pub trait Logger: Send + Sync {
    fn error(&self, message: &str, context: Option<&[(&str, &str)]>);
    fn warn(&self, message: &str, context: Option<&[(&str, &str)]>);
    fn info(&self, message: &str, context: Option<&[(&str, &str)]>);
    fn debug(&self, message: &str, context: Option<&[(&str, &str)]>);
    fn trace(&self, message: &str, context: Option<&[(&str, &str)]>);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_error_display() {
        let err = PortError::NotFound("test".to_string());
        assert_eq!(err.to_string(), "not found: test");
    }

    struct MockLogger;

    impl Logger for MockLogger {
        fn error(&self, _message: &str, _context: Option<&[(&str, &str)]>) {}
        fn warn(&self, _message: &str, _context: Option<&[(&str, &str)]>) {}
        fn info(&self, _message: &str, _context: Option<&[(&str, &str)]>) {}
        fn debug(&self, _message: &str, _context: Option<&[(&str, &str)]>) {}
        fn trace(&self, _message: &str, _context: Option<&[(&str, &str)]>) {}
    }

    #[test]
    fn test_logger_creation() {
        let _logger = MockLogger;
    }
}
