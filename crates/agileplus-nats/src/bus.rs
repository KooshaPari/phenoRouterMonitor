//! Core `EventBus` trait and in-memory implementation.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use tokio::sync::oneshot;

use crate::config::NatsConfig;
use crate::envelope::Envelope;
use crate::handler::Handler;
use crate::health::BusHealth;
use crate::subject::Subject;

/// Errors produced by the event bus.
#[derive(Debug, thiserror::Error)]
pub enum EventBusError {
    #[error("Connection error: {0}")]
    ConnectionError(String),
    #[error("Publish error: {0}")]
    PublishError(String),
    #[error("Subscribe error: {0}")]
    SubscribeError(String),
    #[error("Request timeout")]
    Timeout,
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Handler error: {0}")]
    HandlerError(String),
}

/// Trait abstracting the event bus backend. The default implementation is an
/// in-memory bus suitable for testing; a NATS-backed implementation can be
/// provided when a real broker is available.
#[async_trait]
pub trait EventBus: Send + Sync {
    /// Publish an envelope to a subject.
    async fn publish(&self, envelope: Envelope) -> Result<(), EventBusError>;

    /// Subscribe a handler to a subject pattern. Returns a subscription ID.
    async fn subscribe(
        &self,
        subject: Subject,
        handler: Arc<dyn Handler>,
    ) -> Result<String, EventBusError>;

    /// Unsubscribe by subscription ID.
    async fn unsubscribe(&self, subscription_id: &str) -> Result<(), EventBusError>;

    /// Request/reply: publish and wait for a single response.
    async fn request(
        &self,
        envelope: Envelope,
        timeout: Duration,
    ) -> Result<Envelope, EventBusError>;

    /// Check the health of the bus connection.
    async fn health(&self) -> BusHealth;
}

/// The primary event bus store wrapping a backend implementation.
pub struct EventBusStore {
    backend: Box<dyn EventBus>,
    #[allow(dead_code)]
    config: NatsConfig,
}

impl EventBusStore {
    /// Create a new `EventBusStore` with the given backend.
    pub fn new(config: NatsConfig, backend: Box<dyn EventBus>) -> Self {
        Self { backend, config }
    }

    /// Create an `EventBusStore` backed by the in-memory implementation.
    pub fn in_memory(config: NatsConfig) -> Self {
        Self {
            backend: Box::new(InMemoryBus::new()),
            config,
        }
    }

    pub fn backend(&self) -> &dyn EventBus {
        &*self.backend
    }

    pub async fn publish(&self, envelope: Envelope) -> Result<(), EventBusError> {
        self.backend.publish(envelope).await
    }

    pub async fn subscribe(
        &self,
        subject: Subject,
        handler: Arc<dyn Handler>,
    ) -> Result<String, EventBusError> {
        self.backend.subscribe(subject, handler).await
    }

    pub async fn unsubscribe(&self, id: &str) -> Result<(), EventBusError> {
        self.backend.unsubscribe(id).await
    }

    pub async fn request(
        &self,
        envelope: Envelope,
        timeout: Duration,
    ) -> Result<Envelope, EventBusError> {
        self.backend.request(envelope, timeout).await
    }

    pub async fn health(&self) -> BusHealth {
        self.backend.health().await
    }
}

// ---------------------------------------------------------------------------
// In-memory backend for testing
// ---------------------------------------------------------------------------

struct Subscription {
    subject: Subject,
    handler: Arc<dyn Handler>,
}

/// An in-memory event bus that dispatches published messages to matching
/// subscriptions synchronously. Suitable for unit and integration tests.
pub struct InMemoryBus {
    subscriptions: Mutex<HashMap<String, Subscription>>,
    /// All published envelopes, for test assertions.
    published: Mutex<Vec<Envelope>>,
    /// Pending request/reply waiters: reply-subject -> sender.
    reply_waiters: Mutex<HashMap<String, oneshot::Sender<Envelope>>>,
    next_sub_id: Mutex<u64>,
}

impl InMemoryBus {
    pub fn new() -> Self {
        Self {
            subscriptions: Mutex::new(HashMap::new()),
            published: Mutex::new(Vec::new()),
            reply_waiters: Mutex::new(HashMap::new()),
            next_sub_id: Mutex::new(0),
        }
    }

    /// Return all envelopes published so far (test helper).
    pub fn published(&self) -> Vec<Envelope> {
        self.published.lock().unwrap().clone()
    }
}

impl Default for InMemoryBus {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EventBus for InMemoryBus {
    async fn publish(&self, envelope: Envelope) -> Result<(), EventBusError> {
        // Check if the published subject matches any reply waiter.
        {
            let mut waiters = self.reply_waiters.lock().unwrap();
            if let Some(sender) = waiters.remove(&envelope.subject) {
                let _ = sender.send(envelope.clone());
            }
        }

        // Record the envelope.
        self.published.lock().unwrap().push(envelope.clone());

        // Collect matching handlers while holding the lock, then release
        // the lock before awaiting (MutexGuard is not Send).
        let handlers: Vec<Arc<dyn Handler>> = {
            let subs = self.subscriptions.lock().unwrap();
            let concrete = Subject::new(&envelope.subject);
            subs.values()
                .filter(|sub| sub.subject.matches(&concrete))
                .map(|sub| sub.handler.clone())
                .collect()
        };

        for handler in handlers {
            let _ = handler.handle(&envelope).await;
        }

        Ok(())
    }

    async fn subscribe(
        &self,
        subject: Subject,
        handler: Arc<dyn Handler>,
    ) -> Result<String, EventBusError> {
        let mut id_counter = self.next_sub_id.lock().unwrap();
        *id_counter += 1;
        let id = format!("sub-{}", *id_counter);
        drop(id_counter);

        self.subscriptions
            .lock()
            .unwrap()
            .insert(id.clone(), Subscription { subject, handler });

        Ok(id)
    }

    async fn unsubscribe(&self, subscription_id: &str) -> Result<(), EventBusError> {
        self.subscriptions
            .lock()
            .unwrap()
            .remove(subscription_id);
        Ok(())
    }

    async fn request(
        &self,
        mut envelope: Envelope,
        timeout: Duration,
    ) -> Result<Envelope, EventBusError> {
        // Create an inbox subject for the reply.
        let inbox = format!("_INBOX.{}", uuid::Uuid::new_v4());
        envelope.reply_to = Some(inbox.clone());

        // Register a waiter for the inbox subject.
        let (tx, rx) = oneshot::channel();
        self.reply_waiters.lock().unwrap().insert(inbox, tx);

        // Publish the request.
        self.publish(envelope).await?;

        // Wait for the reply with a timeout.
        match tokio::time::timeout(timeout, rx).await {
            Ok(Ok(reply)) => Ok(reply),
            Ok(Err(_)) => Err(EventBusError::Timeout),
            Err(_) => Err(EventBusError::Timeout),
        }
    }

    async fn health(&self) -> BusHealth {
        BusHealth::Connected
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handler::FnHandler;

    #[tokio::test]
    async fn in_memory_health() {
        let store = EventBusStore::in_memory(NatsConfig::default());
        assert_eq!(store.health().await, BusHealth::Connected);
    }

    #[tokio::test]
    async fn publish_and_subscribe() {
        let bus = InMemoryBus::new();

        let received: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let received_clone = received.clone();
        let handler = Arc::new(FnHandler(move |env: &Envelope| {
            received_clone.lock().unwrap().push(env.subject.clone());
            Ok(())
        }));

        bus.subscribe(Subject::all_for_entity("agileplus", "feature"), handler)
            .await
            .unwrap();

        let env = Envelope::new(
            &Subject::for_event("agileplus", "feature", 1, "created"),
            serde_json::json!({"title": "Login"}),
        );
        bus.publish(env).await.unwrap();

        let got = received.lock().unwrap();
        assert_eq!(got.len(), 1);
        assert_eq!(got[0], "agileplus.feature.1.created");
    }

    #[tokio::test]
    async fn unsubscribe_stops_delivery() {
        let bus = InMemoryBus::new();

        let count: Arc<Mutex<u32>> = Arc::new(Mutex::new(0));
        let count_clone = count.clone();
        let handler = Arc::new(FnHandler(move |_: &Envelope| {
            *count_clone.lock().unwrap() += 1;
            Ok(())
        }));

        let sub_id = bus
            .subscribe(Subject::all_for_entity("agileplus", "feature"), handler)
            .await
            .unwrap();

        let env = Envelope::new(
            &Subject::for_event("agileplus", "feature", 1, "created"),
            serde_json::json!({}),
        );
        bus.publish(env).await.unwrap();
        assert_eq!(*count.lock().unwrap(), 1);

        bus.unsubscribe(&sub_id).await.unwrap();

        let env2 = Envelope::new(
            &Subject::for_event("agileplus", "feature", 2, "created"),
            serde_json::json!({}),
        );
        bus.publish(env2).await.unwrap();
        assert_eq!(*count.lock().unwrap(), 1);
    }

    #[tokio::test]
    async fn request_reply() {
        let bus = Arc::new(InMemoryBus::new());

        // Subscribe a responder that replies to the reply_to subject.
        let bus_clone = bus.clone();
        let handler = Arc::new(FnHandler(move |env: &Envelope| {
            if let Some(reply_to) = &env.reply_to {
                let reply = Envelope::new(
                    &Subject::new(reply_to),
                    serde_json::json!({"answer": 42}),
                );
                let bus_inner = bus_clone.clone();
                // Spawn the reply asynchronously to avoid deadlock.
                tokio::spawn(async move {
                    let _ = bus_inner.publish(reply).await;
                });
            }
            Ok(())
        }));

        bus.subscribe(Subject::new("agileplus.rpc.triage"), handler)
            .await
            .unwrap();

        let req = Envelope::new(
            &Subject::new("agileplus.rpc.triage"),
            serde_json::json!({"feature_id": 1}),
        );
        let reply = bus.request(req, Duration::from_secs(2)).await.unwrap();
        assert_eq!(reply.payload["answer"], 42);
    }

    #[tokio::test]
    async fn request_timeout() {
        let bus = InMemoryBus::new();
        let req = Envelope::new(
            &Subject::new("agileplus.rpc.nobody"),
            serde_json::json!({}),
        );
        let result = bus.request(req, Duration::from_millis(50)).await;
        assert!(matches!(result, Err(EventBusError::Timeout)));
    }

    #[tokio::test]
    async fn event_bus_store_delegates() {
        let store = EventBusStore::in_memory(NatsConfig::default());
        let env = Envelope::new(
            &Subject::for_event("agileplus", "wp", 7, "created"),
            serde_json::json!({"title": "WP07"}),
        );
        store.publish(env).await.unwrap();
        assert_eq!(store.health().await, BusHealth::Connected);
    }
}
