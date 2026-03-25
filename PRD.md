# AgilePlus: Product Requirements Document

**Version:** 1.0 | **Status:** Draft | **Date:** 2026-03-25

## Product Vision

AgilePlus is a schema-driven project management platform built on gRPC. This repository is the **single source of truth** for all inter-service Protocol Buffer definitions, providing Rust and Python codegen for the AgilePlus ecosystem (core, MCP, agents, integrations).

## Epics

### E1: Core Service Contract (AgilePlusCoreService)

| ID | Story | Acceptance Criteria |
|----|-------|-------------------|
| E1.1 | Feature lifecycle management (create, update, transition) | gRPC endpoints for full feature CRUD with status transitions |
| E1.2 | Governance enforcement (approval workflows, policy gates) | Governance rules evaluated on state transitions |
| E1.3 | Audit trail (all mutations logged with actor and timestamp) | AuditEntry messages emitted for every state change |

### E2: Agent Dispatch Contract (AgentDispatchService)

| ID | Story | Acceptance Criteria |
|----|-------|-------------------|
| E2.1 | Agent spawn and lifecycle management | gRPC endpoints to spawn, monitor, and terminate agents |
| E2.2 | Review loop orchestration | Agents can request and receive code reviews via gRPC |

### E3: Integrations Contract (IntegrationsService)

| ID | Story | Acceptance Criteria |
|----|-------|-------------------|
| E3.1 | Plane.so integration (issue sync) | Bidirectional sync between AgilePlus features and Plane issues |
| E3.2 | GitHub integration (PR/issue linking) | Features linked to GitHub PRs and issues |
| E3.3 | Triage automation | Incoming items auto-classified and routed |

### E4: Multi-Language Codegen

| ID | Story | Acceptance Criteria |
|----|-------|-------------------|
| E4.1 | Rust crate with tonic/prost codegen | `cargo build` produces working gRPC client/server stubs |
| E4.2 | Python package with grpcio stubs | `uv sync` produces working Python gRPC stubs |
| E4.3 | Breaking change detection via buf | CI blocks breaking proto changes without version bump |

### E5: Shared Message Types

| ID | Story | Acceptance Criteria |
|----|-------|-------------------|
| E5.1 | Common types (Feature, AuditEntry, Agent, etc.) | Shared message definitions used by all three services |
| E5.2 | Versioning (v1 namespace) | All protos under `agileplus.v1` namespace |
