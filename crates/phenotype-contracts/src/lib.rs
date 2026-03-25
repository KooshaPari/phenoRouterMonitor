//! # Phenotype Contracts
//!
//! Hexagonal architecture ports and contracts for phenotype-infrakit.
//!
//! ## Architecture
//!
//! This crate defines the **ports** (interfaces) for the hexagonal architecture.
//! Adapters implement these ports to provide concrete functionality.
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                      Hexagonal Architecture                       │
//! ├─────────────────────────────────────────────────────────────────┤
//! │                                                                  │
//! │   ┌─────────────┐      ┌────────────────┐      ┌──────────────┐  │
//! │   │    Ports    │      │     Domain     │      │   Adapters  │  │
//! │   │ (Interfaces)│◄────▶│    (Core)      │◄────▶│(Implementations│
//! │   │             │      │                │      │              │  │
//! │   │ Inbound:     │      │  Business      │      │ Outbound:    │  │
//! │   │ - UseCase   │      │  Logic         │      │ - Repository │  │
//! │   │ - Command   │      │                │      │ - CachePort  │  │
//! │   │ - Query     │      │                │      │ - SecretPort │  │
//! │   │             │      │                │      │ - EventBus   │  │
//! │   └─────────────┘      └────────────────┘      └──────────────┘  │
//! │                                                                  │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Design Principles
//!
//! - **SOLID**: Interface Segregation, Dependency Inversion
//! - **GRASP**: Low Coupling, High Cohesion, Information Expert
//! - **Law of Demeter**: Minimal knowledge between modules
//! - **DRY**: Don't Repeat Yourself
//! - **KISS**: Keep It Simple, Stupid
//! - **YAGNI**: You Aren't Gonna Need It
//!
//! ## Modules
//!
//! - [`ports::inbound`] - Driving ports (use cases, commands, queries)
//! - [`ports::outbound`] - Driven ports (repositories, cache, secrets)
//! - [`models`] - Domain models and value objects

pub mod ports;
pub mod models;

#[cfg(test)]
mod tests;

/// Result type alias using our custom error
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
