//! Outbound ports - adapters and infrastructure from the application's perspective.

pub mod repository;
pub mod cache;
pub mod event;
pub mod secret;

pub use repository::{Repository, UnitOfWork};
pub use cache::{CachePort, CacheJsonPort, CacheCounterPort, CacheLockPort};
pub use event::{EventPublisher, EventSubscriber};
pub use secret::{SecretPort, VersionedSecretPort, SecretRotator};
