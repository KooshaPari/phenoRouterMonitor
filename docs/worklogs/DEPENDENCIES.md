# Dependencies Worklogs

**Category:** DEPENDENCIES | **Updated:** 2026-03-31

───────────────────────────────────────

## 2026-03-31 - Current Crate Inventory

**Project:** [phenotype-infrakit]
**Status:** completed
**Priority:** P0

### Crate Status

| Crate | LOC | Status | Action |
|-------|-----|--------|--------|
| `phenotype-event-sourcing` | 1,576 | Production | Keep, publish |
| `phenotype-contracts` | 1,440 | Production | Keep, publish |
| `phenotype-policy-engine` | 1,398 | Production | Keep, unique TOML loader |
| `phenotype-git-core` | 1,056 | Production | Keep, integrate |
| `phenotype-config-core` | 949 | UNUSED | Migrate to edition 2024 |
| `phenotype-retry` | 400+ | IN USE | Keep, integrate |
| `phenotype-telemetry` | 400+ | IN USE | Keep, expand |
| `phenotype-error-core` | NEW | IN PROGRESS | Create |
| `phenotype-cost-core` | NEW | IN PROGRESS | Create |

### Already Implemented

| Crate | Implementation | Status |
|-------|-----------------|--------|
| `phenotype-retry` | `retry`, `RetryConfig`, `retry_with_config!` | ✅ Done |
| `phenotype-config-core` | TOML loader, `FromEnv`, `ConfigError` | ✅ Done |
| `phenotype-telemetry` | OTLP export, `ConsoleExporter` | ✅ Done |
| `phenotype-git-core` | Git operations | ✅ Done |

───────────────────────────────────────

## 2026-03-29 - External Dependencies & Package Modernization Audit

**Project:** [cross-repo]
**Category:** dependencies
**Status:** in_progress
**Priority:** P0

### Summary

Comprehensive audit of external dependencies, package modernization opportunities, and fork candidates.

### Fork Candidates (Internal → Shared Libraries)

| ID | Source | Target | LOC | Priority | Status |
|----|--------|--------|-----|----------|--------|
| FORK-001 | `utils/pty` | `phenotype-process` | ~750 | 🔴 CRITICAL | TODO |
| FORK-002 | `error.rs` pattern | `phenotype-error` | ~400 | 🔴 CRITICAL | IN PROGRESS |
| FORK-003 | `utils/git` | `phenotype-git` | ~300 | 🟠 MEDIUM | DONE |
| FORK-004 | `utils/config` | `phenotype-config` | ~200 | 🟠 MEDIUM | DONE |

### External Dependencies Assessment

#### Standard Crates (Optimal - No Action Needed) ✅

| Crate | Version | Assessment |
|-------|---------|------------|
| `serde` | 1.x | Standard - no action needed |
| `serde_json` | 1.x | Standard - no action needed |
| `tokio` | 1.x | Standard - no action needed |
| `thiserror` | 2.x | Standard - pattern upgrade only |
| `anyhow` | 1.x | Standard - pattern upgrade only |
| `rusqlite` | 0.32 | Standard - no action needed |
| `axum` | 0.8 | Standard - no action needed |
| `tonic` | 0.13 | Standard - no action needed |
| `tracing` | 0.1 | Standard - no action needed |
| `clap` | 4.x | Standard - no action needed |

#### Modern Tooling Already Integrated ✅

| Tool | Usage | Location |
|------|-------|----------|
| `uv` | Python package management | `python/Dockerfile.python`, `python/pyproject.toml` |
| `ruff` | Python linting/formatting | `python/ruff.toml`, CI pipeline |
| `gix` | Git operations (v0.79) | `Cargo.toml`, `agileplus-git` |

───────────────────────────────────────

## 2026-03-29 - External Repo Dependency Audit (Blackbox vs Whitebox)

**Project:** [cross-repo]
**Category:** dependencies
**Status:** completed
**Priority:** P0

### Blackbox Dependency Assessment (Usage As-Is)

| Dependency | Project | Status | Rationale |
|---|---|---|---|
| **mcp-sdk-rust** | heliosCLI | ✅ ADOPT | Official Anthropic SDK |
| **rig-core** | thegent | ✅ ADOPT | Cleanest Rust LLM orchestration |
| **sqlx v0.8** | phenotype-infrakit | ✅ UPGRADE | Native async SQLite/Postgres |
| **axum v0.8** | All Rust APIs | ✅ STANDARD | Modern, tower-based HTTP |

### Graybox Dependency Assessment (Wrapping/Adapting)

| Dependency | Project | Wrapper | Purpose |
|---|---|---|---|
| **gix** | AgilePlus | `phenotype-git` | Git ops with Port traits |
| **wasmtime** | thegent | `phenotype-sandbox` | Sandbox tool execution |
| **figment** | All Rust | `phenotype-config` | Hierarchical config loading |

### Whitebox Dependency Assessment (Forking/Modification)

| Dependency | Project | Reason | Est. Value |
|---|---|---|---|
| **eventually-rs** | phenotype-infrakit | Need native NATS/SQLite adapters | `phenotype-event-sourcing` |
| **helios-pty** | heliosCLI | Custom process group + terminal resizing | `phenotype-process` (750 LOC) |

───────────────────────────────────────

## 2026-03-30 - Error Core Deep Dive

**Project:** [phenotype-infrakit]
**Category:** dependencies
**Status:** in_progress
**Priority:** P0

### Summary

Analysis of error core library for centralized error handling.

### Error Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    phenotype-error                          │
├─────────────────────────────────────────────────────────────┤
│  Core Error Types                                          │
│  ├── Source (IO, Parse, Network, Timeout)                 │
│  ├── Domain (Validation, NotFound, Conflict)               │
│  ├── Application (Config, Auth, Permission)                │
│  └── Infrastructure (Database, Cache, Queue)                │
├─────────────────────────────────────────────────────────────┤
│  Error Traits                                             │
│  ├── Error::source() - cause chain                       │
│  ├── Error::report() - structured reporting              │
│  └── Error::retryable() - automatic retry                │
├─────────────────────────────────────────────────────────────┤
│  Derive Macros                                            │
│  ├── #[from] - automatic From implementations            │
│  ├── #[context] - structured context                     │
│  └── #[retry] - retry policy                           │
└─────────────────────────────────────────────────────────────┘
```

### Design Principles

#### 1. Error Hierarchy

```rust
// Core error type
pub enum Error {
    // Source errors
    Io { #[from] source: std::io::Error },
    Parse { #[from] source: serde_json::Error },

    // Domain errors
    NotFound { entity: &'static str, id: String },
    ValidationFailed { field: String, message: String },

    // Application errors
    Unauthorized { reason: String },

    // Infrastructure errors
    Database { #[from] source: sqlx::Error },
}
```

#### 2. Retryable Trait

```rust
pub trait Retryable {
    fn retry_policy(&self) -> RetryPolicy;
    fn is_retryable(&self) -> bool;
}

#[derive(Debug, Clone)]
pub enum RetryPolicy {
    None,
    Fixed { max_attempts: u32, delay: Duration },
    Exponential { max_attempts: u32, base: Duration, max_delay: Duration },
}
```

### Action Items

- [x] ERR-001: Create phenotype-error-core crate
- [ ] ERR-002: Define core error types
- [ ] ERR-003: Implement Retryable trait
- [ ] ERR-004: Add context propagation
- [ ] ERR-005: Migrate phenotype-contracts
- [ ] ERR-006: Migrate phenotype-event-sourcing

───────────────────────────────────────

## 2026-03-30 - External Package Optimization Opportunities

**Project:** [cross-repo]
**Category:** dependencies
**Status:** in_progress
**Priority:** P1

### External Package Decision Matrix

| Category | Current | External | Decision | LOC Savings |
|---------|---------|----------|----------|-------------|
| Process Management | Manual Command | command-group | ADOPT | ~1,000 |
| Config Loading | 4 loaders | figment | ADOPT | ~600 |
| Error Handling | 36 enums | phenotype-error | IN PROGRESS | ~400 |
| Git Operations | git2 | gix | DONE | ~200 |
| Serialization | JSON only | rkyv | EVALUATE | ~300 |
| Health Checks | 6 enums | health-check | EVALUATE | ~100 |

### Critical Adoptions

#### 1. command-group (Process Management)

```toml
[dependencies]
command-group = "5.0"
```

```rust
// Automatic process group
use command_group::{AsyncCommandGroup, AsyncSignalSafeCommand};
let group = AsyncCommandGroup::new()
    .command("docker")
    .args(["run", "-it", "image"])
    .spawn()?;

// Automatic kill on drop - all children terminated
drop(group);
```

#### 2. figment (Configuration) - ALREADY DONE ✅

```rust
// phenotype-config-core already implements figment-based loading
let config: Config = Figment::new()
    .merge(Toml::file("config.toml"))
    .merge(Env::prefixed("APP_"))
    .extract()?;
```

#### 3. gix (Git Operations) - ALREADY DONE ✅

```rust
// phenotype-git-core already uses gix
let repo = gix::discover(path)?;
let remote = repo.find_remote("origin")?;
```

### Version Recommendations

```toml
[workspace.dependencies]

# Critical - Adopt now
command-group = "5.0"
figment = "0.10"
indicatif = "0.18"
console = "0.16"

# High - Evaluate this quarter
gix = { version = "0.80", features = ["worktree-mutation"] }
rkyv = "0.8"
parking_lot = "0.12"

# Medium - Monitor
anyhow = "1.0"
thiserror = "2.0"
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.42", features = ["full"] }
```

───────────────────────────────────────

## 2026-03-30 - Performance Optimization: Async Patterns

**Project:** [cross-repo]
**Category:** dependencies
**Status:** in_progress
**Priority:** P1

### Async Runtime Analysis

| Metric | Current | Target | Improvement |
|--------|---------|--------|-------------|
| Task spawn overhead | High | Low | 40% reduction |
| Channel capacity | 1 | Dynamic | 50% reduction |
| Lock contention | High | Low | 60% reduction |

### Tokio Best Practices

#### 1. Task Spawning

```rust
// AFTER: Bounded concurrency
async fn process_batch(items: Vec<Item>) -> Vec<Result<()>> {
    let semaphore = Arc::new(Semaphore::new(32));
    let mut handles = Vec::with_capacity(items.len());

    for item in items {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        handles.push(tokio::spawn(async move {
            let result = process(item).await;
            drop(permit);
            result
        }));
    }

    futures::future::join_all(handles).await
}
```

#### 2. Channel Sizing

```rust
// BEFORE: Unbuffered
let (tx, rx) = tokio::sync::mpsc::channel::<Message>(1);

// AFTER: Bounded with backpressure
let (tx, rx) = tokio::sync::mpsc::channel::<Message>(100);
```

#### 3. Lock Optimization

```rust
// AFTER: RwLock for read-heavy workloads
let data = Arc::new(RwLock::new(HashMap::new()));
let guard = data.read().await; // Multiple readers
```

### Action Items

- [ ] ASYNC-001: Audit unbounded tokio::spawn
- [ ] ASYNC-002: Replace unbuffered channels
- [ ] ASYNC-003: Migrate to RwLock where applicable
- [ ] ASYNC-004: Implement connection pooling
- [ ] ASYNC-005: Add memory profiling

───────────────────────────────────────

## 2026-03-29 - 2026 Crate Landscape Assessment

**Project:** [cross-repo]
**Category:** dependencies
**Status:** completed
**Priority:** P1

### AI/LLM Integration (2026)

| Crate | Version | Purpose | Assessment |
|-------|---------|---------|------------|
| `anthropic` | 0.3.0 | Claude API SDK | **ADD** - First-class async |
| `anthropic-sdk-core` | 0.3.0 | Core types | **ADD** - Streaming, tools |
| `llm-chain` | 0.5.0 | Multi-provider LLM | **EVALUATE** - Tool use |
| `tiktoken` | 0.5.0 | BPE tokenization | **EVALUATE** - Cost tracking |

### Agent Frameworks (2026)

| Crate | Version | Purpose | Assessment |
|-------|---------|---------|------------|
| `agent-p` | 0.2.0 | Agent primitives | **EVALUATE** - MCP integration |
| `open-agent` | 0.1.0 | OpenAI agents | **EVALUATE** - Tool calling |
| `mcp-sdk` | 0.1.0 | Model Context Protocol | **EVALUATE** - Standard tool protocol |

### Observability & Tracing (2026)

| Crate | Version | Purpose | Assessment |
|-------|---------|---------|------------|
| `ratatui` | 0.28.0 | Terminal UI | **ADOPT** - TUI dashboards |
| `tokio-console` | 0.2.0 | Async debugging | **ADOPT** - Debugging |
| `tracing-flame` | 0.2.0 | Flame graphs | **EVALUATE** - Performance |

### Performance & Optimization (2026)

| Crate | Version | Purpose | Assessment |
|-------|---------|---------|------------|
| `cargo-flamegraph` | 0.6.0 | Profiling | **ADOPT** - Already using |
| `cargo-nextest` | 0.9.0 | Test runner | **ADOPT** - Parallel tests |
| `cargo-hack` | 0.5.0 | Feature flags | **EVALUATE** - CI |
| `sccache` | 0.8.0 | Shared cache | **EVALUATE** - CI caching |

### Database & Storage (2026)

| Crate | Version | Purpose | Assessment |
|-------|---------|---------|------------|
| `sqlx` | 0.8.0 | Async SQL | **EVALUATE** - Migration from rusqlite |
| `sea-orm` | 1.0.0 | Async ORM | **EVALUATE** - Complex queries |
| `sled` | 0.34.0 | Embedded KV | **EVALUATE** - Local caching |
| `rocksdb` | 0.22.0 | RocksDB bindings | **EVALUATE** - Performance |

### Serialization (2026)

| Crate | Version | Purpose | Assessment |
|-------|---------|---------|------------|
| `rkyv` | 0.8.0 | Zero-copy | **EVALUATE** - Performance |
| `postcard` | 1.0.0 | No-std | **EVALUATE** - Embedded |
| `speedy` | 0.13.0 | Fast | **EVALUATE** - Cross-language |

───────────────────────────────────────

## 2026-03-29 - Security Advisories

**Project:** [cross-repo]
**Category:** dependencies
**Status:** completed
**Priority:** P0

### Crates with Known Issues

| Crate | Issue | Severity | Mitigation |
|-------|-------|----------|------------|
| `git2` | CVE-2024-XXXX | 🟠 MEDIUM | Pin to `>=0.20.0`, use `gix` |
| `rusqlite` | Memory safety | 🟢 LOW | Use `bundled` feature |
| `chrono` | Timezone issues | 🟡 MEDIUM | Use `chrono-tz` for TZ handling |

### Recommended Versions (2026-03)

```toml
[workspace.dependencies]
# Security-sensitive
git2 = { version = "=0.20.0", features = ["vendored-openssl"] }
rusqlite = { version = "0.32", features = ["bundled"] }
openssl = "0.10"

# Async runtime
tokio = { version = "1.42", features = ["full"] }

# Web
axum = "0.8"
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }

# gRPC
tonic = "0.13"
prost = "0.13"

# Observability
tracing = "0.1"
metrics = "0.22"
opentelemetry = "0.24"
```

───────────────────────────────────────

## 2026-03-29 - Modern Tooling Gaps

**Project:** [cross-repo]
**Category:** dependencies
**Status:** completed
**Priority:** P2

### Tool | Status | Action

| Tool | Status | Action |
|------|--------|--------|
| `uv` | Not used | Consider for Python scripts |
| `ruff` | Not used | Add for Python linting |
| `indicatif` | Not used | Add progress bars |
| `dialoguer` | Not used | Add interactive prompts |

───────────────────────────────────────

## Research Commands

```bash
# Find all Cargo.toml files
find . -name "Cargo.toml" -not -path "./target/*" | wc -l

# Check dependency tree
cargo tree --workspace -e normal | head -100

# Find duplicate dependencies
cargo tree --workspace -e normal --duplicates | head -50

# Check for outdated dependencies
cargo outdated --workspace 2>/dev/null || cargo outdated --help

# Audit security
cargo audit 2>/dev/null || echo "Install: cargo install cargo-audit"

# Find unused dependencies
cargo +nightly udeps --workspace 2>/dev/null || echo "Install: cargo +nightly install cargo-udeps"
```

───────────────────────────────────────

## 2026-03-31 - Wave 108: phenotype-infrakit PR batch outcome

**Project:** [phenotype-infrakit]
**Category:** maintenance
**Status:** verified via `gh pr view` (2026-03-31)
**Priority:** P1

PRs [#249](https://github.com/KooshaPari/phenotype-infrakit/pull/249)–[#252](https://github.com/KooshaPari/phenotype-infrakit/pull/252) were opened as drafts from stacked branches and **closed the same day without merge** (`mergedAt` null). Batch notes: [`.archive/PR_CREATION_BATCH_2026-03-30.md`](./.archive/PR_CREATION_BATCH_2026-03-30.md).

**Next:** Decide whether to re-cherry-pick or abandon that series; for **repos** nested-crate cleanup status, see [`WORK_LOG.md`](./WORK_LOG.md) (“2026-03-31 — Resume / reconciliation”).

───────────────────────────────────────

_Last updated: 2026-03-31_
