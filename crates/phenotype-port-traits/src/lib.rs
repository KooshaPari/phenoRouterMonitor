//! Phenotype canonical hexagonal architecture port trait definitions.
//!
//! This crate provides minimal, generic port traits that form the foundation
//! of hexagonal architecture in the Phenotype ecosystem. All adapters implementing
//! these ports maintain clear separation between domain logic and external concerns.
//!
//! # Architecture
//!
//! Ports are divided into two categories:
//!
//! - **Inbound Ports (Driving)**: Entry points for use cases, commands, and queries
//! - **Outbound Ports (Driven)**: Interfaces for repositories, caches, secrets, and events
//!
//! All traits are `Send + Sync` to support async runtimes and concurrent access.

mod repository;
mod event_publisher;
mod cache;
mod secret_store;
mod health;
mod message_queue;

pub use repository::{Repository, RepositoryError};
pub use event_publisher::{EventPublisher, EventPublisherError};
pub use cache::{CachePort, CacheError};
pub use secret_store::{SecretStore, SecretStoreError};
pub use health::{HealthCheck, HealthStatus};
pub use message_queue::{MessageQueue, MessageQueueError};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_status_variants() {
        let healthy = HealthStatus::Healthy;
        assert_eq!(healthy, HealthStatus::Healthy);

        let degraded = HealthStatus::Degraded("partial failure".to_string());
        assert!(matches!(degraded, HealthStatus::Degraded(_)));

        let unhealthy = HealthStatus::Unhealthy("critical failure".to_string());
        assert!(matches!(unhealthy, HealthStatus::Unhealthy(_)));
    }
}
