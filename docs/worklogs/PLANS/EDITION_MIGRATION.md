# Edition Migration Plan

**Created:** 2026-03-29
**Status:** Planned
**Priority:** P0 (Critical)

## Problem Statement

The `libs/` directory contains 16 crates with `edition = "2021"`, but the main workspace uses `edition = "2024"`. This prevents integration of well-designed, hexagonal-compliant libraries into the main codebase.

**Key Finding:** `hexagonal-rs` is already `edition = "2024"` but is NOT imported anywhere in the codebase despite having the exact patterns needed.

## Edition Inventory

| Library | Edition | Workspace Member | In Use |
|---------|---------|------------------|--------|
| hexagonal-rs | 2024 | Yes | **NO** |
| config-core | 2021 | Yes | **NO** |
| logger | 2021 | Yes | **NO** |
| tracing | 2021 | Yes | **NO** |
| metrics | 2021 | Yes | **NO** |
| hexkit | 2021 | Yes | **NO** |
| cipher | 2021 | Yes | Partial |
| gauge | 2021 | Yes | Partial |
| nexus | 2021 | Yes | Partial |
| xdd-lib-rs | 2021 | Yes | Partial |
| cli-framework | 2021 | Yes | Partial |

## Migration Options

### Option A: Migrate All to 2024 (Recommended)

**Approach:** Update `edition = "2021"` → `"2024"` in all 15 affected libs.

**Pros:**
- Clean solution, no workarounds
- Unifies all code under one edition
- Enables first-class dependency management

**Cons:**
- Must verify all dependencies support 2024
- Potential compilation issues with older deps
- 15 Cargo.toml files to update

**Steps:**
1. Audit all deps for 2024 compatibility
2. Create migration branch: `libs/edition-migration-2024`
3. Update each Cargo.toml
4. Run `cargo check` for each crate
5. Run full workspace build
6. Create PR

### Option B: Conditional Compilation with Feature Flags

**Approach:** Keep 2021 edition, use feature flags to conditionally enable integration.

**Pros:**
- Minimal code changes
- Can maintain backward compatibility

**Cons:**
- Complexity, two code paths
- Not idiomatic Rust

**Not recommended.**

### Option C: Fork as agileplus-* Crates

**Approach:** Copy needed libs into `crates/agileplus-*` with 2024 edition.

**Pros:**
- Complete control
- No external dependencies

**Cons:**
- Duplicates code
- Maintenance burden
- Loses shared versioning

**Not recommended - defeats purpose of libification.**

## Implementation Plan (Option A)

### Phase 1: Dependency Audit (1-2 hours)

```bash
# For each lib, check dependency compatibility
cd libs/config-core
cargo tree -e normal  # Check all deps
```

**Known compatible deps:**
- `anyhow`, `serde`, `serde_json`, `toml` - all 2024 ready
- `thiserror` - has 2024 edition support

**Potential issues:**
- `tokio` - need to verify 2024 support (likely fine)
- `async-trait` - need to verify 2024 support (likely fine)

### Phase 2: Batch Update (2-3 hours)

```bash
# Script to update all editions
for dir in libs/*/; do
  if grep -q 'edition = "2021"' "$dir/Cargo.toml"; then
    sed -i '' 's/edition = "2021"/edition = "2024"/g' "$dir/Cargo.toml"
    echo "Updated: $dir"
  fi
done
```

### Phase 3: Verification (3-4 hours)

```bash
# Check each lib individually
cargo check -p config-core
cargo check -p logger
cargo check -p tracing
# ... etc

# Check full workspace
cargo check --workspace
```

### Phase 4: Integration (4-8 hours)

Once editions match, integrate the libs:

1. **hexagonal-rs integration:**
   - Replace duplicated `EventBus` trait with `hexagonal_rs::ports::EventBus`
   - Replace duplicated `Repository` trait with `hexagonal_rs::ports::Repository`
   - Remove duplicate trait definitions from crates

2. **config-core integration:**
   - Replace `agileplus-domain/src/config/loader.rs` with `config_core::ConfigLoader`
   - Replace `agileplus-telemetry/src/config.rs` with `config_core::ConfigLoader<Yaml>`
   - Replace `vibe-kanban/backend/src/models/config.rs` with `config_core::ConfigLoader<Json>`

## Effort Estimate

| Phase | Effort | Risk |
|-------|--------|------|
| Dependency Audit | 2 hours | Low |
| Batch Update | 2 hours | Low |
| Verification | 4 hours | Medium |
| Integration | 8 hours | Medium |
| **Total** | **16 hours** | — |

## Action Items

- [ ] 🔴 **CRITICAL** Run `cargo tree -e normal` on each 2021 lib to audit deps
- [ ] 🔴 **CRITICAL** Create branch: `libs/edition-migration-2024`
- [ ] 🟡 **HIGH** Batch update all 15 libs to `edition = "2024"`
- [ ] 🟡 **HIGH** Run `cargo check` per lib, fix any issues
- [ ] 🟠 **MEDIUM** Run full workspace build
- [ ] 🟠 **MEDIUM** Integrate hexagonal-rs traits
- [ ] 🟠 **MEDIUM** Integrate config-core
- [ ] 🟢 **LOW** Document migration in worklogs/ARCHITECTURE.md

## Related

- worklogs/ARCHITECTURE.md (libs/ Directory Analysis)
- worklogs/DUPLICATION.md (Duplicated traits that need hexagonal-rs)
