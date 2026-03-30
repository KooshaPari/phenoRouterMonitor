# Local-First Tech Stack: Quick Reference Guide

## Crate Dependency List (Copy-Paste Ready)

```toml
# In Cargo.toml [workspace.dependencies]

# Message Queuing
async-nats = "0.35"

# Caching
redis = { version = "0.25", features = ["aio", "tokio-comp", "json"] }
bb8 = "0.16"
bb8-redis = "0.16"

# Graph Database
neo4rs = "0.7"

# Object Storage
aws-sdk-s3 = "1.15"
aws-config = "1.1"
aws-smithy-runtime = "1.1"

# Web Framework (for API)
axum = "0.7"
tower = "0.4"
hyper = "1"

# Async Runtime
tokio = { version = "1", features = ["full"] }

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Time
chrono = { version = "0.4", features = ["serde"] }

# Observability
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt"] }

# Error Handling
thiserror = "2"
anyhow = "1"
```

---

## Quick Start Checklist

### 1. NATS JetStream

**Verify Connection**:
```rust
let client = async_nats::connect("nats://localhost:4222").await?;
println!("Connected to NATS");
```

**Create Stream**:
```rust
let jetstream = jetstream::new(client);
jetstream
    .get_or_create_stream(jetstream::stream::Config {
        name: "wp-events".to_string(),
        subjects: vec!["wp.>".to_string()],
        ..Default::default()
    })
    .await?;
```

### 2. Dragonfly Redis

**Verify Connection**:
```rust
let manager = RedisConnectionManager::new("redis://localhost:6379")?;
let pool = bb8::Pool::builder().max_size(16).build(manager).await?;
let mut conn = pool.get().await?;
let pong: String = redis::cmd("PING").query_async(&mut *conn).await?;
assert_eq!(pong, "PONG");
```

**Set Value with TTL**:
```rust
use redis::AsyncCommands;
let key = "snapshot:wp-001";
let value = serde_json::to_string(&snapshot)?;
conn.set_ex(key, value, 3600).await?;  // 1 hour TTL
```

### 3. Neo4j

**Verify Connection**:
```rust
let graph = Graph::new("bolt://localhost:7687", "neo4j", "password").await?;
let mut result = graph.execute(neo4rs::query("RETURN 1 AS test")).await?;
```

**Create Node and Query**:
```rust
graph
    .execute(
        neo4rs::query(
            "CREATE (n:WorkPackage {id: $id, title: $title}) RETURN n"
        )
        .param("id", "WP-001")
        .param("title", "Example WP"),
    )
    .await?;

let mut result = graph
    .execute(neo4rs::query("MATCH (n:WorkPackage {id: $id}) RETURN n.title")
        .param("id", "WP-001"))
    .await?;

while let Ok(Some(row)) = result.next().await {
    let title: String = row.get("n.title")?;
    println!("Title: {}", title);
}
```

### 4. MinIO / S3

**Verify Connection**:
```rust
let s3 = aws_sdk_s3::Client::from_conf(
    aws_sdk_s3::config::Builder::from(&config)
        .endpoint_url("http://localhost:9000")
        .build()
);

s3.list_buckets().send().await?;
```

**Upload Artifact**:
```rust
use aws_sdk_s3::primitives::ByteStream;

let body = ByteStream::from_path(Path::new("artifact.tar.gz")).await?;
s3
    .put_object()
    .bucket("artifacts")
    .key("wp-001/snapshot.tar.gz")
    .body(body)
    .send()
    .await?;
```

### 5. Health Check Endpoints

```rust
use axum::{
    routing::get,
    http::StatusCode,
    Router,
    extract::State,
};
use std::sync::Arc;

async fn health() -> (StatusCode, &'static str) {
    (StatusCode::OK, "OK")
}

async fn ready(State(state): State<Arc<AppState>>) -> Result<(StatusCode, &'static str), StatusCode> {
    if !state.nats_connected.load(std::sync::atomic::Ordering::Relaxed) {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    }
    if !state.neo4j_ready.load(std::sync::atomic::Ordering::Relaxed) {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    }
    Ok((StatusCode::OK, "Ready"))
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/health", get(health))
        .route("/ready", get(ready))
}
```

---

## Process Compose Minimal Template

```yaml
version: "0.5"

processes:
  nats:
    command: "nats-server -js"
    ports: ["4222:4222", "8222:8222"]
    health_check:
      test: ["CMD", "curl", "-f", "http://localhost:8222/varz"]
      interval: 5s
      retries: 3

  dragonfly:
    command: "dragonfly"
    ports: ["6379:6379"]
    depends_on:
      nats: complete
    health_check:
      test: ["CMD", "redis-cli", "PING"]
      interval: 5s
      retries: 3

  neo4j:
    command: "neo4j console"
    ports: ["7687:7687", "7474:7474"]
    depends_on:
      dragonfly: complete
    health_check:
      test: ["CMD", "curl", "-f", "http://localhost:7474/browser/"]
      interval: 10s
      retries: 5

  minio:
    command: "minio server /data --console-address :9001"
    ports: ["9000:9000", "9001:9001"]
    depends_on:
      neo4j: complete
    health_check:
      test: ["CMD", "curl", "-f", "http://localhost:9000/minio/health/live"]
      interval: 10s
      retries: 3
```

---

## Environment Variable Reference

```bash
# NATS
NATS_URL=nats://localhost:4222
NATS_JETSTREAM_DOMAIN=local

# Redis/Dragonfly
REDIS_URL=redis://localhost:6379
REDIS_PASSWORD=  # empty for local dev

# Neo4j
NEO4J_URL=bolt://localhost:7687
NEO4J_USER=neo4j
NEO4J_PASSWORD=password

# MinIO/S3
S3_ENDPOINT=http://localhost:9000
S3_ACCESS_KEY=minioadmin
S3_SECRET_KEY=minioadmin
S3_BUCKET=artifacts
S3_REGION=us-east-1

# Logging
RUST_LOG=info,agileplus=debug
LOG_LEVEL=debug
```

---

## Integration Testing Pattern

```rust
#[cfg(test)]
mod tests {
    use super::*;

    async fn setup() -> TestFixture {
        let nats = async_nats::connect("nats://localhost:4222").await.unwrap();
        let redis = RedisConnectionManager::new("redis://localhost:6379")
            .unwrap();
        let pool = bb8::Pool::builder().max_size(4).build(redis).await.unwrap();
        let neo4j = Graph::new("bolt://localhost:7687", "neo4j", "password")
            .await
            .unwrap();

        TestFixture {
            nats,
            redis_pool: pool,
            neo4j,
        }
    }

    #[tokio::test]
    async fn test_wp_event_flow() {
        let fixture = setup().await;

        // Publish event
        fixture.nats
            .publish("wp.created", serde_json::to_vec(&event).unwrap().into())
            .await
            .unwrap();

        // Verify in Neo4j
        let mut result = fixture.neo4j
            .execute(neo4rs::query("MATCH (n:WorkPackage {id: $id}) RETURN count(n)")
                .param("id", "WP-001"))
            .await
            .unwrap();

        while let Ok(Some(row)) = result.next().await {
            let count: i64 = row.get("count(n)").unwrap();
            assert_eq!(count, 1);
        }
    }
}
```

---

## Troubleshooting Common Issues

### NATS Connection Refused
```bash
# Check if NATS is running
ps aux | grep nats-server

# Start with verbose logging
nats-server -v
```

### Redis/Dragonfly Connection Issues
```rust
// Test with different retry strategy
let client = redis::Client::open("redis://localhost:6379")?;
let conn_manager = client
    .get_tokio_connection_manager()
    .await
    .map_err(|e| {
        eprintln!("Redis connection failed: {}", e);
        e
    })?;
```

### Neo4j Authentication Failed
```bash
# Verify credentials match container environment
# Default: neo4j / password

# Test connection from command line
cypher-shell -a bolt://localhost:7687 -u neo4j -p password
```

### MinIO Bucket Not Found
```rust
// Create bucket if it doesn't exist
match s3.head_bucket().bucket("artifacts").send().await {
    Ok(_) => println!("Bucket exists"),
    Err(_) => {
        s3.create_bucket()
            .bucket("artifacts")
            .send()
            .await?;
    }
}
```

### Process Compose Port Conflicts
```bash
# Check what's using port 4222
lsof -i :4222

# Kill process if needed
kill -9 <PID>
```

---

## Performance Tuning

### NATS Optimization
```conf
# In nats.conf
jetstream {
  store_dir: /ssd/jetstream  # Use SSD if available
  max_memory: 2GB

  limits {
    max_file_store: 10GB
  }
}

# Connection settings
max_connections: 10000
```

### Dragonfly/Redis Optimization
```bash
# Increase client buffer
CONFIG SET client-output-buffer-limit "normal 0 0 0"

# Increase slowlog
CONFIG SET slowlog-log-slower-than 1000
CONFIG SET slowlog-max-len 128
```

### Neo4j Optimization
```conf
# In docker/neo4j.conf
dbms.memory.heap.initial_size=512m
dbms.memory.heap.max_size=2g
dbms.memory.pagecache.size=500m
```

---

## Version Compatibility Matrix (March 2026)

| Component | Rust Edition | Min Tokio | Min Rust |
|-----------|---|---|---|
| async-nats 0.35 | 2021 | 1.25 | 1.70 |
| redis 0.25 | 2021 | 1.25 | 1.70 |
| neo4rs 0.7 | 2021 | 1.25 | 1.70 |
| aws-sdk-s3 1.15 | 2021 | 1.28 | 1.72 |
| axum 0.7 | 2021 | 1.25 | 1.70 |

**Project Requirement**: Rust 1.85 (from project Cargo.toml), which satisfies all crates.

---

## Links and Resources

### Official Documentation
- [NATS Rust Client](https://github.com/nats-io/nats.rs)
- [NATS JetStream Docs](https://docs.nats.io/nats-concepts/jetstream)
- [Dragonfly Docs](https://www.dragonflydb.io/)
- [Neo4j Rust Driver](https://github.com/neo4j-labs/neo4rs)
- [AWS SDK for Rust](https://github.com/awslabs/aws-sdk-rust)
- [Process Compose](https://github.com/F1bonacci/process-compose)

### Example Repositories
- [NATS Examples](https://github.com/nats-io/nats-by-example)
- [Neo4rs Examples](https://github.com/neo4j-labs/neo4rs/tree/main/examples)
- [AWS SDK Examples](https://github.com/awslabs/aws-sdk-rust/tree/main/examples)

### Benchmarks & Performance
- [NATS Benchmarks](https://github.com/nats-io/nats-bench)
- [Dragonfly vs Redis Benchmarks](https://www.dragonflydb.io/benchmarks)

---

**Last Updated**: March 2026 | **Status**: Ready for Implementation
