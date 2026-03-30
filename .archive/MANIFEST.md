# Archive Manifest

**Last updated:** 2026-03-30

## Overview

This directory contains archived specifications, work packages, and related artifacts that have been superseded, migrated, or completed. Items are preserved here for historical reference and auditability rather than being destructively deleted.

All archived items are indexed below with provenance information and restoration instructions if needed.

## Archived Items

| Name | Size | Type | Archive Date | Reason |
|------|------|------|--------------|--------|
| `kitty-specs/` | 16K | Specification Archive | 2026-03-28 | Migrated to AgilePlus format; legacy BMAD refs archived for historical reference |

## Directory Structure

```
.archive/
└── kitty-specs/
    └── phenotype-infrakit-lockfile-repair/
        ├── spec.md                          # Problem statement and acceptance criteria
        ├── plan.md                          # Work package breakdown (1 WP)
        ├── tasks/
        │   └── WP01-initial-implementation.md
        └── contracts/
            └── governance-v1.json
```

## Provenance Notes

### kitty-specs/

**Original Location:**
Legacy specification directory in project root before migration to AgilePlus format.

**Archive Reason:**
These specifications were migrated to the AgilePlus format (spec-driven development methodology) and preserved in `.archive/` for historical auditability. The kitty-specs naming convention predates the current AgilePlus framework.

**Archive Date:**
2026-03-28 (Commit `72f43911d`)

**Related PRs:**
- PR #38: "chore: archive kitty-specs" — Primary archival
- PR #96: "chore(specs): migrate kitty-specs to AgilePlus format, archive BMAD refs" — Spec format migration

### phenotype-infrakit-lockfile-repair

**Specification Summary:**
Work package for adding missing `Cargo.lock` to phenotype-infrakit workspace and enabling locked dependency resolution.

**Status:**
Completed — This specification has been fully implemented and is retained for historical reference.

**Work Package Details:**
- WP01: Initial Implementation — Add Cargo.lock and verify `cargo test --locked` succeeds

**Key Files:**
- `spec.md` — Problem statement and acceptance criteria
- `plan.md` — Work package structure (1 WP, no dependencies)
- `tasks/WP01-initial-implementation.md` — Task breakdown for initial implementation
- `contracts/governance-v1.json` — Governance contract metadata

**Related Issues:**
This work was part of the infrakit lockfile repair initiative to ensure reproducible builds with locked dependencies.

## Restore Instructions

### To restore an archived item:

1. **Identify the item** in the table above.

2. **Copy from archive** to its target location:
   ```bash
   # Example: restore phenotype-infrakit-lockfile-repair spec
   cp -r .archive/kitty-specs/phenotype-infrakit-lockfile-repair \
         docs/specs/phenotype-infrakit-lockfile-repair
   ```

3. **Update references** if the item is a Rust crate:
   - Add to `Cargo.toml` workspace members
   - Update dependency tree in dependent crates
   - Verify with `cargo check --workspace`

4. **Verify integrity**:
   ```bash
   # For spec archives
   ls -la docs/specs/phenotype-infrakit-lockfile-repair/

   # For Rust crates
   cargo check --workspace
   ```

5. **Commit** with restoration context:
   ```bash
   git add <restored-paths>
   git commit -m "restore: restore <item-name> from archive

   Reason: <explain why restoration is needed>

   This item was archived on <date> in commit <sha> but is
   now needed for <purpose>.
   "
   ```

## Historical Context

The `.archive/` directory follows the **Phenotype Long-Term Stability and Non-Destructive Change Protocol**, which mandates:

- **No destructive deletions** — Items are moved to `.archive/` for retention and auditability
- **Preserve git history** — Original commit history is intact
- **Enable restoration** — Any archived item can be restored with clear provenance
- **Maintain references** — This MANIFEST.md serves as a central index

### Why Archive Instead of Delete?

1. **Auditability** — Maintains record of all work, decisions, and implementations
2. **Traceability** — Preserves git history and commit references
3. **Future reference** — Enables pattern analysis and historical learning
4. **Regulatory compliance** — Some organizations require retention of specifications
5. **Restoration path** — Supports undoing archival if needed

## Maintenance

This manifest is automatically maintained by:
- Git commit messages (new archival events are logged)
- Manual updates when new items are archived

To update this manifest:
1. List new archived items in the table
2. Add provenance notes section for each item
3. Commit with message: `docs: update archive manifest`

## Access and Permissions

The `.archive/` directory is **read-only** for normal operations:
- View archived items: Always allowed
- Restore archived items: Use `cp` and `git add <specific-paths>` workflow
- Do not use `git add -A` when working with archived content (can hang due to embedded repos)

## Questions?

For questions about archived items or restoration, refer to:
- Original PR/issue in GitHub
- Commit message in git log: `git log --all -- .archive/`
- Related work packages in AgilePlus
