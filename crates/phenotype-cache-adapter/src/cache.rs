//! Cache implementation.

use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::hash::Hash;
use std::fmt::Debug;

pub trait Cache<K, V>: Send + Sync {
    fn get(&self, key: &K) -> Option<V> where K: Hash + Eq;
    fn set(&self, key: K, value: V) where K: Hash + Eq + Clone;
}

pub struct TwoTierCache<K, V> {
    l1: Arc<DashMap<K, V>>,
    l2: Arc<RwLock<HashMap<K, V>>>,
}

impl<K, V> TwoTierCache<K, V> {
    pub fn new() -> Self {
        Self {
            l1: Arc::new(DashMap::new()),
            l2: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl<K, V> Default for TwoTierCache<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> Cache<K, V> for TwoTierCache<K, V>
where K: Hash + Eq + Clone + Debug {
    fn get(&self, key: &K) -> Option<V> {
        if let Some(v) = self.l1.get(key) {
            return Some(v.clone());
        }
        let l2 = self.l2.clone();
        let key = key.clone();
        let result = tokio::task::block_in_place(|| {
            futures::executor::block_on(async { l2.read().await.get(&key).cloned() })
        });
        result
    }

    fn set(&self, key: K, value: V) {
        self.l1.insert(key.clone(), value.clone());
        let l2 = self.l2.clone();
        tokio::spawn(async move {
            let mut cache = l2.write().await;
            cache.insert(key, value);
        });
    }
}
