//! Event sourcing ports: EventStore, Snapshot.
//!
//! Previously in `phenotype-event-sourcing` (now removed) and partially in
//! `phenotype-contracts`.

use async_trait::async_trait;

use crate::domain::{AggregateRoot, DomainEvent};
use crate::error::PortError;

/// Persists and replays domain events for event-sourced aggregates.
#[async_trait]
pub trait EventStore: Send + Sync {
    /// Append events for a given aggregate.
    async fn append(
        &self,
        aggregate_id: &str,
        expected_version: u64,
        events: &[Box<dyn DomainEvent>],
    ) -> Result<(), PortError>;

    /// Load all events for an aggregate (in order).
    async fn load(&self, aggregate_id: &str) -> Result<Vec<Box<dyn DomainEvent>>, PortError>;

    /// Load events starting from a specific version.
    async fn load_from(
        &self,
        aggregate_id: &str,
        from_version: u64,
    ) -> Result<Vec<Box<dyn DomainEvent>>, PortError>;
}

/// Snapshot support for event-sourced aggregates.
///
/// Snapshots store a materialized aggregate state to avoid replaying all events.
#[async_trait]
pub trait Snapshot<A: AggregateRoot>: Send + Sync {
    /// Load the latest snapshot for an aggregate.
    async fn load_snapshot(&self, aggregate_id: &str) -> Result<Option<(A, u64)>, PortError>;

    /// Save a snapshot at a given version.
    async fn save_snapshot(
        &self,
        aggregate_id: &str,
        aggregate: &A,
        version: u64,
    ) -> Result<(), PortError>;
}
