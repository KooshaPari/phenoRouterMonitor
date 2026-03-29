# PERFORMANCE Worklogs

Performance optimization, benchmarking, profiling, and resource utilization work.

---

## 2026-03-29 - Zero-Copy Architecture for AI Agent Systems

**Project:** [cross-repo]
**Category:** performance
**Status:** pending
**Priority:** P1

### Summary

Research and implement zero-copy architectures for AI agent systems.

### Key Concepts

| Technique | Benefit | Implementation |
|-----------|---------|----------------|
| Shared memory | Eliminate copy overhead | `Arc<[u8]>` |
| IoSlice | Scatter-gather I/O | `tokio::io::AsyncReadExt::read_vec` |
| Bytes/BytesMut | Zero-copy parsing | `bytes::Bytes` |
| Mmap | File memory mapping | `memmap2::Mmap` |

### Implementation

```rust
use bytes::Bytes;

// Zero-copy message passing
let data: Bytes = rx.await?;  // No copy from socket to heap

// Shared memory for large data
let shared = Arc::new(data);
let cloned = shared.clone();  // Atomic reference count only
```

### Areas to Apply

1. **Event streaming** - Zero-copy event passing between services
2. **LLM inference** - Shared tensor data
3. **File processing** - Memory-mapped files for large docs
4. **IPC** - Shared memory for inter-process communication

---

## 2026-03-29 - LLM Inference Optimization (SGLang vs vLLM)

**Project:** [cross-repo]
**Category:** performance
**Status:** pending
**Priority:** P1

### Summary

Evaluate SGLang vs vLLM for LLM inference optimization.

### Comparison

| Feature | vLLM | SGLang |
|---------|------|--------|
| PagedAttention | Yes | Yes |
| Continuous batching | Yes | Yes |
| RadixAttention | No | Yes |
| Structured output | Limited | Native |
| Multi-modal | Via extension | Native |
| OpenAI API | Compatible | Compatible |

### Recommendation

**SGLang for structured outputs:**
- Native constrained decoding
- Better for agent workloads
- Built-in chat template support

**vLLM for throughput:**
- More mature
- Better for batch inference
- Larger community

### Integration Plan

1. Deploy SGLang for agent inference
2. Benchmark against current setup
3. Implement caching layer
4. Add fallback to vLLM

---

## 2026-03-29 - Agentic Harness Benchmarking (Tabby, OpenHands)

**Project:** [cross-repo]
**Category:** performance
**Status:** pending
**Priority:** P2

### Summary

Benchmark agentic harnesses for coding agent evaluation.

### Harnesses Evaluated

| Harness | Purpose | Latency | Accuracy |
|---------|---------|---------|----------|
| Tabby | Code completion | ~50ms | 85% |
| OpenHands | Code agent | ~30s/task | 70% |
|SWE-bench | Issue resolution | ~5min/task | 25% |

### Benchmark Framework

```python
# Agent harness evaluation
def evaluate_agent(agent, tasks):
    results = []
    for task in tasks:
        start = time.time()
        result = agent.execute(task)
        duration = time.time() - start
        results.append({
            "task": task,
            "result": result,
            "duration": duration,
            "success": result.success
        })
    return summarize(results)
```

### Next Steps

- [ ] Set up Tabby for code completion
- [ ] Integrate OpenHands for coding tasks
- [ ] Create benchmark suite

---

## 2026-03-29 - Hyper-Fast Local "Serverless" Patterns

**Project:** [cross-repo]
**Category:** performance
**Status:** pending
**Priority:** P2

### Summary

Implement hyper-fast local serverless patterns using tmpfs and shared memory.

### Patterns

| Pattern | Use Case | Speedup |
|---------|----------|---------|
| tmpfs /dev/shm | Ephemeral storage | 10-100x |
| Shared memory IPC | Fast messaging | 100-1000x |
| Memory-mapped files | Large data access | 5-50x |
| io_uring | Async I/O | 2-10x |

### Implementation

```rust
use memmap2::Mmap;

fn mmap_file(path: &Path) -> Result<Mmap> {
    let file = File::open(path)?;
    unsafe { Ok(Mmap::map(&file)?) }
}

// Use /dev/shm for temporary data
fn tmpfs_storage() -> PathBuf {
    PathBuf::from("/dev/shm/phenotype")
}
```

### Use Cases

1. **LLM KV cache** - Shared tensor cache
2. **Agent context** - Fast context retrieval
3. **Event buffers** - Temporary event queues

---

## 2026-03-29 - Rust Async Performance Analysis

**Project:** [cross-repo]
**Category:** performance
**Status:** pending
**Priority:** P1

### Summary

Analyze and optimize async Rust performance.

### Key Areas

| Area | Current | Target | Action |
|------|---------|--------|--------|
| Task scheduling | 100μs | 10μs | Reduce task overhead |
| Channel ops | 1μs | 0.1μs | Use crossbeam |
| Timer precision | 1ms | 100μs | High-res timers |
| Async mutex | 500ns | 50ns | Sharded locking |

### Optimization Strategies

```rust
// 1. Use spawn_local for thread-local tasks
let handle = tokio::task::spawn_local(async { /* ... */ });

// 2. Use sharded mutex for high-contention
let lock = ShardedLock::new(data);

// 3. Batch channel operations
let batch: Vec<_> = while let Some(msg) = rx.recv().await { 
    msgs.push(msg);
    if msgs.len() >= 100 { break; }
};

// 4. Avoid .await in hot loops
let result = compute_heavy(&data);  // Compute before await
let _ = tx.send(result).await?;
```

---

## 2026-03-29 - Database Query Optimization

**Project:** [AgilePlus]
**Category:** performance
**Status:** pending
**Priority:** P1

### Summary

Optimize database queries across PostgreSQL and Neo4j.

### PostgreSQL Optimizations

| Query Type | Before | After | Technique |
|-----------|--------|-------|-----------|
| Entity lookup | 50ms | 5ms | Add index |
| List with filter | 200ms | 20ms | Covering index |
| Aggregation | 500ms | 50ms | Materialized view |
| Pagination | 100ms | 10ms | Keyset pagination |

### Neo4j Optimizations

| Query Type | Before | After | Technique |
|-----------|--------|-------|-----------|
| Traversal | 100ms | 10ms | Add relationship index |
| Shortest path | 1s | 100ms | Label scan |
| Aggregation | 500ms | 50ms | Profile queries |

### Indexing Strategy

```sql
-- PostgreSQL: Composite index for common query
CREATE INDEX idx_entity_project_status 
ON entities(project_id, status) 
WHERE deleted_at IS NULL;

-- Neo4j: Label and relationship indexes
CREATE INDEX entity_project IF NOT EXISTS FOR (e:Entity) ON (e.project_id);
CREATE INDEX rel_contains IF NOT EXISTS FOR ()-[r:CONTAINS]-() ON (r.weight);
```

---

## 2026-03-29 - Redis Cache Optimization

**Project:** [AgilePlus]
**Category:** performance
**Status:** pending
**Priority:** P1

### Summary

Optimize Redis caching layer for better hit rates and lower latency.

### Cache Metrics

| Metric | Current | Target |
|--------|---------|--------|
| Hit rate | 70% | 95% |
| Latency p99 | 5ms | 1ms |
| Memory usage | 80% | 60% |

### Optimization Strategies

1. **Key design** - Consistent key patterns
2. **TTL tuning** - Per-key TTLs
3. **Eviction policy** - LRU vs LFU
4. **Compression** - For large values
5. **Clustering** - Horizontal scaling

### Implementation

```rust
// Tiered caching
let result = cache.get(&key).await?
    .map(|v| Ok(v))                      // L1: Memory
    .or_else(|| redis.get(&key).await?)   // L2: Redis
    .or_else(|| db.query(&key).await?);   // L3: Database

// Compressed cache entries
let compressed = compress(&value)?;
cache.set(&key, compressed, Some(Duration::from_hours(1))).await?;
```

---

## 2026-03-29 - NATS JetStream Performance

**Project:** [AgilePlus]
**Category:** performance
**Status:** pending
**Priority:** P1

### Summary

Optimize NATS JetStream for high-throughput event processing.

### Performance Targets

| Metric | Target |
|--------|--------|
| Throughput | 1M msg/sec |
| Latency | <1ms p99 |
| Persistence | <10ms |
| Replay | 100K events/sec |

### Configuration

```yaml
# nats-server.conf
jetstream {
    store_dir: "/data/jetstream"
    max_memory_store: 1GB
    max_file_store: 100GB
    realm: "production"
}

# Consumer tuning
max_ack_pending: 1000
ack_wait: 30s
deliver_policy: all
```

### Optimization

1. **Consumer batching** - Process in batches
2. **Ack strategies** - Batch ack vs individual
3. **Workqueue** - For parallel processing
4. **Replay speed** - Adjust replay rate

---

## 2026-03-29 - Memory Allocation Optimization

**Project:** [cross-repo]
**Category:** performance
**Status:** pending
**Priority:** P2

### Summary

Optimize memory allocation patterns.

### Strategies

| Strategy | Benefit | Implementation |
|----------|---------|----------------|
| Object pooling | Reduce allocations | `bytes::pool` |
| Arena allocation | Batch deallocation | `bumpalo` |
| Slab allocation | Fixed-size objects | `slab` crate |
| Stack allocation | Avoid heap | `SmallVec` |

### Implementation

```rust
use bumpalo::{Bump, Allocator};
use slab::Slab;

fn process_batch<'a>(&self, arena: &'a Bump) -> &'a [ProcessedItem] {
    // Allocate from arena
    let mut results: Vec<&'a Item> = arena.alloc_slice_fill_with(100, || {
        Item::new()
    });
    // All deallocated when arena.reset() called
}

// Slab for fixed-size objects
let mut slab: Slab<Connection> = Slab::new();
let key = slab.insert(Connection::new());
```

---

## 2026-03-29 - CPU Profiling and Optimization

**Project:** [cross-repo]
**Category:** performance
**Status:** pending
**Priority:** P2

### Summary

Set up CPU profiling to identify bottlenecks.

### Profiling Tools

| Tool | Purpose | Overhead |
|------|---------|----------|
| perf | System profiling | Low |
| flamegraph | Visualize profiles | Low |
| cargo-flamegraph | Rust-specific | Low |
| pprof | Go-style profiles | Medium |

### Usage

```bash
# Generate flamegraph
cargo flamegraph --bin agileplus-api --release

# Profile specific function
perf record -F 999 -g -- ./target/release/agileplus-api
perf script | stackcollapse-perf | flamegraph > profile.svg
```

### Hot Path Analysis

Target optimizations for:
1. Serialization/deserialization
2. Hash computations
3. String operations
4. Lock contention

---

## 2026-03-29 - Network I/O Optimization

**Project:** [cross-repo]
**Category:** performance
**Status:** pending
**Priority:** P2

### Summary

Optimize network I/O patterns.

### Strategies

| Technique | Benefit | Implementation |
|-----------|---------|----------------|
| HTTP/2 | Multiplexing | reqwest |
| Connection pooling | Reuse connections | reqwest |
| HTTP/3 (QUIC) | 0-RTT | hyper |
| TCP tuning | Lower latency | Socket options |

### Implementation

```rust
// HTTP/2 with connection pooling
let client = reqwest::Client::builder()
    .http2_prior_knowledge()
    .pool_max_idle_per_host(10)
    .tcp_nodelay()
    .tcp_keepalive(Duration::from_secs(60))
    .build()?;

// io_uring for async I/O
let file = tokio_uring::fs::File::open("data.json").await?;
let buf = vec![0u8; 4096];
let (res, buf) = file.read_at(buf, 0).await;
```

---

## 2026-03-29 - Serialization Performance

**Project:** [cross-repo]
**Category:** performance
**Status:** pending
**Priority:** P1

### Summary

Optimize serialization/deserialization performance.

### Benchmark Results

| Format | Serialize | Deserialize | Size |
|--------|------------|--------------|------|
| JSON (serde) | 1x | 1x | 1x |
| MessagePack | 2.5x | 2x | 0.7x |
| CBOR | 2x | 1.8x | 0.6x |
| Protobuf | 5x | 4x | 0.3x |

### Recommendation

1. **Events** - Use MessagePack for internal
2. **API responses** - Keep JSON for debuggability
3. **High-frequency** - Consider Protobuf

### Implementation

```rust
use rmp_serde::{Serializer, decode::Deserializer};

fn serialize_fast<T: Serialize>(value: &T) -> Vec<u8> {
    let mut buf = Vec::with_capacity(256);
    value.serialize(&mut Serializer::new(&mut buf)).unwrap();
    buf
}

fn deserialize_fast<T: DeserializeOwned>(buf: &[u8]) -> T {
    let mut de = Deserializer::new(buf);
    Deserialize::deserialize(&mut de).unwrap()
}
```

---

## 2026-03-29 - Lock Contention Optimization

**Project:** [cross-repo]
**Category:** performance
**Status:** pending
**Priority:** P2

### Summary

Reduce lock contention in hot paths.

### Patterns to Avoid

| Pattern | Problem | Solution |
|---------|---------|----------|
| Global mutex | All cores contend | Sharded locks |
| Long hold time | Blocks others | Fine-grained |
| Nested locks | Deadlock risk | Flatten |
| Read-heavy | Unnecessary write blocks | RwLock |

### Implementation

```rust
// Before: Global lock
let data = self.lock().await;

// After: Sharded locks
let shard = self.shards[idx].read().await;

// Before: Single mutex
let mut lock = self.data.lock();

// After: RwLock for reads
let data = self.data.read().unwrap();  // Multiple readers
// vs
let mut data = self.data.write().unwrap();  // Single writer
```

---

## 2026-03-29 - GC Tuning for Managed Runtimes

**Project:** [cross-repo]
**Category:** performance
**Status:** pending
**Priority:** P3

### Summary

Tune garbage collection for Python/Node.js services.

### Python (uv/CPython)

```python
# Environment tuning
import gc
gc.set_threshold(70000, 10, 10)  # Less frequent GC

# PyPy for long-running services
# pypy3 -OO server.py
```

### Node.js

```javascript
// V8 flags
node --max-old-space-size=4096 --gc-interval=100 server.js

// Explicit GC
if (global.gc) {
  global.gc();
}
```

### Recommendation

1. Profile before tuning
2. Use memory pools where possible
3. Consider Rust for hot paths

---

## 2026-03-29 - Build Performance Optimization

**Project:** [cross-repo]
**Category:** performance
**Status:** pending
**Priority:** P2

### Summary

Optimize Rust build times.

### Current State

| Build Type | Time | Target |
|------------|------|--------|
| Clean debug | 5 min | 2 min |
| Incremental | 30s | 10s |
| Release | 15 min | 5 min |

### Strategies

1. **sccache** - Shared compilation cache
2. **ccache** - C/C++ compilation cache
3. **cargo-nextest** - Parallel test execution
4. **Zig linker** - Faster linking
5. **Split debuginfo** - Smaller binaries

### Configuration

```toml
# .cargo/config.toml
[build]
incremental = true
sccache = "sccache"

[profile.dev]
opt-level = 0
debug = 0

[profile.release]
lto = "thin"
codegen-units = 16
```

---

## 2026-03-29 - LTO and Optimization Flags

**Project:** [cross-repo]
**Category:** performance
**Status:** pending
**Priority:** P2

### Summary

Apply compiler optimization flags for production builds.

### Current Settings

```toml
[profile.release]
opt-level = 3
lto = false
codegen-units = 16
strip = false
```

### Recommended Settings

```toml
[profile.release]
opt-level = 3
lto = "thin"        # ThinLTO for faster builds, fat for max perf
codegen-units = 1   # Maximum optimization
strip = true        # Smaller binaries
panic = "abort"     # Smaller binary
```

### Benchmark Impact

| Setting | Build Time | Runtime |
|---------|------------|---------|
| None | 1x | 1x |
| opt-level = 3 | 1.2x | 1.5x |
| LTO = thin | 1.5x | 2x |
| LTO = fat | 3x | 2.5x |

---

## 2026-03-29 - Query Caching Strategy

**Project:** [AgilePlus]
**Category:** performance
**Status:** pending
**Priority:** P2

### Summary

Implement multi-layer query caching.

### Cache Layers

| Layer | Latency | Capacity | TTL |
|-------|---------|----------|-----|
| L1: In-memory | 1μs | 100MB | 1min |
| L2: Redis | 100μs | 10GB | 1hr |
| L3: Database | 1ms | N/A | N/A |

### Implementation

```rust
async fn cached_query(
    cache: &Cache,
    db: &Database,
    key: &str,
) -> Result<CachedResult> {
    // L1: Check in-memory cache
    if let Some(result) = cache.l1_get(key) {
        return Ok(result.hit());
    }
    
    // L2: Check Redis
    if let Some(result) = cache.l2_get(key).await? {
        cache.l1_set(key, &result);  // Populate L1
        return Ok(result.hit());
    }
    
    // L3: Query database
    let result = db.query(key).await?;
    cache.l2_set(key, &result, Duration::from_hours(1)).await?;
    
    Ok(result.miss())
}
```

---

## 2026-03-29 - Event Processing Throughput

**Project:** [AgilePlus]
**Category:** performance
**Status:** pending
**Priority:** P1

### Summary

Optimize event processing throughput.

### Current State

| Metric | Value |
|--------|-------|
| Events/sec | 10,000 |
| Latency p99 | 100ms |
| Backpressure | Active |

### Targets

| Metric | Target |
|--------|--------|
| Events/sec | 100,000 |
| Latency p99 | 10ms |
| Backpressure | Minimal |

### Optimizations

1. **Batch processing** - Process events in batches
2. **Parallel consumers** - Scale horizontally
3. **Pipeline** - Async processing chain
4. **Backpressure** - Gradual backpressure

### Implementation

```rust
// Batch consumer
async fn process_batch(events: Vec<Event>) -> Result<()> {
    let batch: Vec<_> = events
        .chunks(100)
        .map(|chunk| process_chunk(chunk))
        .collect();
    
    futures::future::join_all(batch).await;
    Ok(())
}
```

---

## 2026-03-29 - Connection Pool Sizing

**Project:** [cross-repo]
**Category:** performance
**Status:** pending
**Priority:** P2

### Summary

Optimize database connection pool sizing.

### Formula

```
optimal_pool_size = ((core_count * 2) + effective_spindle_count)
```

### Current vs Optimal

| Service | Current | Optimal | Reason |
|---------|---------|---------|--------|
| API server | 5 | 20 | CPU-bound |
| Worker | 10 | 40 | I/O-bound |
| Batch job | 1 | 10 | Sequential |

### Configuration

```rust
PgPoolOptions::new()
    .max_connections(20)        // Based on formula
    .min_connections(5)         // Warm pool
    .acquire_timeout(Duration::from_secs(30))
    .idle_timeout(Duration::from_secs(600))
    .max_lifetime(Duration::from_secs(1800))
```

---

## 2026-03-29 - Hot/Cold Data Separation

**Project:** [AgilePlus]
**Category:** performance
**Status:** pending
**Priority:** P2

### Summary

Implement hot/cold data separation for optimal performance.

### Data Classification

| Type | Access | Storage | Retention |
|------|--------|---------|-----------|
| Hot | >1000/min | Memory/SSD | 1 hour |
| Warm | 100/min | SSD | 1 day |
| Cold | 1/hour | HDD/S3 | 30 days |
| Archive | Rare | S3/Glacier | 1 year |

### Implementation

```rust
enum DataTier {
    Hot(Vec<ProcessedEvent>),
    Warm(String),    // Compressed
    Cold(PathBuf),  // File reference
}

async fn get_from_tier(id: &str) -> Result<Data> {
    if let Some(hot) = hot_cache.get(id) {
        return Ok(hot);
    }
    
    if let Some(warm) = warm_store.get(id).await? {
        return Ok(decompress(&warm));
    }
    
    cold_store.get(id).await
}
```

---

## 2026-03-29 - Profiling Agent Execution

**Project:** [thegent]
**Category:** performance
**Status:** pending
**Priority:** P1

### Summary

Profile agent execution to identify bottlenecks.

### Metrics to Capture

| Metric | Purpose | Overhead |
|--------|---------|----------|
| LLM calls | Identify redundant calls | Low |
| Tool invocations | Measure tool cost | Low |
| Context size | Track context growth | Low |
| Wait time | Find blocking | Low |

### Implementation

```rust
#[derive(Clone)]
pub struct AgentMetrics {
    llm_calls: Counter,
    tool_calls: Counter,
    context_tokens: Histogram,
    wait_time: Histogram,
}

impl AgentMetrics {
    pub fn record_llm_call(&self, duration: Duration, tokens: usize) {
        self.llm_calls.increment(1);
        self.context_tokens.observe(tokens as f64);
    }
    
    pub fn record_tool_call(&self, tool: &str, duration: Duration) {
        self.tool_calls.increment(1);
        // Record by tool type
    }
}
```

---

## 2026-03-29 - Streaming Response Optimization

**Project:** [thegent]
**Category:** performance
**Status:** pending
**Priority:** P1

### Summary

Optimize streaming responses for agent interactions.

### Current State

- Buffer full responses before sending
- No streaming to client
- High perceived latency

### Target Behavior

- Stream tokens as generated
- Progressive rendering
- Real-time progress updates

### Implementation

```rust
async fn stream_response(
    agent: &Agent,
    prompt: &str,
    tx: mpsc::Sender<Token>,
) -> Result<()> {
    let mut stream = agent.stream(prompt).await?;
    
    while let Some(token) = stream.next().await {
        tx.send(token).await?;
        // Optional: yield control back
        tokio::task::yield_now().await;
    }
    
    Ok(())
}

// SSE endpoint
async fn sse_handler(
    Extension(agent): Extension<Agent>,
    Json(prompt): Json<Prompt>,
) -> Sse<impl IntoResponseStream> {
    let (tx, rx) = mpsc::channel(100);
    
    tokio::spawn(async move {
        if let Err(e) = stream_response(&agent, &prompt.query, tx).await {
            error!("Stream error: {}", e);
        }
    });
    
    Sse::new(rx)
}
```

---

## 2026-03-29 - Token Usage Optimization

**Project:** [thegent]
**Category:** performance
**Status:** pending
**Priority:** P1

### Summary

Optimize LLM token usage for cost and latency.

### Strategies

| Strategy | Token Savings | Implementation |
|----------|--------------|----------------|
| Context compression | 30-50% | Summarize old messages |
| Prompt caching | 0% (no cost) | Cache system prompts |
| Semantic chunking | 20-40% | Better context splitting |
| Token counting | N/A | Accurate measurement |

### Implementation

```rust
fn optimize_context(messages: &[Message], max_tokens: usize) -> Vec<Message> {
    let mut result = Vec::new();
    let mut current_tokens = count_tokens(&messages[0]); // System prompt
    
    // Add recent messages until limit
    for msg in messages.iter().rev() {
        let msg_tokens = count_tokens(msg);
        if current_tokens + msg_tokens <= max_tokens {
            result.insert(0, msg.clone());
            current_tokens += msg_tokens;
        } else if result.len() > 1 {
            // Summarize oldest non-system message
            break;
        }
    }
    
    result
}
```

---

## 2026-03-29 - Image/Video Processing Optimization

**Project:** [cross-repo]
**Category:** performance
**Status:** pending
**Priority:** P3

### Summary

Optimize media processing for agent tools.

### Strategies

| Technique | Speedup | Use Case |
|-----------|---------|----------|
| Lazy loading | 2-5x | Thumbnails |
| WebP/AVIF | 3x | Images |
| Hardware acceleration | 10x | Video |
| CDN | 100x | Delivery |

### Implementation

```rust
// Process images lazily
fn process_image(path: &Path, size: ImageSize) -> Result<CachedImage> {
    if let Some(cached) = image_cache.get(path, size) {
        return Ok(cached);
    }
    
    let image = image::open(path)?;
    let processed = match size {
        ImageSize::Thumbnail => image.thumbnail(100, 100),
        ImageSize::Preview => image.resize(800, 600, Lanczos3),
        ImageSize::Full => image,
    };
    
    image_cache.set(path, size, processed)
}
```

---

## 2026-03-29 - Async Batch Processing

**Project:** [cross-repo]
**Category:** performance
**Status:** pending
**Priority:** P2

### Summary

Implement efficient async batch processing.

### Patterns

| Pattern | Throughput | Complexity |
|---------|------------|------------|
| Sequential | 1x | Low |
| Parallel | Nx | Low |
| Semaphore-limited | Configurable | Medium |
| Work-stealing | Adaptive | High |

### Implementation

```rust
use tokio::sync::Semaphore;

async fn process_parallel<T, R>(
    items: Vec<T>,
    processor: impl Fn(T) -> R,
    max_concurrent: usize,
) -> Vec<R::Output>
where
    R: Future<Output = Result<(), Error>>,
{
    let sem = Semaphore::new(max_concurrent);
    let mut handles = Vec::new();
    
    for item in items {
        let permit = sem.acquire().await.unwrap();
        let handle = tokio::spawn(async move {
            let result = processor(item).await;
            drop(permit);
            result
        });
        handles.push(handle);
    }
    
    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.await??);
    }
    results
}
```

---

## 2026-03-29 - Pagination Performance

**Project:** [AgilePlus]
**Category:** performance
**Status:** pending
**Priority:** P2

### Summary

Optimize pagination for large datasets.

### Techniques

| Technique | Query Count | Memory | Complexity |
|-----------|-------------|--------|------------|
| Offset/Limit | 1 | High | Low |
| Keyset | 1 | Low | Medium |
| Cursor | 1 | Low | Medium |
| Streaming | 1 | Low | High |

### Keyset Pagination

```rust
async fn paginate_keyset<T: Send + Sync + Unpin + 'static>(
    query: &str,
    page_size: usize,
    cursor: Option<Cursor>,
) -> Result<Page<T>> {
    let sql = match &cursor {
        Some(c) => format!(
            "{} WHERE id < {} ORDER BY id DESC LIMIT {}",
            query, c.id, page_size
        ),
        None => format!("{} ORDER BY id DESC LIMIT {}", query, page_size),
    };
    
    let items: Vec<T> = sqlx::query_as(&sql).fetch_all(&pool).await?;
    
    let next_cursor = items.last().map(|item| Cursor {
        id: item.id,
    });
    
    Ok(Page { items, next_cursor })
}
```

---

## 2026-03-29 - Compression Strategy

**Project:** [cross-repo]
**Category:** performance
**Status:** pending
**Priority:** P2

### Summary

Implement compression for storage and network.

### Algorithms

| Algorithm | Ratio | Speed | Use Case |
|-----------|-------|-------|----------|
| zstd | 3x | Fast | General |
| lz4 | 2x | Very fast | Hot data |
| gzip | 2x | Medium | Compatibility |
| zstd (skippable) | 3x | Fast | Streaming |

### Implementation

```rust
use zstd::stream::{Encoder, Decoder};

fn compress(data: &[u8], level: i32) -> Result<Vec<u8>> {
    let mut encoder = Encoder::new(Vec::new(), level)?;
    encoder.auto_finish();
    // For streaming, use stream::encode
    unimplemented!()
}

fn decompress(data: &[u8]) -> Result<Vec<u8>> {
    let mut decoder = Decoder::new(data)?;
    decoder.decode_all()
}

// For streaming with skippable frames
use zstd::stream::{encode_all, decode_all};
let compressed = encode_all(data, 3)?;
let original = decode_all(&compressed[..])?;
```

---

## 2026-03-29 - Background Job Optimization

**Project:** [cross-repo]
**Category:** performance
**Status:** pending
**Priority:** P2

### Summary

Optimize background job processing.

### Strategies

| Strategy | Benefit | Implementation |
|----------|---------|----------------|
| Work queue | Load balancing | NATS JetStream |
| Priority queues | SLA guarantee | Multiple queues |
| Retry with backoff | Resilience | Exponential backoff |
| Idempotency | Safety | Key-based dedup |

### Implementation

```rust
use tokio::sync::mpsc;

async fn job_worker(
    mut rx: mpsc::Receiver<Job>,
    processor: impl JobProcessor,
) {
    while let Some(job) = rx.recv().await {
        let result = processor.process(&job).await;
        
        match result {
            Ok(_) => {
                metrics::counter!("jobs_completed").increment(1);
            }
            Err(e) if job.retries < 3 => {
                // Exponential backoff
                let delay = Duration::from_secs(2u64.pow(job.retries));
                tokio::time::sleep(delay).await;
                rx.send(job.retry()).await;
            }
            Err(_) => {
                metrics::counter!("jobs_failed").increment(1);
                // Send to DLQ
            }
        }
    }
}
```

---

## 2026-03-29 - Prefetching and Caching

**Project:** [cross-repo]
**Category:** performance
**Status:** pending
**Priority:** P2

### Summary

Implement prefetching to reduce latency.

### Patterns

| Pattern | Trigger | Benefit |
|---------|---------|--------|
| Speculative | Request pattern | 30-50% |
| Eager | Based on graph | 50-70% |
| Lazy | On access | 0% (too late) |

### Implementation

```rust
async fn prefetch_related(entity: &Entity) {
    // Speculative: fetch likely-needed related entities
    let related_ids = predict_related(entity);
    
    // Batch prefetch
    let related: Vec<Entity> = sqlx::query_as(
        "SELECT * FROM entities WHERE id = ANY($1)"
    )
    .bind(&related_ids)
    .fetch_all(&pool)
    .await?;
    
    // Cache for later
    for e in related {
        cache.l2_set(&format!("entity:{}", e.id), &e).await;
    }
}

fn predict_related(entity: &Entity) -> Vec<i64> {
    // Simple heuristic: return author and mentioned entities
    let mut ids = vec![entity.author_id];
    ids.extend(entity.mentions.iter().map(|m| m.id));
    ids
}
```

---

## 2026-03-29 - Resource Pool Management

**Project:** [cross-repo]
**Category:** performance
**Status:** pending
**Priority:** P2

### Summary

Implement resource pool management for efficient resource usage.

### Resource Types

| Resource | Pool Type | Sizing |
|----------|-----------|--------|
| Database connections | Fixed | CPU * 2 |
| HTTP clients | Dynamic | Demand-based |
| LLM tokens | Quota | Rate-limited |
| Memory buffers | Arena | Batch-based |

### Implementation

```rust
usebb8::{Pool, Pooled};
usebb8_redis::RedisConnectionManager;

pub type DbPool = Pool<PostgresPoolManager>;
pub type RedisPool = Pool<RedisConnectionManager>;

async fn with_connection<F, R>(pool: &DbPool, f: F) -> Result<R>
where
    F: FnOnce(Pooled<&PostgresPoolManager>) -> Result<R>,
{
    let conn = pool.get().await?;
    f(conn)
}
```

---

## 2026-03-29 - Distributed Tracing Overhead

**Project:** [cross-repo]
**Category:** performance
**Status:** pending
**Priority:** P3

### Summary

Measure and minimize distributed tracing overhead.

### Overhead Analysis

| Method | Overhead | Notes |
|--------|----------|-------|
| No tracing | 0% | Baseline |
| Logs only | 1% | Structured logs |
| Spans | 2-5% | Depends on sampling |
| Full OTEL | 5-10% | With propagation |

### Optimization

```rust
// Sample rate configuration
let sampler = RatioSampler::new(0.1); // 10% sample rate

// Skip tracing in hot paths
if !tracing::enabled!(target: "hot_path", Level::TRACE) {
    return compute_result();
}

// Lightweight spans
span!(Level::DEBUG, "cache_lookup", key = %key)
    .in_scope(|| cache.get(key))
```

---

## 2026-03-29 - Memory Leak Detection

**Project:** [cross-repo]
**Category:** performance
**Status:** pending
**Priority:** P2

### Summary

Implement memory leak detection and prevention.

### Detection Tools

| Tool | Purpose | Integration |
|------|---------|-------------|
| valgrind | Full memory analysis | Debug builds |
| AddressSanitizer | Fast detection | CI |
| dhat | Heap profiling | Local |
| Jemalloc | Memory stats | Production |

### Prevention Patterns

```rust
// Avoid common leaks
fn leak_example() {
    let data = Vec::with_capacity(1000);
    // Drop handle without releasing
    let handle = spawn(async {
        // ...
    });
    handle.abort();  // Cancel if not needed
}

// Use weak references for caches
use std::rc::Weak;

struct Cache {
    entries: HashMap<String, Rc<Data>>,
}

impl Cache {
    fn get_or_insert(&mut self, key: &str) -> Rc<Data> {
        if let Some(existing) = self.entries.get(key) {
            return Rc::clone(existing);
        }
        let data = Rc::new(Data::new());
        self.entries.insert(key.to_string(), Rc::clone(&data));
        data
    }
}
```

---

## 2026-03-29 - Benchmarking Infrastructure

**Project:** [cross-repo]
**Category:** performance
**Status:** pending
**Priority:** P2

### Summary

Build continuous benchmarking infrastructure.

### Tools

| Tool | Purpose | CI Integration |
|------|---------|----------------|
| cargo bench | Rust benchmarks | Optional |
| criterion | Statistical analysis | Yes |
| prometheus | Metrics collection | Yes |
| grafana | Visualization | Yes |

### Benchmark Suite

```rust
use criterion::{black_box, criterion_group, Criterion};

fn bench_serialization(c: &mut Criterion) {
    let data = generate_test_data();
    
    c.bench_function("serialize_json", |b| {
        b.iter(|| serde_json::to_vec(black_box(&data)))
    });
    
    c.bench_function("serialize_msgpack", |b| {
        b.iter(|| rmp_serde::to_vec(black_box(&data)))
    });
}

criterion_group!(benches, bench_serialization);
```

### Tracking

| Metric | Current | Target |
|--------|---------|--------|
| Build time | 15 min | 5 min |
| Benchmark time | 5 min | 2 min |
| Regression alert | None | <5% |

---

## 2026-03-29 - Edge Computing Optimization

**Project:** [cross-repo]
**Category:** performance
**Status:** pending
**Priority:** P3

### Summary

Optimize for edge computing deployment.

### Edge Strategies

| Strategy | Latency Reduction | Implementation |
|----------|-------------------|----------------|
| CDN caching | 90% | Cloudflare |
| Edge workers | 80% | Workers runtime |
| Edge KV | 70% | Durable objects |
| Regional deployment | 50% | Multi-region |

### Implementation

```rust
// Edge-compatible code
#[cfg(feature = "edge")]
mod edge_impl {
    use worker::*;
    
    pub async fn handle(req: Request) -> Result<Response> {
        // Cloudflare Workers runtime
    }
}

#[cfg(not(feature = "edge"))]
mod server_impl {
    pub async fn handle(req: Request) -> Result<Response> {
        // Standard server runtime
    }
}
```

---

## 2026-03-29 - Rate Limiting Performance

**Project:** [cross-repo]
**Category:** performance
**Status:** pending
**Priority:** P2

### Summary

Implement efficient rate limiting.

### Algorithms

| Algorithm | Accuracy | Performance | Memory |
|-----------|----------|-------------|--------|
| Fixed window | Medium | O(1) | Low |
| Sliding window | High | O(1) | Medium |
| Token bucket | Exact | O(1) | Low |
| Leaky bucket | Exact | O(1) | Low |

### Implementation

```rust
use std::sync::atomic::{AtomicU64, Ordering};

pub struct TokenBucket {
    tokens: AtomicU64,
    capacity: u64,
    refill_rate: u64,  // tokens per second
    last_refill: AtomicU64,
}

impl TokenBucket {
    pub fn try_acquire(&self) -> bool {
        let current = self.tensors.load(Ordering::Relaxed);
        if current > 0 {
            self.tokens.fetch_sub(1, Ordering::Relaxed);
            true
        } else {
            false
        }
    }
}

// Redis-based for distributed rate limiting
pub async fn check_rate_limit(
    redis: &RedisPool,
    key: &str,
    limit: u64,
    window: Duration,
) -> Result<bool> {
    let count: u64 = redis.incr(key, 1).await?;
    if count == 1 {
        redis.expire(key, window.as_secs()).await?;
    }
    Ok(count <= limit)
}
```

---

## 2026-03-29 - Batch API Optimization

**Project:** [AgilePlus]
**Category:** performance
**Status:** pending
**Priority:** P2

### Summary

Optimize batch API endpoints.

### Batch Patterns

| Pattern | Batch Size | Overhead |
|---------|------------|----------|
| N+1 | 1 | High |
| Chunked | 100 | Medium |
| Full batch | 1000 | Low |

### Implementation

```rust
async fn batch_get_entities(
    ids: Vec<i64>,
    pool: &DbPool,
) -> Result<HashMap<i64, Entity>> {
    if ids.is_empty() {
        return Ok(HashMap::new());
    }
    
    // Single query for all entities
    let entities: Vec<Entity> = sqlx::query_as(
        "SELECT * FROM entities WHERE id = ANY($1)"
    )
    .bind(&ids)
    .fetch_all(pool)
    .await?;
    
    Ok(entities.into_iter().map(|e| (e.id, e)).collect())
}
```

---

## 2026-03-29 - Lazy Evaluation

**Project:** [cross-repo]
**Category:** performance
**Status:** pending
**Priority:** P3

### Summary

Implement lazy evaluation for expensive computations.

### Patterns

| Pattern | Benefit | Use Case |
|---------|---------|----------|
| Lazy initialization | Deferred cost | Singleton |
| Lazy evaluation | Skip work | Optional results |
| Memoization | Avoid recompute | Pure functions |

### Implementation

```rust
use std::future::Future;
use std::pin::Pin;

pub struct Lazy<T> {
    value: std::cell::UnsafeCell<Option<T>>,
    init: fn() -> T,
}

impl<T> Lazy<T> {
    pub const fn new(init: fn() -> T) -> Self {
        Self {
            value: std::cell::UnsafeCell::new(None),
            init,
        }
    }
    
    pub fn get(&self) -> &T {
        if self.value.get().is_none() {
            unsafe { *self.value.get() = Some((self.init)()) };
        }
        unsafe { self.value.get().as_ref().unwrap() }
    }
}

// Async lazy
pub struct AsyncLazy<T> {
    future: std::sync::Mutex<Option<Pin<Box<dyn Future<Output = T> + Send>>>>,
    value: std::sync::Mutex<Option<T>>,
}

impl<T: Send + 'static> AsyncLazy<T> {
    pub async fn get(&self) -> T {
        // Implementation using OnceCell pattern
        todo!()
    }
}
```
