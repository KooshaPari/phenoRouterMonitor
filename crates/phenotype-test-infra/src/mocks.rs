//! Mock implementations.
use dashmap::DashMap;
use serde_json::Value;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct MockRepository {
    store: Arc<DashMap<String, Value>>,
}

impl MockRepository {
    pub fn new() -> Self {
        Self {
            store: Arc::new(DashMap::new()),
        }
    }

    pub fn insert(&self, key: impl Into<String>, value: Value) {
        self.store.insert(key.into(), value);
    }

    pub fn get(&self, key: &str) -> Option<Value> {
        self.store.get(key).map(|entry| entry.clone())
    }

    pub fn len(&self) -> usize {
        self.store.len()
    }
}

impl Default for MockRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct MockCache {
    store: Arc<DashMap<String, Value>>,
    hits: Arc<std::sync::atomic::AtomicU64>,
    misses: Arc<std::sync::atomic::AtomicU64>,
}

impl MockCache {
    pub fn new() -> Self {
        Self {
            store: Arc::new(DashMap::new()),
            hits: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            misses: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }

    pub fn set(&self, key: impl Into<String>, value: Value) {
        self.store.insert(key.into(), value);
    }

    pub fn get(&self, key: &str) -> Option<Value> {
        if let Some(entry) = self.store.get(key) {
            self.hits.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            Some(entry.clone())
        } else {
            self.misses.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            None
        }
    }

    pub fn hit_count(&self) -> u64 {
        self.hits.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn miss_count(&self) -> u64 {
        self.misses.load(std::sync::atomic::Ordering::Relaxed)
    }
}

impl Default for MockCache {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct MockEventStore {
    events: Arc<parking_lot::Mutex<Vec<Value>>>,
}

impl MockEventStore {
    pub fn new() -> Self {
        Self {
            events: Arc::new(parking_lot::Mutex::new(Vec::new())),
        }
    }

    pub fn append(&self, event: Value) {
        self.events.lock().push(event);
    }

    pub fn event_count(&self) -> usize {
        self.events.lock().len()
    }

    pub fn has_event<F: Fn(&Value) -> bool>(&self, predicate: F) -> bool {
        self.events.lock().iter().any(predicate)
    }
}

impl Default for MockEventStore {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct MockLogger {
    messages: Arc<parking_lot::Mutex<Vec<String>>>,
}

impl MockLogger {
    pub fn new() -> Self {
        Self {
            messages: Arc::new(parking_lot::Mutex::new(Vec::new())),
        }
    }

    pub fn info(&self, message: impl Into<String>) {
        self.messages.lock().push(message.into());
    }

    pub fn message_count(&self) -> usize {
        self.messages.lock().len()
    }
}

impl Default for MockLogger {
    fn default() -> Self {
        Self::new()
    }
}
