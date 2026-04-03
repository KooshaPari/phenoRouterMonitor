use std::sync::{Arc, Mutex};
use std::time::Duration;

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

    let bus_clone = bus.clone();
    let handler = Arc::new(FnHandler(move |env: &Envelope| {
        if let Some(reply_to) = &env.reply_to {
            let reply = Envelope::new(&Subject::new(reply_to), serde_json::json!({"answer": 42}));
            let bus_inner = bus_clone.clone();
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
    let req = Envelope::new(&Subject::new("agileplus.rpc.nobody"), serde_json::json!({}));
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
