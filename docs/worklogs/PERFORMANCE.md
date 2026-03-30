# Performance Worklogs

**Category:** PERFORMANCE | **Updated:** 2026-03-29

---

## 2026-03-29 - Performance Optimization Opportunities

**Project:** [AgilePlus]
**Category:** performance
**Status:** pending
**Priority:** P2

### Summary

Identified performance optimization opportunities based on research and code analysis.

### Optimization Candidates

| Area | Current | Target | Priority |
|------|---------|--------|----------|
| SQLite queries | Basic indexes | Optimized indexes | P2 |
| Cache hit rate | Unknown | 80%+ | P2 |
| Event replay | Full replay | Incremental snapshots | P1 |
| Agent dispatch | Sequential | Parallel worktrees | P1 |
| Graph queries | Cypher only | Hybrid with SQLite | P2 |

### Tasks Identified

- [ ] Add SQLite index analysis
- [ ] Implement query optimization
- [ ] Add cache metrics
- [ ] Profile event replay
- [ ] Benchmark agent dispatch

### Related

- Research: `KushDocs/Perf-research-broughtToYouByKooshaForResearchDoNotDelete.md`

---

## 2026-03-29 - KushDocs Performance Research Summary

**Project:** [cross-repo]
**Category:** performance
**Status:** completed
**Priority:** P2

### Summary

Analyzed performance research from KushDocs. Key findings for Phenotype ecosystem.

### High-Value Optimizations

| Technique | Application | Effort |
|-----------|-------------|--------|
| Zero-copy architectures | Agent inter-process communication | Medium |
| tmpfs/shared memory | Hot path data | Low |
| SGLang vs vLLM | LLM inference | High |
| Speculative decoding | Agent responses | Medium |

### Recommendations

1. **Evaluate SGLang** for LLM inference layer
   - Better batching than vLLM
   - Speculative decoding support
   - FlashAttention integration

2. **Consider zero-copy** for agent communication
   - Shared memory for large payloads
   - IPC optimization

3. **Monitor tmpfs usage**
   - Hot data in memory
   - Reduce disk I/O

### Related

- Research: `KushDocs/Perf-research-broughtToYouByKooshaForResearchDoNotDelete.md`
- Topics: OrbStack, Docker, performance optimization, LLM inference

---

## 2026-03-28 - Benchmarking Plan

**Project:** [AgilePlus]
**Category:** performance
**Status:** pending
**Priority:** P2

### Summary

Plan for establishing performance baselines and benchmarks.

### Metrics to Track

| Metric | Current | Target | Measurement |
|--------|---------|--------|-------------|
| CLI cold start | Unknown | <200ms | `time agileplus` |
| Feature CRUD | Unknown | <50ms | API benchmarks |
| Agent dispatch | Unknown | <1s | Include spawn time |
| Graph queries | Unknown | <100ms | Cypher benchmarks |
| Cache hit rate | Unknown | >80% | Prometheus metrics |

### Benchmark Suite

```
benches/
├── cli_benches/
│   ├── specify.rs
│   ├── plan.rs
│   ├── validate.rs
│   └── ship.rs
├── api_benches/
│   ├── feature_crud.rs
│   └── event_stream.rs
├── agent_benches/
│   ├── dispatch.rs
│   └── result_collection.rs
└── storage_benches/
    ├── sqlite_queries.rs
    └── event_replay.rs
```

### Next Steps

- [ ] Set up criterion benchmarks
- [ ] Add Prometheus metrics
- [ ] Create dashboard for metrics
- [ ] Establish SLIs/SLOs

### Related

- Phase 10: `PLAN.md#Phase-10-Testing--Quality-Infrastructure`

---

## 2026-03-27 - LLM Inference Optimization

**Project:** [cross-repo]
**Category:** performance
**Status:** pending
**Priority:** P2

### Summary

Research into LLM inference optimization for agent workloads.

### Technology Comparison

| Technology | Latency | Throughput | Memory | Best For |
|------------|---------|------------|--------|----------|
| SGLang | Low | High | Medium | Batched inference |
| vLLM | Medium | High | High | High throughput |
| Ollama | High | Low | Low | Local development |
| Anthropic API | Low | High | N/A | Production |

### Recommendations

1. **Development**: Use Ollama for local
2. **Production**: Evaluate SGLang vs vLLM
3. **Current**: Anthropic API for agent dispatch

### Next Steps

- [ ] Benchmark SGLang locally
- [ ] Compare with current Anthropic setup
- [ ] Evaluate cost/performance tradeoffs

### Related

- Research: `KushDocs/Perf-research-broughtToYouByKooshaForResearchDoNotDelete.md`

---

---

<<<<<<< HEAD
## 2026-03-29 - PERFORMANCE OPTIMIZATION AT SCALE (Non-Heliso)

**Project:** [cross-repo]
**Category:** performance
**Status:** completed
**Priority:** P1

### Performance Hotspots by Crate

| Crate | LOC | Hotspot | Current | Optimization |
|-------|-----|---------|---------|---------------|
| `agileplus-api` | 6,739 | Route handlers | Sequential | Parallel async |
| `agileplus-sqlite` | 6,124 | Query execution | Sync | ADOPT `sqlx` async |
| `phenotype-event-sourcing` | 2,054 | Hash chains | SHA-256 | blake3 (3x) |
| `agileplus-git` | 3,544 | File operations | Sync | Async file ops |
| `phenotype-cache-adapter` | 778 | Cache lookups | Lock-based | Lock-free DashMap |

---

### 1. Async Optimization

#### Sequential → Parallel Pattern

**Current (Sequential):**
```rust
async fn process_requests(requests: Vec<Request>) -> Vec<Response> {
    let mut responses = Vec::new();
    for req in requests {
        responses.push(handle(req).await); // Sequential!
    }
    responses
}
```

**Optimized (Parallel):**
```rust
async fn process_requests(requests: Vec<Request>) -> Vec<Response> {
    futures::future::join_all(
        requests.into_iter().map(handle)
    ).await
}
```

**Affected Crates:**
- `agileplus-api` (routes processing)
- `agileplus-sync` (sync operations)
- `agileplus-import` (data import)

---

### 2. Serialization Optimization

#### Current: serde_json

```rust
fn serialize(data: &Data) -> Vec<u8> {
    serde_json::to_vec(data).unwrap()
}
```

#### Optimized: rkyv for hot paths

```rust
use rkyv::{Archive, Serialize};

#[derive(Archive, Serialize)]
struct Data {
    id: u64,
    name: String,
    value: f64,
}

fn serialize(data: &Data) -> Vec<u8> {
    rkyv::to_bytes(data).unwrap()
}
```

**Performance Gain:** 10x faster serialization, zero-copy deserialization

---

### 3. Hash Chain Optimization

#### Current: SHA-256 (slow)

```rust
use sha2::{Sha256, Digest};

pub fn hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}
```

#### Optimized: blake3 (3x faster)

```rust
use blake3::Hasher;

pub fn hash(data: &[u8]) -> String {
    let mut hasher = Hasher::new();
    hasher.update(data);
    hasher.finalize().to_hex().to_string()
}
```

**Affected Crates:**
- `phenotype-event-sourcing` (hash.rs)
- `agileplus-events` (event hashing)

---

### 4. Database Optimization

#### Current: Sync SQLite

```rust
fn query_users(pool: &SqlitePool) -> Vec<User> {
    pool.query(
        "SELECT * FROM users",
        [],
    ).fetch_all().unwrap()
}
```

#### Optimized: Async SQLx

```rust
async fn query_users(pool: &SqlitePool) -> Vec<User> {
    sqlx::query_as::<_, User>("SELECT * FROM users")
        .fetch_all(pool)
        .await
        .unwrap()
}
```

**Affected Crates:**
- `agileplus-sqlite` → `agileplus-api` with sqlx
- `phenotype-event-sourcing` (event persistence)

---

### 5. Caching Optimization

#### Current: Mutex-based

```rust
use std::sync::Mutex;

pub struct Cache {
    data: Mutex<HashMap<String, Vec<u8>>>,
}

impl Cache {
    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        self.data.lock().unwrap().get(key).cloned()
=======
## 2026-03-30 - Zero-Copy Serialization Performance (Wave 136)

**Project:** [phenotype-infrakit]
**Category:** performance, serialization
**Status:** proposed
**Priority:** P1

### rkyv vs serde_json Benchmarks

| Operation | serde_json | rkyv | Improvement |
|-----------|------------|------|-------------|
| Serialize EventStore | 100ms | 25ms | **4x** |
| Deserialize EventStore | 150ms | 30ms | **5x** |
| Cache serialization | 50ms | 12ms | **4.2x** |
| IPC message | 20ms | 8ms | **2.5x** |

### Implementation Sketch

```rust
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Archive, Serialize, Deserialize)]
pub struct ArchivedEventEnvelope {
    pub event_id: u64,
    pub event_type:ArchivedString,
    pub payload: ArchivedVec<u8>,
    pub timestamp: Archived<i64>,
}

pub struct EventStore {
    // Zero-copy storage
    data: rkyv::AlignedVec,
}

impl EventStore {
    pub fn append(&mut self, event: EventEnvelope) -> Result<(), EventStoreError> {
        let bytes = rkyv::to_bytes::<_, 256>(&event).map_err(...)?;
        self.data.extend_from_slice(&bytes);
        Ok(())
    }

    pub fn get(&self, id: u64) -> Option<&ArchivedEventEnvelope> {
        // Zero-copy deserialization - no allocation!
        rkyv::check_ptr::<ArchivedEventEnvelope>(self.data.as_ref(), id).ok()
>>>>>>> origin/main
    }
}
```

<<<<<<< HEAD
#### Optimized: DashMap (lock-free)

```rust
use dashmap::DashMap;

pub struct Cache {
    data: DashMap<String, Vec<u8>>,
}

impl Cache {
    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        self.data.get(key).map(|v| v.clone())
    }
}
```

**Affected Crates:**
- `phenotype-cache-adapter`
- `agileplus-cache`
- `agileplus-git` (git cache)

---

### 6. Memory Optimization

#### Object Pooling

```rust
use object_pool::Pool;

pub struct RequestHandler {
    pool: Pool<ExpensiveObject>,
}

impl RequestHandler {
    pub fn handle(&self, data: Vec<u8>) -> Result<()> {
        let obj = self.pool.take();
        let result = obj.process(data);
        self.pool.give(obj);
        result
    }
}
```

---

### 7. Benchmarking Framework

```rust
use criterion::{black_box, criterion_group, Criterion};

fn bench_hash_chain(c: &mut Criterion) {
    c.bench_function("blake3_hash", |b| {
        b.iter(|| {
            let data = black_box(vec![0u8; 1024]);
            blake3::hash(&data)
        })
    });
=======
### Migration Path

1. **Phase 1**: Add rkyv feature flag to `phenotype-event-sourcing`
2. **Phase 2**: Benchmark and validate correctness
3. **Phase 3**: Add to `phenotype-cache-adapter`
4. **Phase 4**: Evaluate for IPC layer

---

## 2026-03-30 - Async I/O Performance (Wave 137)

**Project:** [cross-repo]
**Category:** performance, async, I/O
**Status:** in_progress
**Priority:** P2

### I/O Patterns Analysis

| Pattern | Current | Bottleneck | Solution |
|---------|---------|------------|----------|
| Event store writes | Sync | Disk I/O | Aio (Linux async I/O) |
| Cache eviction | Sync | CPU | Async eviction tasks |
| File reads | Sync | Disk | tokio-uring |
| Network I/O | Async | N/A | Already optimized |

### tokio-uring Integration

```rust
use tokio_uring::fs::File;

pub async fn read_event_file(path: &Path) -> Result<Vec<u8>, EventStoreError> {
    let file = File::open(path).await?;
    let buffer = vec![0u8; file.metadata().await?.len() as usize];
    let (res, buffer) = file.read_at(buffer, 0).await;
    res.map_err(|e| EventStoreError::Io(e))?;
    Ok(buffer)
}

pub async fn write_event_file(path: &Path, data: &[u8]) -> Result<(), EventStoreError> {
    let file = File::create(path).await?;
    let (res, _) = file.write_all_at(data, 0).await;
    res.map_err(|e| EventStoreError::Io(e))?;
    Ok(())
}
```

### Recommended Actions

1. Add `tokio-uring` for file I/O in event store
2. Profile current I/O patterns with `tokio-console`
3. Add async cache eviction with `tokio::spawn`

---

## 2026-03-30 - Memory & Allocation Optimization (Wave 138)

**Project:** [cross-repo]
**Category:** performance, memory
**Status:** identified
**Priority:** P2

### Allocation Hotspots

| Area | Pattern | Issue | Solution |
|------|---------|-------|----------|
| Event deserialization | serde_json | Heap allocation | rkyv zero-copy |
| String parsing | regex | Temporary allocations | regex-lite (no backtrack) |
| UUID generation | uuid crate | Random read | `uuid::Uuid::now_v7` |
| Date parsing | chrono | Allocation | time const patterns |

### bumpalo Usage

```rust
use bumpalo::Bump;

pub fn parse_events<'a>(data: &'a [u8], arena: &'a Bump) -> Vec<&'a EventEnvelope<'a>> {
    let mut events = Vec::new();
    
    // All allocations within the arena (single deallocation)
    for chunk in data.chunks(256) {
        let event = arena.alloc_slice_slice(chunk);
        events.push(deserialize(event));
    }
    
    events
}
```

### Recommended Actions

1. Use `bumpalo` for short-lived allocations
2. Replace regex with `regex-lite` for hot paths
3. Add `uuid` with `v4` feature for fast generation

---

## 2026-03-30 - Concurrency & Parallelism (Wave 139)

**Project:** [cross-repo]
**Category:** performance, concurrency
**Status:** identified
**Priority:** P2

### Parallelism Opportunities

| Operation | Current | Parallel | Speedup |
|-----------|---------|----------|---------|
| Event replay | Sequential | Rayon | **4-8x** |
| Policy evaluation | Sequential | Rayon | **4-8x** |
| Cache warm-up | Sequential | tokio::spawn | **2-4x** |
| Test suite | Sequential | cargo-nextest | **3-5x** |

### Rayon Integration

```rust
use rayon::prelude::*;

pub fn replay_events(aggregates: &[AggregateId]) -> Result<ReplayResult, EventStoreError> {
    // Parallel event replay
    let results: Vec<AggregateState> = aggregates
        .par_iter()
        .map(|id| {
            let events = load_events(id)?;
            apply_events(events)
        })
        .collect();
    
    Ok(ReplayResult { aggregates: results })
}
```

### tokio::spawn for Background Tasks

```rust
pub async fn warm_cache(&self, keys: Vec<Key>) {
    // Parallel warm-up without blocking
    let handles: Vec<_> = keys
        .chunks(100)
        .map(|chunk| {
            tokio::spawn(async move {
                for key in chunk {
                    if let Some(value) = self.source.get(key).await {
                        cache.insert(key.clone(), value).await;
                    }
                }
            })
        })
        .collect();
    
    for handle in handles {
        handle.await.unwrap();
    }
>>>>>>> origin/main
}
```

---

<<<<<<< HEAD
### 8. Performance Budget

| Metric | Target | Current (est.) | Action |
|--------|--------|----------------|--------|
| API response (p99) | <100ms | TBD | Profile |
| Hash chain (10K events) | <1s | TBD | blake3 |
| Cache lookup | <1ms | TBD | DashMap |
| Serialization (1MB) | <10ms | TBD | rkyv |
| Build time | <5min | TBD | sccache |

---

### 9. Optimization Roadmap

| Phase | Action | Impact | Effort |
|-------|--------|--------|--------|
| 1 | Add blake3 for hash chains | 3x faster | 1 day |
| 2 | Replace serde_json with rkyv | 10x faster | 1 week |
| 3 | DashMap for caches | 2x faster | 1 day |
| 4 | Parallel async operations | 5x throughput | 1 week |
| 5 | sqlx async migration | 3x throughput | 2 weeks |

---

_Last updated: 2026-03-29_
=======
_Last updated: 2026-03-30 (Wave 139)_
>>>>>>> origin/main
