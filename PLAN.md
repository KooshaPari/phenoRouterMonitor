# AgilePlus: Implementation Plan

## Phase 1: Proto Definitions (Complete)

| Task | Description | Depends On |
|------|-------------|------------|
| P1.1 | Define common.proto shared message types | - |
| P1.2 | Define core.proto (AgilePlusCoreService) | P1.1 |
| P1.3 | Define agents.proto (AgentDispatchService) | P1.1 |
| P1.4 | Define integrations.proto (IntegrationsService) | P1.1 |

## Phase 2: Codegen and Tooling (Complete)

| Task | Description | Depends On |
|------|-------------|------------|
| P2.1 | buf v2 lint and breaking change config | P1.1 |
| P2.2 | buf codegen plugin config (buf.gen.yaml) | P2.1 |
| P2.3 | Rust crate with tonic/prost build.rs | P2.2 |
| P2.4 | Python package with grpcio stubs | P2.2 |

## Phase 3: CI and Quality Gates

| Task | Description | Depends On |
|------|-------------|------------|
| P3.1 | CI: buf lint on every PR | P2.1 |
| P3.2 | CI: buf breaking against main | P2.1 |
| P3.3 | CI: cargo build for Rust crate | P2.3 |
| P3.4 | CI: uv sync for Python package | P2.4 |

## Phase 4: Downstream Consumer Validation

| Task | Description | Depends On |
|------|-------------|------------|
| P4.1 | Validate agileplus-core consumes Rust stubs | P3.3 |
| P4.2 | Validate agileplus-mcp consumes Python stubs | P3.4 |
| P4.3 | Validate agileplus-agents consumes stubs | P3.3, P3.4 |
| P4.4 | Validate agileplus-integrations consumes stubs | P3.3, P3.4 |
