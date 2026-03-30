//! Hexagonal architecture ports and domain model contracts for Phenotype.

pub mod adapters;
pub mod error;
pub mod inbound;
pub mod models;
pub mod outbound;

pub use adapters::{
    InMemoryCache, InMemoryEventBus, InMemoryRepository, InMemorySecretManager,
};
pub use error::{ContractError, Result};
pub use inbound::{CommandHandler, EventHandler, QueryHandler, UseCase};
pub use models::{AggregateRoot, DomainEntity, DomainEvent, ValueObject};
pub use outbound::{CachePort, ConfigLoader, EventBus, Repository, SecretManager};
