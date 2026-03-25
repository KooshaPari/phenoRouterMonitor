# phenotype-infrakit

Rust infrastructure toolkit extracted from the Phenotype ecosystem. Generic, domain-agnostic crates for event sourcing, caching, policy evaluation, and state machine management.

## Crates

| Crate | Description | Tests |
|-------|-------------|-------|
| [`phenotype-event-sourcing`](crates/phenotype-event-sourcing) | Append-only event store with SHA-256 hash chain verification, snapshot management, and pluggable storage backends | 15 |
| [`phenotype-cache-adapter`](crates/phenotype-cache-adapter) | Two-tier cache (L1 LRU + L2 DashMap) with TTL expiration and pluggable `MetricsHook` for observability | 7 |
| [`phenotype-policy-engine`](crates/phenotype-policy-engine) | Rule-based policy evaluation engine with allow/deny/require rules, TOML config loading, and severity levels | 43 |
| [`phenotype-state-machine`](crates/phenotype-state-machine) | Generic finite state machine with transition guards, forward-only enforcement, skip-state config, and history tracking | 11 |

## Quick Start

Add any crate as a git dependency:

```toml
[dependencies]
phenotype-event-sourcing = { git = "https://github.com/KooshaPari/phenotype-infrakit" }
phenotype-cache-adapter = { git = "https://github.com/KooshaPari/phenotype-infrakit" }
phenotype-policy-engine = { git = "https://github.com/KooshaPari/phenotype-infrakit" }
phenotype-state-machine = { git = "https://github.com/KooshaPari/phenotype-infrakit" }
```

## Usage Examples

### Event Sourcing

```rust
use phenotype_event_sourcing::{EventEnvelope, EventStore};
use phenotype_event_sourcing::memory::InMemoryEventStore;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct UserCreated { user_id: String, email: String }

let store = InMemoryEventStore::new();
let event = EventEnvelope::new(
    UserCreated { user_id: "u-1".into(), email: "a@b.com".into() },
    "system",
);
let seq = store.append(&event, "UserCreated").unwrap();
```

### Cache

```rust
use phenotype_cache_adapter::TieredCache;
use std::time::Duration;

let cache = TieredCache::new(100, Duration::from_secs(300), None);
cache.insert("key".to_string(), "value".to_string());
assert_eq!(cache.get(&"key".to_string()), Some("value".to_string()));
```

### Policy Engine

```rust
use phenotype_policy_engine::{PolicyEngine, PolicyContext};

let engine = PolicyEngine::new();
engine.load_toml_str(r#"
[[rules]]
name = "require-auth"
action = "require"
field = "auth_token"
"#).unwrap();

let mut ctx = PolicyContext::new();
ctx.set("auth_token", "abc123");
let result = engine.evaluate(&ctx).unwrap();
assert!(result.passed());
```

### State Machine

```rust
use phenotype_state_machine::{State, StateMachine};

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
enum Status { Draft, Review, Approved, Done }

impl State for Status {
    fn ordinal(&self) -> u32 {
        match self { Self::Draft => 0, Self::Review => 1, Self::Approved => 2, Self::Done => 3 }
    }
}

let mut sm = StateMachine::new(Status::Draft);
sm.transition(Status::Review).unwrap();
assert_eq!(sm.current(), &Status::Review);
```

## Development

```bash
cargo test --workspace      # Run all 76 tests
cargo clippy --workspace    # Lint
cargo fmt --check           # Format check
```

## Architecture

Each crate is fully independent with no inter-crate dependencies. They share workspace-level dependency versions for consistency but can be consumed individually.

```
phenotype-infrakit/
  Cargo.toml              # Workspace root
  crates/
    phenotype-event-sourcing/   # EventStore trait + InMemoryEventStore + hash chains
    phenotype-cache-adapter/    # TieredCache<K,V> with L1/L2 + TTL + MetricsHook
    phenotype-policy-engine/    # PolicyEngine + Rule + TOML loader + Context
    phenotype-state-machine/    # StateMachine<S> + TransitionGuard + history
```

## License

MIT
