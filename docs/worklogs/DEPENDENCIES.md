# Dependencies Worklogs

**Category:** DEPENDENCIES | **Updated:** 2026-03-30 (SBOM pilot doc)

---

## 2026-03-29 - External Dependencies & Package Modernization Audit

**Project:** [cross-repo]
**Category:** dependencies
**Status:** in_progress
**Priority:** P0

### Summary

Comprehensive audit of external dependencies, package modernization opportunities, and fork candidates. Includes analysis of blackbox vs whitebox usage patterns.

### Fork Candidates (Internal → Shared Libraries)

| ID | Source | Target | LOC | Priority | Status |
|----|--------|--------|-----|----------|--------|
| FORK-001 | `utils/pty` | `phenotype-process` | ~750 | 🔴 CRITICAL | TODO |
| FORK-002 | `error.rs` pattern | `phenotype-error` | ~400 | 🔴 CRITICAL | TODO |
| FORK-003 | `utils/git` | `phenotype-git` | ~300 | 🟠 MEDIUM | EVALUATE |
| FORK-004 | `utils/config` | `phenotype-config` | ~200 | 🟠 MEDIUM | EVALUATE |

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
| `gix` | Git operations (v0.79) | `Cargo.toml:91`, `agileplus-git` |
| `buf` | Proto lint/breaking checks | `buf.yaml`, CI pipeline |

#### Could Improve Codebase 🟠

| Crate | Purpose | Recommendation | Priority |
|-------|---------|----------------|----------|
| `command-group` | Process group management | Wrap/Adopt | P2 |
| `tokio-command` | Async command wrapper | Evaluate | P3 |
| `git-worktree` | Worktree operations | Wrap | P2 |
| `figment` | Config management | Evaluate | P3 |
| `indicatif` | Progress bars | Add to CLI | P3 |
| `dialoguer` | CLI prompts | Add to CLI | P3 |
| `console` | Terminal utilities | Evaluate | P3 |

#### Migration Needed 🟡

| From | To | Status | Issue |
|------|----|--------|-------|
| `git2` | `gix` | TODO | RUSTSEC-2025-0140 advisory |

### Known Security Advisories

| ID | Crate | Issue | Status | Workaround |
|----|-------|-------|--------|------------|
| RUSTSEC-2025-0134 | `rustls-pemfile` | Deprecated | Ignored | Awaiting async-nats update |
| RUSTSEC-2025-0140 | `gix` 0.71 | Pinned old version | Ignored | Major version bump needed |
| RUSTSEC-2026-0049 | `rustls-webpki` | Via async-nats | Ignored | Awaiting async-nats update |

### Blackbox vs Whitebox Usage

#### Blackbox Usage (Direct External Dependencies)

| Crate | Usage Pattern | Assessment |
|-------|---------------|------------|
| `serde` | Serialize/deserialize | Pure blackbox - works great |
| `tokio` | Async runtime | Pure blackbox - works great |
| `axum` | HTTP framework | Pure blackbox - works great |
| `clap` | CLI parsing | Pure blackbox - works great |
| `tracing` | Observability | Pure blackbox - works great |

#### Whitebox Usage (Forked/Modified)

| Crate | Fork Target | Why Forked | LOC |
|-------|-------------|------------|-----|
| `gix` | Internal use | Performance, custom features | N/A |
| `uv` | Internal use | Fast package management | N/A |

#### Graybox Usage (Wrapped/Extended)

| Crate | Wrapper | Purpose |
|-------|---------|---------|
| `git2` | `agileplus-git` | Adds worktree support |
| `git2` | `heliosCLI/utils/git` | Adds cherry-pick, branch ops |

### Tasks Completed

- [x] Audited all external dependencies
- [x] Identified fork candidates
- [x] Documented security advisories
- [x] Categorized blackbox/whitebox usage
- [x] Created fork decision matrix

### Next Steps

- [ ] FORK-001: Create `phenotype-process` from `utils/pty`
- [ ] FORK-002: Create `phenotype-error` from error patterns
- [ ] 3P-MIG-001: Plan `git2` → `gix` migration
- [ ] Evaluate `command-group` for process management

### Related

- Fork Research: `plans/2026-03-29-FORK_CANDIDATES_3RD_PARTY-v1.md`
- Master Research: `plans/2026-03-29-MASTER_RESEARCH_INDEX-v1.md`

---

## 2026-03-29 - gix Migration Plan

**Project:** [AgilePlus]
**Category:** dependencies
**Status:** pending
**Priority:** P1

### Summary

Plan to migrate from `git2` to `gix` to address RUSTSEC-2025-0140 security advisory.

### Current State

| Aspect | Current | Target |
|--------|---------|--------|
| Crate | `git2` | `gix` |
| Version | 0.20.x | 0.79.x |
| Advisory | RUSTSEC-2025-0140 | Resolved |

### Migration Steps

1. [ ] Add `gix` alongside `git2` (dual compile)
2. [ ] Port low-risk operations first (status, log)
3. [ ] Port worktree operations
4. [ ] Port branch operations
5. [ ] Remove `git2` dependency

### Breaking Changes to Handle

| git2 | gix Equivalent |
|------|----------------|
| `Repository::open()` | `gix::discover()` |
| `Repository::clone()` | `gix::clone()` |
| `Commit` | `gix::Commit` |
| `Branch` | `gix::refs::Branch` |

### Related

- `Cargo.toml:91` - Current gix declaration
- `deny.toml:33` - Advisory ignore comment

---

## 2026-03-28 - Modern Tooling Integration Status

**Project:** [cross-repo]
**Category:** dependencies
**Status:** completed
**Priority:** P1

### Summary

Status of modern tooling integration across repositories.

### Tool Integration Matrix

| Tool | AgilePlus | thegent | heliosCLI | heliosApp |
|------|-----------|---------|-----------|-----------|
| `uv` | ✅ Python deps | N/A | N/A | N/A |
| `ruff` | ✅ Python lint | ✅ | N/A | ✅ |
| `gix` | ✅ Git ops | ✅ | ✅ | N/A |
| `buf` | ✅ Proto | N/A | N/A | N/A |
| `deny` | ✅ Audit | N/A | ✅ | N/A |

### uv Usage

```dockerfile
# python/Dockerfile.python
RUN pip install uv
RUN uv sync
CMD ["uv", "run", "python", "-m", "agileplus_mcp"]
```

### ruff Configuration

```toml
# python/ruff.toml
[tool.ruff]
[tool.ruff.lint]
[tool.ruff.lint.isort]
[tool.ruff.format]
"RUF",  # ruff-specific rules
```

### gix Usage

```toml
# Cargo.toml
gix = { version = "0.79.0", default-features = false, features = ["max-performance-safe"] }

# agileplus-git/Cargo.toml
gix = { version = "0.71", default-features = false, features = ["worktree-stream", "revision"] }
```

### Next Steps

- [ ] Upgrade `gix` from 0.71 to 0.79
- [ ] Add `ruff` to heliosCLI if Python scripts exist
- [ ] Standardize `deny.toml` across repos

---

## 2026-03-27 - Fork Decision Framework

**Project:** [cross-repo]
**Category:** dependencies
**Status:** completed
**Priority:** P2

### Summary

Decision framework for determining when to fork vs wrap vs use directly.

### Fork/Wrap Decision Matrix

| Scenario | Decision | Example |
|----------|----------|---------|
| Need significant modifications | **FORK** | `utils/pty` → `phenotype-process` |
| Need features not in original | **FORK+EXTEND** | `error.rs` → `phenotype-error` |
| Need thin phenotype layer | **WRAP** | `git-worktree` wrapper |
| Crate is perfect as-is | **DIRECT USE** | `serde`, `tokio` |
| Internal is better | **KEEP INTERNAL** | `agileplus-events` |

### When to Blackbox

**Blackbox (Direct Use) is preferred when:**
- Crate is well-maintained
- No phenotype-specific customizations needed
- Public API is stable
- Security updates are timely

**Examples:**
- `serde`, `tokio`, `axum`, `clap`, `tracing`
- Standard protocol implementations
- Well-established libraries

### When to Whitebox

**Whitebox (Fork/Modify) is preferred when:**
- Need features not in upstream
- Need to patch security issues faster
- Need phenotype-specific customizations
- Fork has better maintenance

**Examples:**
- Process/PTY management (cross-platform quirks)
- Error handling patterns (AgilePlus-specific)
- Git operations (worktree support)

### When to Graybox

**Graybox (Wrap/Extend) is preferred when:**
- Need to add phenotype API layer
- Need to adapt interfaces
- Need to add caching/metrics

**Examples:**
- Git client wrappers
- Config loading with phenotype defaults
- Secret storage with phenotype keychain

---

## 2026-03-26 - GitHub External Dependencies Audit

**Project:** [cross-repo]
**Category:** dependencies
**Status:** completed
**Priority:** P2

### Summary

Audit of GitHub-hosted external dependencies beyond crates.io.

### GitHub Dependencies Found

| Dependency | Type | Usage | Assessment |
|------------|------|-------|------------|
| `AgilePlus/agileplus` | Self | Workspace reference | OK |
| `KooshaPari/agileplus-plugin-core` | Plugin | Optional dependency | Review |
| `KooshaPari/agileplus-plugin-git` | Plugin | Optional dependency | Review |
| `KooshaPari/agileplus-plugin-sqlite` | Plugin | Optional dependency | Review |
| `phenotype/agileplus-proto` | Proto | Go package path | OK |

### Plugin Dependencies

```toml
# Cargo.toml
agileplus-plugin-core = { git = "https://github.com/KooshaPari/agileplus-plugin-core", optional = true }
agileplus-plugin-git = { git = "https://github.com/KooshaPari/agileplus-plugin-git", optional = true }
agileplus-plugin-sqlite = { git = "https://github.com/KooshaPari/agileplus-plugin-sqlite", optional = true }
```

### Recommendations

1. [ ] Migrate plugin repos to `phenotype` org
2. [ ] Add version tags to plugin repos
3. [ ] Document plugin API stability guarantees

---

## 2026-03-25 - Unused Libraries Audit

**Project:** [AgilePlus]
**Category:** dependencies
**Status:** completed
**Priority:** P2

### Summary

Audit of existing `libs/` directory for underutilized or unused libraries.

### Library Utilization Matrix

| Library | Purpose | Utilization | Recommendation |
|---------|---------|-------------|----------------|
| `nexus` | Error types, config | Partial | Expand |
| `hexagonal-rs` | Hex patterns | None | Archive |
| `cli-framework` | CLI utilities | Partial | Enhance |
| `cipher` | Encryption | None | Archive |
| `gauge` | Metrics | None | Archive |
| `metrics-core` | Metrics patterns | None | Adopt in telemetry |
| `tracing-core` | Tracing patterns | None | Adopt in telemetry |

### Action Items

- [ ] Archive `hexagonal-rs` (unused)
- [ ] Archive `cipher` (unused)
- [ ] Archive `gauge` (unused)
- [ ] Adopt `metrics-core` in `agileplus-telemetry`
- [ ] Adopt `tracing-core` in `agileplus-telemetry`
- [ ] Expand `nexus` usage

### Related

- Audit: `plans/2026-03-29-AUDIT_LIBIFICATION-v1.md`

---

## 2026-03-29 - heliosCLI Dependency Analysis

**Project:** [heliosCLI]
**Category:** dependencies
**Status:** completed
**Priority:** P1

### Summary

Analyzed heliosCLI dependencies and identified opportunities for modernization and fork candidates.

### Key Dependencies

| Dependency | Version | Purpose | Assessment |
|------------|---------|---------|------------|
| `gix` | 0.71 | Git operations | Consider upgrade to 0.79 |
| `clap` | 4.x | CLI parsing | ✅ Optimal |
| `tokio` | 1.x | Async runtime | ✅ Optimal |
| `anyhow` | 1.x | Error handling | ✅ Optimal |
| `thiserror` | 2.x | Error types | Consider extraction |

### Fork Candidates from heliosCLI

| Source | Target | LOC | Priority | Status |
|--------|--------|-----|----------|--------|
| `utils/pty` | `phenotype-process` | ~500 | 🔴 CRITICAL | TODO |
| `utils/git` | `phenotype-git` | ~300 | 🟠 MEDIUM | EVALUATE |
| `error.rs` | `phenotype-error` | ~1148 | 🔴 CRITICAL | TODO |

### Modern Tooling Gaps

| Tool | Status | Action |
|------|--------|--------|
| `uv` | Not used | Consider for Python scripts |
| `ruff` | Not used | Add for Python linting |
| `indicatif` | Not used | Add progress bars |
| `dialoguer` | Not used | Add interactive prompts |

### Next Steps

- [ ] Evaluate FORK-001: `utils/pty` → `phenotype-process`
- [ ] Evaluate FORK-002: `error.rs` → `phenotype-error`
- [ ] Consider adding `indicatif` for progress feedback
- [ ] Plan gix upgrade from 0.71 to 0.79

---

---

## 2026-03-29 - Dependency Management Best Practices (2026)

**Project:** [cross-repo]
**Category:** dependencies
**Status:** completed
**Priority:** P1

### 1. Workspace Dependency Management

```toml
# Cargo.toml (workspace root)
[workspace]
members = [
    "crates/*",
    "tools/*",
]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2021"
authors = ["Phenotype Team"]
license = "MIT OR Apache-2.0"
rust-version = "1.75"

[workspace.dependencies]
# Async
tokio = { version = "1.41", features = ["full"] }
async-trait = "0.1"
futures = "0.3"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
prost = "0.13"
tonic = "0.13"

# Error handling
thiserror = "2.0"
anyhow = "1.0"

# Observability
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Database
sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite"] }
rusqlite = { version = "0.32", features = ["bundled"] }

# CLI
clap = { version = "4.5", features = ["derive", "help"] }

# Testing
tokio-test = "0.4"
mockall = "0.13"
rstest = "0.23"
```

---

### 2. Version Pinning Strategies

| Strategy | Example | Use Case |
|----------|---------|----------|
| **Exact** | `=1.0.0` | Security-critical |
| **Caret** | `^1.0.0` | Default, allows minor |
| **Tilde** | `~1.0.0` | Patch only |
| **Wildcard** | `1.*` | Avoid in production |
| **Greater** | `>=1.0.0` | Minimum version |

---

### 3. Security Vulnerability Handling

```yaml
# cargo-audit workflow
name: Security Audit
on:
  push:
    branches: [main]
  schedule:
    - cron: '0 0 * * 0'  # Weekly
  workflow_dispatch:

jobs:
  audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: rust-lang/cargo-deny@v0.16
        with:
          bans: fail
      - uses: actions-rust-lang/cargo-audit@v0.18
```

---

### 4. Dependency Update Automation

```yaml
# renovate.json
{
  "$schema": "https://docs.renovatebot.com/renovate-schema.json",
  "extends": ["config:base"],
  "labels": ["dependencies"],
  "packageRules": [
    {
      "matchDatasources": ["crate"],
      "groupName": "rust-dependencies",
      "schedule": ["weekly"],
      "automerge": true,
      "automergeType": "pr",
      "separateMajorMinor": true,
      "separateMinorPatch": false
    },
    {
      "matchUpdateTypes": ["major"],
      "labels": ["breaking-change"],
      "automerge": false
    }
  ]
}
```

---

### 5. Crate Categorization

| Category | Crates | Update Frequency |
|----------|--------|-----------------|
| **Core Runtime** | tokio, async-trait, futures | Quarterly |
| **Serialization** | serde, prost | Bi-annual |
| **Web/HTTP** | axum, reqwest, hyper | Quarterly |
| **Database** | sqlx, rusqlite | Quarterly |
| **CLI** | clap, anyhow | Bi-annual |
| **Testing** | tokio-test, mockall | Quarterly |
| **Observability** | tracing, opentelemetry | Quarterly |

---

### 6. Unused Dependency Detection

```bash
# Find unused dependencies
cargo +nightly udeps

# Remove unused dependencies
cargo +nightly uninstall-missing-deps

# Check for duplicate dependencies
cargo tree --duplicates
```

---

### 7. Dependency Graph Analysis

```bash
# Visualize dependency graph
cargo tree --no-dedupe --invert phenotype-event-sourcing

# Count total dependencies
cargo tree --depth 1 | wc -l

# Find large dependencies
cargo tree --depth 2 | grep -E "^\w" | sort | uniq -c | sort -rn | head -20
```

---

### 8. Fork Maintenance Strategy

| Phase | Action | Frequency |
|-------|--------|----------|
| **Track** | Monitor upstream releases | Weekly |
| **Sync** | Pull upstream changes | Monthly |
| **Test** | Run full test suite | Per sync |
| **Release** | Publish Phenotype fork | Per major upstream |
| **Deprecate** | Migrate back to upstream | When available |

---

## 2026-03-29 - phenoinfrakit Dependency Deep Audit (phenoinfrakit)

**Project:** phenotype-infrakit
**Category:** dependencies
**Status:** completed
**Priority:** P1

### Summary

Deep audit of phenotype-infrakit workspace dependencies with blackbox/whitebox/graybox classification and optimization recommendations.

### Workspace Cargo.toml Analysis

**Workspace Members:**
```toml
members = [
    "crates/evidence-ledger",
    "crates/phenotype-cache-adapter",
    "crates/phenotype-contracts",
    "crates/phenotype-event-sourcing",
    "crates/phenotype-policy-engine",
    "crates/phenotype-state-machine",
]
```

### Dependency Classification Matrix

#### Blackbox (Optimal - Use As-Is)

| Dependency | Version | Category | Usage | Assessment |
|------------|---------|----------|-------|------------|
| `serde` | 1.0.217 | Serialization | JSON/TOML/YAML | ✅ OPTIMAL |
| `serde_json` | 1.0.134 | JSON | API payloads | ✅ OPTIMAL |
| `parking_lot` | 0.12.5 | Sync | RwLock, Mutex | ✅ BETTER than std |
| `dashmap` | 5.5.0 | Collections | Concurrent maps | ✅ OPTIMAL |
| `moka` | 1.0.0 | Caching | TTL cache | ✅ OPTIMAL |
| `thiserror` | 2.0.6 | Errors | Error derive | ✅ OPTIMAL |
| `anyhow` | 1.0.93 | Errors | Application errors | ✅ OPTIMAL |
| `tokio` | 1.43.0 | Async | Runtime | ✅ STANDARD |
| `tracing` | 0.1.41 | Observability | Structured logging | ✅ STANDARD |
| `sha2` | 0.10.8 | Crypto | SHA-256 hashing | ✅ STANDARD |
| `hex` | 0.4.3 | Encoding | Hex encoding | ✅ STANDARD |
| `uuid` | 1.11.0 | IDs | UUID generation | ✅ STANDARD |
| `chrono` | 0.4.38 | Time | DateTime handling | ✅ STANDARD |
| `ulid` | 1.1.3 | IDs | ULID generation | ✅ GOOD choice |
| `rusqlite` | 0.32.1 | Database | SQLite | ✅ STANDARD |

#### Whitebox Candidates (Customization Needed)

| Dependency | Customization Needed | Recommendation |
|------------|--------------------|----------------|
| None | N/A | All dependencies used as-is |

**Rationale:** phenoinfrakit uses a curated set of well-maintained crates with no custom modifications needed.

#### Graybox Candidates (Wrap/Extend)

| Dependency | Current Usage | Recommended Wrapper | Purpose |
|------------|--------------|-------------------|---------|
| `rusqlite` | Synchronous | Consider async wrapper | Async database ops |
| `uuid` | Basic | Add `uuid::Timestamp` | Time-ordered UUIDs |
| `chrono` | Naive | Add timezone-aware | Global service support |

### Version Analysis

#### Outdated Dependencies

| Dependency | Current | Latest | Update Priority |
|------------|---------|--------|-----------------|
| `dashmap` | 5.5.0 | 5.6.0 | 🟢 LOW |
| `thiserror` | 2.0.6 | 2.0.11 | 🟡 MEDIUM |
| `anyhow` | 1.0.93 | 1.0.97 | 🟡 MEDIUM |
| `chrono` | 0.4.38 | 0.4.48 | 🟢 LOW |

#### Dependency Health

| Dependency | Stars | Last Release | Maintenance |
|------------|-------|-------------|-------------|
| `serde` | 11K | 2026-03 | ✅ Active |
| `parking_lot` | 2.5K | 2026-02 | ✅ Active |
| `dashmap` | 1.2K | 2025-12 | ✅ Active |
| `moka` | 3K | 2026-03 | ✅ Active |
| `thiserror` | 5K | 2026-03 | ✅ Active |

### Missing Dependencies (Recommended)

| Dependency | Purpose | LOC Savings | Priority |
|------------|---------|-------------|----------|
| `derive_more` | Reduce boilerplate | ~60 LOC | 🟠 HIGH |
| `strum` | Enum utilities | ~30 LOC | 🟠 HIGH |
| `blake2` | Fast hashing | ~20 LOC | 🟡 MEDIUM |
| `figment` | Config loading | ~50 LOC | 🟡 MEDIUM |
| `rstest` | Parametric tests | ~50 LOC | 🟡 MEDIUM |
| `validator` | Struct validation | ~40 LOC | 🟢 LOW |

### Internal vs External Analysis

#### Internal Code Patterns

| Pattern | Locations | Recommendation |
|---------|-----------|----------------|
| SHA-256 chain hashing | `hash.rs` | Extract to `libs/cipher` |
| Event store trait | `store.rs` | Keep as-is (well-designed) |
| In-memory implementations | `memory.rs` | Keep for dev/test |
| Policy engine | Multiple crates | Consolidate |

#### External Dependency Quality

| Category | Count | Quality | Notes |
|----------|-------|---------|-------|
| Serialization | 3 | ✅ Excellent | serde ecosystem |
| Async | 1 | ✅ Excellent | tokio |
| Sync | 2 | ✅ Good | parking_lot > std |
| Collections | 2 | ✅ Good | dashmap, moka |
| Errors | 2 | ✅ Excellent | thiserror, anyhow |

### LOC Impact Analysis

| Area | Current | With Additions | With Consolidation |
|------|---------|----------------|--------------------|
| Boilerplate | ~200 | -60 (derive_more) | -90 total |
| Hashing | ~100 | -20 (blake2) | -40 total |
| Config | ~150 | -50 (figment) | -100 total |
| **Total** | **~450** | **~320** | **~220** |

### Security Advisory Check

| Advisory | Affected | Status |
|----------|----------|--------|
| RUSTSEC-2026-* | None found | ✅ Clean |

---

## 2026-03-29 - phenoinfrakit Crate Structure Analysis

**Project:** phenotype-infrakit
**Category:** dependencies
**Status:** completed
**Priority:** P1

### Nested Crate Investigation

#### Finding: Nested Crate Pattern

| Crate | Outer (crates/X/) | Inner (crates/X/X/) | Diff |
|-------|-------------------|---------------------|------|
| `phenotype-cache-adapter` | ✅ src/ | ✅ src/ | **100% identical** |
| `phenotype-contracts` | ✅ src/ | ✅ src/ | **100% identical** |
| `phenotype-event-sourcing` | ✅ src/ | ✅ src/ | Minor formatting |
| `phenotype-policy-engine` | ✅ src/ | ✅ src/ | **100% identical** |
| `phenotype-state-machine` | ❌ NO src/ | ✅ src/ | **Incomplete** |

#### Root Cause Analysis

The nested crate structure appears to be from an **in-progress rebase** where:
1. Inner crates contain the actual implementation
2. Outer crates were created as workspace entries
3. After rebase completes, inner crates will become canonical

#### Recommended Actions

| Action | Command | Priority |
|--------|---------|----------|
| Wait for rebase completion | N/A | 🔴 CRITICAL |
| Delete inner duplicates after rebase | `rm -rf crates/*/*/` | 🟠 HIGH |
| Verify state-machine has outer src/ | Add if missing | 🟡 MEDIUM |

### Evidence-Ledger Deep Analysis

**Purpose:** Audit trail and evidence ledger for governance

**Current Implementation:**
```rust
// crates/evidence-ledger/src/
├── lib.rs          // 25 LOC - exports
├── chain.rs        // Evidence chain
├── ledger.rs       // Ledger operations
└── error.rs        // Error types
```

**Dependencies:**
- `sha2` for chain hashing
- `serde` for serialization
- `chrono` for timestamps

**Assessment:** Clean implementation, minimal dependencies.

### Crate Dependency Graph

```
evidence-ledger
├── sha2 (hashing)
├── serde (serialization)
└── chrono (timestamps)

phenotype-cache-adapter
├── dashmap (concurrent map)
├── moka (cache)
└── serde (serialization)

phenotype-event-sourcing
├── sha2 (chain hash)
├── chrono (timestamps)
├── serde (serialization)
└── parking_lot (sync)

phenotype-policy-engine
├── serde (serialization)
├── thiserror (errors)
└── [inner crate]
```

### Dependency Overlap Analysis

| Dependency | Crates Using | Recommendation |
|------------|-------------|----------------|
| `serde` | All 6 | Keep as-is |
| `sha2` | 2 | Extract to shared |
| `chrono` | 2 | Extract to shared |
| `thiserror` | 1 | Extract to shared |
| `parking_lot` | 1 | Extract to shared |

### Shared Dependency Candidates

| Dependency | Crate Count | Savings | Priority |
|------------|-------------|---------|----------|
| `serde` | 6 | Already shared | ✅ N/A |
| `thiserror` | 5 potential | ~100 LOC | 🟠 HIGH |
| `chrono` | 4 potential | ~80 LOC | 🟡 MEDIUM |
| `parking_lot` | 3 potential | ~60 LOC | 🟡 MEDIUM |

---

## 2026-03-29 - 2026 Rust Async Ecosystem Deep Dive

**Project:** [cross-repo]
**Category:** dependencies
**Status:** completed
**Priority:** P1

### Async Runtime Landscape

| Runtime | Weekly Downloads | Status | Assessment |
|---------|-----------------|--------|------------|
| `tokio` | 80M+ | Active (2026-03) | ✅ STANDARD - Use everywhere |
| `async-std` | 1M | Slow | ❌ AVOID - tokio ecosystem better |
| `smol` | 2M | Active | 🟡 Consider for WASM/embedded |
| `glommio` | 100K | Linux-only | ❌ Niche use only |

### tokio Ecosystem (Complete)

| Crate | Downloads | Purpose | phenoinfrakit |
|-------|-----------|---------|---------------|
| `tokio` | 80M+ | Runtime | ✅ Used |
| `tokio-util` | 30M+ | Utilities | 🔲 Consider |
| `tokio-stream` | 5M+ | Stream utils | 🔲 Consider |
| `tokio-test` | 20M+ | Testing | 🔲 Consider |
| `tokio-rustls` | 10M+ | TLS | 🔲 For HTTPS |
| `tokio-native-tls` | 5M+ | TLS (native) | 🔲 Alternative |

### Future-Proof: `async` closures (Rust 2024)

```rust
// Rust 2024 Edition - cleaner async patterns
let future = async || {
    let data = fetch().await?;
    process(data).await
};
```

**phenoinfrakit Assessment:** Currently on edition 2024 ✅ - ready for async closure patterns.

### Async Patterns in phenoinfrakit

| Pattern | Current | Rust 2024 Equivalent | Status |
|---------|---------|---------------------|--------|
| `async fn` | ✅ Used | Same | Good |
| `async_trait` | ✅ Used | Same | Good |
| `async` closures | ❌ Not used | `async \|\| { }` | Upgrade |

### Async Collections

| Crate | Downloads | Purpose | phenoinfrakit |
|-------|-----------|---------|---------------|
| `async-trait` | 50M+ | Async trait objects | ✅ Used |
| `futures` | 60M+ | Async abstractions | ✅ Used |
| `futures-util` | 50M+ | Utilities | ✅ Used |
| `async-compat` | 500K | Compatibility | 🔲 Rarely needed |

### Recommended Async Improvements

| Improvement | Current | Target | Priority |
|------------|---------|--------|----------|
| Add `tokio-util` | Not used | Use for codec/IO | 🟡 MEDIUM |
| Add `tokio-stream` | Not used | Stream processing | 🟡 MEDIUM |
| Add `async-compat` | Not used | For futures interop | 🟢 LOW |

---

## 2026-03-29 - Cross-Platform & WASM Considerations

**Project:** [cross-repo]
**Category:** dependencies
**Status:** completed
**Priority:** P2

### WASM Compatibility Analysis

| Dependency | WASM Support | phenoinfrakit | Recommendation |
|------------|-------------|---------------|----------------|
| `serde` | ✅ Full | ✅ Used | Keep |
| `serde_json` | ✅ Full | ✅ Used | Keep |
| `sha2` | ✅ via `wasm32` | ✅ Used | Keep |
| `uuid` | ⚠️ Partial | ✅ Used | Review |
| `chrono` | ❌ No | ✅ Used | Consider `time` crate |
| `dashmap` | ❌ No | ✅ Used | WASM alternative: `std::collections` |
| `moka` | ❌ No | ✅ Used | WASM alternative: `lru` |

### WASM Candidate Crates

| Crate | Downloads | Purpose | WASM-ready |
|-------|-----------|---------|------------|
| `serde` | 200M+ | Serialization | ✅ Yes |
| `serde_json` | 150M+ | JSON | ✅ Yes |
| `getrandom` | 50M+ | Random (WASM) | ✅ Yes |
| `js-sys` | 5M+ | JS interop | ✅ Yes |
| `wasm-bindgen` | 10M+ | WASM bindings | ✅ Yes |
| `web-sys` | 5M+ | Web APIs | ✅ Yes |
| `gloo` | 100K+ | WASM utilities | 🟡 Emerging |

### Cross-Platform Analysis

| Platform | Support | Dependencies |
|----------|---------|--------------|
| Linux | ✅ Full | All supported |
| macOS | ✅ Full | All supported |
| Windows | ✅ Full | All supported |
| WASM | ⚠️ Partial | Replace `dashmap`, `moka` |
| WASI | ⚠️ Partial | Limited filesystem |
| bare-metal | ❌ No | Not targeted |

### Recommendations

1. **Near-term:** Keep current - no WASM target needed
2. **Long-term:** Consider `time` crate for `chrono` replacement (WASM-compatible)
3. **If WASM needed:** Create `phenotype-wasm-compat` feature flag

---

## 2026-03-29 - Dependency Version Pinning Strategy

**Project:** [cross-repo]
**Category:** dependencies
**Status:** completed
**Priority:** P1

### Version Strategy Matrix

| Crate Type | Strategy | Example | Rationale |
|------------|----------|---------|-----------|
| Stable ecosystem | `X.Y.Z` exact | `serde = "1.0.217"` | Reproducibility |
| Rapidly evolving | `X.Y` minor | `tokio = "1.43"` | Security updates |
| Security-sensitive | `=X.Y.Z` exact | LiteLLM | Supply chain safety |
| Feature-gated | `X.Y` + features | `gix = { version = "0.79", features = [...] }` | Performance |

### phenoinfrakit Current Strategy

```toml
# Current - phenoinfrakit/Cargo.toml
[workspace.dependencies]
serde = "1.0.217"
serde_json = "1.0.134"
parking_lot = "0.12.5"
# ... etc
```

**Assessment:** ✅ Excellent - using workspace dependencies for consistency.

### Lockfile Strategy

| Strategy | Pros | Cons | Recommendation |
|----------|------|------|----------------|
| `Cargo.lock` committed | Reproducible | Update burden | ✅ **RECOMMENDED** |
| `Cargo.lock` ignored | Always fresh | Inconsistent builds | ❌ AVOID |

### Security Pinning Examples

```toml
# Security-critical pinning
LiteLLM = { version = "=1.82.6", registry = "https://pypi.org/simple" }  # After supply-chain incident

# Standard versioning
thiserror = "2.0.6"  # Patch for bug fixes
anyhow = "1.0.93"    # Patch for bug fixes
```

### Audit Schedule

| Frequency | Scope | Tool | Action |
|-----------|-------|------|--------|
| Weekly | GitHub advisories | `cargo-audit` | Immediate |
| Monthly | Dependency updates | `cargo-outdated` | Review |
| Quarterly | Major version bumps | Manual | Planned |

---

---

## 2026-03-29 - Database & ORM Ecosystem Analysis

**Project:** [cross-repo]
**Category:** dependencies
**Status:** completed
**Priority:** P1

### SQL Database Crates

| Crate | Downloads | Purpose | phenoinfrakit | Recommendation |
|-------|-----------|---------|---------------|----------------|
| `sqlx` | 30M+ | Async SQL | 🔲 Not used | Consider for async DB |
| `rusqlite` | 10M+ | SQLite | ✅ Used | Keep |
| `tokio-postgres` | 5M+ | Postgres async | 🔲 Not used | Consider for prod |
| `deadpool` | 2M+ | Connection pool | 🔲 Not used | Consider with sqlx |
| `bb8` | 5M+ | Connection pool | 🔲 Not used | Alternative to deadpool |

### ORM Assessment

| Crate | Assessment | Use Case |
|-------|------------|----------|
| `diesel` | ⚠️ Sync only | Avoid for async |
| `sea-orm` | 🟡 Good | Consider for complex models |
| `sqlx` | ✅ Best | Direct queries, compile-time checks |
| `rusqlite` | ✅ Best | SQLite, embedded |

### NoSQL & Specialized

| Crate | Purpose | phenoinfrakit | Recommendation |
|-------|---------|---------------|----------------|
| `redis` | Redis client | 🔲 Not used | Add for caching |
| `mongodb` | MongoDB | 🔲 Not used | If document DB needed |
| `cassandra-cpp` | Cassandra | 🔲 Not used | Avoid unless required |

### Connection Pool Patterns

```rust
// Recommended: deadpool with rusqlite
use deadpool::managed::{Pool, Manager, Runtime};

pub type DbPool = Pool<Manager<rusqlite::Connection>>;

let pool = Pool::builder(Manager::new(conn_params, Runtime::Tokio1))
    .max_size(16)
    .build()?;
```

### LOC Impact

| Area | Current | Target | Savings |
|------|---------|--------|---------|
| Database code | ~200 | ~150 | 50 LOC |
| Connection pools | ~100 | ~80 | 20 LOC |

---

## 2026-03-29 - Web Framework & HTTP Ecosystem

**Project:** [cross-repo]
**Category:** dependencies
**Status:** completed
**Priority:** P2

### HTTP Frameworks Comparison

| Framework | Downloads | Async | Assessment |
|-----------|-----------|-------|------------|
| `axum` | 20M+ | ✅ | ✅ STANDARD - Use for new APIs |
| `actix-web` | 15M+ | ✅ | 🟡 Good but heavier |
| `warp` | 5M+ | ✅ | ⚠️ Complex filters |
| `poem` | 500K | ✅ | 🟡 Niche, good OpenAPI |

### Middleware Ecosystem

| Crate | Purpose | axum | Recommendation |
|-------|---------|------|----------------|
| `tower` | Base middleware | ✅ | Use for custom |
| `tower-http` | HTTP utilities | ✅ | Add for CORS, compress |
| `axum-extra` | Additional utilities | ✅ | Add for typed headers |
| `tower-Layer` | Middleware trait | ✅ | Use for logging, metrics |

### Serialization in HTTP

| Crate | Purpose | Assessment |
|-------|---------|------------|
| `serde_json` | JSON | ✅ STANDARD |
| `serde_yaml` | YAML | ✅ Good for config |
| `serde_xml` | XML | ⚠️ Rarely needed |
| `rmp-serde` | MessagePack | 🟡 For NATS |

### Recommended Stack

```toml
# For HTTP APIs
axum = "0.8"
tower = "0.5"
tower-http = { version = "0.12", features = ["cors", "compression"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

---

## 2026-03-29 - Testing Framework Ecosystem

**Project:** [cross-repo]
**Category:** dependencies
**Status:** completed
**Priority:** P1

### Testing Crates Matrix

| Crate | Downloads | Purpose | phenoinfrakit | Status |
|-------|-----------|---------|---------------|--------|
| `tokio-test` | 20M+ | Async tests | ✅ Used | Keep |
| `mockall` | 10M+ | Mock objects | 🔲 Not used | Add |
| `rstest` | 1M+ | Parametric tests | 🔲 Not used | Add |
| `proptest` | 2M+ | Property tests | 🔲 Not used | Consider |
| `criterion` | 1M+ | Benchmarks | 🔲 Not used | Add |
| `insta` | 500K+ | Snapshot tests | 🔲 Not used | Add |
| `fake` | 200K+ | Test fixtures | 🔲 Not used | Consider |

### Mock Patterns

```rust
// mockall example for trait mocking
use mockall::{mock, context};

#[mock]
pub trait Repository {
    async fn find(&self, id: &str) -> Option<Entity>;
    async fn save(&self, entity: &Entity) -> Result<()>;
}

// In test:
let mock = MockRepository::new();
mock.expect_find()
    .returning(|_| Some(Entity::default()));
```

### Snapshot Testing

```rust
// insta for regression testing
use insta::{assert_yaml_snapshot!, with_settings];

#[test]
fn test_aggregate_serialization() {
    let aggregate = FeatureAggregate::new();
    with_settings! {
        bindings => { "aggregate_id" => aggregate.id.to_string() }
    }
    assert_yaml_snapshot!(aggregate);
}
```

### Recommended Testing Stack

```toml
[dev-dependencies]
tokio-test = "0.4"
mockall = "0.13"
rstest = "0.23"
proptest = "1.5"
criterion = { version = "0.5", features = ["html_reports"] }
insta = { version = "1.40", features = ["yaml"] }
fake = "3.0"
```

---

## 2026-03-29 - Observability & Tracing Ecosystem

**Project:** [cross-repo]
**Category:** dependencies
**Status:** completed
**Priority:** P1

### Tracing Stack

| Crate | Downloads | Purpose | phenoinfrakit | Status |
|-------|-----------|---------|---------------|--------|
| `tracing` | 50M+ | Core | 🔲 Not used | **ADD** |
| `tracing-subscriber` | 30M+ |_fmt, JSON | 🔲 Not used | Add with fmt |
| `tracing-appender` | 5M+ | File logging | 🔲 Not used | Add |
| `tracing-opentelemetry` | 2M+ | OTel export | 🔲 Not used | Consider |

### Metrics Crates

| Crate | Purpose | Assessment |
|-------|---------|------------|
| `metrics` | Prometheus metrics | ✅ Standard |
| `metrics-exporter-prometheus` | Prometheus export | ✅ Good |
| `opentelemetry` | Tracing API | ✅ Add for distributed |

### Logging Configuration

```rust
// Recommended: tracing with fmt layer
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

tracing_subscriber::registry()
    .with(
        fmt::layer()
            .with_target(true)
            .with_thread_ids(true)
            .with_file(true)
            .with_line_number(true)
    )
    .with(
        // JSON for production
        tracing_subscriber::fmt::layer()
            .json()
    )
    .init();
```

### Metrics Pattern

```rust
use metrics::{describe_counter, describe_histogram, increment_counter};

pub struct EventStore {
    events_appended: Counter,
    append_duration: Histogram,
}

impl EventStore {
    pub fn new() -> Self {
        describe_counter!("events_appended_total", "Total events appended");
        describe_histogram!("event_append_duration_ms", "Event append duration");
        Self {
            events_appended: Counter::new("events_appended_total"),
            append_duration: Histogram::new("event_append_duration_ms"),
        }
    }
}
```

### OpenTelemetry Integration

```rust
// For distributed tracing
use tracing_opentelemetry::OpenTelemetryLayer;
use opentelemetry_sdk::{trace, runtime};

let tracer = opentelemetry_sdk::trace::TracerProvider::builder()
    .with_batch_exporter(opentelemetry_otlp::new_exporter(), runtime::Tokio)
    .build();

let telemetry = OpenTelemetryLayer::new(tracer);
```

### LOC Impact

| Area | Current | With Additions | Impact |
|------|---------|----------------|--------|
| Logging | 0 LOC | +50 LOC | Better observability |
| Metrics | 0 LOC | +30 LOC | Better monitoring |
| Tracing | 0 LOC | +100 LOC | Debugging |

---

## 2026-03-29 - CLI & Terminal Ecosystem

**Project:** [cross-repo]
**Category:** dependencies
**Status:** completed
**Priority:** P2

### CLI Parsing

| Crate | Downloads | Assessment | heliosCLI |
|-------|-----------|------------|-----------|
| `clap` | 30M+ | ✅ STANDARD | ✅ Used |
| `structopt` | 10M+ | ⚠️ Deprecated | ❌ Not used |
| `gum` | 100K+ | 🟡 For scripts | 🔲 Not used |
| `argh` | 50K+ | 🟡 Google style | 🔲 Not used |

### Progress & Feedback

| Crate | Purpose | Downloads | Status |
|-------|---------|-----------|--------|
| `indicatif` | Progress bars | 2M+ | 🔲 Not used |
| `dialoguer` | Interactive prompts | 1M+ | 🔲 Not used |
| `console` | Terminal styling | 500K+ | 🔲 Not used |
| `colored` | Colors | 1M+ | 🔲 Not used |

### Recommended CLI Stack

```toml
# For full CLI experience
clap = { version = "4.5", features = ["derive", "help", "usage"] }
indicatif = "0.17"
dialoguer = "0.11"
console = "0.15"
anyhow = "1.0"
```

### Example: Progress Bar

```rust
use indicatif::{ProgressBar, ProgressStyle};

let pb = ProgressBar::new(100);
pb.set_style(
    ProgressStyle::default_bar()
        .template("[{bar:40}] {pos}/{len} {msg}")?
        .progress_char("━"),
);

for i in 0..100 {
    pb.set_message(format!("Processing item {}", i));
    pb.inc(1);
    tokio::time::sleep(Duration::from_millis(10)).await;
}
pb.finish_with_message("Done");
```

---

## 2026-03-29 - Rust Workspace Dependency Audit: Unused & Version Drift

**Project:** [AgilePlus, phenotype-infrakit]
**Category:** dependencies
**Status:** completed
**Priority:** P0

## Summary

Comprehensive audit identified unused dependencies and version inconsistencies in the Phenotype workspace root `Cargo.toml`. Found 3 unused workspace dependencies declared but never referenced in any crate.

## Key Findings

### Unused Workspace Dependencies (REMOVE)

| Dependency | Version | Declared At | Used In | Status | Priority |
|-----------|---------|-------------|---------|--------|----------|
| `lru` | 0.12 | workspace | NONE (0 crates) | **UNUSED** | 🔴 CRITICAL |
| `parking_lot` | 0.12 | workspace | NONE (0 crates) | **UNUSED** | 🔴 CRITICAL |
| `moka` | 0.12 | workspace | NONE (0 crates) | **UNUSED** | 🔴 CRITICAL |

**Action:** Remove from `/Users/kooshapari/CodeProjects/Phenotype/repos/Cargo.toml` lines 24-26.

### Version Drift (Update Required)

| Crate | Workspace | Sub-Crate | File | Status |
|-------|-----------|-----------|------|--------|
| `tokio` | Not declared | 1.40 (phenotype-event-sourcing) | `crates/phenotype-event-sourcing/phenotype-event-sourcing/Cargo.toml` | Inconsistent |
| `tokio` | Not declared | 1.0 (phenotype-contracts) | `crates/phenotype-contracts/phenotype-contracts/Cargo.toml` | **PINNED TOO LOW** |

**Recommendation:** Add `tokio = { version = "1.40", features = [...] }` to workspace deps; update phenotype-contracts to use workspace version.

### Workspace Consistency

| Aspect | Status | Details |
|--------|--------|---------|
| Workspace dependency usage | PARTIAL | Only 3 deps used (serde, thiserror, chrono). Most are declared but not used |
| Inconsistent tokio versions | **CRITICAL** | phenotype-contracts pinned to 1.0; phenotype-event-sourcing uses 1.40 |
| Feature flag fragmentation | ISSUE | `tokio` uses different feature sets across crates |

## Detailed Dependency Analysis

### Unused Deps: Code Search Results

```bash
# lru: No references found
$ grep -r "use lru\|lru::\|LRU\|Lru" crates/phenotype-*/ --include="*.rs"
# (0 results)

# parking_lot: No references found
$ grep -r "use parking_lot\|parking_lot::\|Mutex\|RwLock" crates/phenotype-*/ --include="*.rs"
# (0 results - only std::sync primitives used)

# moka: No references found
$ grep -r "use moka\|moka::\|Cache\|Moka" crates/phenotype-*/ --include="*.rs"
# (0 results)
```

### Declared But Unused Dependencies in Workspace

| Dep | Workspace? | Crates Using | Recommendation |
|-----|-----------|--------------|-----------------|
| `serde` | YES | All | Keep (essential) |
| `serde_json` | YES | Implicit (JSON serialization) | Keep (essential) |
| `thiserror` | YES | phenotype-event-sourcing, contracts | Keep (essential) |
| `chrono` | YES | Likely time handling | Keep (verify usage) |
| `sha2` | YES | Hash operations | Keep (verify usage) |
| `hex` | YES | Hex encoding | Keep (verify usage) |
| `dashmap` | YES | None visible | REMOVE if unused |
| `lru` | YES | None visible | **REMOVE** |
| `parking_lot` | YES | None visible | **REMOVE** |
| `moka` | YES | None visible | **REMOVE** |
| `toml` | YES | Config parsing | Keep (verify usage) |
| `regex` | YES | Pattern matching | Keep (verify usage) |
| `uuid` | YES | ID generation | Keep (verify usage) |

### Latest Versions Check

| Crate | Declared | Latest | Update Available? | Status |
|-------|----------|--------|------------------|--------|
| `serde` | 1.0 | 1.0.215 | Minor update | OK |
| `serde_json` | 1.0 | 1.0.136 | Minor update | OK |
| `thiserror` | 2.0 | 2.0.10 | Minor update | OK |
| `tokio` | 1.0/1.40 | 1.40.2 | 1.40 latest (event-sourcing OK, contracts OUTDATED) | **UPDATE** |
| `chrono` | 0.4 | 0.4.45 | Minor update | OK |
| `sha2` | 0.10 | 0.10.8 | Minor update | OK |
| `uuid` | 1.11 | 1.11.0 | Current | OK |
| `gix` | 0.79.0 | 0.81.0 | Major available (0.81) | Minor lag |
| `regex` | 1.11 | 1.11.1 | Minor update | OK |
| `axum` | Not declared | 0.8.1 | N/A (used in heliosCLI) | N/A |

---

## 2026-03-29 - TypeScript/Node Dependency Audit: Versions & Drift

**Project:** [thegent, heliosCLI]
**Category:** dependencies
**Status:** completed
**Priority:** P1

## Summary

Scanned TypeScript/JavaScript package.json files across Phenotype workspace. Found dependency version consistency issues and modern tooling opportunities.

### Key Findings

#### Main Package.json Files Scanned

| Project | Location | Manager | Status |
|---------|----------|---------|--------|
| phenotype-infrakit-docs | `/repos/package.json` | npm | Minimal deps (VitePress) |
| heliosCLI | `/repos/heliosCLI/package.json` | pnpm | Workspace root only |
| thegent-docs | `/repos/platforms/thegent/package.json` | bun | VitePress + modern tools |
| heliosCLI docs | `/repos/heliosCLI/docs/package.json` | pnpm | VitePress docs |

### Version Consistency Issues

| Package | heliosCLI docs | phenotype-docs | thegent-docs | Status |
|---------|---|---|---|---|
| `vitepress` | ^1.6.3 | ^1.6.3 | ^1.6.4 | DRIFT (1.6.3 vs 1.6.4) |
| `vue` | ^3.5.13 | ^3.5.13 | ^3.5.28 | DRIFT (3.5.13 vs 3.5.28) |
| `mermaid` | ^11.4.1 | ^11.4.1 | ^11.12.3 | DRIFT (11.4.1 vs 11.12.3) |
| `markdown-it-emoji` | Not declared | Not declared | ^3.0.0 | Only in thegent |
| `prettier` | ^3.5.3 (heliosCLI workspace root) | N/A | N/A | Only in heliosCLI |

### Outdated Versions (Major Updates Available)

| Package | Current | Latest | Recommendation |
|---------|---------|--------|---|
| `vitepress` | 1.6.4 | 1.6.4 | Current (acceptable) |
| `vue` | 3.5.28 | 3.5.28+ | Current |
| `mermaid` | 11.12.3 | 11.12.3 | Current |
| `prettier` | 3.5.3 | 3.5.3+ | Current |

### Package Manager Configuration

| Project | Manager | Config File | Status |
|---------|---------|-------------|--------|
| heliosCLI | pnpm | `pnpm@10.29.3+sha512...` | Modern (10.x) |
| thegent | bun | `bun@latest` | Cutting-edge |
| phenotype-docs | npm | (implicit) | Default npm |

**Recommendation:** Standardize to pnpm 10.x or bun across all TypeScript projects.

### Overrides/Resolutions Issues

#### heliosCLI (Workspace Root)

```json
"resolutions": {
  "braces": "^3.0.3",      // Version constraint override
  "micromatch": "^4.0.8",  // Version constraint override
  "semver": "^7.7.1"       // Version constraint override
},
"overrides": {
  "punycode": "^2.3.1",    // Constraint override
  "esbuild": ">=0.25.0"    // Constraint override
}
```

**Issue:** Multiple resolution strategies (npm `resolutions` vs pnpm `overrides`). Suggests mixed tooling or legacy configuration.

#### thegent

```json
"overrides": {
  "minimatch": ">=3.1.4",   // Security constraint
  "ajv": ">=8.17.1",        // Security constraint
  "esbuild": ">=0.25.0"     // Security constraint
}
```

**Assessment:** Clean overrides for security constraints; well-managed.

---

## 2026-03-29 - Python Dependency Audit: Modern Tooling & Version Drift

**Project:** [heliosCLI, thegent]
**Category:** dependencies
**Status:** completed
**Priority:** P1

## Summary

Comprehensive Python dependency audit across heliosCLI and thegent. Found modern tooling already adopted (uv, ruff); identified version drift and outdated minimum versions.

### Projects Scanned

| Project | File | Manager | Python Version |
|---------|------|---------|-----------------|
| heliosCLI | `heliosCLI/pyproject.toml` | uv (managed) | >=3.14 |
| thegent | `platforms/thegent/pyproject.toml` | uv (managed) | >=3.13 |
| heliosCLI harness | `heliosCLI/harness/pyproject.toml` | uv | (check) |

### Modern Tooling Already Integrated

| Tool | Project | Status | Config |
|------|---------|--------|--------|
| **uv** | heliosCLI | Adopted | `[tool.uv]` managed=true |
| **uv** | thegent | Adopted | `[tool.uv]` managed=true, workspace members |
| **ruff** | heliosCLI | Adopted | 150+ lint rules, format enabled |
| **ruff** | thegent | Adopted | 200+ lint rules, aggressive |
| **pytest** | Both | Adopted | pytest-asyncio, coverage, xdist |
| **mypy** | Both | Adopted | Type checking configured |

**Assessment:** EXCELLENT adoption of modern Python tooling.

### Dependency Version Drift

#### Core Dependencies: heliosCLI vs thegent

| Dependency | heliosCLI | thegent | Latest | Drift? |
|-----------|-----------|---------|--------|--------|
| `streamlit` | >=1.44.0 | Not used | 1.44.1 | N/A |
| `pandas` | >=2.2.0 | Not used | 2.2.3 | N/A |
| `numpy` | >=2.0.0 | Not used | 2.2.0 | N/A |
| `asyncio-mqtt` | >=0.16.0 | Not used | 0.16.1 | N/A |
| `nats-py` | >=0.24.0 | Not used | 0.24.0 | Current |
| `httpx` | Not in core | >=0.28.1 | 0.28.1 | Current |
| `typer` | Not in core | >=0.16.0 | 0.16.0 | Current |
| `pydantic` | Not in core | >=2.12.5 | 2.12.5 | Current |
| `pytest` | >=8.2.0 (dev) | >=9.0.2 (dev) | 9.0.2 | DRIFT (8.2 vs 9.0) |
| `ruff` | >=0.8.0 (dev) | >=0.15.1 (dev) | 0.15.1 | DRIFT (0.8 vs 0.15) |

**Critical Drift:** pytest versions differ by major version (8.2 vs 9.0).

### Optional Dependencies Not Used

| Project | Optional | Declared | Likely Unused |
|---------|----------|----------|---------------|
| heliosCLI | `server` | uvicorn, httpx | Check if used |
| thegent | `fast` | curl-cffi | Conditional on system/arch |

### Python Version Requirements

| Project | Declared | Rationale |
|---------|----------|-----------|
| heliosCLI | >=3.14 | Cutting-edge (latest) |
| thegent | >=3.13 | Slightly behind |

**Recommendation:** Align thegent to >=3.14 for consistency; verify 3.13 constraint is actually needed.

### Unused Development Dependencies

| Project | Dep | Purpose | Status | Remove? |
|---------|-----|---------|--------|---------|
| heliosCLI | `pytest-cov` | Coverage | Used (config exists) | Keep |
| thegent | `hypothesis` | Property-based testing | No tests found | **EVALUATE** |
| thegent | `litellm==1.82.5` | LLM client (pinned old!) | Check if current | **UPDATE** |

**Issue:** thegent has `litellm==1.82.5` pinned (exact version) while latest is 2.x. This is intentional lock but may be stale.

### Security Considerations: Overrides

Both projects use dependency overrides for security:
- heliosCLI: `punycode`, `esbuild` (JavaScript)
- thegent: Uses pyproject.toml for Python (cleaner)

---

## 2026-03-29 - Cross-Repo Dependency Consolidation Opportunities

**Project:** [cross-repo]
**Category:** dependencies
**Status:** identified
**Priority:** P1

## Consolidation Opportunities

### 1. Workspace Dependency Standardization

#### Current State

| Repo | Workspace Manager | Status |
|------|------------------|--------|
| Phenotype infrakit (Rust) | Yes | Partial (many unused deps) |
| heliosCLI (Rust) | Yes | No workspace deps |
| thegent (Python) | Partial (uv) | Using uv workspaces |
| All (TypeScript) | Mixed (pnpm, bun, npm) | Fragmented |

#### Recommendation: Standardize on Workspace Dependencies

**For Rust:**
- Keep workspace root (`/Cargo.toml`) for common deps
- Remove unused: `lru`, `parking_lot`, `moka`, `dashmap`
- Add: `tokio` (with unified version 1.40+)
- Enforce: All sub-crates must inherit workspace versions

**For Python:**
- thegent uses uv workspaces correctly
- Standardize heliosCLI on uv workspaces if multi-package

**For TypeScript:**
- Choose: pnpm 10.x (already used) or bun (cutting-edge)
- Migrate heliosCLI workspace root from npm to pnpm
- Standardize VitePress: upgrade all to 1.6.4

### 2. Shared Crate Extraction (phenotype-cache-adapter)

| Crate | Purpose | Status | Extract? |
|-------|---------|--------|----------|
| `phenotype-cache-adapter` | Cache abstractions | Declared but minimal usage | No (too small) |
| `phenotype-contracts` | Domain models | Workspace member | Keep |
| `phenotype-event-sourcing` | Event patterns | Workspace member | Keep |

### 3. Version Alignment Matrix

Create a unified dependency version policy:

```
| Dependency | Min Version | Max Version | Policy | Why |
|-----------|-------------|-------------|--------|-----|
| serde | 1.0 | 1.0.x | Minor bumps OK | Stable 1.0 API |
| tokio | 1.40 | 1.x | Stay on 1.40+ | Critical feature parity |
| gix | 0.79 | 0.81 | Can bump to 0.81 | No breaking changes |
| pytest | 9.0 | 9.x | Upgrade heliosCLI | Feature parity with thegent |
| ruff | 0.15 | 0.x | Upgrade heliosCLI | Lint rules sync |
| vitepress | 1.6.4 | 1.6.x | Align all to 1.6.4 | No minor drift |
```

---

## 2026-03-29 - Action Plan & Implementation Priority

## P0 - CRITICAL (Remove/Fix Now)

| ID | Action | Repo | Effort | Impact |
|----|--------|------|--------|--------|
| CLEAN-001 | Remove unused: `lru`, `parking_lot`, `moka` | Phenotype/Cargo.toml | 5 min | Cleaner workspace |
| CLEAN-002 | Update phenotype-contracts to tokio 1.40 | phenotype-contracts/Cargo.toml | 5 min | Version consistency |
| CLEAN-003 | Add tokio to workspace deps with 1.40 | Phenotype/Cargo.toml | 5 min | Single source of truth |
| VER-001 | Upgrade heliosCLI pytest from 8.2 to 9.0 | heliosCLI/pyproject.toml | 10 min | Test suite consistency |
| VER-002 | Upgrade heliosCLI ruff from 0.8 to 0.15 | heliosCLI/pyproject.toml | 10 min | Lint rule parity |

## P1 - HIGH (Next Sprint)

| ID | Action | Repo | Effort | Impact |
|----|--------|------|--------|--------|
| SYNC-001 | Standardize all VitePress to 1.6.4 | phenotype-docs, heliosCLI, thegent | 15 min | No drift |
| SYNC-002 | Standardize Vue to 3.5.28 | phenotype-docs, heliosCLI | 10 min | Latest stable |
| SYNC-003 | Standardize Mermaid to 11.12.3 | phenotype-docs, heliosCLI | 10 min | Latest features |
| PKG-001 | Migrate heliosCLI npm workspace to pnpm | heliosCLI/package.json | 20 min | Consistency with others |
| PYTHON-001 | Align Python min version thegent 3.13→3.14 | thegent/pyproject.toml | 5 min | Consistency |

## P2 - MEDIUM (Plan)

| ID | Action | Repo | Effort | Impact |
|----|--------|------|--------|--------|
| AUDIT-001 | Verify chrono, sha2, hex, toml, regex actually used | Phenotype/Cargo.toml | 15 min | Remove more unused |
| AUDIT-002 | Verify dashmap usage | Phenotype/Cargo.toml | 5 min | Possible remove |
| MIGRATE-001 | Plan `gix` 0.79→0.81 upgrade | AgilePlus/git | 30 min | Latest gix features |
| LIT-001 | Check if `litellm==1.82.5` pinning is intentional | thegent/pyproject.toml | 10 min | May need major update |
| HYPOTHESIS-001 | Verify `hypothesis` package is actually used | thegent/pyproject.toml | 10 min | Remove if unused |

---

## 2026-03-29 - Remaining Questions & Follow-ups

| Question | Impact | Owner |
|----------|--------|-------|
| Are chrono, sha2, hex, regex actually used in phenotype crates? | Could remove 5+ more deps | Dependency audit |
| Is `dashmap` used anywhere? | Could remove 1 more dep | Dependency audit |
| Why is `litellm==1.82.5` pinned to exact old version? | Update or document reason | thegent maintainer |
| Is `hypothesis` actively used in tests? | Remove if unused | thegent test lead |
| Can heliosCLI's `server` optional dep be removed? | Cleaner optional deps | heliosCLI maintainer |

---

## 2026-03-29 - Wave 92: Supply chain, wrapping, expanded fork matrix

**Project:** [cross-repo] | **Status:** in_progress | **Priority:** P1

### Additional fork / extract candidates

| ID | Source | Target | Est. LOC |
|----|--------|--------|----------|
| FORK-005 | `thegent-hooks` `*Error` enums | `thegent-hooks-error` | 180 |
| FORK-006 | `heliosCLI` harness `*Error` | `harness-core-error` | 120 |
| FORK-007 | Nested `crates/*/*/src` | Remove duplicate root | 400+ |
| FORK-008 | PTY + process groups | `phenotype-process` | 750 |
| FORK-009 | JSON Schema TS+Rust | `phenotype-jsonschema` | 200 |

### Supply chain / provenance

| Tool | Action |
|------|--------|
| `cargo-cyclonedx` | **PILOT:** `.github/workflows/sbom.yml` → artifacts `cyclonedx-sbom-<crate>` per matrix member (JSON spec 1.5); includes `phenotype-error-core`, contracts, errors, event-sourcing, policy-engine, cache-adapter, `phenotype-config-core` |
| `syft` | WRAP in release pipeline |
| OSV-Scanner | ADOPT for batch triage |
| `cargo audit` + `cargo deny advisories` | Run both weekly |

### Black-box wraps

`tower`, `jsonwebtoken`, `argon2`, `opentelemetry`, `@modelcontextprotocol/sdk`, `zod` — use only at adapter boundaries; no domain imports.

---

## 2026-03-29 - Wave 93: Verified workspace `Cargo.toml` usage (`crates/*.rs` only)

**Method:** `rg -l '<pattern>' repos/crates --glob '*.rs'` on 2026-03-29. Scoped to workspace member trees under `crates/` (excludes vendored `*-wtrees`).

| Workspace dep | Pattern | Files hitting |
|-----------------|---------|---------------|
| (effective) `chrono` | `chrono::` | 6 |
| `toml` | `toml::` | 2 |
| `regex` | `regex::` | 2 |
| `dashmap` | `dashmap::` | 1 |
| `sha2` | `sha2::` | 1 |
| `hex` | `hex::` | 1 |
| `moka` | `moka::` | 0 |
| `lru` | `lru::` | 0 |
| `parking_lot` | `parking_lot::` | 0 |

**Conclusion:** `moka`, `lru`, `parking_lot` have **no** Rust source references under `crates/` — safe candidates to drop from **workspace** `[workspace.dependencies]` after `cargo check` confirms no transitive need. `dashmap` is **narrow** (single file); confirm it is not only in a nested duplicate crate before removal.

**Resolves follow-ups:** Rows in “Remaining Questions” for chrono/sha2/hex/regex/dashmap — **answered** for this repo slice; re-run after deleting nested `crates/<pkg>/<pkg>/` trees.

---

## 2026-03-30 - CycloneDX SBOM pilot (CI)

**Project:** phenotype-infrakit | **Status:** implemented | **Priority:** P1

### What shipped

| Item | Detail |
|------|--------|
| Workflow | `.github/workflows/sbom.yml` |
| Triggers | `push` to `main`, `pull_request`, `workflow_dispatch` |
| Tool | `cargo-cyclonedx@0.5.9` via `taiki-e/install-action` |
| Scope | Matrix of workspace members (see workflow `matrix.include`) |
| Output | One `sbom-<crate-id>.json` next to each crate manifest |
| Artifact names | `cyclonedx-sbom-<crate-id>` (one upload per matrix row) |

### Stacked delivery (historical)

Earlier stacked PRs (#99–#101) were closed without merge; workflow initially landed via #95. Docs + session + matrix expansion ship together on `main` (see session folder + current PR).

### Next expansions

- [ ] Add remaining workspace members to the matrix (`agileplus-api-types`, crypto, iter, port-traits, state-machine, string, telemetry, test-infra, time).
- [ ] Attach SBOM to GitHub Releases for tagged builds (release workflow).

---

_Last updated: 2026-03-30_
