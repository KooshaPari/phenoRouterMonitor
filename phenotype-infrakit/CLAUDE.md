# CLAUDE.md — phenotype-infrakit

## Identity

**Name**: phenotype-infrakit
**Type**: Rust workspace (internal infrastructure library)
**Location**: `/Users/kooshapari/CodeProjects/Phenotype/repos/phenotype-infrakit`

## Authority and Scope

This is the canonical reference for phenotype-infrakit. This file provides project-specific guidance that supplements the shelf-level `AGENTS.md`.

## Project Structure

```
phenotype-infrakit/
├── crates/                     # Independent crates (no inter-crate deps)
├── tests/                     # Integration tests
├── docs/                      # Architecture documentation
├── Cargo.toml                 # Workspace manifest
├── clippy.toml                # Linter config
├── deny.toml                  # Dependency audit config
└── _typos.toml               # Typo detection config
```

## Build & Test Commands

```bash
# Full quality check
cargo test --workspace && cargo clippy --workspace -- -D warnings && cargo fmt --check

# Build all crates
cargo build --workspace

# Build specific crate
cargo build -p phenotype-event-sourcing

# Run tests for specific crate
cargo test -p phenotype-cache-adapter

# Generate docs
cargo doc --workspace --no-deps
```

## Crate Inventory

| Crate | Purpose |
|-------|---------|
| `phenotype-event-sourcing` | Append-only event store with SHA-256 hash chains |
| `phenotype-cache-adapter` | Two-tier LRU + DashMap cache with TTL |
| `phenotype-policy-engine` | Rule-based policy evaluation with TOML config |
| `phenotype-state-machine` | Generic FSM with transition guards |
| `phenotype-contracts` | Shared traits and types |
| `phenotype-error-core` | Canonical error types |
| `phenotype-health` | Health check abstraction |
| `phenotype-config-core` | Configuration management |

## Key Constraints

1. **No inter-crate dependencies** - Each crate must be independently consumable
2. **Full type annotations** - No `impl Trait` in public APIs unless necessary
3. **Error handling** - Use `thiserror` with proper `#[from]` conversions
4. **Size limits** - ≤500 lines per source file

## See Also

- `AGENTS.md` - Full agent rules and governance
- `docs/adr/` - Architecture decision records
