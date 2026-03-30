//! EventSourced trait: unified event emission, storage, and replay pattern.
//!
//! This trait provides a type-safe, portable pattern for aggregates that:
//! - Emit domain events during mutations
//! - Store events immutably in the event store
//! - Reconstruct from event streams (replay)
//! - Support event versioning and metadata
//!
//! Traces to: FR-PHENO-008

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use uuid::Uuid;

use crate::{EventEnvelope, EventSourcingError, EventStore, Result};

/// Metadata attached to sourced events for correlation and causation tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    /// Unique identifier for this event.
    pub event_id: Uuid,
    /// Correlation ID: trace related events across the system.
    pub correlation_id: Option<Uuid>,
    /// Causation ID: what event caused this one (optional parent).
    pub causation_id: Option<Uuid>,
    /// Actor who triggered this event (user ID, service name, etc.).
    pub actor: String,
    /// Custom tags for filtering and searching (e.g., ["high-priority", "batch-import"]).
    pub tags: Vec<String>,
}

impl EventMetadata {
    pub fn new(actor: String) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            correlation_id: None,
            causation_id: None,
            actor,
            tags: vec![],
        }
    }

    pub fn with_correlation(mut self, correlation_id: Uuid) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }

    pub fn with_causation(mut self, causation_id: Uuid) -> Self {
        self.causation_id = Some(causation_id);
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }
}

/// Unified trait for event-sourced aggregates.
///
/// Types implementing EventSourced can:
/// - Emit events through type-safe methods
/// - Have events persisted atomically
/// - Be reconstructed from event streams
/// - Support querying uncommitted events
///
/// # Type Parameters
/// - `Event`: The domain event type (must implement DomainEvent)
/// - `Id`: The aggregate identifier type
///
/// # Example
/// ```ignore
/// pub struct Order {
///     id: OrderId,
///     items: Vec<Item>,
///     status: OrderStatus,
///     pending_events: Vec<OrderEvent>,
/// }
///
/// impl EventSourced for Order {
///     type Event = OrderEvent;
///     type Id = OrderId;
///
///     fn id(&self) -> &Self::Id { &self.id }
///
///     fn pending_events(&self) -> &[Self::Event] { &self.pending_events }
///
///     fn clear_events(&mut self) { self.pending_events.clear(); }
///
///     fn apply_event(&mut self, event: Self::Event) -> Result<()> {
///         match event {
///             OrderEvent::ItemAdded { item } => {
///                 self.items.push(item);
///             }
///             OrderEvent::OrderPlaced => {
///                 self.status = OrderStatus::Placed;
///             }
///         }
///         Ok(())
///     }
/// }
/// ```
pub trait EventSourced: Send + Sync + Serialize + DeserializeOwned {
    /// The domain event type emitted by this aggregate.
    type Event: Serialize + DeserializeOwned + Send + Sync + 'static;

    /// The aggregate identifier type.
    type Id: Send + Sync + Clone + std::fmt::Display + Serialize + DeserializeOwned + 'static;

    /// Returns this aggregate's unique identifier.
    fn id(&self) -> &Self::Id;

    /// Returns uncommitted (pending) events.
    fn pending_events(&self) -> &[Self::Event];

    /// Clears uncommitted events after successful persistence.
    fn clear_events(&mut self);

    /// Applies an event to this aggregate's state.
    ///
    /// This is the core mutation method. Implementations should:
    /// 1. Validate the event against current state
    /// 2. Update internal fields
    /// 3. Return Err if the event is invalid for current state
    ///
    /// This method is called during:
    /// - Replay (reconstructing from event stream)
    /// - Real-time event application
    fn apply_event(&mut self, event: Self::Event) -> Result<()>;

    /// Emits an event by appending it to pending events.
    ///
    /// Called during aggregate mutations. The event is stored locally
    /// until persist_events() is called.
    fn emit_event(&mut self, event: Self::Event) {
        // This method can be overridden for custom emission logic,
        // but the default is to collect into pending events.
        // Subclasses should maintain self.pending_events vector.
    }
}

/// Event sourcing operations: persist, load, replay.
#[async_trait]
pub trait EventSourcingPort: Send + Sync {
    type Event: Serialize + DeserializeOwned + Send + Sync + 'static;
    type Id: Send + Sync + Clone + std::fmt::Display + Serialize + DeserializeOwned + 'static;

    /// Persist all pending events for an aggregate.
    ///
    /// Implementation should:
    /// 1. Validate the aggregate state
    /// 2. Iterate pending_events
    /// 3. Create EventEnvelopes with metadata
    /// 4. Append to the event store
    /// 5. Return the sequence number of the last appended event
    async fn persist_events<T: EventSourced<Event = Self::Event, Id = Self::Id> + Send>(
        &self,
        aggregate: &T,
        metadata: &EventMetadata,
    ) -> Result<i64>;

    /// Load all events for an aggregate by ID and reconstruct it.
    ///
    /// Implementation should:
    /// 1. Retrieve all events from the store
    /// 2. Create a fresh aggregate
    /// 3. Apply each event in sequence
    /// 4. Return the reconstructed aggregate
    ///
    /// Returns EventSourcingError::EntityNotFound if no events exist.
    async fn load_aggregate<T: EventSourced<Event = Self::Event, Id = Self::Id>>(
        &self,
        id: &T::Id,
    ) -> Result<T>;

    /// Get all events for an aggregate without reconstruction.
    async fn get_events(&self, id: &Self::Id) -> Result<Vec<Self::Event>>;

    /// Reconstruct aggregate from a specific event sequence range.
    ///
    /// Useful for time-travel queries and event stream analysis.
    async fn load_aggregate_at<T: EventSourced<Event = Self::Event, Id = Self::Id>>(
        &self,
        id: &T::Id,
        up_to_sequence: i64,
    ) -> Result<T>;
}

/// Event handler registry: auto-discovery and dispatch.
#[async_trait]
pub trait EventHandlerRegistry: Send + Sync {
    type Event: Serialize + Send + Sync + 'static;

    /// Register a handler for a specific event type.
    ///
    /// Handlers can be registered multiple times for the same event type;
    /// all matching handlers are invoked on dispatch.
    async fn register<H: EventHandler + 'static>(
        &self,
        event_type: &str,
        handler: H,
    ) -> Result<()>;

    /// Dispatch an event to all registered handlers.
    ///
    /// Returns a vector of handler results. If any handler fails,
    /// the error is returned but other handlers still execute.
    async fn dispatch(&self, event_type: &str) -> Result<Vec<()>>;

    /// Get the number of handlers registered for an event type.
    async fn handler_count(&self, event_type: &str) -> Result<usize>;

    /// Unregister all handlers for an event type.
    async fn unregister_all(&self, event_type: &str) -> Result<()>;
}

/// Generic event handler: react to events.
#[async_trait]
pub trait EventHandler: Send + Sync {
    async fn handle(&self) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metadata_with_correlation() {
        let corr_id = Uuid::new_v4();
        let meta = EventMetadata::new("actor1".to_string())
            .with_correlation(corr_id)
            .with_tags(vec!["test".to_string()]);
        assert_eq!(meta.correlation_id, Some(corr_id));
        assert_eq!(meta.tags.len(), 1);
    }

    #[test]
    fn metadata_with_causation() {
        let caus_id = Uuid::new_v4();
        let meta = EventMetadata::new("actor1".to_string())
            .with_causation(caus_id)
            .with_tags(vec!["batch".to_string()]);
        assert_eq!(meta.causation_id, Some(caus_id));
    }

    // Mock aggregate for testing
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum MockEvent {
        Created { id: String },
        Updated { value: i32 },
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct MockAggregate {
        id: String,
        value: i32,
        pending: Vec<MockEvent>,
    }

    impl EventSourced for MockAggregate {
        type Event = MockEvent;
        type Id = String;

        fn id(&self) -> &Self::Id {
            &self.id
        }

        fn pending_events(&self) -> &[Self::Event] {
            &self.pending
        }

        fn clear_events(&mut self) {
            self.pending.clear();
        }

        fn apply_event(&mut self, event: Self::Event) -> Result<()> {
            match event {
                MockEvent::Created { id } => {
                    self.id = id;
                    self.value = 0;
                    Ok(())
                }
                MockEvent::Updated { value } => {
                    self.value = value;
                    Ok(())
                }
            }
        }

        fn emit_event(&mut self, event: Self::Event) {
            self.pending.push(event);
        }
    }

    // Traces to: FR-PHENO-008
    #[test]
    fn mock_aggregate_emit_and_pending() {
        let mut agg = MockAggregate {
            id: "a1".to_string(),
            value: 0,
            pending: vec![],
        };

        agg.emit_event(MockEvent::Updated { value: 42 });
        assert_eq!(agg.pending_events().len(), 1);

        agg.clear_events();
        assert!(agg.pending_events().is_empty());
    }

    // Traces to: FR-PHENO-008
    #[test]
    fn mock_aggregate_apply_event() {
        let mut agg = MockAggregate {
            id: "a1".to_string(),
            value: 0,
            pending: vec![],
        };

        let event = MockEvent::Updated { value: 99 };
        agg.apply_event(event).unwrap();
        assert_eq!(agg.value, 99);
    }

    // Traces to: FR-PHENO-008
    #[test]
    fn metadata_default_fields() {
        let meta = EventMetadata::new("test-actor".to_string());
        assert_eq!(meta.actor, "test-actor");
        assert!(meta.correlation_id.is_none());
        assert!(meta.causation_id.is_none());
        assert!(meta.tags.is_empty());
    }
}
