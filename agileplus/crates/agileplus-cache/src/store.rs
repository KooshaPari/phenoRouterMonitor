//! Typed cache store with serde serialization.

use crate::pool::CachePool;
use async_trait::async_trait;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Redis error: {0}")]
    RedisError(String),
    #[error("Key not found")]
    NotFound,
    #[error("Connection error: {0}")]
    ConnectionError(String),
}

#[async_trait]
pub trait CacheStore: Send + Sync {
    async fn get<T: for<'de> Deserialize<'de> + Send>(
        &self,
        key: &str,
    ) -> Result<Option<T>, CacheError>;

    async fn set<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
        ttl: Option<Duration>,
    ) -> Result<(), CacheError>;

    async fn delete(&self, key: &str) -> Result<(), CacheError>;

    async fn exists(&self, key: &str) -> Result<bool, CacheError>;
}

/// Redis/Dragonfly-backed cache store.
pub struct RedisCacheStore {
    pool: CachePool,
    default_ttl: Duration,
}

impl RedisCacheStore {
    pub fn new(pool: CachePool, default_ttl_secs: u64) -> Self {
        Self {
            pool,
            default_ttl: Duration::from_secs(default_ttl_secs),
        }
    }
}

#[async_trait]
impl CacheStore for RedisCacheStore {
    async fn get<T: for<'de> Deserialize<'de> + Send>(
        &self,
        key: &str,
    ) -> Result<Option<T>, CacheError> {
        let mut conn = self
            .pool
            .get_connection()
            .await
            .map_err(|e| CacheError::ConnectionError(e.to_string()))?;

        let value: Option<String> = conn
            .get(key)
            .await
            .map_err(|e| CacheError::RedisError(e.to_string()))?;

        match value {
            Some(v) => serde_json::from_str(&v)
                .map(Some)
                .map_err(|e| CacheError::SerializationError(e.to_string())),
            None => Ok(None),
        }
    }

    async fn set<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
        ttl: Option<Duration>,
    ) -> Result<(), CacheError> {
        let mut conn = self
            .pool
            .get_connection()
            .await
            .map_err(|e| CacheError::ConnectionError(e.to_string()))?;

        let serialized = serde_json::to_string(value)
            .map_err(|e| CacheError::SerializationError(e.to_string()))?;

        let ttl_secs = ttl.unwrap_or(self.default_ttl).as_secs() as i64;

        conn.set_ex::<_, _, ()>(key, &serialized, ttl_secs as u64)
            .await
            .map_err(|e| CacheError::RedisError(e.to_string()))?;

        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<(), CacheError> {
        let mut conn = self
            .pool
            .get_connection()
            .await
            .map_err(|e| CacheError::ConnectionError(e.to_string()))?;

        conn.del::<_, ()>(key)
            .await
            .map_err(|e| CacheError::RedisError(e.to_string()))?;

        Ok(())
    }

    async fn exists(&self, key: &str) -> Result<bool, CacheError> {
        let mut conn = self
            .pool
            .get_connection()
            .await
            .map_err(|e| CacheError::ConnectionError(e.to_string()))?;

        conn.exists(key)
            .await
            .map_err(|e| CacheError::RedisError(e.to_string()))
    }
}
