# AgilePlus: Functional Requirements

## FR-CORE: Core Service

| ID | Requirement | Traces To |
|----|------------|-----------|
| FR-CORE-001 | System SHALL define gRPC endpoints for feature CRUD (Create, Get, Update, Delete, List) | E1.1 |
| FR-CORE-002 | System SHALL define status transition RPCs with governance validation | E1.2 |
| FR-CORE-003 | System SHALL emit AuditEntry messages for all state mutations | E1.3 |
| FR-CORE-004 | System SHALL define Feature message with fields: id, name, status, owner, metadata, timestamps | E5.1 |

## FR-AGENT: Agent Dispatch Service

| ID | Requirement | Traces To |
|----|------------|-----------|
| FR-AGENT-001 | System SHALL define RPCs to spawn agents with configuration | E2.1 |
| FR-AGENT-002 | System SHALL define RPCs to monitor agent health and status | E2.1 |
| FR-AGENT-003 | System SHALL define RPCs for review loop (request review, submit review) | E2.2 |
| FR-AGENT-004 | System SHALL define Agent message with type, config, and lifecycle status | E5.1 |

## FR-INT: Integrations Service

| ID | Requirement | Traces To |
|----|------------|-----------|
| FR-INT-001 | System SHALL define RPCs for Plane.so bidirectional issue sync | E3.1 |
| FR-INT-002 | System SHALL define RPCs for GitHub PR/issue linking | E3.2 |
| FR-INT-003 | System SHALL define RPCs for triage classification and routing | E3.3 |

## FR-CODEGEN: Multi-Language Code Generation

| ID | Requirement | Traces To |
|----|------------|-----------|
| FR-CODEGEN-001 | Rust crate SHALL compile with `cargo build` producing tonic gRPC stubs | E4.1 |
| FR-CODEGEN-002 | Python package SHALL install with `uv sync` producing grpcio stubs | E4.2 |
| FR-CODEGEN-003 | CI SHALL block PRs with breaking proto changes (buf breaking) | E4.3 |

## FR-PROTO: Proto Quality

| ID | Requirement | Traces To |
|----|------------|-----------|
| FR-PROTO-001 | All proto files SHALL pass buf lint with zero warnings | E4.3 |
| FR-PROTO-002 | All proto files SHALL use `agileplus.v1` package namespace | E5.2 |
| FR-PROTO-003 | Common message types SHALL be defined in `common.proto` | E5.1 |
