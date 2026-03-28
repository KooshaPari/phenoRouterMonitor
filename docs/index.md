---
layout: home
hero:
  name: phenotype-infrakit
  tagline: Rust infrastructure toolkit for the Phenotype ecosystem.
---

# phenotype-infrakit

Rust infrastructure toolkit extracted from the Phenotype ecosystem. Generic, domain-agnostic crates for event sourcing, caching, policy evaluation, and state machine management.

## Crates

- **event-sourcing** — event store and projection primitives
- **cache-adapter** — unified caching abstraction over multiple backends
- **policy-engine** — policy evaluation and enforcement
- **state-machine** — typed state machine implementation

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
phenotype-infrakit = { git = "https://github.com/KooshaPari/phenotype-infrakit" }
```
