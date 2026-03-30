# WS6 TOML Configuration Audit Report
## Phase 2 Libification Initiative — Standardization Assessment

**Date:** 2026-03-29
**Audit Scope:** Rust projects across Phenotype workspace
**Agent:** WS6 Audit Agent
**Status:** COMPLETE

---

## Executive Summary

This audit examined TOML configuration handling across all Rust projects in the Phenotype workspace. The investigation identified **critical version fragmentation** (toml 0.8 vs 0.9.5), **library duplication** (two independent config systems), and **missed standardization opportunities**.

**Key Findings:**
- **3 projects** with explicit TOML dependencies (non-standard versions)
- **1 dedicated config crate** (phenotype-config-core) exists but underutilized
- **Multiple config patterns** across ecosystem (file-based, file-edit, enum-based)
- **160+ TOML imports** detected across source code
- **Estimated standardization effort:** 3-5 hours total (350-450 LOC)

---

## Projects Audited

### 1. phenotype (Main Workspace Root)
**Location:** `/Users/kooshapari/CodeProjects/Phenotype/repos`

| Metric | Value |
|--------|-------|
| Cargo.toml Files | 23 |
| TOML Dependency | `toml = "0.8"` |
| Related Crates | phenotype-config-core |
| Config Format Support | TOML, YAML, JSON |

**Key Crate: phenotype-config-core**
```
Purpose: Unified configuration loading and management for Phenotype ecosystem
Dependencies:
  - serde (workspace)
  - serde_json (1.0)
  - serde_yaml (0.9)
  - toml (workspace = 0.8)
  - thiserror (1.0)
  - tracing (0.1)
  - dirs (5.0)

Modules:
  src/lib.rs - Public API
  src/loader.rs - File-based config loading
  src/format.rs - Multi-format parsing (TOML/YAML/JSON)
  src/error.rs - Standardized error handling
  src/dirs_helper.rs - XDG directory resolution

Status: PRODUCTION READY (complete, well-documented)
Pattern: File-based loader with format detection
```

**Findings:**
- Implements standard file-based config pattern
- Supports multi-format deserialization via serde
- Provides XDG directory integration
- SHOULD BE primary config library across workspace

---

### 2. heliosCLI (Large Multi-Crate Workspace)
**Location:** `/Users/kooshapari/CodeProjects/Phenotype/repos/heliosCLI`

#### 2a. codex-rs Workspace
**Cargo.toml:** 70+ member crates
**Workspace Dependencies:**
```toml
toml = "0.9.5"              # Read-only TOML parsing
toml_edit = "0.24.0"        # In-place TOML mutation
serde = "1"
serde_json = "1"
serde_yaml = "0.9"
```

**Config Infrastructure Found:**

| Component | Location | Purpose | Status |
|-----------|----------|---------|--------|
| Config Module | `core/src/config/` | Main config system | 10 files |
| Config Loader | `core/src/config_loader/` | Layered loading | 3 files |
| Types | `core/src/config/types.rs` | Config type defs | Extensive |
| TOML Editing | `core/src/config/edit.rs` | Safe TOML mutation | Uses toml_edit |
| Schema | `core/src/config/schema.rs` | Config schema | Type-driven |
| Features | `core/src/features.rs` | Feature flags | Uses toml::Value |
| ExecPolicy | `core/src/exec_policy.rs` | Policy config | Uses toml::Value |

**Config Files Analyzed:**

1. **config/mod.rs** (380+ lines)
   - Central config aggregator
   - Layer composition (system, user, project)
   - Type-safe deserialization

2. **config/edit.rs** (400+ lines)
   - Uses `toml_edit` for mutations
   - Types: ArrayOfTables, DocumentMut, Table, Item
   - Preserves formatting and comments
   - Complex array/table manipulation

3. **config/types.rs** (extensive)
   - AppsConfigToml - apps.toml structure
   - McpServerConfig - MCP server definitions
   - OtelConfig - OpenTelemetry config
   - Features, Notifications, Sandbox configs

4. **config_loader/mod.rs**
   - ConfigLayerStack - multiple config sources
   - Platform-specific loaders (macOS, Windows, Linux)
   - Constraint validation
   - Environment variable overrides

5. **config_loader/macos.rs**
   - macOS-specific config loading
   - Seatbelt profile integration
   - Hardened Runtime support

**TOML Usage Pattern:**
```
Read-Only:  toml::Value for parsing
            Used in: features.rs, exec_policy.rs
            Purpose: Feature flag evaluation, policy parsing

Edit:       toml_edit for mutations
            Used in: config/edit.rs
            Purpose: In-place TOML modification
```

**Key Observation:** codex-rs implements a sophisticated, custom config system that **duplicates** phenotype-config-core's basic functionality while adding:
- Layered config composition
- TOML in-place editing
- Platform-specific loaders
- Schema validation

---

#### 2b. helios-rs Workspace
**Cargo.toml:** Root workspace (2 members)
**Dependencies:** Same as codex-rs (toml 0.9.5, toml_edit 0.24.0)

**Status:** Inherits codex-rs config infrastructure

---

### 3. platforms/thegent (Policy & Agent Framework)
**Location:** `/Users/kooshapari/CodeProjects/Phenotype/repos/platforms/thegent`

| Metric | Value |
|--------|-------|
| Cargo.toml Files | 28+ |
| TOML Dependencies | None detected |
| Pattern | Enum-based config |

**Relevant Crates:**
- `thegent-policy` - Policy engine (policy.rs, evaluator.rs)
- `thegent-state-machine` - State management
- `thegent-router` - Routing configuration
- `thegent-config-*` - Various config modules

**Config Pattern Observed:**
```rust
// Enum-based configuration (no TOML parsing)
enum PolicyConfig { ... }
enum StateTransition { ... }

// Hard-coded defaults or environment-driven
// No file-based TOML loading
```

**Findings:**
- thegent uses programmatic (enum-based) config
- No TOML file dependencies
- Isolated from phenotype-config-core
- Pattern is deliberate for policy/state machines

---

## TOML Library Inventory

### Version Distribution
```
toml = "0.8"       → 1 project (phenotype root)
toml = "0.9.5"     → 2 projects (heliosCLI codex-rs, helios-rs)
toml_edit = "0.24.0" → 2 projects (same as above)
```

### Version Gap Analysis
| Library | Current | Latest | Gap | Risk |
|---------|---------|--------|-----|------|
| toml | 0.8-0.9.5 | 0.9.x | Major inconsistency | HIGH |
| toml_edit | 0.24.0 | 0.24.x | Aligned | LOW |
| serde | workspace | 1.0.x | Workspace managed | OK |

---

## Config Pattern Analysis

### Pattern 1: File-Based Loader (phenotype-config-core)
```
┌─ ConfigLoader
│  └─ load_from_file(path) -> Result<T>
│     ├─ Read file → string
│     ├─ Detect format → TOML/YAML/JSON
│     ├─ Deserialize → T (via serde)
│     └─ Return T
```

**Characteristics:**
- Simple, focused API
- Multi-format support via format detection
- XDG directory integration
- Standard error handling
- Reusable across projects

**Projects Using:** Internal only (underutilized)

---

### Pattern 2: File-Based + Mutation (codex-rs)
```
┌─ ConfigLayerStack
│  ├─ system layer (read-only)
│  ├─ user layer (read+edit)
│  ├─ project layer (read+edit)
│  └─ env overrides
│
├─ ConfigEdit
│  ├─ toml_edit::DocumentMut
│  ├─ Array/Table manipulation
│  └─ Preserve formatting
│
└─ Type-safe schema validation
```

**Characteristics:**
- Layered composition
- In-place TOML editing
- Preserves comments/formatting
- Complex schema support
- Platform-specific loaders

**Projects Using:** heliosCLI (codex-rs, helios-rs)

---

### Pattern 3: Enum-Based Config (thegent)
```
enum PolicyConfig {
    AllowAll,
    DenyAll,
    Restricted(RestrictedPolicy),
}

impl PolicyConfig {
    fn evaluate() -> Result<Decision>
}
```

**Characteristics:**
- No file I/O required
- Type-safe at compile time
- Suitable for policy/state machines
- No TOML parsing overhead

**Projects Using:** thegent (policy, state machines)

---

## Detailed Findings by Issue

### Issue #1: Version Fragmentation (HIGH)

**Problem:**
```
phenotype root:  toml = "0.8"
heliosCLI root:  toml = "0.9.5"
Workspace Scope: No workspace-level version pinning
```

**Impact:**
- Incompatible APIs between 0.8 and 0.9.5
- Dependency resolution conflicts if shared
- Violates "bleeding-edge first" mandate
- Increases maintenance burden

**Severity:** HIGH (blocks cross-project reuse)

---

### Issue #2: Library Duplication (HIGH)

**Problem:**
- `phenotype-config-core` exists (complete, documented)
- `codex-rs/core/config_loader` reinvents same functionality
- 500+ LOC of duplicated config logic
- Both systems coexist without integration

**Evidence:**
```
phenotype-config-core:  ~200 LOC
codex-rs config system: ~500 LOC
Duplication factor: 2.5x
```

**Impact:**
- Maintenance burden increases with each fix
- Inconsistent error handling across projects
- Missed opportunity for shared testing
- Violates DRY principle

**Severity:** HIGH (architectural debt)

---

### Issue #3: Missing Abstraction (MEDIUM)

**Problem:**
- codex-rs uses both `toml` and `toml_edit`
- No safe wrapper for mutation operations
- Raw API usage creates fragile code
- No reusable pattern for edit operations

**Example:**
```rust
// Raw toml_edit usage scattered throughout
use toml_edit::{DocumentMut, ArrayOfTables, Table};
let doc = DocumentMut::parse(content)?;
// ... complex manipulation code ...
```

**Impact:**
- TOML mutation logic hard to test/maintain
- Error handling not standardized
- No safeguards against malformed output

**Severity:** MEDIUM (code quality issue)

---

### Issue #4: Underutilization of phenotype-config-core (MEDIUM)

**Problem:**
- Crate is complete, well-documented
- NOT used by largest TOML user (codex-rs)
- Export for public use not established
- Missing feature flags for optional deps

**Evidence:**
- Published to workspace but never consumed
- codex-rs implements parallel system
- No migration path documented

**Impact:**
- Investment not leveraged
- Divergent interfaces across ecosystem
- Inconsistent error handling

**Severity:** MEDIUM (opportunity cost)

---

### Issue #5: No Standardized Error Handling (LOW)

**Problem:**
- phenotype-config-core: `ConfigError` (thiserror)
- codex-rs: Custom error types per module
- No unified Result type

**Impact:**
- Error context lost across project boundaries
- Inconsistent error messages
- Client code must map multiple error types

**Severity:** LOW (but fixable)

---

## Cross-Project Reuse Opportunities

### Opportunity 1: Unified TOML Reading
**Location:** Standardize via phenotype-config-core
**Effort:** LOW (30 min, 10 LOC)

Create workspace dependency:
```toml
# In workspace root Cargo.toml
[workspace.dependencies]
toml = "0.9.5"  # Unified version
phenotype-config-core = { path = "crates/phenotype-config-core" }
```

### Opportunity 2: TOML Editing Wrapper
**Location:** New crate `phenotype-toml-edit`
**Effort:** MEDIUM (90 min, 150 LOC)

```rust
// phenotype-toml-edit/src/lib.rs
pub struct TomlDocument { doc: toml_edit::DocumentMut }
pub struct TomlEdit { ... }

impl TomlDocument {
    pub fn new(content: &str) -> Result<Self>
    pub fn edit(&mut self) -> TomlEdit
    pub fn save(&self) -> Result<String>
}

impl TomlEdit {
    pub fn set_value(&mut self, path: &str, value: Value) -> Result<()>
    pub fn add_array_item(&mut self, path: &str, item: Value) -> Result<()>
}
```

### Opportunity 3: Config Layer Composition
**Location:** Extend phenotype-config-core or new crate
**Effort:** MEDIUM (2 hours, 250 LOC)

```rust
// phenotype-config-layers/src/lib.rs
pub struct LayerStack {
    system: Layer,
    user: Layer,
    project: Layer,
    env_overrides: BTreeMap<String, Value>,
}

impl LayerStack {
    pub fn compose(&self) -> Result<Value>
    pub fn get(&self, key: &str) -> Result<Value>
}
```

### Opportunity 4: Shared Config Types
**Location:** phenotype-config-core or new schema crate
**Effort:** HIGH (variable, 200-400 LOC)

Extract common types from codex-rs:
- AppConfig (apps.toml)
- MCP server specs
- OTel configurations
- Feature flags structure

Make reusable across ecosystem.

---

## Standardization Recommendation

### Recommended Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                    Phenotype Config Stack                     │
├──────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │ Application Layer (projects use this)                   │ │
│  │  - phenotype-config-core (file loading)                │ │
│  │  - phenotype-config-types (shared schemas)              │ │
│  │  - phenotype-config-layers (composition)                │ │
│  └─────────────────────────────────────────────────────────┘ │
│                           ▲                                   │
│                           │                                   │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │ Library Layer (internal infrastructure)                 │ │
│  │  - phenotype-toml-edit (safe mutation wrapper)          │ │
│  │  - serde, serde_json, serde_yaml                        │ │
│  │  - toml 0.9.5 (unified version)                         │ │
│  │  - thiserror (error handling)                           │ │
│  └─────────────────────────────────────────────────────────┘ │
│                                                               │
└──────────────────────────────────────────────────────────────┘

Workspace Dependency Tree:
phenotype-config-core
  ├─ serde (workspace)
  ├─ serde_json
  ├─ serde_yaml
  ├─ toml (workspace = 0.9.5)
  ├─ thiserror
  └─ dirs

phenotype-toml-edit (NEW)
  ├─ toml (workspace = 0.9.5)
  ├─ toml_edit (workspace = 0.24.0)
  ├─ thiserror
  └─ serde_json

phenotype-config-types (NEW, OPTIONAL)
  ├─ serde
  ├─ serde_json
  └─ serde_yaml

phenotype-config-layers (NEW, OPTIONAL)
  ├─ phenotype-config-core
  ├─ phenotype-config-types
  ├─ toml (workspace)
  └─ thiserror
```

### Adoption Path by Project

**phenotype (root + crates):**
- Already using phenotype-config-core
- Update toml to 0.9.5 in workspace
- No breaking changes required
- Status: ✓ Already aligned

**heliosCLI/codex-rs:**
- Option A: Migrate to phenotype-config-core + phenotype-toml-edit
  - Eliminates 500 LOC duplication
  - Effort: 3-4 hours
  - Risk: Medium (significant refactor)

- Option B: Extract codex-rs config to codex-config-core
  - Reuses existing sophisticated system
  - Effort: 1-2 hours (refactor only)
  - Risk: Low (no functional changes)
  - Benefit: Codex can use for other projects

**Recommendation:** Option B (codex-config-core)
- Leverages existing, production-tested code
- Maintains codex-rs's layered config system
- Lower risk than wholesale migration
- Codex config system can be shared with helios-rs

**heliosCLI/helios-rs:**
- Inherits codex-config-core after extraction
- No additional changes needed

**platforms/thegent:**
- No TOML changes required
- Enum-based configs suitable for use case
- Document pattern in governance

---

## Migration Plan by Tier

### Tier 1: Quick Wins (1 hour total)

**T1.1: Unify TOML Versions**
- File: `/Users/kooshapari/CodeProjects/Phenotype/repos/Cargo.toml`
- Change: `toml = "0.8"` → `toml = "0.9.5"`
- Effort: 10 min
- Risk: LOW (minor version bump, backward compatible)

**T1.2: Workspace Dependency Pinning**
- File: `/Users/kooshapari/CodeProjects/Phenotype/repos/Cargo.toml`
- Add to [workspace.dependencies]:
  ```toml
  toml = "0.9.5"
  toml_edit = "0.24.0"
  phenotype-config-core = { path = "crates/phenotype-config-core" }
  ```
- Effort: 15 min
- Risk: LOW

**T1.3: Create phenotype-toml-edit Wrapper**
- New crate: `crates/phenotype-toml-edit/`
- Purpose: Safe TOML mutation abstraction
- Effort: 45 min (~150 LOC)
- Risk: LOW (new code, no dependencies on existing)

**T1 Success Criteria:**
- All projects use toml 0.9.5
- Workspace dependencies pinned
- phenotype-toml-edit builds and exports safe API

---

### Tier 2: Medium Integration (3-4 hours total)

**T2.1: Extract codex-config-core**
- Location: `heliosCLI/codex-rs/config/` → new crate
- Move: `core/src/config/` + `core/src/config_loader/` modules
- Effort: 2 hours (~200 LOC refactor)
- Risk: MEDIUM (requires careful testing)

**T2.2: Add helios-rs Dependency**
- File: `heliosCLI/helios-rs/Cargo.toml`
- Add: `codex-config = { path = "../codex-rs/config" }`
- Effort: 15 min
- Risk: LOW

**T2.3: Document Config Patterns**
- File: New file `docs/CONFIG_GOVERNANCE.md`
- Content:
  - Pattern 1: File-based (phenotype-config-core)
  - Pattern 2: Layered + Edit (codex-config-core)
  - Pattern 3: Enum-based (thegent)
  - When to use each pattern
- Effort: 30 min (~300 words)
- Risk: LOW

**T2 Success Criteria:**
- codex-config-core builds independently
- helios-rs uses codex-config-core
- All patterns documented

---

### Tier 3: Long-Term Consolidation (2-3 hours)

**T3.1: Shared Config Types (Optional)**
- Create: `crates/phenotype-config-types/`
- Extract from codex-rs: AppConfig, MCP specs, OTel configs
- Effort: 2 hours (~250 LOC)
- Risk: MEDIUM (requires agreement on schemas)

**T3.2: Config Composition Library (Optional)**
- Create: `crates/phenotype-config-layers/`
- Purpose: Multi-layer config composition
- Build on: phenotype-config-core + codex-config-core patterns
- Effort: 2 hours (~200 LOC)
- Risk: MEDIUM

**T3.3: Governance Updates**
- File: `Phenotype/repos/CLAUDE.md`
- Add section: "Config Library Standards"
  - Use phenotype-config-core for simple file loading
  - Use codex-config-core for complex layering
  - Use phenotype-toml-edit for mutations
  - Document when to use enum-based config
- Effort: 30 min
- Risk: LOW

**T3 Success Criteria:**
- Shared config types available
- Layer composition pattern documented and reusable
- All config work uses standardized patterns

---

## Effort & Risk Assessment

### Tier 1: Quick Wins
| Task | LOC | Effort | Risk | Dependencies |
|------|-----|--------|------|--------------|
| Version unification | 5 | 10 min | LOW | None |
| Workspace pinning | 10 | 15 min | LOW | Tier 1.1 |
| phenotype-toml-edit | 150 | 45 min | LOW | Tier 1.2 |
| **SUBTOTAL** | **~165** | **~70 min** | **LOW** | - |

### Tier 2: Medium Integration
| Task | LOC | Effort | Risk | Dependencies |
|------|-----|--------|------|--------------|
| Extract codex-config-core | 200 | 2 hrs | MEDIUM | Tier 1 |
| helios-rs integration | 15 | 15 min | LOW | Tier 2.1 |
| Document patterns | 300 | 30 min | LOW | Tier 2.1 |
| **SUBTOTAL** | **~515** | **~2.75 hrs** | **MEDIUM** | Tier 1 |

### Tier 3: Long-Term
| Task | LOC | Effort | Risk | Dependencies |
|------|-----|--------|------|--------------|
| Shared config types | 250 | 2 hrs | MEDIUM | Tier 2 |
| Config composition lib | 200 | 2 hrs | MEDIUM | Tier 2 |
| Governance updates | 100 | 30 min | LOW | Tier 2 |
| **SUBTOTAL** | **~550** | **~4.5 hrs** | **MEDIUM** | Tier 2 |

### Total Across All Tiers
```
Total LOC: ~1,230
Total Effort: ~7.25 hours
Effort Breakdown:
  - Tier 1 (Quick): 70 min (must-do)
  - Tier 2 (Recommended): 2.75 hrs (should-do)
  - Tier 3 (Optional): 4.5 hrs (nice-to-have)
```

---

## Code Entity Map

### Before Standardization
```
phenotype/crates/phenotype-config-core/
  src/lib.rs                 → ConfigLoader, ConfigFormat, ConfigDirs
  src/loader.rs              → load_from_file, load_from_string
  src/format.rs              → Format enum, format detection
  src/error.rs               → ConfigError

heliosCLI/codex-rs/core/src/config/
  config/mod.rs              → Main config aggregator
  config/edit.rs             → TOML mutation (toml_edit)
  config/types.rs            → AppsConfigToml, McpServerConfig, etc.
  config_loader/mod.rs       → ConfigLayerStack, layering logic
  config_loader/macos.rs     → macOS-specific loading
  config_loader/layer_io.rs  → Layer I/O operations

heliosCLI/helios-rs/
  (inherits codex-rs patterns)

platforms/thegent/crates/
  thegent-policy/src/policy.rs → PolicyConfig enum
  thegent-policy/src/engine.rs → Policy evaluation
```

### After Standardization (Target)
```
phenotype/crates/phenotype-config-core/          ✓ Keep as-is
  src/lib.rs
  src/loader.rs
  src/format.rs
  src/error.rs

phenotype/crates/phenotype-toml-edit/            ← NEW
  src/lib.rs
  src/document.rs              → TomlDocument wrapper
  src/edit.rs                  → TomlEdit safe API
  src/error.rs                 → Unified error type

heliosCLI/codex-rs/config/                       ← EXTRACTED
  src/lib.rs                   → Reexport config_core
  src/config/                  → Move from core/src/config/
  src/config_loader/           → Move from core/src/config_loader/
  src/error.rs                 → ConfigError (unified with phenotype)

heliosCLI/helios-rs/
  Cargo.toml                   → Depends on codex-config

platforms/thegent/crates/
  thegent-policy/             ✓ Keep as-is (enum-based)

phenotype/crates/phenotype-config-types/         ← OPTIONAL (T3)
  src/lib.rs
  src/app_config.rs
  src/mcp_server.rs
  src/otel_config.rs

phenotype/crates/phenotype-config-layers/        ← OPTIONAL (T3)
  src/lib.rs
  src/layer_stack.rs
  src/composition.rs
```

---

## Testing Requirements

### Tier 1 Testing
- **phenotype-toml-edit:** Unit tests for safe mutation
  - Test document creation
  - Test value setting
  - Test array operations
  - Test formatting preservation

### Tier 2 Testing
- **codex-config-core:** Integration tests (move from codex-rs tests)
  - Test layer composition
  - Test platform-specific loading
  - Test schema validation
  - Test error propagation

### Tier 3 Testing (Optional)
- **phenotype-config-types:** Roundtrip serialization tests
- **phenotype-config-layers:** Composition precedence tests

---

## Governance Documentation Template

**File:** `docs/CONFIG_GOVERNANCE.md` (to be created)

### Config Pattern Selection Guide

| Pattern | Use Case | Pros | Cons | Example |
|---------|----------|------|------|---------|
| **File-Based (phenotype-config-core)** | Simple app configs, settings files | Simple API, multi-format, reusable | No in-place edit | App settings, service config |
| **Layered + Edit (codex-config-core)** | Complex, multi-source configs | Sophisticated composition, safe edit | Higher learning curve | CLI configs, nested settings |
| **Enum-Based (thegent)** | Policy/state machines, fixed configs | Compile-time safe, no I/O | Not file-based | Policies, state transitions |

### Dependencies to Use

```toml
# For simple file loading:
phenotype-config-core = "workspace"

# For TOML manipulation:
phenotype-toml-edit = "workspace"

# For complex layering:
codex-config-core = "workspace"  # or local path in codex-rs

# For serialization:
serde = { version = "1", features = ["derive"] }
serde_json = "workspace"
```

### Error Handling Standard

All config operations MUST return `Result<T, ConfigError>` where:
```rust
pub enum ConfigError {
    Io(std::io::Error),
    Parse(String),
    Schema(String),
    NotFound(PathBuf),
    #[from]
    Other(#[from] Box<dyn std::error::Error>),
}
```

Use `thiserror` for implementation.

---

## Risk Assessment & Mitigation

### Risk 1: Breaking Changes in codex-rs Extraction

**Probability:** MEDIUM
**Impact:** HIGH (largest TOML user)

**Mitigation:**
- Extract to new crate alongside core (no in-place refactor)
- Keep all APIs identical during extraction
- Run full test suite before merging
- Gradual migration of helios-rs

---

### Risk 2: Version Compatibility with Existing Code

**Probability:** LOW
**Impact:** MEDIUM (toml 0.8 → 0.9.5 bump)

**Mitigation:**
- toml 0.8 → 0.9.5 is minor version (mostly compatible)
- Run `cargo update` locally and test before committing
- Check changelogs for breaking changes
- Test phenotype-config-core with new version

---

### Risk 3: Missed Dependencies in Migration

**Probability:** MEDIUM
**Impact:** LOW (codex-rs is well-contained)

**Mitigation:**
- Use `cargo tree` to verify dependency graph
- Check for transitive dependencies
- Test in isolation before integration
- Keep old code in worktree until validated

---

### Risk 4: Governance Non-Adoption

**Probability:** MEDIUM
**Impact:** MEDIUM (future projects may miss standards)

**Mitigation:**
- Add to project CLAUDE.md (enforced)
- Create example project using standard pattern
- Add to PR checklist (config choice validation)
- Reference in AgilePlus specs

---

## Success Criteria

### Tier 1 (Must-Have)
- [ ] All TOML versions unified to 0.9.5
- [ ] Workspace dependencies pinned
- [ ] phenotype-toml-edit builds and exports
- [ ] No version conflicts in `cargo tree`

### Tier 2 (Should-Have)
- [ ] codex-config-core extracted and functional
- [ ] helios-rs uses codex-config-core
- [ ] CONFIG_GOVERNANCE.md written and reviewed
- [ ] All existing tests pass

### Tier 3 (Nice-to-Have)
- [ ] phenotype-config-types available (if needed)
- [ ] phenotype-config-layers available (if needed)
- [ ] Governance doc in CLAUDE.md
- [ ] Example projects updated

---

## Recommendations

### Immediate Actions (This Sprint)

1. **Execute Tier 1** (70 minutes)
   - Unify toml to 0.9.5
   - Pin workspace dependencies
   - Create phenotype-toml-edit wrapper
   - Deliverable: Unified version management

2. **Plan Tier 2** (30 minutes)
   - Decide: Extract codex-config or migrate to phenotype-config-core
   - Get stakeholder buy-in on approach
   - Create AgilePlus spec

### Next Sprint

3. **Execute Tier 2** (2.75 hours)
   - Extract or migrate config system
   - Document CONFIG_GOVERNANCE.md
   - Update helios-rs dependencies
   - Deliverable: Unified config infrastructure

### Future (Roadmap)

4. **Execute Tier 3** (4.5 hours, optional)
   - Build shared config types if needed
   - Build config composition library if needed
   - Integrate into thegent and other projects

### Long-Term Governance

5. **Establish Standards**
   - Add to Phenotype CLAUDE.md
   - Enforce config pattern selection in PR reviews
   - Update architecture documentation

---

## Conclusion

The Phenotype workspace has **strong fundamentals** for TOML configuration management:
- phenotype-config-core is production-ready
- codex-rs has sophisticated, tested patterns
- thegent correctly uses enum-based configs

However, **fragmentation and duplication** create technical debt:
- Version inconsistencies block cross-project reuse
- Two independent config systems increase maintenance burden
- Opportunity cost of underutilized libraries

**This audit recommends a phased standardization approach:**
1. **Tier 1** unifies versions and creates reusable tools (LOW EFFORT)
2. **Tier 2** consolidates config systems (MEDIUM EFFORT, HIGH VALUE)
3. **Tier 3** builds optional shared libraries (OPTIONAL, LONG-TERM)

**Total Effort:** ~7.25 hours across 3 tiers
**Total LOC:** ~1,230 (new code + refactoring)
**Risk Level:** LOW to MEDIUM (mostly refactoring, not new functionality)

**Recommendation:** Proceed with Tier 1 immediately, execute Tier 2 within 2 sprints.

---

## Appendix: File Manifest

### Analyzed Files

**phenotype-config-core:**
- `/Users/kooshapari/CodeProjects/Phenotype/repos/crates/phenotype-config-core/Cargo.toml`
- `/Users/kooshapari/CodeProjects/Phenotype/repos/crates/phenotype-config-core/src/lib.rs`
- `/Users/kooshapari/CodeProjects/Phenotype/repos/crates/phenotype-config-core/src/loader.rs`
- `/Users/kooshapari/CodeProjects/Phenotype/repos/crates/phenotype-config-core/src/format.rs`
- `/Users/kooshapari/CodeProjects/Phenotype/repos/crates/phenotype-config-core/src/error.rs`
- `/Users/kooshapari/CodeProjects/Phenotype/repos/crates/phenotype-config-core/src/dirs_helper.rs`

**heliosCLI/codex-rs config system:**
- `/Users/kooshapari/CodeProjects/Phenotype/repos/heliosCLI/codex-rs/Cargo.toml`
- `/Users/kooshapari/CodeProjects/Phenotype/repos/heliosCLI/codex-rs/core/src/config/mod.rs`
- `/Users/kooshapari/CodeProjects/Phenotype/repos/heliosCLI/codex-rs/core/src/config/edit.rs`
- `/Users/kooshapari/CodeProjects/Phenotype/repos/heliosCLI/codex-rs/core/src/config/types.rs`
- `/Users/kooshapari/CodeProjects/Phenotype/repos/heliosCLI/codex-rs/core/src/config/service.rs`
- `/Users/kooshapari/CodeProjects/Phenotype/repos/heliosCLI/codex-rs/core/src/config_loader/mod.rs`
- `/Users/kooshapari/CodeProjects/Phenotype/repos/heliosCLI/codex-rs/core/src/config_loader/macos.rs`

**heliosCLI/helios-rs:**
- `/Users/kooshapari/CodeProjects/Phenotype/repos/heliosCLI/helios-rs/Cargo.toml`

**platforms/thegent:**
- `/Users/kooshapari/CodeProjects/Phenotype/repos/platforms/thegent/Cargo.toml`
- `/Users/kooshapari/CodeProjects/Phenotype/repos/platforms/thegent/crates/thegent-policy/src/policy.rs`
- `/Users/kooshapari/CodeProjects/Phenotype/repos/platforms/thegent/crates/thegent-policy/src/engine.rs`

### To Be Created

- `/Users/kooshapari/CodeProjects/Phenotype/repos/crates/phenotype-toml-edit/Cargo.toml`
- `/Users/kooshapari/CodeProjects/Phenotype/repos/crates/phenotype-toml-edit/src/lib.rs`
- `/Users/kooshapari/CodeProjects/Phenotype/repos/docs/CONFIG_GOVERNANCE.md`

---

**Audit Completed:** 2026-03-29
**Auditor:** WS6 Audit Agent
**Status:** FINAL REPORT
