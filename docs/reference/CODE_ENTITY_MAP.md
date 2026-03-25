# AgilePlus: Code Entity Map

## Proto Definitions

| Entity | Path | Maps To |
|--------|------|---------|
| Common types (Feature, AuditEntry) | `proto/agileplus/v1/common.proto` | FR-CORE-004, FR-AGENT-004, FR-PROTO-003 |
| AgilePlusCoreService | `proto/agileplus/v1/core.proto` | FR-CORE-001, FR-CORE-002, FR-CORE-003 |
| AgentDispatchService | `proto/agileplus/v1/agents.proto` | FR-AGENT-001, FR-AGENT-002, FR-AGENT-003 |
| IntegrationsService | `proto/agileplus/v1/integrations.proto` | FR-INT-001, FR-INT-002, FR-INT-003 |

## Rust Crate

| Entity | Path | Maps To |
|--------|------|---------|
| Rust crate (agileplus-proto) | `rust/` | FR-CODEGEN-001 |
| Cargo.toml | `rust/Cargo.toml` | FR-CODEGEN-001 |
| Build script (prost codegen) | `rust/build.rs` | FR-CODEGEN-001 |

## Python Package

| Entity | Path | Maps To |
|--------|------|---------|
| Python package (agileplus-proto) | `python/` | FR-CODEGEN-002 |
| pyproject.toml | `python/pyproject.toml` | FR-CODEGEN-002 |

## Tooling

| Entity | Path | Maps To |
|--------|------|---------|
| buf config | `buf.yaml` | FR-PROTO-001, FR-CODEGEN-003 |
| buf codegen config | `buf.gen.yaml` | FR-CODEGEN-001, FR-CODEGEN-002 |
| Makefile | `Makefile` | All |
