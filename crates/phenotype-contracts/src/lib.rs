//! Hexagonal architecture ports and domain model contracts for Phenotype.

pub mod adapters;
pub mod error;
pub mod outbound;

pub use adapters::{InMemoryCache, InMemoryEventBus, InMemoryRepository, InMemorySecretManager};
pub use error::{DomainError, Result};
pub use outbound::{CachePort, EventBus, Repository, SecretManager};
