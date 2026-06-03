> **Work state:** MISLABELED-SHELF · **Progress:** `███░░░░░░░ 30%`
> ⚠ Repo name is a MISNOMER — this is **Phenotype InfraKit** (shared infra crates), NOT a router monitor. ~11 crates duplicate the canonical kit [HexaKit](https://github.com/KooshaPari/HexaKit); canonical routing = [OmniRoute](https://github.com/KooshaPari/OmniRoute) (LLM) + [Tokn](https://github.com/KooshaPari/Tokn) `tokenledger::routing` (Rust). Pending dedup/rehome decision (dom-services-routing #24). · updated 2026-06-02

# Phenotype InfraKit

Shared infrastructure crates extracted from the Phenotype ecosystem.

This workspace contains generic infrastructure components that are shared across Phenotype services.

## Crates

- `phenotype-error-core`: Canonical error types for the Phenotype ecosystem
- `phenotype-git-core`: Phenotype git core crate
- `phenotype-health`: Shared health check abstraction for Phenotype services
- `phenotype-config-core`: Unified configuration loading and management for Phenotype ecosystem
- `phenotype-telemetry`: Telemetry and observability infrastructure
- `phenotype-validation`: Data validation infrastructure
- `phenotype-event-sourcing`: Append-only event store with SHA-256 hash chains
- `phenotype-cache-adapter`: Two-tier LRU + DashMap cache with TTL
- `phenotype-policy-engine`: Rule-based policy evaluation with TOML config
- `phenotype-state-machine`: Generic FSM with transition guards
- `phenotype-contracts`: Shared traits and types

## Usage

Add to your Cargo.toml:

```toml
[dependencies]
phenotype-error-core = { path = "../phenotype-infrakit/crates/phenotype-error-core" }
# ... other phenotype crates
```

## License

MIT
