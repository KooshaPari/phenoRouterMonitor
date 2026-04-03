# phenotype-health — CLAUDE.md

## Project Overview

A Rust library providing health checking and observability for Phenotype ecosystem services. Implements liveness, readiness, and startup probes for cloud-native deployments.

**Language**: Rust
**Location**: `crates/phenotype-health/`
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
- Maintain `/FUNCTIONAL_REQUIREMENTS.md` with FR-HEALTH-XXX IDs
- Tag all tests with comment: `// Traces to: FR-HEALTH-NNN`
- Map code entities in `docs/reference/CODE_ENTITY_MAP.md`
- Create worklog entries in `docs/worklogs/` per phase

## Specification Documents

**Root-level files** (in monorepo root):
- `/FUNCTIONAL_REQUIREMENTS.md` — Granular requirements for phenotype-health
- `/docs/worklogs/` — Phase-based worklog entries
- `/docs/reference/CODE_ENTITY_MAP.md` — Code ↔ requirements mapping

## Testing & Traceability

All tests MUST reference an FR:

```rust
// Traces to: FR-HEALTH-001
#[test]
fn test_liveness_probe_check() {
    // Test body
}
```

Run: `cargo test --lib phenotype_health`

## Build & Quality

```bash
cd crates/phenotype-health
cargo test
cargo clippy
cargo fmt
```

## Development Notes

- Kubernetes-native health check implementation (liveness, readiness, startup)
- Pluggable checker system for custom health validators
- Integration with observability/metrics pipelines
- Used by AgilePlus dashboard and agent services

## See Also

- **AgilePlus Governance**: `docs/reference/AGILEPLUS_GOVERNANCE_CHASSIS.md`
- **Phenotype Docs**: `docs/reference/PHENOTYPE_DOCS_CHASSIS_INTERFACE.md`
- **Monorepo Root**: `../../../CLAUDE.md`

