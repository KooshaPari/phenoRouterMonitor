# Comparison Matrix

## Feature Comparison

This document compares **phenotype-infrakit** with similar tools in the Rust infrastructure crates space.

| Repository | Purpose | Key Features | Language/Framework | Maturity | Comparison |
|------------|---------|--------------|-------------------|----------|------------|
| **phenotype-infrakit (this repo)** | Rust infrastructure toolkit | Event sourcing, Caching, Policy engine, State machine | Rust | Stable | Phenotype ecosystem |
| [Tokio](https://github.com/tokio-rs/tokio) | Async runtime | Futures, Tasks, Timers | Rust | Stable | Async foundation |
| [Serde](https://github.com/serde-rs/serde) | Serialization | JSON, TOML, YAML, etc. | Rust | Stable | Serialization std |
| [DashMap](https://github.com/xacrimon/dashmap) | Concurrent map | Sharded map | Rust | Stable | Concurrent hashmap |
| [LRU Cache](https://github.com/jeromefroe/lru-rs) | LRU cache | Memory-bounded | Rust | Stable | Simple cache |
| [Rhai](https://github.com/rhaiscript/rhai) | Scripting engine | Embedded scripting | Rust | Stable | Policy scripting |
| [XState](https://github.com/statelyai/xstate) | State machines | Formal models | TypeScript | Stable | Industry standard |

## Detailed Feature Comparison

### Crates

| Crate | phenotype-infrakit | tokio | serde | DashMap | LRU |
|-------|-------------------|-------|-------|---------|-----|
| Event Sourcing | ✅ | ❌ | ❌ | ❌ | ❌ |
| Cache (Two-tier) | ✅ | ❌ | ❌ | ✅ | ✅ |
| Policy Engine | ✅ | ❌ | ❌ | ❌ | ❌ |
| State Machine | ✅ | ❌ | ❌ | ❌ | ❌ |

### Event Sourcing

| Feature | phenotype-infrakit | EventStoreDB | Event Sourcing |
|---------|-------------------|--------------|----------------|
| Append-only store | ✅ | ✅ | N/A |
| SHA-256 hash chain | ✅ | ❌ | N/A |
| Snapshot management | ✅ | ✅ | N/A |
| Pluggable backends | ✅ | ✅ | N/A |
| In-memory implementation | ✅ | ❌ | N/A |

### Cache Adapter

| Feature | phenotype-infrakit | LRU | DashMap | go-redis |
|---------|-------------------|-----|---------|----------|
| Two-tier (L1/L2) | ✅ | ❌ | ❌ | ❌ |
| LRU (L1) | ✅ | ✅ | ❌ | ❌ |
| DashMap (L2) | ✅ | ❌ | ✅ | ❌ |
| TTL expiration | ✅ | ❌ | ❌ | ✅ |
| Metrics hooks | ✅ | ❌ | ❌ | ✅ |

### Policy Engine

| Feature | phenotype-infrakit | OPA | Rhai | custom |
|---------|-------------------|-----|------|--------|
| TOML config | ✅ | ❌ | ❌ | N/A |
| Allow/deny/require | ✅ | ✅ | ✅ | N/A |
| Severity levels | ✅ | ❌ | ❌ | N/A |
| Field-based rules | ✅ | ✅ | ✅ | N/A |
| Lightweight | ✅ | ❌ | ✅ | N/A |

### State Machine

| Feature | phenotype-infrakit | XState | simple_state_machine |
|---------|-------------------|--------|---------------------|
| Generic states | ✅ | ✅ | ✅ |
| Transition guards | ✅ | ✅ | ❌ |
| Forward-only | ✅ | ❌ | ❌ |
| History tracking | ✅ | ✅ | ❌ |
| Skip-state config | ✅ | ❌ | ❌ |

## Unique Value Proposition

phenotype-infrakit provides:

1. **Independent Crates**: No inter-crate dependencies, consume individually
2. **Event Sourcing with Hash Chain**: SHA-256 verification for append-only stores
3. **Two-Tier Cache**: L1 LRU + L2 DashMap for optimized caching
4. **Lightweight Policy Engine**: TOML-based rules without full OPA overhead

## Crates

| Crate | Tests | Description |
|-------|-------|-------------|
| `phenotype-event-sourcing` | 15 | Append-only event store with hash chain verification |
| `phenotype-cache-adapter` | 7 | Two-tier cache with TTL and MetricsHook |
| `phenotype-policy-engine` | 43 | Rule-based policy evaluation with TOML config |
| `phenotype-state-machine` | 11 | Generic FSM with transition guards |

## References

- Tokio: [tokio-rs/tokio](https://github.com/tokio-rs/tokio)
- Serde: [serde-rs/serde](https://github.com/serde-rs/serde)
- XState: [statelyai/xstate](https://github.com/statelyai/xstate)
- OPA: [open-policy-agent/opa](https://github.com/open-policy-agent/opa)
