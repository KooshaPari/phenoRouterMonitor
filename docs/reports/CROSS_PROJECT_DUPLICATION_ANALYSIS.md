# Cross-Project Duplication & Fork Strategy Analysis

**Generated:** 2026-03-29
**Priority:** P0-P1
**Status:** in_progress

---

## Executive Summary

This document consolidates findings from subagent analysis on:
1. Duplicate code patterns across projects (LOC reduction opportunities)
2. External fork/wrap decisions (third-party integration strategy)
3. Inactive/non-canonical folder audit
4. Whitebox vs blackbox usage patterns

### Key Metrics

| Metric | Value |
|--------|-------|
| Cross-project duplication | ~2,100 LOC |
| Savings potential (consolidation) | ~800 LOC |
| Inactive folders identified | 6 |
| Fork candidates | 4 |
| Wrap candidates | 8 |

---

## Part 1: Cross-Project Duplication Analysis

### 1.1 Error Type Duplication (~400 LOC)

**Finding:** 12+ error type definitions across crates with significant overlap.

| Error Type | Locations | LOC | Duplicated Variants |
|------------|-----------|-----|---------------------|
| `ApiError` | 1 | 14 | NotFound, Internal |
| `DomainError` | 1 | 47 | NotFound, Conflict |
| `SyncError` | 2 (sync, p2p) | 41 | Nats, Serialization |
| `EventSourcingError` | 1 | 46 | Store, Hash |
| `PolicyEngineError` | 1 | 65 | Regex, Config, Load |
| `PortError` | 1 | 51 | NotFound, Validation |
| `GraphError` | 1 | 12 | Store, Query |
| `CacheError` | 1 | 10 | Store, Serialization |

**Variant Overlap:**

| Variant | Appears In | Recommendation |
|---------|-----------|----------------|
| `NotFound(String)` | 5 types | Extract to `StorageError` |
| `Serialization(String)` | 3 types | Extract to `SerializationError` |
| `Storage(String)` | 3 types | Extract to `StorageError` |
| `Internal(String)` | 2 types | Keep domain-specific |

**Action:** Create `libs/error-core/` with canonical error types

---

### 1.2 Config Loading Patterns (~600 LOC)

**Finding:** 4 independent config loaders across projects.

| Loader | Location | Format | LOC |
|--------|----------|--------|-----|
| TOML parser | `policy-engine/loader.rs` | TOML | 238 |
| JSON loader | `event-sourcing/snapshot.rs` | JSON | 92 |
| Builder pattern | `cache/config.rs` | Struct | ~100 |
| YAML parser | `telemetry/config.rs` | YAML | ~200 |

**Duplicated Patterns:**

```rust
// Pattern: File Loading with Error Conversion (appears 3+ times)
pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| Error::LoadError(format!("...{}", e)))?;
    toml::from_str(&content)
        .map_err(|e| Error::SerializationError(format!("...{}", e)))
}
```

**Action:** Migrate `libs/config-core/` to edition 2024 + integrate `figment`

---

### 1.3 Builder Patterns (~400 LOC)

**Finding:** Extensive manual builder patterns in policy-engine.

| Struct | Builder Methods | LOC |
|--------|----------------|-----|
| `Policy` | `with_description`, `add_rule`, `set_enabled` | 45 |
| `Rule` | `with_description` | 32 |
| `EvaluationContext` | `set`, `set_string`, `set_number` | 93 |

**Action:** Add `derive_builder` crate (100M+ downloads)

---

### 1.4 Repository/Store Traits (~300 LOC)

**Finding:** 5 store traits with overlap.

| Trait | Crate | Methods | Overlap |
|-------|-------|---------|---------|
| `EventStore` | event-sourcing | 6 | HIGH |
| `SyncMappingStore` | sync | 4 | MEDIUM |
| `CacheStore` | cache | 5 | HIGH |
| `GraphBackend` | graph | 3 | LOW |
| `SnapshotStore` | event-sourcing | 4 | MEDIUM |

**Action:** Integrate `libs/hexagonal-rs/` (has Repository patterns)

---

### 1.5 Display/AsStr Duplication (~28 LOC)

**Finding:** Identical `as_str()` + `Display` implementations.

```rust
// rule.rs and result.rs - IDENTICAL PATTERN
impl RuleType {
    pub fn as_str(&self) -> &'static str {
        match self { RuleType::Allow => "Allow", ... }
    }
}
impl std::fmt::Display for RuleType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
```

**Action:** Create macro for reusable implementations

---

## Part 2: External Package Fork/Wrap Strategy

### 2.1 Fork Decision Matrix

| Scenario | Decision | Example |
|----------|----------|---------|
| Need significant modifications | **FORK** | `utils/pty` → `phenotype-process` |
| Need features not in original | **FORK+EXTEND** | `error.rs` → `phenotype-error` |
| Need thin phenotype layer | **WRAP** | `git-worktree` wrapper |
| Crate is perfect as-is | **DIRECT USE** | `serde`, `tokio` |
| Internal is better | **KEEP INTERNAL** | `agileplus-events` |

### 2.2 LOC Threshold for Fork/Wrap

| LOC Saved | Threshold | Action |
|-----------|-----------|--------|
| > 500 LOC | Always justify | FORK or WRAP |
| 200-500 LOC | Strong case | Evaluate fork vs wrap |
| 50-200 LOC | Moderate case | Consider adoption |
| < 50 LOC | Low justification | Direct use |

### 2.3 Fork Candidates

| ID | Source | Target | LOC | Priority | Status |
|----|--------|--------|-----|----------|--------|
| FORK-001 | `utils/pty` | `phenotype-process` | ~750 | 🔴 CRITICAL | TODO |
| FORK-002 | `error.rs` pattern | `phenotype-error` | ~400 | 🔴 CRITICAL | TODO |
| FORK-003 | `utils/git` | `phenotype-git` | ~300 | 🟠 MEDIUM | EVALUATE |
| FORK-004 | `utils/config` | `phenotype-config` | ~200 | 🟠 MEDIUM | EVALUATE |

### 2.4 Wrap Candidates (External Packages)

| Package | Downloads | Purpose | Wrapper Name |
|---------|-----------|---------|--------------|
| `eventually` | ~500 stars | Event sourcing | `phenotype-event-sourcing-wrapper` |
| `casbin` | 2k stars | Policy engine | `phenotype-policy-wrapper` |
| `temporal-sdk` | ~500 stars | Workflows | `phenotype-workflow-wrapper` |
| `figment` | 50M+ | Config | Already adopting |
| `derive_builder` | 100M+ | Builders | Already adopting |

### 2.5 Whitebox vs Blackbox Usage

#### Blackbox (Direct - Works Great ✅)

| Crate | Usage | Assessment |
|-------|-------|------------|
| `serde` | Serialize/deserialize | Pure blackbox |
| `tokio` | Async runtime | Pure blackbox |
| `axum` | HTTP framework | Pure blackbox |
| `clap` | CLI parsing | Pure blackbox |
| `tracing` | Observability | Pure blackbox |

#### Graybox (Wrapped/Extended)

| Crate | Wrapper | Purpose |
|-------|---------|---------|
| `git2` | `agileplus-git` | Adds worktree support |
| `git2` | `heliosCLI/utils/git` | Adds cherry-pick, branch ops |
| `git2` | Custom in thegent | Adds worktree management |

#### Whitebox (Forked)

| Crate | Fork Target | Why Forked |
|-------|-------------|------------|
| `gix` | Internal use | Performance, custom features |
| `uv` | Internal use | Fast package management |

---

## Part 3: Inactive/Non-Canonical Folder Audit

### 3.1 Phenotype Parent Level Folders

| Directory | Git State | Stashes | Recommendation |
|-----------|-----------|---------|----------------|
| `phenotype-gauge-temp` | 2f5e69f | 1 stash | **DELETE** - archived, target merged |
| `phenotype-nexus-temp` | 9c34d85 | 1 stash | **DELETE** - archived, target merged |
| `phenotype-shared-temp` | bb9f9c7 | 2 stashes | **ACTIVATE** - has valuable 10 crates |
| `phenotype-go-kit-temp` | 519fb86 | 1 stash | **EVALUATE** - Go patterns, may duplicate |
| `template-commons-temp` | Active | None | **KEEP** - templates are valid |
| `backups/` | N/A | N/A | **ARCHIVE** - cold storage |
| `isolated/` | N/A | N/A | **DELETE** - stale postmerge work |

### 3.2 .worktrees/ Status

| Worktree | Branch | Status | Action |
|----------|--------|--------|--------|
| `gh-pages-deploy` | Unknown | Needs verification | Verify main at origin |
| `phench-fix` | Unknown | Needs verification | Verify main at origin |
| `thegent` | `chore/cleanup-worklogs` | ✅ Clean | Already on main |

### 3.3 Action Items

- [ ] DELETE: `phenotype-gauge-temp` (has stash - extract first)
- [ ] DELETE: `phenotype-nexus-temp` (has stash - extract first)
- [ ] ACTIVATE: `phenotype-shared-temp` (10 crates to integrate)
- [ ] EVALUATE: `phenotype-go-kit-temp` (Go patterns)
- [ ] ARCHIVE: `backups/` to cold storage
- [ ] DELETE: `isolated/` (stale)
- [ ] VERIFY: `.worktrees/gh-pages-deploy` on main at origin
- [ ] VERIFY: `.worktrees/phench-fix` on main at origin

---

## Part 4: LOC Reduction Summary

| Category | Current | Target | Savings | Effort |
|----------|---------|--------|---------|--------|
| Error Types | 400 | 150 | **250** | 2 days |
| Config Loading | 600 | 200 | **400** | 3 days |
| Builder Patterns | 400 | 100 | **300** | 2 days |
| Repository Traits | 300 | 100 | **200** | 2 days |
| Display/AsStr | 28 | 8 | **20** | 0.5 day |
| **TOTAL** | **1,728** | **558** | **1,170** | **9.5 days** |

---

## Cross-Reference

| Document | Location |
|----------|----------|
| Master Duplication Audit | `docs/reports/MASTER_DUPLICATION_AUDIT.md` |
| Decomposition Audit | `docs/reports/DECOMPOSITION_AUDIT.md` |
| Dependencies Analysis | `docs/worklogs/DEPENDENCIES.md` |
| Research Findings | `docs/worklogs/RESEARCH.md` |

---

*Report generated by FORGE (2026-03-29)*
