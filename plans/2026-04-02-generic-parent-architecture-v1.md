# Generic Parent Architecture Plan

**Date**: 2026-04-02
**Status**: COMPLETED ✓

## Vision

Create a **unified polyrepo platform** under `platforms/` that serves as the generic parent for all product lines (phenotype, thegent, future platforms). This anticipates cross-platform convergence before it happens.

---

## Target Architecture

```
platforms/                                    ← GENERIC PARENT WORKSPACE
├── Cargo.toml                                ← Workspace root with shared deps
├── phenotype/   (symlink)                   ← phenotype-infrakit crates
│   └── crates/phenotype-*/
├── thegent/                                  ← Agent orchestration platform
│   ├── pyproject.toml
│   └── crates/thegent-*/
└── crates/                                   ← Cross-platform shared crates
    ├── platform-error-core/
    ├── platform-logging/
    └── platform-telemetry/
```

### Key Principles

1. **Platform Workspace** owns all product line workspaces
2. **Cross-platform crates** go in `platforms/crates/`
3. **Product-specific crates** stay in their respective directories
4. **Symlinks** for `phenotype/` to avoid duplication

---

## Current State

| Directory | Type | Role |
|-----------|------|------|
| `repos/` | Git repo | phenotype-infrakit (main workspace) |
| `repos/phenotype-infrakit/` | Submodule | Duplicate stub crates |
| `platforms/thegent/` | Git repo | Agent platform |
| `platforms/` | Directory | Parent container (no workspace) |

---

## Execution Steps

### Step 1: Create Generic Parent Workspace

Create `platforms/Cargo.toml` with:
- Workspace definition for all product lines
- Shared dependencies
- Version management

### Step 2: Symlink phenotype/ into platforms/

```bash
ln -s ../../ phenotype  # from platforms/ directory
```

### Step 3: Archive phenotype-infrakit duplicates

Archive 7 stub crates that duplicate root implementations:
- phenotype-error-core
- phenotype-casbin-wrapper  
- phenotype-config-core
- phenotype-config-loader
- phenotype-cost-core
- phenotype-git-core
- phenotype-capital

### Step 4: Add Cross-Platform Crate Pattern

Create `platforms/crates/platform-commons/` as a template for future cross-platform crates.

### Step 5: Verify Build

```bash
cargo build --manifest-path platforms/Cargo.toml --workspace
```

---

## Verification Checklist
## Verification Checklist:

- [x] `platforms/Cargo.toml` created with workspace definition
- [x] `repos/crates/phenotype-*` symlinked into `platforms/crates/`
- [x] phenotype-infrakit duplicates archived (completed 2026-04-02)
- [ ] platforms/thegent/ included in workspace (future - when cross-platform deps emerge)
- [x] Cross-platform crate template created (`platform-commons`)
- [x] Workspace builds successfully
- [x] PRs created for each change

## PRs Created:

| # | Repo | Title | Status |
|---|------|-------|--------|
| 553 | phenotype-infrakit | feat(platforms): add generic parent workspace + test-infra | OPEN |
| 281 | AgilePlus | fix(fixtures): add missing timestamps to WorkPackage builder | OPEN |

## Completed Actions (2026-04-02):

1. **phenotype-infrakit cleanup**:
   - Archived 7 duplicate stub crates to `crates/.archive/`
   - Workspace reduced to single member: `phenotype-sentry-config`
   - Builds successfully

2. **Generic parent workspace created**:
   - `platforms/Cargo.toml` workspace root
   - `platforms/crates/platform-commons/` cross-platform crate
   - 19 phenotype-* crates symlinked
   - Builds successfully

3. **AgilePlus fixtures fix**:
   - Added missing timestamps to WorkPackage builder
   - PR #281 created

4. **Stub consolidation (2026-04-02)**:
   - Consolidated 18 phenotype-* stubs into `.archive/phenotype-stubs/`
   - Archived crates: phenotype-async-traits, phenotype-capital, phenotype-casbin-wrapper, phenotype-config-core, phenotype-config-loader, phenotype-contract, phenotype-cost-core, phenotype-crypto, phenotype-error-core, phenotype-error-macros, phenotype-event-sourcing, phenotype-git-core, phenotype-http-client-core, phenotype-macros, phenotype-mcp, phenotype-ports-canonical, phenotype-process, phenotype-test-infra
   - Updated projects/INDEX.md with consolidated archive
   - Cleaned up duplicate archive directories

## Future Work:

- [x] Archive phenotype-infrakit stub crates (consolidated to repos/.archive/phenotype-stubs/ - 18 crates)
- [ ] Add thegent-* crates when cross-platform dependencies emerge
- [ ] Consider adding phenotype-infrakit to platforms/ workspace
- [x] Create projects/INDEX.md for shelf catalog
- [ ] Clean up vibeproxy-monitoring archive (embedded git repo)
