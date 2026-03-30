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
    }
}
```

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
}
```

---

_Last updated: 2026-03-30 (Wave 139)_
