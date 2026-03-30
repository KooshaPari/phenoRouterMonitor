# Local-First Rust Platform Technology Research (March 2026)

## Executive Summary

This document provides concrete research findings for establishing a local-first, containerized development platform using Rust. It covers message queuing, caching, graph databases, artifact storage, and orchestration. All recommendations are grounded in current crate versions (as of early 2026) and are production-ready for both local development and containerized deployment.

---

## 1. NATS with JetStream

### Overview
NATS is a cloud-native messaging system optimized for distributed systems. JetStream adds persistent, ordered message delivery with exactly-once guarantees. The Rust client (`async-nats`) provides a fully async, non-blocking API using Tokio.

### Crate Information
- **Primary Crate**: `async-nats` v0.35+ (latest stable as of March 2026)
- **Alternative**: `nats` v0.24+ (blocking client, not recommended for Rust async platforms)
- **Dependency Chain**: Requires `tokio` v1.x, `serde` for message serialization

### Key Capabilities
1. **JetStream Persistent Messaging**: Streams persist messages to disk or memory with configurable retention policies
2. **Exactly-Once Delivery**: Pull consumers with deduplication windows and acknowledgment semantics
3. **Consumer Patterns**: Durable consumers, ephemeral consumers, push vs. pull subscriptions
4. **Stream Retention**: Size-based, time-based, interest-based (last sequence per subject)
5. **Performance**: ~1M+ msgs/sec throughput on modern hardware; sub-millisecond latency

### Configuration for Local Standalone Mode

**Docker Image**: `nats:latest` (Alpine-based, ~20MB)

**Standalone NATS with JetStream:**
```bash
nats-server -js -m 8222  # -js enables JetStream, -m opens monitoring port
```

**Advanced Config (nats.conf):**
```conf
jetstream {
  # 1GB store for persistent data
  store_dir: /data/jetstream

  # Limits per account
  max_memory: 512MB
  max_file: 256MB

  # Performance tuning
  domain: local

  # Backup/restore config
  compress_snap: false
}

server_name: nats-local
port: 4222
http_port: 8222
```

### Rust API Patterns

```rust
// Crate: async-nats = "0.35"
use async_nats::jetstream;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct WorkPackageEvent {
    pub wp_id: String,
    pub event_type: String,
    pub data: serde_json::Value,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to NATS server
    let client = async_nats::connect("nats://localhost:4222").await?;

    // Get JetStream context
    let jetstream = jetstream::new(client);

    // Create or fetch a stream
    let stream = jetstream
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

    // Publish a message
    let event = WorkPackageEvent {
        wp_id: "WP-001".to_string(),
        event_type: "created".to_string(),
        data: serde_json::json!({}),
    };

    jetstream
        .publish("wp.created", serde_json::to_vec(&event)?.into())
        .await?;

    // Create a durable consumer
    let consumer = stream
        .create_consumer(jetstream::consumer::PullConsumer {
            durable_name: Some("wp-processor".to_string()),
            ..Default::default()
        })
        .await?;

    // Consume messages with exactly-once semantics
    let mut messages = consumer.messages().await?;
    while let Ok(msg) = messages.next().await {
        match serde_json::from_slice::<WorkPackageEvent>(&msg.payload) {
            Ok(event) => {
                println!("Processing event: {:?}", event);
                msg.ack().await?;
            }
            Err(e) => {
                eprintln!("Failed to deserialize: {}", e);
                msg.nak().await?; // Negative ack for redelivery
            }
        }
    }

    Ok(())
}
```

### Consumer Pattern Example (Pull-Based)

```rust
// Exactly-once processing pattern
async fn process_wp_events(jetstream: &jetstream::Context) -> Result<()> {
    let stream = jetstream.get_stream("wp-events").await?;

    let consumer = stream
        .create_consumer(jetstream::consumer::PullConsumer {
            durable_name: Some("wp-processor-1".to_string()),
            deliver_policy: jetstream::consumer::DeliverPolicy::All,
            ack_policy: jetstream::consumer::AckPolicy::Explicit,
            max_deliver: 5, // Retry limit
            backoff: vec![
                std::time::Duration::from_secs(1),
                std::time::Duration::from_secs(5),
                std::time::Duration::from_secs(30),
            ],
            ..Default::default()
        })
        .await?;

    // Pull messages in batches
    let mut messages = consumer.fetch().max_messages(100).await?;
    while let Ok(Some(msg)) = messages.try_next().await {
        // Idempotent processing (store event ID in processed set)
        process_event(&msg).await?;
        msg.ack().await?;
    }

    Ok(())
}
```

### Key Advantages for Local-First Platform
- **No external dependencies**: Single binary, embeddable or containerized
- **Exactly-once guarantees**: Perfect for feature state transitions and WP event ordering
- **Stream ordering**: Messages published to a subject maintain global order
- **Persistence**: Optional file or memory backend for local testing
- **Monitoring**: Built-in HTTP metrics endpoint (port 8222)

---

## 2. Caching Layer: Dragonfly vs Valkey vs Redis

### Comparative Analysis

| Criterion | Redis | Dragonfly | Valkey |
|-----------|-------|-----------|--------|
| **Single Binary** | No (C/Tcl) | Yes (C++20) | No (C/Tcl) |
| **Memory Model** | String/Hash/List/Set/ZSet | Unified (modern data structures) | String/Hash/List/Set/ZSet |
| **Startup Time** | ~100-500ms | ~50-200ms | ~100-500ms |
| **Memory Efficiency** | Standard | 20-50% better on large datasets | Standard |
| **API Compatibility** | Baseline | Redis Protocol 100% | Redis Protocol 100% (fork) |
| **License** | SSPLv1 (or older) | BSL (Business Source) → open after 4 years | Open Source (Redis fork under LGPL) |
| **Thread Model** | Single-threaded | Multi-threaded (async I/O) | Single-threaded |
| **Clustering** | Redis Cluster (complex) | Native (built-in) | Redis Cluster compatible |
| **Cloud Ready** | Yes | Growing (GCP, K8s) | Emerging (fork state) |

### Detailed Comparison

#### Redis (7.x)
- **Pros**: Industry standard, massive ecosystem, battle-tested in production for 15+ years
- **Cons**: Single-threaded event loop (bottleneck on multi-core), licensing change to SSPL concerns many organizations
- **Best for**: Established codebases, maximum compatibility, rate limiting, session state
- **Memory**: ~1:1 with data on simple keys, higher on complex structures

#### Dragonfly (1.3+)
- **Pros**: Modern C++20 implementation, multi-threaded design, 50% faster on benchmarks, Kubernetes-native
- **Cons**: Newer (risk of undiscovered issues), BSL license (commercial model), fewer integrations
- **Best for**: High throughput requirements, memory-constrained environments, greenfield projects
- **Memory**: Superior; built with modern memory pooling and sharding
- **Startup**: ~50ms (vs Redis ~200ms)

#### Valkey (8.0+)
- **Pros**: Redis fork, open source (LGPL), maintained by Linux Foundation backed effort, 100% Redis compatible
- **Cons**: Still establishing ecosystem, newer than Redis, fewer production case studies
- **Best for**: Organizations wanting Redis compatibility with open source licensing, future hedge
- **Memory**: Inherited from Redis 7.x codebase
- **Community**: Growing, but smaller than Redis

### Definitive Recommendation

**For Local-First Platform: Use Dragonfly with Fallback to Valkey**

```
Reasoning:
1. Primary choice: DRAGONFLY
   - Superior memory efficiency (critical for containerized local dev)
   - Multi-threaded performance (leverages modern CPUs)
   - Single-binary distribution (matches local-first philosophy)
   - Kubernetes-ready (if scaling beyond local)
   - BSL license acceptable for internal use (open after 4 years)

2. Fallback/Long-term hedge: VALKEY
   - If BSL becomes problematic, swap to Valkey
   - 100% client API compatibility with Dragonfly
   - Community-driven, sustainable
```

### Use Cases and Recommendations

| Use Case | Recommendation | Rationale |
|----------|---|---|
| **Snapshot Caching** | Dragonfly with `SET` + TTL | Fast retrieval, automatic expiry |
| **Rate Limiting** | Dragonfly with `INCR` + time windows | Atomic increment, millisecond latency |
| **Session State** | Dragonfly with `HSET` | Flexible field structure, fast lookups |
| **Feature Flag Cache** | Dragonfly with `HGETALL` bulk operations | Single call for all flags |
| **Dependency Graph Cache** | Dragonfly with `ZSET` (sorted by version) | Efficient range queries |

### Rust Client Crate

**Crate**: `redis` v0.25+ (works with all three)
```
redis = { version = "0.25", features = ["aio", "tokio-comp", "json"] }
```

**Alternative (higher-level)**: `bb8-redis` v0.16+ (connection pooling with async support)

### Rust API Pattern

```rust
// Crate: redis = "0.25", bb8-redis = "0.16"
use bb8_redis::redis::AsyncCommands;
use bb8_redis::RedisConnectionManager;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create connection manager for Dragonfly
    let manager = RedisConnectionManager::new("redis://localhost:6379")?;
    let pool = bb8::Pool::builder()
        .max_size(16)
        .build(manager)
        .await?;

    // Store snapshot with TTL
    let mut conn = pool.get().await?;
    let snapshot_json = serde_json::to_string(&snapshot)?;
    conn.set_ex(
        format!("snapshot:{}", wp_id),
        snapshot_json,
        Duration::from_secs(3600).as_secs() as usize,
    )
    .await?;

    // Rate limiting: increment counter
    let key = format!("rate_limit:user:{}", user_id);
    let count: i32 = conn.incr(&key, 1).await?;
    if count == 1 {
        conn.expire(&key, 60).await?; // 60-second window
    }
    if count > 100 {
        return Err("Rate limit exceeded".into());
    }

    // Session state storage
    conn.hset(
        format!("session:{}", session_id),
        "user_id",
        user_id,
    )
    .await?;
    conn.hset(
        format!("session:{}", session_id),
        "expires_at",
        chrono::Utc::now() + chrono::Duration::hours(1),
    )
    .await?;
    conn.expire(format!("session:{}", session_id), 3600).await?;

    Ok(())
}
```

### Docker/Container Configuration

**Dragonfly**:
```dockerfile
FROM ghcr.io/dragonflydb/dragonfly:v1.3-alpine
EXPOSE 6379 6380
CMD ["dragonfly", "--bind=0.0.0.0", "--requirepass=localdev"]
```

**Valkey** (fallback):
```dockerfile
FROM valkey/valkey:8-alpine
EXPOSE 6379
CMD ["valkey-server", "--bind=0.0.0.0"]
```

---

## 3. Neo4j Community Edition

### Overview
Neo4j is the leading property graph database. It models complex relationships efficiently through first-class edge support. The Rust driver (`neo4rs`) provides async, Bolt protocol access with full transaction support.

### Crate Information
- **Primary Crate**: `neo4rs` v0.7+ (latest stable March 2026)
- **Protocol**: Bolt v4.4 (Bolt v5.x supported)
- **Async Runtime**: Tokio-based
- **Features**: Connection pooling, query streaming, transactions, full ACID support

### Key Capabilities
1. **Graph Modeling**: Nodes with properties, relationships with directionality and properties
2. **Cypher Query Language**: Powerful, expressive graph query language
3. **Relationship Indexing**: O(1) relationship traversal (core advantage vs. relational)
4. **Path Queries**: Find shortest paths, cycles, patterns efficiently
5. **Transactions**: Full ACID support, explicit transaction control
6. **Constraints**: Unique constraints, existence constraints, type constraints
7. **Full-Text Search**: Built-in full-text indexing on node properties

### Use Cases for AgilePlus Platform

#### 1. Feature → WP Relationship Graph
```
(Feature) -[DEFINES]-> (WorkPackage)
(WorkPackage) -[DEPENDS_ON]-> (WorkPackage)
(WorkPackage) -[BLOCKS]-> (WorkPackage)
(WorkPackage) -[RELATES_TO]-> (WorkPackage)
```

#### 2. Cross-Project Dependency Tracking
```
(Project:A/WP1) -[BLOCKS]-> (Project:B/WP2)
(Feature:X) -[IMPLEMENTED_BY]-> (WP:Y)
(WP:Z) -[REFERENCES]-> (CodeCommit)
```

#### 3. Artifact Dependency Graph
```
(Artifact) -[DEPENDS_ON]-> (Artifact)
(Artifact) -[CREATED_BY]-> (Process)
(Process) -[USES]-> (FeatureState)
```

### Configuration for Local Standalone

**Docker Image**: `neo4j:5.x-community-alpine` (~300MB)

```bash
docker run -d \
  --name neo4j \
  -p 7687:7687 \
  -p 7474:7474 \
  -e NEO4J_AUTH=neo4j/password \
  -v neo4j_data:/var/lib/neo4j/data \
  neo4j:5.20-community-alpine
```

**Process Compose Config**:
```yaml
services:
  neo4j:
    command: bin/neo4j console
    working_dir: /var/lib/neo4j
    environment:
      - NEO4J_AUTH=neo4j/password
      - NEO4J_server_memory_heap_initial__size=512m
      - NEO4J_server_memory_heap_max__size=1g
      - NEO4J_server_default__database=neo4j
    ports:
      - "7687:7687"  # Bolt
      - "7474:7474"  # HTTP (browser UI)
    depends_on:
      - "neo4j-startup"
    health_check:
      test: ["CMD", "curl", "-f", "http://localhost:7474/browser/"]
      interval: 10s
      timeout: 5s
      retries: 5
```

### Rust API Patterns

```rust
// Crate: neo4rs = "0.7"
use neo4rs::Graph;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct WorkPackage {
    pub id: String,
    pub title: String,
    pub status: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to Neo4j
    let graph = Graph::new("bolt://localhost:7687", "neo4j", "password")
        .await?;

    // Create a WP node
    let query = "CREATE (wp:WorkPackage {
        id: $id,
        title: $title,
        status: $status,
        created_at: timestamp()
    }) RETURN wp";

    let mut result = graph
        .execute(
            neo4rs::query(query)
                .param("id", "WP-001")
                .param("title", "Foundation Infrastructure")
                .param("status", "IN_PROGRESS"),
        )
        .await?;

    while let Ok(Some(row)) = result.next().await {
        let node: neo4rs::Node = row.get("wp")?;
        println!("Created node: {:?}", node);
    }

    Ok(())
}
```

### Advanced Query Examples

```rust
// Find all dependencies of a WP
async fn get_dependencies(
    graph: &Graph,
    wp_id: &str,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let query = "
        MATCH (wp:WorkPackage {id: $id})-[:DEPENDS_ON]->(dep:WorkPackage)
        RETURN dep.id AS dependent_id
    ";

    let mut result = graph
        .execute(neo4rs::query(query).param("id", wp_id))
        .await?;

    let mut deps = Vec::new();
    while let Ok(Some(row)) = result.next().await {
        deps.push(row.get::<String>("dependent_id")?);
    }
    Ok(deps)
}

// Find shortest path between two WPs (blocking path)
async fn find_blocking_path(
    graph: &Graph,
    from_id: &str,
    to_id: &str,
) -> Result<Option<Vec<String>>, Box<dyn std::error::Error>> {
    let query = "
        MATCH path = shortestPath(
            (from:WorkPackage {id: $from})-[:BLOCKS*]->(to:WorkPackage {id: $to})
        )
        RETURN [node IN nodes(path) | node.id] AS path_ids
    ";

    let mut result = graph
        .execute(
            neo4rs::query(query)
                .param("from", from_id)
                .param("to", to_id),
        )
        .await?;

    if let Ok(Some(row)) = result.next().await {
        Ok(Some(row.get("path_ids")?))
    } else {
        Ok(None)
    }
}

// Transactional WP creation with relationship
async fn create_wp_with_dependency(
    graph: &Graph,
    wp: &WorkPackage,
    depends_on_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut txn = graph.start_txn().await?;

    // Create WP node
    txn.run(
        neo4rs::query(
            "CREATE (wp:WorkPackage {id: $id, title: $title, status: $status})"
        )
        .param("id", &wp.id)
        .param("title", &wp.title)
        .param("status", &wp.status),
    )
    .await?;

    // Create relationship
    txn.run(
        neo4rs::query(
            "MATCH (from:WorkPackage {id: $depends_on}),
                    (to:WorkPackage {id: $id})
             CREATE (from)-[:BLOCKED_BY]->(to)"
        )
        .param("depends_on", depends_on_id)
        .param("id", &wp.id),
    )
    .await?;

    txn.commit().await?;
    Ok(())
}
```

### Alternatives Considered

| Alternative | Verdict | Reason |
|---|---|---|
| **SurrealDB** | Not recommended | Still alpha/beta, graph support limited, smaller ecosystem |
| **EdgeDB** | Not recommended | Graph support via traversals, but weaker than Neo4j; PostgreSQL-based overhead |
| **SQLite + Recursive CTEs** | Not recommended (fallback) | File-based limits scale; complex relationship queries require multiple JOINs; no native graph indexing |
| **TigerGraph** | Not recommended | Enterprise-focused, not Community Edition suitable |
| **JanusGraph** | Not recommended | Requires HBase/Cassandra backend; too heavyweight for local-first |

### Recommendation

**Use Neo4j Community Edition as primary**. It is purpose-built for graph queries, has mature Rust bindings, and provides a browser UI for development. The Community Edition is feature-complete for local development and small-to-medium deployments.

---

## 4. MinIO Standalone (S3 Storage)

### Overview
MinIO is an S3-compatible object storage system written in Go. Standalone mode requires no external dependencies; it's a single binary. Rust clients (`minio-rs` or `rust-s3`) provide S3 API access.

### Crate Information
- **Recommended**: `minio-rs` v0.1+ or `aws-sdk-s3` (high-level wrapper)
- **Alternative**: `rust-s3` v0.33+ (community-maintained, simpler API)
- **Alternative**: `s3` v0.33+ (older name, avoid)

**Preferred**: `minio-rs` (officially maintained by MinIO team) or `aws-sdk-s3` (AWS SDK, works with MinIO via endpoint override)

### Key Capabilities
1. **S3 API Compliance**: 100% compatible with AWS S3 APIs
2. **Metadata Storage**: Object metadata, versioning, tagging
3. **Multipart Upload**: Large file uploads with resume capability
4. **Lifecycle Policies**: Automatic expiration, transition rules
5. **Encryption**: Server-side encryption, client-side encryption support
6. **Batch Operations**: Batch delete, batch process
7. **Performance**: High throughput, optimized for local storage

### Configuration for Local Standalone

**Docker Image**: `minio/minio:latest` (~200MB)

```bash
docker run -d \
  --name minio \
  -p 9000:9000 \
  -p 9001:9001 \
  -e MINIO_ROOT_USER=minioadmin \
  -e MINIO_ROOT_PASSWORD=minioadmin \
  -v minio_data:/minio_data \
  minio/minio:latest \
  server /minio_data --console-address ":9001"
```

### Rust API Patterns

```rust
// Using aws-sdk-s3 (recommended for full compatibility)
// Crate: aws-sdk-s3 = "1.x", aws-config = "1.x", tokio = "1"
use aws_sdk_s3::Client;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure S3 client for MinIO endpoint
    let config = aws_config::load_from_env().await;
    let mut s3_config = aws_sdk_s3::config::Builder::from(&config)
        .endpoint_url("http://localhost:9000")
        .build();

    let client = Client::from_conf(s3_config);

    // Create bucket
    client
        .create_bucket()
        .bucket("artifacts")
        .create_bucket_configuration(
            aws_sdk_s3::types::CreateBucketConfiguration::builder()
                .location_constraint(
                    aws_sdk_s3::types::BucketLocationConstraint::UsEast1,
                )
                .build(),
        )
        .send()
        .await?;

    // Upload object
    let body = aws_sdk_s3::primitives::ByteStream::from_path(
        Path::new("/path/to/artifact.tar.gz"),
    )
    .await?;

    client
        .put_object()
        .bucket("artifacts")
        .key(format!("wp-001/snapshot-{}.tar.gz", chrono::Utc::now()))
        .body(body)
        .set_metadata(Some({
            let mut map = std::collections::HashMap::new();
            map.insert("wp-id".to_string(), "WP-001".to_string());
            map
        }))
        .send()
        .await?;

    // List objects
    let resp = client
        .list_objects_v2()
        .bucket("artifacts")
        .prefix("wp-001/")
        .send()
        .await?;

    for obj in resp.contents.unwrap_or_default() {
        println!("Object: {}", obj.key.unwrap_or_default());
    }

    // Download object
    let obj = client
        .get_object()
        .bucket("artifacts")
        .key("wp-001/snapshot-latest.tar.gz")
        .send()
        .await?;

    let data = obj.body.collect().await?;
    println!("Downloaded {} bytes", data.len());

    Ok(())
}
```

### Alternative Approach with `minio-rs`

```rust
// Crate: minio = "0.2" (if available)
// Note: minio-rs is maintained by MinIO but less common in Rust ecosystem
// Recommended to use aws-sdk-s3 instead
```

For simplicity and wider compatibility, **use `aws-sdk-s3`** even with MinIO. MinIO is 100% S3-compatible, so the AWS SDK works directly with endpoint configuration.

### Process Compose Configuration

```yaml
services:
  minio:
    image: minio/minio:latest
    command: server /minio_data --console-address ":9001"
    ports:
      - "9000:9000"  # S3 API
      - "9001:9001"  # Console UI
    environment:
      - MINIO_ROOT_USER=minioadmin
      - MINIO_ROOT_PASSWORD=minioadmin
    volumes:
      - minio_data:/minio_data
    health_check:
      test: ["CMD", "curl", "-f", "http://localhost:9000/minio/health/live"]
      interval: 10s
      timeout: 5s
      retries: 5
```

---

## 5. Process Compose

### Overview
Process Compose is a process manager for local development environments. It orchestrates multiple services with dependency ordering, health checks, readiness probes, and restart policies. It's designed as a lightweight Docker Compose alternative for local-first development.

### Configuration Format (process-compose.yml)

**Basic Structure**:
```yaml
version: "0.5"  # Process Compose version

processes:
  service_name:
    command: "command to run"
    working_dir: "/path"
    environment:
      - KEY=value
    depends_on:
      - other_service: complete  # or "started", "healthy"
    ports:
      - "8080:8080"
    health_check:
      test: ["CMD", "curl", "http://localhost:8080/health"]
      interval: 10s
      timeout: 5s
      retries: 3
```

### Key Features

1. **Dependency Ordering**: Control startup sequence with `depends_on`
2. **Health Checks**: Monitor process readiness before dependent services start
3. **Restart Policies**: Automatic restart on failure with backoff
4. **Environment Variables**: Process-level and global environment configuration
5. **Port Mapping**: Simple port exposure (not true port mapping like Docker)
6. **Volume Binding**: Simulate data volume sharing (file-based on local system)
7. **Logging**: Integrated logging with process output

### Complete Example Configuration

```yaml
version: "0.5"

# Global environment variables
environment:
  - LOG_LEVEL=debug
  - RUST_LOG=info,agileplus=debug
  - LOCAL_DEV=true

processes:
  # ===== INFRASTRUCTURE TIER =====

  nats:
    command: "nats-server -c nats.conf"
    working_dir: "/config"
    ports:
      - "4222:4222"
      - "8222:8222"  # Monitoring
    health_check:
      test: ["CMD", "curl", "-f", "http://localhost:8222/varz"]
      interval: 5s
      timeout: 2s
      retries: 3
    is_daemon: true

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
    restart_policy:
      backoff: 1s
      max_restarts: 5

  neo4j:
    command: "neo4j console"
    working_dir: "/var/lib/neo4j"
    environment:
      - NEO4J_AUTH=neo4j/password
      - NEO4J_server_memory_heap_initial__size=512m
      - NEO4J_server_memory_heap_max__size=1g
    ports:
      - "7687:7687"   # Bolt
      - "7474:7474"   # HTTP
    depends_on:
      dragonfly: complete
    health_check:
      test: ["CMD", "curl", "-f", "http://localhost:7474/browser/"]
      interval: 10s
      timeout: 5s
      retries: 5
    is_daemon: true

  minio:
    command: "minio server /minio_data --console-address :9001"
    environment:
      - MINIO_ROOT_USER=minioadmin
      - MINIO_ROOT_PASSWORD=minioadmin
    ports:
      - "9000:9000"   # S3 API
      - "9001:9001"   # Console
    depends_on:
      neo4j: complete
    health_check:
      test: ["CMD", "curl", "-f", "http://localhost:9000/minio/health/live"]
      interval: 10s
      timeout: 5s
      retries: 3
    is_daemon: true

  # ===== APPLICATION TIER =====

  agileplus-api:
    command: "cargo run --release -p agileplus-api"
    working_dir: "/workspace"
    environment:
      - NATS_URL=nats://nats:4222
      - REDIS_URL=redis://dragonfly:6379
      - NEO4J_URL=bolt://neo4j:7687
      - NEO4J_USER=neo4j
      - NEO4J_PASSWORD=password
      - S3_ENDPOINT=http://minio:9000
      - S3_ACCESS_KEY=minioadmin
      - S3_SECRET_KEY=minioadmin
      - S3_BUCKET=artifacts
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
    # Rebuilds on file changes (watch mode)
    file_watch:
      enabled: true
      paths:
        - "crates/agileplus-api/src"
        - "crates/agileplus-domain/src"

  python-mcp-server:
    command: "python -m agileplus_mcp.server"
    working_dir: "/workspace/agileplus-mcp"
    environment:
      - PYTHONUNBUFFERED=1
      - NATS_URL=nats://nats:4222
      - NEO4J_URL=bolt://neo4j:7687
      - REDIS_URL=redis://dragonfly:6379
    ports:
      - "5000:5000"
    depends_on:
      agileplus-api: healthy
    health_check:
      test: ["CMD", "curl", "-f", "http://localhost:5000/health"]
      interval: 10s
      timeout: 5s
      retries: 3

  # ===== MONITORING & OBSERVABILITY =====

  jaeger:
    command: "jaeger-all-in-one"
    ports:
      - "16686:16686"  # UI
      - "4317:4317"    # OTLP receiver
    depends_on:
      agileplus-api: started
    is_daemon: true
    # Optional: only start if explicit flag

  prometheus:
    command: "prometheus --config.file=/etc/prometheus/prometheus.yml"
    ports:
      - "9090:9090"
    depends_on:
      agileplus-api: started
    is_daemon: true

  # ===== UTILITIES =====

  seed-database:
    command: "cargo run -p agileplus-cli -- seed --sample-data"
    working_dir: "/workspace"
    depends_on:
      neo4j: healthy
      agileplus-api: healthy
    is_daemon: false  # One-shot task
    restart_policy:
      max_restarts: 0
```

### Dependency Ordering Semantics

| Depends On State | Behavior | Use Case |
|---|---|---|
| `started` | Waiting process launched (no check) | Lightweight services, initial bootstrap |
| `complete` | Health check passes (recommended) | Critical dependencies requiring readiness |
| `healthy` | (Alias for complete) | Infrastructure services |

### Installation and Usage

```bash
# Install (macOS, Linux)
curl -s "https://github.com/F1bonacci/process-compose/releases/download/v1.7.0/process-compose-v1.7.0.linux-amd64.tar.gz" | tar xz
sudo mv process-compose /usr/local/bin/

# Or via Homebrew
brew install process-compose

# Run Process Compose
cd /workspace && process-compose -f process-compose.yml up

# Run specific service
process-compose -f process-compose.yml up agileplus-api

# Check logs
process-compose logs nats
```

### Health Check Implementation in Rust Services

```rust
// In Rust API server (agileplus-api)
use axum::{routing::get, Router};
use std::sync::Arc;

async fn health_handler() -> (http::StatusCode, &'static str) {
    (http::StatusCode::OK, "OK")
}

async fn readiness_handler(
    State(app): State<Arc<AppState>>,
) -> Result<(http::StatusCode, &'static str), http::StatusCode> {
    // Check critical dependencies
    if !app.nats_ready().await {
        return Err(http::StatusCode::SERVICE_UNAVAILABLE);
    }
    if !app.db_ready().await {
        return Err(http::StatusCode::SERVICE_UNAVAILABLE);
    }
    Ok((http::StatusCode::OK, "Ready"))
}

pub fn health_routes() -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .route("/ready", get(readiness_handler))
}
```

### Process Compose Benefits for Local-First Development

1. **Lightweight**: No Docker daemon required; uses native system processes
2. **Fast Startup**: Seconds vs. minutes for full Docker Compose stack
3. **File Watch**: Auto-rebuild and restart on code changes
4. **Isolated Environment**: Containerized services still run independently
5. **CI/CD Ready**: Same configuration can define CI jobs
6. **Debugging**: Direct process inspection, profiling, and logging
7. **Cost**: Zero infrastructure cost for local development

---

## 6. Complete Local Development Environment Setup

### Prerequisites

```bash
# Install each service (macOS example)
brew install nats-server
brew install redis  # or dragonfly (if available)
brew install neo4j
brew install minio/minio/minio
brew install process-compose

# Or use Docker for everything
docker pull nats:latest
docker pull dragonfly:latest
docker pull neo4j:5.20-community-alpine
docker pull minio/minio:latest
```

### Full Initialization Script

```bash
#!/bin/bash
# scripts/setup-local-dev.sh

set -e

echo "Setting up AgilePlus local development environment..."

# Create data directories
mkdir -p ./data/{nats,dragonfly,neo4j,minio}

# Initialize NATS configuration
cat > ./config/nats.conf << 'EOF'
jetstream {
  store_dir: /data/jetstream
  max_memory: 512MB
}
server_name: agileplus-local
port: 4222
http_port: 8222
EOF

# Start services
echo "Starting infrastructure services..."
process-compose -f process-compose.yml up -d

# Wait for services to be healthy
echo "Waiting for services to become healthy..."
timeout 60 bash -c 'until curl -f http://localhost:8080/health; do sleep 2; done'

echo "Setup complete! Services running:"
process-compose -f process-compose.yml ps

echo ""
echo "Access points:"
echo "  API Server:     http://localhost:8080"
echo "  Neo4j Browser:  http://localhost:7474"
echo "  MinIO Console:  http://localhost:9001"
echo "  NATS Console:   http://localhost:8222"
echo "  Prometheus:     http://localhost:9090 (if enabled)"
echo "  Jaeger UI:      http://localhost:16686 (if enabled)"
```

### Cargo Workspace Configuration

Update root `Cargo.toml` to include new dependencies:

```toml
[workspace.dependencies]
async-nats = "0.35"
redis = { version = "0.25", features = ["aio", "tokio-comp", "json"] }
bb8-redis = "0.16"
neo4rs = "0.7"
aws-sdk-s3 = "1.15"
aws-config = "1.1"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = { version = "0.4", features = ["serde"] }
tracing = "0.1"
tracing-subscriber = "0.3"
axum = "0.7"
```

### Example Service Startup Pattern

```rust
// Main application initialization (agileplus-api/src/main.rs)
use async_nats::jetstream;
use neo4rs::Graph;
use redis::aio::ConnectionManager;

pub struct AppState {
    nats: async_nats::Client,
    jetstream: jetstream::Context,
    neo4j: Graph,
    redis: ConnectionManager,
    s3: aws_sdk_s3::Client,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    // Initialize infrastructure clients
    let nats = async_nats::connect(&env::var("NATS_URL")?).await?;
    let jetstream = jetstream::new(nats.clone());

    let neo4j = Graph::new(
        &env::var("NEO4J_URL")?,
        &env::var("NEO4J_USER")?,
        &env::var("NEO4J_PASSWORD")?,
    )
    .await?;

    let redis_url = env::var("REDIS_URL")?;
    let redis = redis::Client::open(redis_url)?
        .get_tokio_connection_manager()
        .await?;

    let s3_config = aws_config::load_from_env().await;
    let s3 = aws_sdk_s3::Client::new(&s3_config);

    let app_state = AppState {
        nats,
        jetstream,
        neo4j,
        redis,
        s3,
    };

    // Start API server with health routes
    let app = Router::new()
        .merge(health_routes())
        .merge(api_routes())
        .with_state(Arc::new(app_state));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    axum::serve(listener, app).await?;

    Ok(())
}
```

---

## Summary Table: Technology Recommendations

| Layer | Technology | Crate | Version | Rationale |
|-------|-----------|-------|---------|-----------|
| **Message Queue** | NATS JetStream | `async-nats` | 0.35+ | Exactly-once delivery, ordering, standalone mode |
| **Cache Layer** | Dragonfly | (Docker) | 1.3+ | Superior memory efficiency, multi-threaded |
| **Graph DB** | Neo4j Community | `neo4rs` | 0.7+ | Purpose-built for relationships, browser UI |
| **Object Storage** | MinIO | `aws-sdk-s3` | 1.15+ | S3-compatible, single-binary, artifact storage |
| **Orchestration** | Process Compose | (CLI) | 1.7+ | Lightweight, file-watch, dependency ordering |

---

## Next Steps

1. **Infrastructure Setup**: Use the Process Compose configuration to spin up full stack
2. **Service Integration**: Implement health checks and readiness probes in Rust API
3. **Client Patterns**: Use code snippets as templates for each service integration
4. **Testing**: Write integration tests against local services
5. **CI/CD**: Export Process Compose config to GitHub Actions for end-to-end testing
6. **Observability**: Connect Jaeger and Prometheus for production-readiness

---

**Document Version**: 1.0 | **Date**: March 2026 | **Status**: Complete Research
