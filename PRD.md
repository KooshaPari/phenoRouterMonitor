# PRD - phenotype-infrakit

## E1: Event Sourcing

### E1.1: Append-Only Event Store
As a backend developer, I persist domain events in an append-only store with SHA-256 hash chain verification for tamper detection.

**Acceptance**: Events appended with sequence numbers; hash chain validates integrity; snapshot support for replay optimization.

## E2: Two-Tier Cache

### E2.1: LRU + DashMap Cache
As a service developer, I use a two-tier cache (L1 LRU in-process, L2 concurrent DashMap) with TTL expiration and metrics hooks.

**Acceptance**: L1 hit avoids L2 lookup; TTL eviction; `MetricsHook` reports hit/miss rates.

## E3: Policy Engine

### E3.1: Rule-Based Policy Evaluation
As a platform operator, I define allow/deny/require rules in TOML and evaluate them against a context to enforce security and compliance policies.

**Acceptance**: TOML config loading; allow/deny/require actions; severity levels; context-based evaluation.

## E4: State Machine

### E4.1: Generic Finite State Machine
As a workflow developer, I use a generic FSM with transition guards, forward-only enforcement, optional skip-state config, and history tracking.

**Acceptance**: Forward-only transitions; guard callbacks; skip-state support; full transition history.
