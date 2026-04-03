# phenotype-git-core — CLAUDE.md

## Project Overview

A Rust library providing Git operations abstractions for Phenotype ecosystem projects. Handles repository operations, revision parsing, and git-based versioning.

**Language**: Rust
**Location**: `crates/phenotype-git-core/`
**Published**: Yes (to crates.io or internal registry)

## Phenotype Federated Hybrid Architecture

This project is part of the Phenotype Federated Hybrid Architecture:

### Phenotype Docs Chassis
Provides VitePress configuration and design tokens for documentation.

See: `docs/reference/PHENOTYPE_DOCS_CHASSIS_INTERFACE.md`

### AgilePlus Governance Chassis
Defines specification-driven delivery with PRD, ADR, FUNCTIONAL_REQUIREMENTS, and FR traceability.

See: `docs/reference/AGILEPLUS_GOVERNANCE_CHASSIS.md`

**For this project**:
- Maintain `/FUNCTIONAL_REQUIREMENTS.md` with FR-GIT-XXX IDs
- Tag all tests with comment: `// Traces to: FR-GIT-NNN`
- Map code entities in `docs/reference/CODE_ENTITY_MAP.md`
- Create worklog entries in `docs/worklogs/` per phase

## Specification Documents

**Root-level files** (in monorepo root):
- `/FUNCTIONAL_REQUIREMENTS.md` — Granular requirements for phenotype-git-core
- `/docs/worklogs/` — Phase-based worklog entries
- `/docs/reference/CODE_ENTITY_MAP.md` — Code ↔ requirements mapping

## Testing & Traceability

All tests MUST reference an FR:

```rust
// Traces to: FR-GIT-001
#[test]
fn test_repository_initialization() {
    // Test body
}
```

Run: `cargo test --lib phenotype_git_core`

## Build & Quality

```bash
cd crates/phenotype-git-core
cargo test
cargo clippy
cargo fmt
```

## Development Notes

- Wraps libgit2 for low-level Git operations
- Provides high-level abstractions for common workflows
- Error handling via phenotype-error-core
- Used by AgilePlus for commit history analysis and verification

## See Also

- **AgilePlus Governance**: `docs/reference/AGILEPLUS_GOVERNANCE_CHASSIS.md`
- **Phenotype Docs**: `docs/reference/PHENOTYPE_DOCS_CHASSIS_INTERFACE.md`
- **Monorepo Root**: `../../../CLAUDE.md`

