//! # AgilePlus Domain
//!
//! Core domain model for AgilePlus: entities, value objects, port interfaces, and aggregates.

pub mod aggregates;
pub mod entities;
pub mod events;
pub mod ports;
pub mod values;

pub use aggregates::ProjectAggregate;
pub use entities::{Project, Sprint, WorkItem};
pub use events::DomainEvent;
pub use values::{Priority, Status};
