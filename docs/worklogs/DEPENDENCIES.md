# Dependencies Worklogs

**Category:** DEPENDENCIES | **Updated:** 2026-03-29

---

## 2026-03-29 - External Dependencies & Package Modernization Audit (v3)

**Project:** [cross-repo]
**Category:** dependencies
**Status:** in_progress
**Priority:** P0

### Summary

Comprehensive audit of external dependencies, package modernization opportunities, and fork candidates. Includes analysis of blackbox vs whitebox usage patterns and 2026 crate evaluations.

### Fork Candidates (Internal → Shared Libraries)

| ID | Source | Target | LOC | Priority | Status |
|----|--------|--------|-----|----------|--------|
| FORK-001 | `utils/pty` | `phenotype-process` | ~750 | 🔴 CRITICAL | TODO |
| FORK-002 | `error.rs` pattern | `phenotype-error` | ~400 | 🔴 CRITICAL | TODO |
| FORK-003 | `utils/git` | `phenotype-git` | ~300 | 🟠 MEDIUM | EVALUATE |
| FORK-004 | `utils/config` | `phenotype-config` | ~200 | 🟠 MEDIUM | EVALUATE |

### External Crates Assessment (2026)

#### 🔴 CRITICAL - Fork/Adopt Now

| Crate | Version | Action | Current LOC | Target LOC | Savings |
|-------|---------|--------|-------------|------------|---------|
| `command-group` | 5.0.1 | ADOPT | ~1,433 | ~300 | **79%** |
| `figment` | 0.10.19 | ADOPT | ~760 | ~150 | **80%** |
| CodexErr pattern | N/A | FORK → phenotype-error | ~400 | ~100 | **75%** |
| `gix` | 0.79.0 | MIGRATE from git2 | ~500 | ~200 | **60%** |

#### 🟠 HIGH - Evaluate

| Crate | Version | Action | Benefit |
|-------|---------|--------|---------|
| `indicatif` | 0.18.4 | ADOPT | CLI progress bars |
| `utils/pty` | N/A | FORK → phenotype-process | ~500 LOC |

#### 🟡 MEDIUM - Consider

| Crate | Version | Action |
|-------|---------|--------|
| `eventually` | 0.4.0 | EVALUATE for event sourcing |
| `signal-hook` | 0.4.3 | EVALUATE for graceful shutdown |
| `miette` | 7.2.0 | EVALUATE for pretty errors |
| `smallvec` | 1.17.0 | EVALUATE for collections |

#### 🟢 LOW - Nice to Have

| Crate | Version | Action |
|-------|---------|--------|
| `console` | 0.16.2 | EVALUATE |
| `dialoguer` | 0.11.0 | EVALUATE |
| `rkyv` | 0.8.0 | EVALUATE |

### Standard Crates (Optimal - No Action Needed) ✅

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

### Modern Tooling Already Integrated ✅

| Tool | Usage | Location |
|------|-------|----------|
| `uv` | Python package management | `python/Dockerfile.python`, `python/pyproject.toml` |
| `ruff` | Python linting/formatting | `python/ruff.toml`, CI pipeline |
| `gix` | Git operations (v0.79) | `Cargo.toml:91`, `agileplus-git` |
| `buf` | Proto lint/breaking checks | `buf.yaml`, CI pipeline |

### Could Improve Codebase 🟠

| Crate | Purpose | Recommendation | Priority |
|-------|---------|----------------|----------|
| `command-group` | Process group management | Wrap/Adopt | P2 |
| `tokio-command` | Async command wrapper | Evaluate | P3 |
| `git-worktree` | Worktree operations | Wrap | P2 |
| `figment` | Config management | Evaluate | P3 |
| `indicatif` | Progress bars | Add to CLI | P3 |
| `dialoguer` | CLI prompts | Add to CLI | P3 |
| `console` | Terminal utilities | Evaluate | P3 |

### Migration Needed 🟡

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

### Total LOC Impact from External Packages

| Category | Current | External + Adoption | Reduction |
|----------|---------|-------------------|-----------|
| Process/PTY | ~1,433 | ~300 (command-group) | **79%** |
| Config loading | ~760 | ~150 (figment) | **80%** |
| Error handling | ~400 | ~100 (phenotype-error) | **75%** |
| Git operations | ~500 | ~200 (gix migration) | **60%** |
| CLI progress | ~100 | ~20 (indicatif) | **80%** |
| **TOTAL** | **~3,193** | **~770** | **~76%** |

### Tasks Completed

- [x] Audited all external dependencies
- [x] Identified fork candidates
- [x] Documented security advisories
- [x] Categorized blackbox/whitebox usage
- [x] Created fork decision matrix
- [x] Evaluated 2026 crate landscape
- [x] Quantified LOC savings potential

### Next Steps

- [ ] FORK-001: Create `phenotype-process` from `utils/pty`
- [ ] FORK-002: Create `phenotype-error` from error patterns
- [ ] 3P-MIG-001: Plan `git2` → `gix` migration
- [ ] Evaluate `command-group` for process management
- [ ] Evaluate `figment` for config loading
- [ ] Evaluate `indicatif` for CLI progress

### Related
### Related

- Fork Research: `plans/2026-03-29-FORK_CANDIDATES_3RD_PARTY-v1.md`
- Master Research: `plans/2026-03-29-MASTER_RESEARCH_INDEX-v1.md`
- Duplication: `docs/worklogs/DUPLICATION.md`

---

## 2026-03-29 - thegent Python Patterns Research

**Project:** [thegent]
**Category:** dependencies
**Status:** completed
**Priority:** P1

### Summary

Comprehensive analysis of thegent Python patterns that could inform shared library development or cross-language tooling.

### Fork Candidates from thegent

| Source | Target | LOC | Priority | Why Fork |
|--------|--------|-----|----------|----------|
| `mesh/git.py` | `phenotype-git-async` | ~426 | 🔴 HIGH | Per-agent index isolation, CAS ref updates |
| `mesh/ipc.py` | `phenotype-ipc` | ~414 | 🔴 HIGH | Atomic locks, maildir queues, intent registry |
| `mesh/coordination.py` | `phenotype-coordination` | ~327 | 🟠 MEDIUM | HLC timestamps, OCC, conflict prediction |
| `agents/state_machine.py` | `phenotype-state-machine` | ~370 | 🟠 MEDIUM | Phase transitions, promotion gates |
| `config/settings.py` | `phenotype-config-python` | ~1034 | 🟠 MEDIUM | Pydantic Settings, validation |

### Key Patterns Discovered

#### 1. Git Parallelism (`mesh/git.py:27-415`)

```python
class GitParallelismManager:
    def ensure_index(self) -> None:  # Per-agent index files
    def create_commit_from_index(self) -> str:  # Plumbing commands
    def update_ref_cas(self, ref, expected, value) -> bool:  # CAS with backoff
    def wait_for_index_lock(self) -> bool:  # Stale lock cleanup
```

**Why Valuable:** Core multi-agent coordination pattern with:
- Per-agent index files via `GIT_INDEX_FILE` env var
- Compare-and-swap ref updates with exponential backoff + jitter
- Plumbing commands (`write-tree`, `commit-tree`) for atomic commits

#### 2. IPC Primitives (`mesh/ipc.py:46-83`)

```python
class IPCMesh:
    def acquire_atomic_lock(self, lock_name: str, ttl: int = 60) -> bool:  # mkdir-based
    def get_maildir_queue(self) -> MaildirQueue:  # tmp/new/cur structure

class MaildirQueue:
    def enqueue(self, item: Any) -> str:  # Atomic os.rename
    def claim(self, receipt: str) -> Any:  # Ownership transfer
```

**Why Valuable:** Filesystem-based distributed coordination:
- tmpfs for high-performance IPC
- mkdir-based atomic locks with TTL
- Maildir-style task queues

#### 3. Hybrid Logical Clocks (`mesh/coordination.py:15-48`)

```python
class HLCTimestamp:
    def update(self, other: Optional["HLCTimestamp"] = None) -> "HLCTimestamp":
        now = int(time.time() * 1000)
        if other:
            self.physical = max(self.physical, other.physical, now)
```

**Why Valuable:** Distributed ordering without synchronized clocks

#### 4. State Machine Pattern (`agents/state_machine.py:60-111`)

```python
_PHASE_TRANSITIONS: ClassVar[Any] = MappingProxyType({
    "pending": ["running", "failed"],
    "running": ["success", "failed", "fallback", "paused"],
})

class FallbackStateMachine:
    def promote(self, evidence: dict) -> bool:  # PromotionGate
    def suggest_fallbacks(self) -> list[str]:  # Safe fallback suggestion
```

**Why Valuable:** Explicit state validation before transitions

### Cross-Language Patterns

| Pattern | Python (thegent) | Rust (AgilePlus) | Shared Crate |
|---------|------------------|-------------------|--------------|
| State machines | `state_machine.py` | `agileplus-domain` | `phenotype-state` |
| IPC | `ipc.py` | Manual `Command` | `phenotype-ipc` |
| Coordination | `coordination.py` | None | `phenotype-coord` |
| Git ops | `git.py` | `agileplus-git` | `phenotype-git` |

---

## 2026-03-29 - phenotype-infrakit Deep Analysis

**Project:** [phenotype-infrakit]
**Category:** dependencies + structure
**Status:** completed
**Priority:** P1

### Summary

Analysis of phenotype-infrakit workspace confirming production-ready crates vs archive candidates.

### Production-Ready Crates

| Crate | LOC | Status | Recommendation |
|-------|-----|--------|----------------|
| `phenotype-event-sourcing` | ~758 | Production | Keep, publish to crates.io |
| `phenotype-policy-engine` | ~1,190 | Production | Keep, unique TOML loader |
| `phenotype-contracts` | ~400 | Production | Keep as canonical ports |

### Archive Candidates

| Crate | LOC | Issue | Action |
|-------|-----|-------|--------|
| `phenotype-state-machine` | 0 | Empty stub | **ARCHIVE** |
| `phenotype-cache-adapter` | 0 | Empty stub | **ARCHIVE** |

### Event Sourcing vs Contracts Overlap

| Aspect | `phenotype-event-sourcing` | `phenotype-contracts` |
|--------|---------------------------|----------------------|
| Event ID | UUID | `EventMetadata.event_id` |
| Timestamp | `DateTime<Utc>` | `EventMetadata.timestamp` |
| Event Type | String | `DomainEvent::event_type()` |
| Payload | Generic `T: Serialize` | Generic `T: Serialize + Deserialize` |

**Finding:** Overlapping but complementary - ES provides hash chain integrity, contracts provide port interfaces.

### TOML Policy Loader (Unique)

`phenotype-policy-engine/src/loader.rs:238 LOC`
- Unique differentiator vs OPA
- Production-ready with 43 tests
- TOML-based policy configuration

---

_Last updated: 2026-03-29_
**Project:** [phenotype-infrakit]
**Category:** dependencies + structure
**Status:** completed
**Priority:** P0

### Summary

Comprehensive audit of phenotype-infrakit workspace structure and dependencies. Identified nested crate duplication issue, confirmed dead code, and assessed whitebox/blackbox dependency usage.

### Workspace Structure Analysis

#### Nested Crate Duplication Issue

| Crate | `crates/X/` | `crates/X/X/` | Difference |
|-------|-------------|---------------|------------|
| `phenotype-cache-adapter` | ✅ Has src/ | ✅ Has src/ | **100% IDENTICAL** |
| `phenotype-contracts` | ✅ Has src/ | ✅ Has src/ | **100% IDENTICAL** |
| `phenotype-event-sourcing` | ✅ Has src/ | ✅ Has src/ | **Formatting only** |
| `phenotype-policy-engine` | ✅ Has src/ | ✅ Has src/ | **100% IDENTICAL** |
| `phenotype-state-machine` | ❌ NO src/ | ✅ Has src/ | **INCOMPLETE** |

**Root Cause:** Work-in-progress rebase created nested copies. The inner copies (`crates/X/X/`) contain the actual implementation.

**Recommended Fix:**
```bash
# Remove outer duplicate directories (keep inner)
rm -rf crates/phenotype-cache-adapter/phenotype-cache-adapter
rm -rf crates/phenotype-contracts/phenotype-contracts
rm -rf crates/phenotype-event-sourcing/phenotype-event-sourcing
rm -rf crates/phenotype-policy-engine/phenotype-policy-engine

# Keep phenotype-state-machine as stub or delete entirely
```

---

### Dependency Analysis (Whitebox/Blackbox)

#### Blackbox (Direct Use - Optimal) ✅

| Dependency | Version | Purpose | Assessment |
|------------|---------|---------|-------------|
| `serde` | 1.0 | Serialization | **OPTIMAL** - Industry standard |
| `serde_json` | 1.0 | JSON | **OPTIMAL** - No fork needed |
| `thiserror` | 2.0 | Error handling | **OPTIMAL** - Already using derive |
| `parking_lot` | 0.12 | Sync primitives | **OPTIMAL** - Better than std Mutex |
| `dashmap` | 6.1 | Concurrent map | **OPTIMAL** - No customization needed |
| `moka` | 0.12 | Cache | **OPTIMAL** - Actively maintained |
| `toml` | 0.8 | TOML parsing | **OPTIMAL** - Standard crate |
| `regex` | 1.11 | Regex | **OPTIMAL** - No fork needed |
| `uuid` | 1.11 | UUIDs | **OPTIMAL** - Using features |
| `chrono` | 0.4 | Time | **OPTIMAL** - Using serde feature |
| `sha2` | 0.10 | Hashing | **OPTIMAL** - Using Digest trait |
| `hex` | 0.4 | Hex encoding | **OPTIMAL** - Simple utility |

**Verdict:** All dependencies are blackbox-optimal. No whitebox or graybox needed.

---

#### Whitebox Potential (Fork Candidates) - NONE IDENTIFIED

| Dependency | Fork Candidate? | Reason |
|------------|-------------------|--------|
| `serde` | ❌ | Perfect as-is |
| `parking_lot` | ❌ | No phenotype-specific needs |
| `dashmap` | ❌ | No phenotype-specific needs |
| `moka` | ❌ | Cache patterns are standard |
| `thiserror` | ❌ | Using derive pattern correctly |

**Analysis:** The codebase uses external crates correctly with no need for customization.

---

#### Graybox Potential (Wrap/Extend) - NONE IDENTIFIED

| Use Case | Current | Recommended |
|----------|---------|-------------|
| Caching | `moka` direct | **Keep** - sufficient |
| Hashing | `sha2` direct | **Keep** - sufficient |
| Errors | `thiserror` direct | **Keep** - sufficient |
| Config | No wrapper | **ADD** `figment` if needed |

---

### Dead Code: phenotype-state-machine

**Status:** CONFIRMED DEAD

| Aspect | Finding |
|--------|---------|
| `src/` directory | **MISSING** - No implementation |
| `Cargo.toml` | Exists but empty dependencies |
| Workspace reference | In `Cargo.toml` members list |
| Usage | ZERO external references |

**Recommended Action:**
```bash
# Option 1: Delete entirely
rm -rf crates/phenotype-state-machine/

# Option 2: Keep as stub with documentation
# Add src/lib.rs with:
//! # phenotype-state-machine
//! 
//! This crate is reserved for future state machine implementations.
//! Currently unused - do not add dependencies.

# Then remove from workspace members in Cargo.toml
```

---

### LOC Analysis

| Crate | Current | After Cleanup | Savings |
|-------|---------|--------------|---------|
| Nested duplicates | 5 × ~100 LOC waste | 0 | **500 LOC** |
| phenotype-state-machine | ~50 LOC (stub) | 0 | **50 LOC** |
| **Total** | | | **550 LOC** |

---

### 2026 External Package Recommendations

Given the workspace is already well-optimized, minimal additions recommended:

#### Immediate (Low LOC, High Value)

| Crate | Purpose | LOC Savings |
|-------|---------|-------------|
| `derive_more` | Reduce boilerplate | ~60 lines |
| `strum` | Enum utilities | ~30 lines |
| `rstest` | Parameterized tests | ~50 lines |

#### Future Consideration

| Crate | Purpose | When |
|-------|---------|------|
| `figment` | Config loading | If config complexity grows |
| `eventually` | Event sourcing | If ES patterns needed |
| `ratatui` | TUI | If CLI dashboards needed |

---

### Action Items

- [ ] 🔴 CRITICAL: Remove nested duplicate directories
- [ ] 🔴 CRITICAL: Delete or stub phenotype-state-machine
- [ ] 🟠 HIGH: Add `derive_more` to workspace
- [ ] 🟠 HIGH: Add `strum` to workspace
- [ ] 🟡 MEDIUM: Add `rstest` for parameterized tests

---

### Related

- `docs/worklogs/DUPLICATION.md` - LOC reduction patterns
- `Cargo.toml` - Workspace configuration

**Project:** [cross-repo]
**Category:** dependencies
**Status:** in_progress
**Priority:** P1

### Deep Dive: command-group

**Why:** Cross-platform process group management with proper signal propagation

**Current State:**
- 3 repos have manual Command wrappers (vibe-kanban, heliosCLI, agileplus)
- ~1,433 LOC of duplicated process management code
- Manual SIGINT/SIGTERM handling in each daemon

**command-group Features:**
```rust
use command_group::{CommandGroup, AsyncCommandGroupExt};

let mut cmd = Command::new("bash");
cmd.arg("-c");
cmd.arg("sleep 100");
let group = cmd.group_spawn()?;

// On drop, kills entire process group
// Proper SIGINT propagation
```

**Integration Plan:**
1. Add to workspace dependencies
2. Replace vibe-kanban process spawning
3. Replace heliosCLI/pty process handling
4. Replace agileplus-daemon signal handling

**Priority:** 🔴 CRITICAL - saves ~1,000 LOC

---

### Deep Dive: figment

**Why:** Mature config management with profiles, env overrides, provenance tracking

**Current State:**
- 4 independent config loaders (TOML, YAML, JSON, Builder)
- ~760 LOC of duplicated config code
- `libs/config-core` exists but UNUSED (edition mismatch)

**figment Features:**
```rust
use figment::{Figment, providers::{Env, Toml, Format}};

let config = Figment::new()
    .merge(Toml::file("config.toml"))
    .merge(Env::prefixed("APP_"))
    .extract::<Config>()?;
```

**Integration Plan:**
1. Migrate `libs/config-core` to use figment
2. Add to workspace
3. Replace TOML loader in agileplus-domain
4. Replace YAML loader in agileplus-telemetry
5. Replace JSON loader in vibe-kanban

**Priority:** 🟠 HIGH - saves ~600 LOC

---

### Deep Dive: signal-hook

**Why:** Structured async signal handling with proper lifetime management

**Current State:**
- Manual signal handling in 5+ daemon processes
- Inconsistent SIGINT/SIGTERM behavior
- Race conditions in shutdown paths

**signal-hook Features:**
```rust
use signal_hook::{async_std::Signals, SIGINT, SIGTERM};

let signals = Signals::new([SIGINT, SIGTERM])?;
signal_hook::async_std::flags::block_signals(&signals)?;

while let Some(signal) = signals.next().await {
    match signal {
        SIGINT => shutdown("SIGINT").await,
        SIGTERM => shutdown("SIGTERM").await,
    }
}
```

**Priority:** 🟡 MEDIUM - improves reliability

---

### Deep Dive: eventually

**Why:** Production-ready event sourcing patterns fromCQRS/ES community

**Current State:**
- `agileplus-events` has basic event store (~300 LOC)
- No upcasting, versioning, or migration support
- `phenotype-event-sourcing` exists but experimental

**eventually Features:**
```rust
use eventually_core::{Aggregate, Event, EventStore};
use eventually_postgres::PostgresEventStore;

pub struct Order {
    pub id: OrderId,
    pub status: OrderStatus,
    pub items: Vec<OrderItem>,
}

#[derive(Event)]
#[event(version = 1)]
enum OrderEvent {
    OrderPlaced { items: Vec<OrderItem> },
    OrderShipped { tracking: String },
}
```

**Integration Plan:**
1. Evaluate eventually as foundation
2. Add phenotype-specific extensions
3. Consider FORK → `phenotype-events`

**Priority:** 🟡 MEDIUM - long-term architecture

---

### Deep Dive: miette

**Why:** Pretty diagnostic errors for CLI tools

**Current State:**
- Basic thiserror in CLI tools
- No source highlighting or code snippets

**miette Features:**
```rust
use miette::{Diagnostic, Help, LabeledSpan};

#[derive(Diagnostic, Error)]
#[error("Parse error")]
struct ParseError {
    #[source_code]
    src: String,
    #[label("here")]
    span: SourceOffset,
    #[help]
    note: Option<String>,
}
```

**Priority:** 🟢 LOW - nice to have

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

_Last updated: 2026-03-29_
