//! Canonical port traits for the Phenotype ecosystem.
//!
//! This crate consolidates all hexagonal architecture port definitions into a
//! single source of truth, eliminating the fragmentation between:
//! - `phenotype-contracts` (outbound ports, sync)
//! - `phenotype-port-traits` (inbound/outbound ports, async)
//! - `phenotype-event-sourcing` (EventStore)
//! - `phenotype-health` (HealthChecker)
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    Inbound Ports                              │
//! │  UseCase | CommandHandler | QueryHandler | EventHandler      │
//! └─────────────────────────────────────────────────────────────┘
//!                              │
//!                              ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │                      Domain Core                            │
//! │              Entity | ValueObject | AggregateRoot          │
//! │                   DomainEvent | DomainError                  │
//! └─────────────────────────────────────────────────────────────┘
//!                              │
//!                              ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │                   Outbound Ports                              │
//! │  Repository | CachePort | EventPublisher | SecretPort        │
//! │  EventStore | EventSubscriber | UnitOfWork | PolicyPort    │
//! └─────────────────────────────────────────────────────────────┘

pub mod error;
pub mod models;
pub mod inbound;
pub mod outbound;
pub mod health;
pub mod eventsourcing;

pub use error::PortError;
pub use models::{Entity, ValueObject, AggregateRoot, DomainEvent, EntityId};
pub use inbound::{UseCase, CommandHandler, QueryHandler, EventHandler, Command, Query, UseCaseInput, UseCaseOutput};
pub use outbound::{
    Repository, UnitOfWork, CachePort, CacheJsonPort, CacheCounterPort, CacheLockPort,
    EventPublisher, EventSubscriber, SecretPort, VersionedSecretPort, SecretRotator,
    PolicyPort, ConfigPort, 
};
pub use health::{HealthChecker, HealthStatus, HealthCheckResult, HealthCheckConfig, HealthResponse, HealthMonitor};
pub use eventsourcing::{EventStore, AsyncEventStore, Snapshot, SnapshotConfig, EventEnvelope};

pub mod prelude {
    pub use super::{
        PortError,
        Entity, ValueObject, AggregateRoot, DomainEvent, EntityId,
        UseCase, CommandHandler, QueryHandler, EventHandler,
        Repository, CachePort, EventPublisher, SecretPort,
        HealthChecker, HealthStatus,
        EventStore, EventEnvelope,
    };
}
