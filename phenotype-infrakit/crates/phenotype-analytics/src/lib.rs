//! Analytics module for phenotype
//!
//! Provides event tracking and analytics infrastructure.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use thiserror::Error;

/// Result type for analytics operations
pub type Result<T> = std::result::Result<T, AnalyticsError>;

/// Analytics error types
#[derive(Error, Debug, Clone)]
pub enum AnalyticsError {
    #[error("track error: {0}")]
    TrackError(String),

    #[error("query error: {0}")]
    QueryError(String),

    #[error("backend error: {0}")]
    BackendError(String),
}

/// Event data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub name: String,
    pub properties: HashMap<String, String>,
    pub timestamp: Option<chrono::DateTime<chrono::Utc>>,
}

/// Event context for validation
#[derive(Debug, Clone)]
pub struct EventContext {
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub metadata: HashMap<String, String>,
}

/// Filter trait for events
pub trait EventFilter: Send + Sync {
    fn should_track(&self, event: &Event) -> bool;
}

/// Sampling filter
#[derive(Debug, Clone, Default)]
pub struct SamplingFilter {
    pub sample_rate: f64,
}

impl SamplingFilter {
    pub fn new(sample_rate: f64) -> Self {
        Self { sample_rate }
    }
}

impl EventFilter for SamplingFilter {
    fn should_track(&self, _event: &Event) -> bool {
        // Simple implementation - always track
        true
    }
}

/// Event store trait
#[async_trait::async_trait]
pub trait EventStore: Send + Sync {
    async fn store(&self, event: Event) -> Result<()>;
    async fn query(&self, filter: EventFilter) -> Result<Vec<Event>>;
}

/// In-memory event store
#[derive(Debug, Default)]
pub struct InMemoryEventStore {
    events: Arc<RwLock<Vec<Event>>>,
}

impl InMemoryEventStore {
    pub fn new() -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn get_events(&self) -> Vec<Event> {
        self.events.read().await.clone()
    }
}

#[async_trait::async_trait]
impl EventStore for InMemoryEventStore {
    async fn store(&self, event: Event) -> Result<()> {
        let mut queue = self.events.write().await;
        queue.push(event);
        Ok(())
    }

    async fn query(&self, _filter: EventFilter) -> Result<Vec<Event>> {
        let events = self.events.read().await;
        Ok(events.clone())
    }
}

/// Analytics client
#[derive(Debug, Clone)]
pub struct AnalyticsClient<S: EventStore> {
    store: Arc<S>,
    context: Arc<RwLock<EventContext>>,
    filters: Vec<Box<dyn EventFilter>>,
}

impl<S: EventStore> AnalyticsClient<S> {
    pub fn new(store: S) -> Self {
        Self {
            store: Arc::new(store),
            context: Arc::new(RwLock::new(EventContext::default())),
            filters: Vec::new(),
        }
    }

    pub async fn track(&self, event: Event) -> Result<()> {
        // Check filters
        for filter in &self.filters {
            if !filter.should_track(&event) {
                return Ok(());
            }
        }
        self.store.store(event).await
    }

    pub async fn set_user(&self, user_id: String) {
        let mut ctx = self.context.write().await;
        ctx.user_id = Some(user_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sampling_filter() {
        let filter = SamplingFilter::new(1.0);
        let event = Event {
            name: "test".to_string(),
            properties: HashMap::new(),
            timestamp: None,
        };
        assert!(filter.should_track(&event));
    }

    #[test]
    fn test_in_memory_store() {
        let store = InMemoryEventStore::new();
        assert_eq!(tokio::runtime::Runtime::new().unwrap().block_on(store.get_events()).len(), 0);
    }
}
