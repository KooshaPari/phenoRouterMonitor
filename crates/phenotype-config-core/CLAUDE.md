# phenotype-config-core — CLAUDE.md

## Project Overview

A Rust library providing configuration management for Phenotype ecosystem projects. Handles file discovery, format detection, and unified configuration loading.

**Language**: Rust
**Location**: `crates/phenotype-config-core/`
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
- Maintain `/FUNCTIONAL_REQUIREMENTS.md` with FR-CONFIG-XXX IDs
- Tag all tests with comment: `// Traces to: FR-CONFIG-NNN`
- Map code entities in `docs/reference/CODE_ENTITY_MAP.md`
- Create worklog entries in `docs/worklogs/` per phase

## Specification Documents

**Root-level files** (in monorepo root):
- `/FUNCTIONAL_REQUIREMENTS.md` — Granular requirements for phenotype-config-core
- `/docs/worklogs/` — Phase-based worklog entries
- `/docs/reference/CODE_ENTITY_MAP.md` — Code ↔ requirements mapping

## Testing & Traceability

All tests MUST reference an FR:

```rust
// Traces to: FR-CONFIG-001
#[test]
fn test_config_file_discovery() {
    // Test body
}
```

Run: `cargo test --lib phenotype_config_core`

## Build & Quality

```bash
cd crates/phenotype-config-core
cargo test
cargo clippy
cargo fmt
```

## Development Notes

- Config loading supports multiple formats (TOML, YAML, JSON)
- Directory discovery follows XDG Base Directory specification
- Error handling via phenotype-error-core
- Logging via phenotype tracing module

## See Also

- **AgilePlus Governance**: `docs/reference/AGILEPLUS_GOVERNANCE_CHASSIS.md`
- **Phenotype Docs**: `docs/reference/PHENOTYPE_DOCS_CHASSIS_INTERFACE.md`
- **Monorepo Root**: `../../../CLAUDE.md`

