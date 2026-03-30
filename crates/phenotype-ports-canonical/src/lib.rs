//! # phenotype-ports-canonical
//!
//! Canonical trait definitions for the Phenotype hexagonal architecture.
//!
//! This crate consolidates core trait definitions (ports/adapters) that were
//! previously duplicated across `phenotype-contracts`, `harness_cache`,
//! `harness_interfaces`, `harness_teammates`, and `phenotype-event-sourcing`.
//!
//! ## Module Structure
//!
//! ```text
//! phenotype-ports-canonical
//! ├── domain     — Entity, AggregateRoot, ValueObject, DomainEvent
//! ├── inbound    — UseCase, CommandHandler, QueryHandler, EventHandler
//! ├── outbound   — Repository, CachePort, SecretPort, EventPublisher, EventSubscriber
//! ├── eventsourcing — EventStore, Snapshot
//! ├── health     — HealthChecker, Auditor
//! └── error      — PortError (unified error type)
//! ```
//!
//! ## Migration
//!
//! Replace imports from `phenotype_contracts::ports::*` and
//! `phenotype_contracts::models::*` with `phenotype_ports_canonical::*`.
//! See `MIGRATION.md` for detailed find-replace patterns.

pub mod domain;
pub mod error;
pub mod eventsourcing;
pub mod health;
pub mod inbound;
pub mod outbound;

// Re-export top-level types for convenience.
pub use domain::{AggregateRoot, DomainEvent, Entity, ValueObject};
pub use error::PortError;
pub use eventsourcing::EventStore;
pub use health::HealthChecker;
pub use inbound::{CommandHandler, EventHandler, QueryHandler, UseCase};
pub use outbound::{CachePort, EventPublisher, EventSubscriber, Repository, SecretPort};
