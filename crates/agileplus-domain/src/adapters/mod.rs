//! In-memory repository adapters for tests and local tooling.

pub mod memory;

pub use memory::{
    InMemoryProjectRepository, InMemorySprintRepository, InMemoryWorkItemRepository,
};
