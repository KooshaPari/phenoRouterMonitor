//! phenotype-contracts
//!
//! Hexagonal architecture ports and contracts for the Phenotype ecosystem.
//! This crate defines the interfaces (ports) and contracts that all other crates depend on.
//!
//! # Architecture
//!
//! Following Hexagonal Architecture (Ports & Adapters):
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │                      Application Core                     │
//! │                    (Domain Logic, Use Cases)             │
//! └─────────────────────────────────────────────────────────┘
//!              ▲                              ▲
//!              │                              │
//!         Inbound Ports                  Outbound Ports
//!    (Driving Ports, APIs)           (Driven Ports, SPIs)
//!              │                              │
//!         ┌────┴────────────┬────────────────┴────┐
//!         │                 │                     │
//!    ┌─────────────┐  ┌────────────┐  ┌──────────────────┐
//!    │   Adapters  │  │  Adapters  │  │    Adapters      │
//!    │  (REST API) │  │  (gRPC)    │  │  (Repository)    │
//!    │             │  │            │  │  (Cache)         │
//!    │             │  │            │  │  (EventBus)      │
//!    └─────────────┘  └────────────┘  └──────────────────┘
//! ```

/// Common error types used across the Phenotype ecosystem.
pub mod error;

/// Inbound ports (driving side) - interfaces for handling external requests.
pub mod inbound;

/// Outbound ports (driven side) - interfaces for accessing external services.
pub mod outbound;

/// Domain models, aggregates, and value objects.
pub mod models;

pub use error::{ContractError, Result};
