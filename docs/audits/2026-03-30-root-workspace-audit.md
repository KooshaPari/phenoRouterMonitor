# Root Workspace (Cargo.toml) Audit — 2026-03-30

## Executive Summary

**Status:** ⚠️ BROKEN — Compilation fails immediately

**Issue:** `phenotype-test-infra` → `phenotype-error-core::CoreError` unresolved import

**Workspace:** 27 total crates
- 13 active members (included in workspace)
- 14 excluded members (2 missing, 12 existing)

## Configuration

**File:** `/Users/kooshapari/CodeProjects/Phenotype/repos/Cargo.toml`

**Package Metadata:**
```toml
[workspace.package]
version = "0.2.0"
edition = "2021"
rust-version = "1.75"
license = "MIT"
```

## Active Members (13)

✅ All directories exist:
phenotype-cache-adapter, phenotype-contracts, phenotype-error-core, phenotype-errors, phenotype-event-sourcing, phenotype-git-core, phenotype-health, phenotype-port-traits, phenotype-policy-engine, phenotype-state-machine, phenotype-telemetry, phenotype-test-infra, phenotype-async-traits

## Compilation Error (CRITICAL)

```
error[E0432]: unresolved import `phenotype_error_core::CoreError`
  --> crates/phenotype-test-infra/src/lib.rs:25:9
```

**Root Cause:** phenotype-error-core does not export CoreError publicly.

**Fix:** Add to `crates/phenotype-error-core/src/lib.rs`:
```rust
pub use crate::core::CoreError;
```

**Effort:** <5 minutes
**Blocks:** All cargo check/build/test commands

## Excluded Crates Issues (14)

| Crate | Status | Issue |
|-------|--------|-------|
| agileplus-api-types | ❌ Missing | Declared but no directory |
| agileplus-domain | ❌ Missing | Declared but no directory |
| phenotype-git-core | ⚠️ Conflict | In BOTH members AND exclude |
| 11 others | ✅ Exist | Why excluded? (phenotype-crypto, logging, mcp, process, etc.) |

### Decisions Needed

For each of 14 excluded crates: Include in members, Archive to .archive/, or Remove from Cargo.toml?

## Workspace Dependencies

✅ 29 packages declared in [workspace.dependencies]
✅ All bleeding-edge versions (tokio 1, serde 1.0, thiserror 2.0, gix 0.81, etc.)

⚠️ Missing: `once_cell` (used by some crates but not declared)

## Recommended Actions (Priority Order)

### 1. CRITICAL: Fix CoreError Export
Edit `crates/phenotype-error-core/src/lib.rs` → Add public export
Run `cargo check` to verify

### 2. HIGH: Remove Missing Excluded Crates
Remove from Cargo.toml exclude:
- "crates/agileplus-api-types"
- "crates/agileplus-domain"

### 3. HIGH: Remove phenotype-git-core from Exclude
Already in members list; remove from exclude list.

### 4. MEDIUM: Clarify Excluded Crates Strategy
Document decision for each of 14 excluded crates.

### 5. LOW: Remove Repository URL Duplication
Keep in [workspace.package], remove from [workspace].

## Summary

| Metric | Count |
|--------|-------|
| Total Crates | 27 |
| Active Members | 13 |
| Excluded Crates | 14 |
| Missing (Excluded) | 2 |
| Compilation Errors | 1 (CoreError) |
| Workspace Dependencies | 29 |
| Builds Passing | ❌ No (blocked) |

## Effort Estimate

- Fix CoreError export: <5 minutes
- Remove missing crates: <2 minutes
- Resolve git-core conflict: <1 minute
- Clarify excluded crates: 1-2 hours
- Add missing workspace.dependencies: <5 minutes

**Total to Unblock Compilation:** <15 minutes

---

**Auditor:** Claude Code | **Date:** 2026-03-30
