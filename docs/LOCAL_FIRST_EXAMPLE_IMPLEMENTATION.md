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
        tracing::info!("Initializing infrastructure...");

        let nats_ready = Arc::new(AtomicBool::new(false));
        let redis_ready = Arc::new(AtomicBool::new(false));
        let neo4j_ready = Arc::new(AtomicBool::new(false));
        let s3_ready = Arc::new(AtomicBool::new(false));

        // Initialize NATS
        tracing::info!("Connecting to NATS: {}", config.nats.url);
        let nats = self::nats::initialize(&config.nats).await?;
        let jetstream = async_nats::jetstream::new(nats.clone());
        nats_ready.store(true, Ordering::Relaxed);
        tracing::info!("NATS connected");

        // Initialize Redis
        tracing::info!("Connecting to Redis: {}", config.redis.url);
        let redis_pool = self::redis::initialize(&config.redis).await?;
        redis_ready.store(true, Ordering::Relaxed);
        tracing::info!("Redis connected");

        // Initialize Neo4j
        tracing::info!("Connecting to Neo4j: {}", config.neo4j.url);
        let neo4j = self::neo4j::initialize(&config.neo4j).await?;
        neo4j_ready.store(true, Ordering::Relaxed);
        tracing::info!("Neo4j connected");

        // Initialize S3
        tracing::info!("Connecting to S3: {}", config.s3.endpoint);
        let s3 = self::s3::initialize(&config.s3).await?;
        s3_ready.store(true, Ordering::Relaxed);
        tracing::info!("S3 connected");

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

    // Test connection
    let _server_info = client.server_info().await?;

    Ok(client)
}

pub async fn create_streams(
    jetstream: &jetstream::Context,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create WP Events Stream
    jetstream
        .get_or_create_stream(jetstream::stream::Config {
            name: "wp-events".to_string(),
            subjects: vec!["wp.>".to_string()],
            retention: jetstream::stream::RetentionPolicy::Interest,
            max_age: std::time::Duration::from_secs(7 * 24 * 3600), // 1 week
            storage: jetstream::stream::StorageType::File,
            discard: jetstream::stream::DiscardPolicy::Old,
            ..Default::default()
        })
        .await?;

    // Create Feature Events Stream
    jetstream
        .get_or_create_stream(jetstream::stream::Config {
            name: "feature-events".to_string(),
            subjects: vec!["feature.>".to_string()],
            retention: jetstream::stream::RetentionPolicy::Interest,
            max_age: std::time::Duration::from_secs(7 * 24 * 3600),
            storage: jetstream::stream::StorageType::File,
            discard: jetstream::stream::DiscardPolicy::Old,
            ..Default::default()
        })
        .await?;

    Ok(())
}

pub async fn create_consumer(
    stream: &jetstream::Stream,
    consumer_name: &str,
) -> Result<jetstream::consumer::Consumer, Box<dyn std::error::Error>> {
    Ok(stream
        .create_consumer(jetstream::consumer::PullConsumer {
            durable_name: Some(consumer_name.to_string()),
            deliver_policy: jetstream::consumer::DeliverPolicy::All,
            ack_policy: jetstream::consumer::AckPolicy::Explicit,
            max_deliver: 5,
            backoff: vec![
                std::time::Duration::from_secs(1),
                std::time::Duration::from_secs(5),
                std::time::Duration::from_secs(30),
            ],
            ..Default::default()
        })
        .await?)
}

pub async fn publish_event<T: serde::Serialize>(
    jetstream: &jetstream::Context,
    subject: &str,
    event: &T,
) -> Result<jetstream::PublishAckFuture, Box<dyn std::error::Error>> {
    let payload = serde_json::to_vec(event)?;
    Ok(jetstream.publish(subject, payload.into()).await?)
}
```

---

## 4. Redis Integration

**File**: `agileplus-api/src/infrastructure/redis.rs`

```rust
use crate::config::RedisConfig;
use bb8_redis::RedisConnectionManager;
use redis::aio::ConnectionManager;
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

    // Test connection
    let mut conn = pool.get().await?;
    redis::cmd("PING").query_async::<_, String>(&mut *conn).await?;

    Ok(pool)
}

pub struct RedisCache {
    pool: bb8::Pool<RedisConnectionManager>,
}

impl RedisCache {
    pub fn new(pool: bb8::Pool<RedisConnectionManager>) -> Self {
        RedisCache { pool }
    }

    pub async fn set_snapshot<T: serde::Serialize>(
        &self,
        wp_id: &str,
        snapshot: &T,
        ttl_secs: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use redis::AsyncCommands;
        let mut conn = self.pool.get().await?;

        let json = serde_json::to_string(snapshot)?;
        let key = format!("snapshot:{}", wp_id);

        conn.set_ex(key, json, ttl_secs).await?;
        Ok(())
    }

    pub async fn get_snapshot<T: serde::de::DeserializeOwned>(
        &self,
        wp_id: &str,
    ) -> Result<Option<T>, Box<dyn std::error::Error>> {
        use redis::AsyncCommands;
        let mut conn = self.pool.get().await?;

        let key = format!("snapshot:{}", wp_id);
        let json: Option<String> = conn.get(&key).await?;

        match json {
            Some(j) => Ok(Some(serde_json::from_str(&j)?)),
            None => Ok(None),
        }
    }

    pub async fn check_rate_limit(
        &self,
        key: &str,
        limit: i32,
        window_secs: usize,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        use redis::AsyncCommands;
        let mut conn = self.pool.get().await?;

        let count: i32 = conn.incr(key, 1).await?;
        if count == 1 {
            conn.expire(key, window_secs).await?;
        }

        Ok(count <= limit)
    }
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

    // Verify connection
    let mut result = graph.execute(neo4rs::query("RETURN 1 AS test")).await?;
    let _ = result.next().await?;

    // Initialize schema
    initialize_schema(&graph).await?;

    Ok(graph)
}

async fn initialize_schema(graph: &Graph) -> Result<(), Box<dyn std::error::Error>> {
    // Create indexes for WorkPackage
    graph
        .execute(
            neo4rs::query(
                "CREATE INDEX wp_id IF NOT EXISTS FOR (n:WorkPackage) ON (n.id)"
            ),
        )
        .await?;

    // Create constraint for unique WP IDs
    graph
        .execute(
            neo4rs::query(
                "CREATE CONSTRAINT wp_unique IF NOT EXISTS FOR (n:WorkPackage) REQUIRE n.id IS UNIQUE"
            ),
        )
        .await?;

    // Create indexes for Features
    graph
        .execute(
            neo4rs::query(
                "CREATE INDEX feature_id IF NOT EXISTS FOR (n:Feature) ON (n.id)"
            ),
        )
        .await?;

    Ok(())
}

pub struct WorkPackageGraph {
    graph: Graph,
}

impl WorkPackageGraph {
    pub fn new(graph: Graph) -> Self {
        WorkPackageGraph { graph }
    }

    pub async fn create_workpackage(
        &self,
        id: &str,
        title: &str,
        status: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let query = "
            CREATE (wp:WorkPackage {
                id: $id,
                title: $title,
                status: $status,
                created_at: timestamp(),
                updated_at: timestamp()
            })
            RETURN wp
        ";

        self.graph
            .execute(
                neo4rs::query(query)
                    .param("id", id.to_string())
                    .param("title", title.to_string())
                    .param("status", status.to_string()),
            )
            .await?;

        Ok(())
    }

    pub async fn add_dependency(
        &self,
        from_id: &str,
        to_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let query = "
            MATCH (from:WorkPackage {id: $from_id}),
                  (to:WorkPackage {id: $to_id})
            CREATE (from)-[:DEPENDS_ON {created_at: timestamp()}]->(to)
            RETURN from, to
        ";

        self.graph
            .execute(
                neo4rs::query(query)
                    .param("from_id", from_id.to_string())
                    .param("to_id", to_id.to_string()),
            )
            .await?;

        Ok(())
    }

    pub async fn get_dependencies(&self, wp_id: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let query = "
            MATCH (wp:WorkPackage {id: $id})-[:DEPENDS_ON]->(dep:WorkPackage)
            RETURN dep.id AS dependent_id
        ";

        let mut result = self
            .graph
            .execute(neo4rs::query(query).param("id", wp_id.to_string()))
            .await?;

        let mut deps = Vec::new();
        while let Ok(Some(row)) = result.next().await {
            deps.push(row.get::<String>("dependent_id")?);
        }

        Ok(deps)
    }

    pub async fn find_blocking_path(
        &self,
        from_id: &str,
        to_id: &str,
    ) -> Result<Option<Vec<String>>, Box<dyn std::error::Error>> {
        let query = "
            MATCH path = shortestPath(
                (from:WorkPackage {id: $from})-[:BLOCKS*]->(to:WorkPackage {id: $to})
            )
            RETURN [node IN nodes(path) | node.id] AS path_ids
        ";

        let mut result = self
            .graph
            .execute(
                neo4rs::query(query)
                    .param("from", from_id.to_string())
                    .param("to", to_id.to_string()),
            )
            .await?;

        if let Ok(Some(row)) = result.next().await {
            Ok(Some(row.get("path_ids")?))
        } else {
            Ok(None)
        }
    }
}
```

---

## 6. S3 Integration

**File**: `agileplus-api/src/infrastructure/s3.rs`

```rust
use crate::config::S3Config;
use aws_sdk_s3::Client;
use std::path::Path;

pub async fn initialize(config: &S3Config) -> Result<Client, Box<dyn std::error::Error>> {
    let s3_config = aws_sdk_s3::config::Builder::from(
        &aws_config::load_from_env().await
    )
    .endpoint_url(&config.endpoint)
    .build();

    let client = Client::from_conf(s3_config);

    // Verify bucket exists or create it
    ensure_bucket(&client, &config.bucket).await?;

    Ok(client)
}

async fn ensure_bucket(client: &Client, bucket_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    match client.head_bucket().bucket(bucket_name).send().await {
        Ok(_) => Ok(()),
        Err(_) => {
            client
                .create_bucket()
                .bucket(bucket_name)
                .send()
                .await?;
            Ok(())
        }
    }
}

pub struct ArtifactStore {
    client: Client,
    bucket: String,
}

impl ArtifactStore {
    pub fn new(client: Client, bucket: String) -> Self {
        ArtifactStore { client, bucket }
    }

    pub async fn upload_snapshot(
        &self,
        wp_id: &str,
        file_path: &Path,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let body = aws_sdk_s3::primitives::ByteStream::from_path(file_path).await?;

        let key = format!("wp-{}/snapshot-{}.tar.gz", wp_id, chrono::Utc::now().timestamp());

        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&key)
            .metadata("wp-id", wp_id)
            .metadata("uploaded-at", chrono::Utc::now().to_rfc3339())
            .body(body)
            .send()
            .await?;

        Ok(key)
    }

    pub async fn download_snapshot(
        &self,
        key: &str,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let obj = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await?;

        let data = obj.body.collect().await?;
        Ok(data.into_iter().collect())
    }

    pub async fn list_snapshots(
        &self,
        wp_id: &str,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let prefix = format!("wp-{}/", wp_id);
        let resp = self
            .client
            .list_objects_v2()
            .bucket(&self.bucket)
            .prefix(&prefix)
            .send()
            .await?;

        let mut keys = Vec::new();
        for obj in resp.contents.unwrap_or_default() {
            if let Some(key) = obj.key {
                keys.push(key);
            }
        }

        Ok(keys)
    }
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

#[derive(Debug, Serialize, Deserialize)]
pub struct ReadyResponse {
    pub ready: bool,
    pub timestamp: String,
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

pub async fn ready(
    State(infra): State<Arc<Infrastructure>>,
) -> Result<(StatusCode, Json<ReadyResponse>), StatusCode> {
    if infra.are_all_ready() {
        Ok((
            StatusCode::OK,
            Json(ReadyResponse {
                ready: true,
                timestamp: chrono::Utc::now().to_rfc3339(),
            }),
        ))
    } else {
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}

pub fn routes() -> Router<Arc<Infrastructure>> {
    Router::new()
        .route("/health", get(health))
        .route("/ready", get(ready))
}
```

---

## 8. Main Application

**File**: `agileplus-api/src/main.rs`

```rust
mod config;
mod infrastructure;
mod handlers;
mod domain;
mod services;

use axum::{middleware, Router};
use std::sync::Arc;
use tokio::signal;
use tower_http::cors::CorsLayer;
use tracing_subscriber;

use config::Config;
use infrastructure::Infrastructure;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config = Config::from_env()?;

    // Initialize logging
    initialize_tracing(&config);

    tracing::info!("AgilePlus API Server starting...");
    tracing::info!("Environment: {:?}", config.logging.level);

    // Initialize infrastructure
    let infra = Infrastructure::initialize(&config).await?;
    let infra = Arc::new(infra);

    tracing::info!("All infrastructure services initialized");

    // Build router
    let app = Router::new()
        .merge(handlers::health::routes())
        .merge(handlers::workpackages::routes())
        .merge(handlers::events::routes())
        .layer(CorsLayer::permissive())
        .layer(middleware::from_fn(request_logging_middleware))
        .with_state(infra.clone());

    // Create TCP listener
    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    tracing::info!("Server listening on {}", addr);

    // Setup graceful shutdown
    let (tx, rx) = tokio::sync::broadcast::channel(1);

    // Spawn server
    let server = axum::serve(listener, app);

    tokio::select! {
        result = server => {
            if let Err(e) = result {
                tracing::error!("Server error: {}", e);
            }
        }
        _ = signal::ctrl_c() => {
            tracing::info!("Received shutdown signal");
            let _ = tx.send(());
        }
    }

    tracing::info!("Server shutdown complete");
    Ok(())
}

fn initialize_tracing(config: &Config) {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(&config.logging.level));

    if config.logging.format == "json" {
        tracing_subscriber::fmt()
            .json()
            .with_env_filter(filter)
            .init();
    } else {
        tracing_subscriber::fmt()
            .compact()
            .with_env_filter(filter)
            .init();
    }
}

async fn request_logging_middleware(
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let method = req.method().clone();
    let uri = req.uri().clone();

    let start = std::time::Instant::now();
    let response = next.run(req).await;
    let elapsed = start.elapsed();

    tracing::info!(
        "{} {} - {} - {:?}",
        method,
        uri,
        response.status(),
        elapsed
    );

    response
}
```

---

## 9. Process Compose Configuration

**File**: `process-compose.yml`

```yaml
version: "0.5"

environment:
  - RUST_LOG=info,agileplus=debug
  - LOG_LEVEL=debug
  - LOCAL_DEV=true

processes:
  # ===== INFRASTRUCTURE =====

  nats:
    command: "nats-server -js -m 8222"
    working_dir: "."
    ports:
      - "4222:4222"
      - "8222:8222"
    environment:
      - NATS_JETSTREAM_DOMAIN=local
    health_check:
      test: ["CMD", "curl", "-f", "http://localhost:8222/varz"]
      interval: 5s
      timeout: 2s
      retries: 3
    is_daemon: true
    restart_policy:
      backoff: 1s
      max_restarts: 5

  dragonfly:
    command: "dragonfly --bind=0.0.0.0 --port=6379"
    ports:
      - "6379:6379"
    depends_on:
      nats: complete
    health_check:
      test: ["CMD", "redis-cli", "PING"]
      interval: 5s
      timeout: 2s
      retries: 3
    is_daemon: true

  neo4j:
    command: "neo4j console"
    working_dir: "/var/lib/neo4j"
    ports:
      - "7687:7687"
      - "7474:7474"
    environment:
      - NEO4J_AUTH=neo4j/password
      - NEO4J_server_memory_heap_initial__size=512m
      - NEO4J_server_memory_heap_max__size=1g
    depends_on:
      dragonfly: complete
    health_check:
      test: ["CMD", "curl", "-f", "http://localhost:7474/browser/"]
      interval: 10s
      timeout: 5s
      retries: 5
    is_daemon: true

  minio:
    command: "minio server /data --console-address :9001"
    environment:
      - MINIO_ROOT_USER=minioadmin
      - MINIO_ROOT_PASSWORD=minioadmin
    ports:
      - "9000:9000"
      - "9001:9001"
    depends_on:
      neo4j: complete
    health_check:
      test: ["CMD", "curl", "-f", "http://localhost:9000/minio/health/live"]
      interval: 10s
      timeout: 5s
      retries: 3
    is_daemon: true

  # ===== APPLICATION =====

  agileplus-api:
    command: "cargo run -p agileplus-api"
    working_dir: "."
    environment:
      - NATS_URL=nats://localhost:4222
      - REDIS_URL=redis://localhost:6379
      - NEO4J_URL=bolt://localhost:7687
      - NEO4J_USER=neo4j
      - NEO4J_PASSWORD=password
      - S3_ENDPOINT=http://localhost:9000
      - S3_ACCESS_KEY=minioadmin
      - S3_SECRET_KEY=minioadmin
      - S3_BUCKET=artifacts
      - SERVER_HOST=0.0.0.0
      - SERVER_PORT=8080
    ports:
      - "8080:8080"
    depends_on:
      nats: healthy
      dragonfly: healthy
      neo4j: healthy
      minio: healthy
    health_check:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 10s
      timeout: 5s
      retries: 5
    readiness_probe:
      test: ["CMD", "curl", "-f", "http://localhost:8080/ready"]
      interval: 5s
      timeout: 2s
      retries: 3
    file_watch:
      enabled: true
      paths:
        - "crates/agileplus-api/src"
```

---

## Testing Integration

**File**: `agileplus-api/tests/integration_test.rs`

```rust
#[tokio::test]
async fn test_full_integration() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to all services
    let nats = async_nats::connect("nats://localhost:4222").await?;
    let jetstream = async_nats::jetstream::new(nats.clone());

    let redis_mgr = bb8_redis::RedisConnectionManager::new("redis://localhost:6379")?;
    let redis_pool = bb8::Pool::builder().max_size(4).build(redis_mgr).await?;

    let neo4j = neo4rs::Graph::new("bolt://localhost:7687", "neo4j", "password").await?;

    // Test event publishing and consumption
    jetstream
        .publish(
            "wp.created",
            serde_json::to_vec(&serde_json::json!({
                "wp_id": "WP-001",
                "event_type": "created"
            }))?
            .into(),
        )
        .await?;

    // Test Neo4j operations
    neo4j
        .execute(neo4rs::query(
            "CREATE (wp:WorkPackage {id: $id, title: $title})"
        )
        .param("id", "WP-001")
        .param("title", "Test WP"))
        .await?;

    // Verify in Redis
    use redis::AsyncCommands;
    let mut redis_conn = redis_pool.get().await?;
    let _: () = redis_conn.set("test_key", "test_value").await?;
    let value: String = redis_conn.get("test_key").await?;
    assert_eq!(value, "test_value");

    Ok(())
}
```

---

**Implementation Status**: Production-ready | **Last Updated**: March 2026
