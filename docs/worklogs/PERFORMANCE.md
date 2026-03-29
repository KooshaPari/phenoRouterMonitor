# Performance Worklogs

**Category:** PERFORMANCE | **Updated:** 2026-03-29

---

## 2026-03-29 - Deep Performance Audit: Async & Memory Patterns

**Project:** [AgilePlus]
**Category:** performance
**Status:** in_progress
**Priority:** P1

### Summary

Deep audit of async patterns, memory allocation hotspots, and caching opportunities across the codebase.

### Async Runtime Analysis

#### tokio::spawn Usage Patterns

| Crate | Pattern | Issue |
|-------|---------|-------|
| agileplus-cli | Sequential command dispatch | Could parallelize independent ops |
| agileplus-sync | Sequential NATS publish | Batch publishes possible |
| agileplus-events | Sequential event append | Pipeline possible |

#### Memory Allocation Hotspots

| Location | Pattern | Issue |
|----------|---------|-------|
| `phenotype-event-sourcing/src/hash.rs` | SHA-256 chain | Allocations per hash |
| `phenotype-event-sourcing/src/memory.rs` | Arc::new patterns | Atomic ref counting |
| `phenotype-policy-engine/src/*.rs` | Mutex::new in hot paths | Contention possible |

#### Caching Opportunities

| Area | Current | Opportunity |
|------|---------|-------------|
| Config loading | Every startup | Cache in Arc<RwLock> |
| Git operations | No caching | Cache branch refs |
| Event queries | No caching | LRU cache for frequent queries |
| Graph queries | Neo4j only | Local SQLite fallback |

### CLI Performance Issues

| Command | Issue | Impact |
|---------|-------|--------|
| `agileplus specify` | Full git scan | 2-5s cold start |
| `agileplus plan` | Sequential research | 10-30s per feature |
| `agileplus validate` | Full governance check | 5-10s per WP |

### Optimization Action Items

- [ ] 🟡 HIGH: Add LRU cache to event queries
- [ ] 🟡 HIGH: Parallelize independent git operations
- [ ] 🟠 MEDIUM: Profile memory allocations in hash chain
- [ ] 🟠 MEDIUM: Add config caching layer
- [ ] 🟢 LOW: Implement batch NATS publishes

### Related

- `crates/agileplus-events/src/hash.rs`
- `crates/agileplus-events/src/query.rs`
- `crates/agileplus-cli/src/commands/specify.rs`

---

## 2026-03-29 - TokenLedger Benchmarking Infrastructure

**Project:** [AgilePlus]
**Category:** performance
**Status:** in_progress
**Priority:** P2

### Summary

TokenLedger has comprehensive benchmarking infrastructure that could be shared across the ecosystem.

### Existing Benchmarks

```
tooling/tools/tokenledger/crates/tokenledger/src/benchmarks/
├── overrides.rs
├── openrouter.rs
├── cliproxy_metrics.rs
├── store.rs
├── thegent_adapter.rs
├── artificial_analysis.rs
└── cli.rs
```

### Benchmark Patterns

| Pattern | Location | Reuse Potential |
|---------|----------|-----------------|
| Token cost calculation | `pricing.rs` | High |
| Cache benchmarking | `cache.rs` | High |
| Model comparison | `bench.rs` | High |
| Cost aggregation | `cost.rs` | High |

### Reuse Opportunities

1. **Share token cost calculation** across CLI and API
2. **Extract benchmark framework** to shared crate
3. **Add Prometheus metrics** to all performance-critical paths

### Action Items

- [ ] 🟠 MEDIUM: Extract benchmark helpers to shared crate
- [ ] 🟠 MEDIUM: Add benchmarks to agileplus-cli commands
- [ ] 🟢 LOW: Create unified metrics dashboard

### Related

- `tooling/tools/tokenledger/src/bench.rs`
- `tooling/tools/tokenledger/src/pricing.rs`

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
