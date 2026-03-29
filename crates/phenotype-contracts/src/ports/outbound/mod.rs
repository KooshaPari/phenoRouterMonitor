//! # Outbound Ports (Driven Ports)
//!
//! Outbound ports define the interfaces for secondary adapters.
//! These are called by the domain to interact with external systems.

pub mod cache;
pub mod event;
pub mod repository;
pub mod secret;

pub use cache::CachePort;
pub use event::EventBusPort;
pub use repository::RepositoryPort;
pub use secret::SecretPort;
