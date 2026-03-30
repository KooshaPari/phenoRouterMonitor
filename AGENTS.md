<!-- Base: platforms/thegent/governance/AGENTS.base.md -->
<!-- Last synced: 2026-03-29 -->

# AGENTS.md — phenotype-infrakit

**Project**: Rust workspace containing generic infrastructure crates (26 crates, ~4K LOC).
**AgilePlus**: Track all work at `/Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus`.

---

## Codebase Map (What's Where)

| Path | Purpose | Lang | Entry Point | Key Pattern |
|------|---------|------|-------------|-------------|
| `crates/phenotype-contracts/` | Shared traits & types | Rust | `src/lib.rs` | Ports & adapters |
| `crates/phenotype-error-core/` | 5 canonical error types | Rust | `src/lib.rs` | thiserror + From |
| `crates/phenotype-event-sourcing/` | Append-only event store | Rust | `src/store.rs` | SHA-256 hash chain |
| `crates/phenotype-cache-adapter/` | LRU + DashMap cache | Rust | `src/cache.rs` | Two-tier TTL |
| `crates/phenotype-health/` | Health check trait | Rust | `src/checker.rs` | Port interface |
| `crates/phenotype-config-core/` | Config loader (figment) | Rust | `src/loader.rs` | UnifiedConfig |
| `crates/phenotype-policy-engine/` | Rule evaluation | Rust | `src/evaluator.rs` | TOML rules |
| `crates/phenotype-state-machine/` | Generic FSM | Rust | `src/fsm.rs` | Transition guards |
| `crates/phenotype-retry/` | Retry strategies | Rust | `src/strategies.rs` | Exponential backoff |
| `crates/phenotype-mcp/` | MCP protocol | Rust | `src/mcp.rs` | Tool registry |
| `crates/phenotype-validation/` | Input validation | Rust | `src/validators.rs` | Trait-based |
| `crates/phenotype-telemetry/` | Observability | Rust | `src/tracer.rs` | OTEL adapter |
| `crates/agileplus-domain/` | AgilePlus domain models | Rust | `src/models.rs` | Entity aggregate |
| `crates/agileplus-api-types/` | API request/response types | Rust | `src/lib.rs` | Serde types |
| `python/phenosdk/` | Python SDK core | Python | `src/phenosdk/__init__.py` | pyproject.toml |
| `docs/adr/` | Architecture decisions | Markdown | - | ADR-XXX format |
| `docs/reference/` | Quick refs & trackers | Markdown | `FR_TRACKER.md` | Query tables |
| `tests/` | Integration tests | Rust | `tests/integration_test.rs` | `.rs` files |
| `.archive/` | Obsolete code | Mixed | - | **Read-only** |

---

## Key Patterns

**Hexagonal Architecture**: Ports (traits) in `phenotype-contracts`, adapters in each crate. No inter-crate deps.

**Error Handling**: 5 canonical types in `phenotype-error-core` (Config, Event, Cache, Policy, Parse). Use `thiserror #[from]`.

**Tests**: Inline `#[cfg(test)]` modules in source files. Trace all to FR-XXX via `// Traces to: FR-PHENO-NNN`.

**Python**: src-layout, pyproject.toml, tox for testing. No setup.py.

---

## Common Tasks

### Add Rust Crate
1. `cargo new --lib crates/phenotype-{name}`
2. Add to root `Cargo.toml` `[workspace]`
3. Create test module: `#[cfg(test)] mod tests { ... }`
4. Export public types in `src/lib.rs`
5. No dependency on sibling crates

### Add Python Package
1. `mkdir -p python/{name}/src/{name}`
2. Create `pyproject.toml` (copy from `phenosdk`)
3. Add `__init__.py` with version from `src/phenosdk/__version__.py`
4. Test via `cd python/{name} && tox`

### Run Quality Checks
```bash
cargo test --workspace              # All tests
cargo clippy --workspace -- -D warnings  # Lint (zero warnings)
cargo fmt --check                   # Format check
python -m pytest python/ -v         # Python tests
```

### Create PR
1. Branch: `git checkout -b chore/feature-name`
2. Code + test (test-first)
3. Lint: `cargo fmt && cargo clippy --workspace -- -D warnings`
4. Commit: small, focused, single concern
5. PR: reference AgilePlus spec, target `main`

---

## Don't Touch

- **`.archive/`** — Obsolete code, read-only reference only
- **`worktrees/`** — Managed by git, never edit directly; use if you need a branch
- **`.agileplus/`** — AgilePlus database, read-only
- **`platforms/thegent/governance/`** — Canonical base docs, extend locally via `AGENTS.md`

---

## Work Requirements

1. **Check AgilePlus spec** before implementing: `agileplus list --filter "in-progress"`
2. **Trace tests to FR**: `// Traces to: FR-PHENO-NNN`
3. **Zero lint errors**: `cargo clippy --workspace -- -D warnings` must pass
4. **No inter-crate deps**: Each crate independent (test with `cargo build -p <crate>`)
5. **Max file size**: 500 lines (prefer ≤350). Split if larger.

---

## Style Constraints

- **Line length**: 100 characters
- **Formatter**: `cargo fmt` (mandatory)
- **Linter**: `cargo clippy -- -D warnings` (zero warnings)
- **File size**: ≤350 lines preferred, hard limit 500 lines
- **Types**: Full annotations required; no `impl Trait` in public APIs

---

## Governance Base

See `platforms/thegent/governance/AGENTS.base.md` for:
- Agent expectations (autonomy, decision rules)
- SWE autopilot loop (review → research → plan → execute → test → polish)
- File modularity & decomposition
- Branch discipline & PR workflow
- Child-agent delegation policy
