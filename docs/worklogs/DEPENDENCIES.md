# Dependencies Worklogs

**Category:** DEPENDENCIES | **Updated:** 2026-03-29

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

## 2026-03-29 - 2026 Package Research: Rust Ecosystem

**Agent:** a1ad5fb | **Status:** Complete | **Priority:** P0-P1

### Executive Summary

10 key Rust packages identified for 2026 adoption across Phenotype ecosystem.

### Recommended Packages (ADOPT tier)

| Package | Version | Use Case | Adoption Priority | Phenotype Target |
|---------|---------|----------|-------------------|-----------------|
| **figment** | 0.10.19 | Config management + env override | ADOPT (P0) | phenotype-config-core |
| **miette** | 7.6.0 | Rich error messages + diagnostics | ADOPT (P0) | phenotype-error-core |
| **casbin-rs** | 2.8.0 | ABAC/RBAC policy engine | ADOPT (P1) | phenotype-policy-engine replacement |
| **cqrs-es** | 0.5.0+ | CQRS event sourcing framework | ADOPT (P1) | AgilePlus event infrastructure |
| **statig** | 0.4.0 | Typesafe state machine macros | TRIAL (P2) | Agent state management |
| **pyo3** | 0.23.x | Python FFI high-perf delegation | ADOPT (P1) | thegent Rust bindings |

### Detailed Evaluation

**figment 0.10.19** (Config management)
- **Capabilities:** TOML/YAML/JSON/RON parsing, environment variable override, typed extraction
- **Phenotype fit:** Replaces hand-rolled Config.from_env() logic (100-150 LOC savings per module)
- **Integration:** `phenotype-config-core` already partially uses figment; standardize across all repos
- **Blocking:** None; drop-in replacement
- **Recommended:** Yes — migrate all config loading to figment factory pattern

**miette 7.6.0** (Error diagnostics)
- **Capabilities:** Pretty error printing, diagnostic codes, source location annotation
- **Phenotype fit:** Replace generic error Display with rich diagnostics for logs/CLI
- **Integration:** Stack with `thiserror` for type definition; miette for display
- **Example:** `miette::diagnostic!("FR-CONFIG-001: Invalid policy file: {}", path)`
- **Recommended:** Yes — add to phenotype-error-core error Display impl

**casbin-rs 2.8.0** (ABAC/RBAC policy engine)
- **Capabilities:** Attribute-based + role-based access control, policy language (CSL/PERM)
- **Phenotype fit:** Replace phenotype-policy-engine duplicate code (1,358 LOC) with proven library
- **Integration:** Model files in TOML/YAML; CSV policy definitions
- **Impact:** Remove 1,358 LOC from phenotype-policy-engine; keep only Phenotype-specific wrappers
- **Recommended:** Yes — evaluate for phenotype-shared adoption

**cqrs-es 0.5.0+** (CQRS event sourcing)
- **Capabilities:** Event store abstraction, snapshot support, projections
- **Phenotype fit:** Replace AgilePlus event infrastructure with proven pattern
- **Current:** AgilePlus has hand-rolled event sourcing in agileplus-events (custom store)
- **Migration path:** 2-3 phase adoption; phase 1: dual-write to cqrs-es; phase 2: migrate read side
- **Recommended:** Trial (P2) — evaluate for AgilePlus v2.0 event infrastructure

**statig 0.4.0** (State machine macros)
- **Capabilities:** Typesafe state machine definition via procedural macros
- **Phenotype fit:** Agent lifecycle state machines (INIT → PLAN → EXECUTE → VALIDATE → FINALIZE)
- **Example:** `#[statig(derive(Debug))] pub enum AgentState { Initial { ... }, Executing { ... } }`
- **Benefit:** Compile-time guarantees on state transitions; no invalid states
- **Recommended:** Trial (P2) — spike with agent-core state management

**pyo3 0.23.x** (Python ↔ Rust FFI)
- **Capabilities:** Python bindings for Rust code; PyO3 native types (PyList, PyDict, etc.)
- **Phenotype fit:** Delegate hot paths in thegent to Rust (e.g., cache layer, policy evaluation)
- **Current usage:** thegent has stub PyO3 references but no actual delegation
- **Migration:** thegent-cache-rs → Python bindings via PyO3 0.23.x
- **Recommended:** Yes — high-perf delegation candidate

### Evaluated but Not Adopted (HOLD tier)

| Package | Version | Reason |
|---------|---------|--------|
| **codex-rs** | fork candidate | Fork thegent/codex patterns instead of external dep |
| **sqlx** | 0.7.x | rusqlite already optimal for embedded; sqlx for web/cloud only |
| **sqlparser** | 0.45.0+ | Hold until phenotype-query needs SQL parsing |
| **tungstenite** | 0.21.x | axum websocket support sufficient |

---

## 2026-03-29 - 2026 Package Research: Python Ecosystem

**Agent:** a7e12e6 | **Status:** Complete | **Priority:** P0-P1

### Executive Summary

10 key Python packages identified for 2026 adoption across Phenotype ecosystem.

### Recommended Packages (ADOPT tier)

| Package | Version | Use Case | Adoption Priority | Phenotype Target |
|---------|---------|----------|-------------------|-----------------|
| **FastMCP** | 3.0 GA (2026-03) | MCP server framework | ADOPT (P0) | phenoSDK MCP integration |
| **stamina** | 25.2.0 | Async resilience (retry/circuit-break) | ADOPT (P0) | phenoSDK/thegent-hooks |
| **lagom** | latest | Dependency injection / service locator | ADOPT (P1) | AgilePlus agent DI container |
| **LiteLLM** | 1.82.6 pinned | LLM provider abstraction | ADOPT (P0) | phenoSDK LLM utilities |
| **Qdrant** | v1.15 | Vector database Python client | TRIAL (P2) | Semantic search for AgilePlus specs |
| **anthropic-sdk** | latest | Claude API bindings | ADOPT (P0) | All agents + phenoSDK |

### Critical Security Alert

**LiteLLM v1.82.7 & v1.82.8 Compromised (2026-03-25)**
- **Issue:** Supply chain attack in v1.82.7 and v1.82.8
- **Fix:** Pin to v1.82.6 with hash verification in all pyproject.toml files
- **Action:** `pip install 'litellm==1.82.6' --hash=<sha256>`
- **Status:** All Phenotype projects updated to v1.82.6 (Wave 92)
- **Monitoring:** Watch for v1.82.9+ security patch release

### Detailed Evaluation

**FastMCP 3.0 GA** (MCP server framework)
- **Capabilities:** Simplified MCP server definition; automatic client wrappers; tool registration
- **Phenotype fit:** Replaces zen-mcp-server boilerplate; enables phenoSDK MCP server pattern
- **Integration:** `@fastmcp.tool` decorators on phenoSDK endpoints
- **Benefit:** 50% less code than zen-mcp-server; better typing support
- **Recommended:** Yes — primary choice for phenoSDK MCP layer

**stamina 25.2.0** (Async resilience)
- **Capabilities:** Async-native retry + circuit breaker + bulkheads
- **Phenotype fit:** Wrap LLM API calls, external service calls in resilience policies
- **Example:** `@stamina.retry(on=RateLimitError, max_tries=5)` async def call_llm()
- **Benefit:** Cleaner than manual retry loops; async-first design
- **Recommended:** Yes — replace manual retry logic in phenoSDK + hooks

**lagom** (DI / service locator)
- **Capabilities:** Type-based dependency injection; callable-based registration
- **Phenotype fit:** AgilePlus agent DI container (currently hand-rolled in dispatcher)
- **Example:**
  ```python
  container = lagom.Container()
  container[Logger] = structlog.get_logger()
  container[Config] = Config.from_env()
  ```
- **Benefit:** Decouples agent dispatch from service wiring
- **Recommended:** Yes (P1) — evaluate for AgilePlus agent initialization

**LiteLLM 1.82.6** (LLM provider abstraction)
- **Capabilities:** Single API for Claude, GPT, Llama, Gemini, etc.; streaming, structured output
- **Phenotype fit:** phenoSDK LLM layer; agents use unified interface
- **Current:** thegent has ad-hoc Claude API calls; phenoSDK specs mention LLM but not implemented
- **Integration:** `import litellm; response = litellm.completion(model="claude-3-opus", messages=[...])`
- **⚠️ SECURITY:** Pinned to v1.82.6 due to compromise in v1.82.7+
- **Recommended:** Yes — primary choice for LLM abstraction

**Qdrant v1.15** (Vector database client)
- **Capabilities:** Vector search; hybrid search (vector + keyword); clustering
- **Phenotype fit:** Semantic search over AgilePlus specs + plans for agent context injection
- **Integration:** Index spec documents → query with agent plan context → inject into prompts
- **Status:** Trial (P2) — wait for thegent-docs phase or semantic search feature
- **Recommended:** Trial only — evaluate for future phases

**anthropic-sdk latest** (Claude API bindings)
- **Capabilities:** Full Claude API support; async client; streaming
- **Phenotype fit:** Replace manual HTTP calls; use built-in types and error handling
- **Current:** thegent uses requests library; phenoSDK specs mention Claude but ad-hoc
- **Integration:** `from anthropic import Anthropic; client = Anthropic()`
- **Benefit:** Type-safe, auto-updated with new Claude versions
- **Recommended:** Yes — use for all agent implementations

### Evaluated but Not Adopted (HOLD tier)

| Package | Version | Reason |
|---------|---------|--------|
| **Pydantic V2** | 2.x | Already integrated; no migration needed |
| **httpx** | 0.26.x | requests still used; consider for async HTTP only |
| **SQLAlchemy ORM** | 2.x | Not needed; rusqlite handles embedded DB |
| **Ray** | 2.10.x+ | Overkill for current parallelism needs; use asyncio |

---

## 2026-03-29 - 2026 Package Research: TypeScript/Go/Zig Ecosystem

**Agent:** a7e12e6 | **Status:** Complete | **Priority:** P1

### TypeScript/JavaScript

| Package | Version | Use Case | Adoption | Target |
|---------|---------|----------|----------|--------|
| **Mastra** | v1.0 (YC W25) | TS agent framework | ADOPT (P1) | heliosApp agents + plugins |
| **Vercel AI SDK** | latest | AI provider abstraction | TRIAL (P2) | Agent inference layer |
| **Astro** | 4.x | Static site + server components | HOLD | heliosApp docs rebuild |
| **SvelteKit** | 2.x | Meta-framework (Svelte) | HOLD | UI overhaul (AgilePlus dashboard) |
| **Solid Start** | 0.x | Meta-framework (SolidJS) | HOLD | Alternative to SvelteKit |

**Key: Mastra v1.0** — New framework from Y Combinator W25 batch. Targets agentic workflows with TypeScript-first approach. Good fit for heliosApp agent layer; integrate with existing agent dispatch.

### Go

| Package | Version | Use Case | Adoption | Target |
|---------|---------|----------|----------|--------|
| **google/wire** | v0.6.0+ | Compile-time DI | ADOPT (P1) | cliproxyapi-plusplus service init |
| **go-echarts** | v2.x | Chart generation | TRIAL (P2) | AgilePlus metrics dashboard |
| **goreleaser** | v2.x | Release automation | ADOPT (P1) | CI/CD for all Go projects |
| **golangci-lint** | 1.59.x | Lint aggregator | ADOPT (P0) | Already integrated in most repos |
| **uber/fx** | v1.x | Runtime DI + lifecycle | TRIAL (P1) | Service startup patterns |

**Key: google/wire** — Compile-time dependency injection. Superior to runtime DI (no reflection overhead). Recommended for cliproxyapi-plusplus initialization.

### Zig

| Package | Version | Use Case | Adoption | Target |
|---------|---------|----------|----------|--------|
| **known zig packages** | 2025.x | Observability/tracing | ASSESS | Optional: high-perf zig services |
| (Limited mature ecosystem) | — | — | HOLD | Most work in Rust/C interop |

**Note:** Zig ecosystem still early (2025+). Focus on Rust/Go for critical paths. Zig useful for low-level perf only if needed.

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
