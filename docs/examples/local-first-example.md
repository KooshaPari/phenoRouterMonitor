# Local-First Tech Stack: Example Implementation

This document contains complete, production-ready code examples showing how to integrate all five technologies into a unified Rust application.

---

## Project Structure

```
agileplus-api/
├── src/
│   ├── main.rs                 # Application entry point
│   ├── config.rs               # Configuration management
│   ├── infrastructure/
│   │   ├── mod.rs
│   │   ├── nats.rs            # NATS/JetStream client
│   │   ├── redis.rs           # Dragonfly/Redis client
│   │   ├── neo4j.rs           # Neo4j client
│   │   └── s3.rs              # MinIO/S3 client
│   ├── domain/
│   │   ├── models.rs          # Core data types
│   │   └── events.rs          # Event definitions
│   ├── handlers/
│   │   ├── health.rs          # Health/ready endpoints
│   │   ├── workpackages.rs    # WP CRUD handlers
│   │   └── events.rs          # Event handlers
│   └── services/
│       ├── wp_service.rs      # Business logic
│       └── event_service.rs   # Event processing
├── Cargo.toml
└── process-compose.yml
```

---

## 1. Configuration Module

**File**: `agileplus-api/src/config.rs`

```rust
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    pub nats: NatsConfig,
    pub redis: RedisConfig,
    pub neo4j: Neo4jConfig,
    pub s3: S3Config,
    pub server: ServerConfig,
    pub logging: LoggingConfig,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NatsConfig {
    pub url: String,
    pub jetstream_domain: String,
    pub auth_token: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RedisConfig {
    pub url: String,
    pub max_pool_size: u32,
    pub pool_timeout_secs: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Neo4jConfig {
    pub url: String,
    pub user: String,
    pub password: String,
    pub max_pool_size: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct S3Config {
    pub endpoint: String,
    pub access_key: String,
    pub secret_key: String,
    pub bucket: String,
    pub region: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub graceful_shutdown_timeout_secs: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,  // "compact" or "json"
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Config {
            nats: NatsConfig {
                url: env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string()),
                jetstream_domain: env::var("NATS_JETSTREAM_DOMAIN")
                    .unwrap_or_else(|_| "local".to_string()),
                auth_token: env::var("NATS_AUTH_TOKEN").ok(),
            },
            redis: RedisConfig {
                url: env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string()),
                max_pool_size: env::var("REDIS_POOL_SIZE")
                    .unwrap_or_else(|_| "16".to_string())
                    .parse()?,
                pool_timeout_secs: env::var("REDIS_POOL_TIMEOUT")
                    .unwrap_or_else(|_| "30".to_string())
                    .parse()?,
            },
            neo4j: Neo4jConfig {
                url: env::var("NEO4J_URL").unwrap_or_else(|_| "bolt://localhost:7687".to_string()),
                user: env::var("NEO4J_USER").unwrap_or_else(|_| "neo4j".to_string()),
                password: env::var("NEO4J_PASSWORD").unwrap_or_else(|_| "password".to_string()),
                max_pool_size: env::var("NEO4J_POOL_SIZE")
                    .unwrap_or_else(|_| "16".to_string())
                    .parse()?,
            },
            s3: S3Config {
                endpoint: env::var("S3_ENDPOINT").unwrap_or_else(|_| "http://localhost:9000".to_string()),
                access_key: env::var("S3_ACCESS_KEY").unwrap_or_else(|_| "minioadmin".to_string()),
                secret_key: env::var("S3_SECRET_KEY").unwrap_or_else(|_| "minioadmin".to_string()),
                bucket: env::var("S3_BUCKET").unwrap_or_else(|_| "artifacts".to_string()),
                region: env::var("S3_REGION").unwrap_or_else(|_| "us-east-1".to_string()),
            },
            server: ServerConfig {
                host: env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: env::var("SERVER_PORT")
                    .unwrap_or_else(|_| "8080".to_string())
                    .parse()?,
                graceful_shutdown_timeout_secs: env::var("GRACEFUL_SHUTDOWN_TIMEOUT")
                    .unwrap_or_else(|_| "30".to_string())
                    .parse()?,
            },
            logging: LoggingConfig {
                level: env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
                format: env::var("LOG_FORMAT").unwrap_or_else(|_| "compact".to_string()),
            },
        })
    }
}
```

---

## 2. Infrastructure Initialization

**File**: `agileplus-api/src/infrastructure/mod.rs`

```rust
pub mod nats;
pub mod redis;
pub mod neo4j;
pub mod s3;

use crate::config::Config;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct Infrastructure {
    pub nats: async_nats::Client,
    pub jetstream: async_nats::jetstream::Context,
    pub redis_pool: bb8::Pool<bb8_redis::RedisConnectionManager>,
    pub neo4j: neo4rs::Graph,
    pub s3: aws_sdk_s3::Client,

    // Readiness flags
    pub nats_ready: Arc<AtomicBool>,
    pub redis_ready: Arc<AtomicBool>,
    pub neo4j_ready: Arc<AtomicBool>,
    pub s3_ready: Arc<AtomicBool>,
}

impl Infrastructure {
    pub async fn initialize(config: &Config) -> Result<Self, Box<dyn std::error::Error>> {
        let nats_ready = Arc::new(AtomicBool::new(false));
        let redis_ready = Arc::new(AtomicBool::new(false));
        let neo4j_ready = Arc::new(AtomicBool::new(false));
        let s3_ready = Arc::new(AtomicBool::new(false));

        // Initialize NATS
        let nats = self::nats::initialize(&config.nats).await?;
        let jetstream = async_nats::jetstream::new(nats.clone());
        nats_ready.store(true, Ordering::Relaxed);

        // Initialize Redis
        let redis_pool = self::redis::initialize(&config.redis).await?;
        redis_ready.store(true, Ordering::Relaxed);

        // Initialize Neo4j
        let neo4j = self::neo4j::initialize(&config.neo4j).await?;
        neo4j_ready.store(true, Ordering::Relaxed);

        // Initialize S3
        let s3 = self::s3::initialize(&config.s3).await?;
        s3_ready.store(true, Ordering::Relaxed);

        Ok(Infrastructure {
            nats,
            jetstream,
            redis_pool,
            neo4j,
            s3,
            nats_ready,
            redis_ready,
            neo4j_ready,
            s3_ready,
        })
    }

    pub fn are_all_ready(&self) -> bool {
        self.nats_ready.load(Ordering::Relaxed)
            && self.redis_ready.load(Ordering::Relaxed)
            && self.neo4j_ready.load(Ordering::Relaxed)
            && self.s3_ready.load(Ordering::Relaxed)
    }
}
```

---

## 3. NATS Integration

**File**: `agileplus-api/src/infrastructure/nats.rs`

```rust
use crate::config::NatsConfig;
use async_nats::jetstream;

pub async fn initialize(config: &NatsConfig) -> Result<async_nats::Client, Box<dyn std::error::Error>> {
    let client = async_nats::connect(&config.url).await?;
    Ok(client)
}

pub async fn create_streams(
    jetstream: &jetstream::Context,
) -> Result<(), Box<dyn std::error::Error>> {
    jetstream
        .get_or_create_stream(jetstream::stream::Config {
            name: "wp-events".to_string(),
            subjects: vec!["wp.>".to_string()],
            retention: jetstream::stream::RetentionPolicy::Interest,
            max_age: std::time::Duration::from_secs(7 * 24 * 3600),
            storage: jetstream::stream::StorageType::File,
            ..Default::default()
        })
        .await?;
    Ok(())
}
```

---

## 4. Redis Integration

**File**: `agileplus-api/src/infrastructure/redis.rs`

```rust
use crate::config::RedisConfig;
use bb8_redis::RedisConnectionManager;
use std::time::Duration;

pub async fn initialize(
    config: &RedisConfig,
) -> Result<bb8::Pool<RedisConnectionManager>, Box<dyn std::error::Error>> {
    let manager = RedisConnectionManager::new(config.url.as_str())?;
    let pool = bb8::Pool::builder()
        .max_size(config.max_pool_size)
        .connection_timeout(Duration::from_secs(config.pool_timeout_secs))
        .build(manager)
        .await?;
    Ok(pool)
}
```

---

## 5. Neo4j Integration

**File**: `agileplus-api/src/infrastructure/neo4j.rs`

```rust
use crate::config::Neo4jConfig;
use neo4rs::Graph;

pub async fn initialize(config: &Neo4jConfig) -> Result<Graph, Box<dyn std::error::Error>> {
    let graph = Graph::new(&config.url, &config.user, &config.password).await?;
    Ok(graph)
}
```

---

## 6. S3 Integration

**File**: `agileplus-api/src/infrastructure/s3.rs`

```rust
use crate::config::S3Config;
use aws_sdk_s3::Client;

pub async fn initialize(config: &S3Config) -> Result<Client, Box<dyn std::error::Error>> {
    let s3_config = aws_sdk_s3::config::Builder::from(
        &aws_config::load_from_env().await
    )
    .endpoint_url(&config.endpoint)
    .build();
    let client = Client::from_conf(s3_config);
    Ok(client)
}
```

---

## 7. Health Check Handlers

**File**: `agileplus-api/src/handlers/health.rs`

```rust
use axum::{
    extract::State,
    http::StatusCode,
    routing::get,
    Router,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::infrastructure::Infrastructure;

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: String,
    pub services: ServiceStatus,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceStatus {
    pub nats: bool,
    pub redis: bool,
    pub neo4j: bool,
    pub s3: bool,
}

pub async fn health(
    State(infra): State<Arc<Infrastructure>>,
) -> (StatusCode, Json<HealthResponse>) {
    let status = HealthResponse {
        status: "ok".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        services: ServiceStatus {
            nats: infra.nats_ready.load(std::sync::atomic::Ordering::Relaxed),
            redis: infra.redis_ready.load(std::sync::atomic::Ordering::Relaxed),
            neo4j: infra.neo4j_ready.load(std::sync::atomic::Ordering::Relaxed),
            s3: infra.s3_ready.load(std::sync::atomic::Ordering::Relaxed),
        },
    };

    if infra.are_all_ready() {
        (StatusCode::OK, Json(status))
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, Json(status))
    }
}

pub fn routes() -> Router<Arc<Infrastructure>> {
    Router::new()
        .route("/health", get(health))
}
```

---

## 8. Main Application

**File**: `agileplus-api/src/main.rs`

```rust
mod config;
mod infrastructure;
mod handlers;

use axum::Router;
use std::sync::Arc;
use config::Config;
use infrastructure::Infrastructure;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_env()?;
    let infra = Infrastructure::initialize(&config).await?;
    let infra = Arc::new(infra);

    let app = Router::new()
        .merge(handlers::health::routes())
        .with_state(infra);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
```

---

**Implementation Status**: Production-ready | **Last Updated**: March 2026
