# Code Entity Map - phenotype-infrakit

## Forward Map (Code -> Requirements)

| Entity | Crate | FR |
|--------|-------|----|
| `EventStore` trait | phenotype-event-sourcing | FR-EVT-001, FR-EVT-004 |
| `InMemoryEventStore` | phenotype-event-sourcing | FR-EVT-001, FR-EVT-004 |
| `EventEnvelope` | phenotype-event-sourcing | FR-EVT-002 |
| `Snapshot` | phenotype-event-sourcing | FR-EVT-003 |
| `TieredCache` | phenotype-cache-adapter | FR-CACHE-001, FR-CACHE-002 |
| `MetricsHook` trait | phenotype-cache-adapter | FR-CACHE-003 |
| `PolicyEngine` | phenotype-policy-engine | FR-POL-001, FR-POL-004 |
| `Rule` | phenotype-policy-engine | FR-POL-002, FR-POL-003 |
| `PolicyContext` | phenotype-policy-engine | FR-POL-004 |
| `StateMachine` | phenotype-state-machine | FR-SM-001, FR-SM-003 |
| `TransitionGuard` | phenotype-state-machine | FR-SM-002 |

## Reverse Map (Requirements -> Code)

| FR | Primary Entities |
|----|-----------------|
| FR-EVT-001 | `EventStore`, `InMemoryEventStore::append` |
| FR-EVT-002 | `EventEnvelope` (hash field) |
| FR-EVT-003 | `Snapshot` |
| FR-EVT-004 | `EventStore` trait |
| FR-CACHE-001 | `TieredCache::get` |
| FR-CACHE-002 | `TieredCache` (TTL check) |
| FR-CACHE-003 | `MetricsHook` |
| FR-POL-001 | `PolicyEngine::load_toml_str` |
| FR-POL-002 | `Rule` (action field) |
| FR-POL-003 | `Rule` (severity field) |
| FR-POL-004 | `PolicyEngine::evaluate`, `PolicyContext` |
| FR-SM-001 | `StateMachine::transition` (ordinal check) |
| FR-SM-002 | `TransitionGuard` |
| FR-SM-003 | `StateMachine` (history vec) |
