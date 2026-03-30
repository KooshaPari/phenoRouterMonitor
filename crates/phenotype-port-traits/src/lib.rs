//! # Phenotype Port Traits
//!
//! Hexagonal architecture port abstractions for domain-driven design.
//!
//! This crate provides traits for implementing Hexagonal Architecture (Ports & Adapters)
//! patterns in Rust. It defines clear boundaries between the application core domain
//! and external systems through inbound and outbound ports.

pub mod errors;
pub mod inbound;
pub mod models;
pub mod outbound;

pub use errors::{PortError, Result};
pub use inbound::{CommandHandler, EventSubscriber, QueryHandler, UseCase, UseCaseError};
pub use models::{AggregateRoot, DomainEvent, Entity, ValueObject};
pub use outbound::{Cache, EventBus, Logger, Repository, SecretVault};
