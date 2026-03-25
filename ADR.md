# AgilePlus: Architecture Decision Records

## ADR-001: Protocol Buffers as Single Source of Truth

- **Status:** Accepted
- **Context:** Five downstream repos need consistent API contracts.
- **Decision:** All inter-service contracts defined as Protocol Buffer files in this repo.
- **Rationale:** Schema-first design ensures type safety across Rust, Python, and future language targets.
- **Alternatives:** OpenAPI (no streaming support), Thrift (less ecosystem tooling).

## ADR-002: buf v2 for Lint and Breaking Change Detection

- **Status:** Accepted
- **Context:** Need automated enforcement of proto quality and backward compatibility.
- **Decision:** Use buf v2 for linting, formatting, and breaking change detection against `main`.
- **Rationale:** buf provides the best-in-class proto tooling; CI integration is straightforward.

## ADR-003: Rust Codegen via tonic/prost

- **Status:** Accepted
- **Context:** Rust services need gRPC client and server stubs.
- **Decision:** Use tonic (gRPC framework) with prost (protobuf codegen) in a dedicated `rust/` crate.
- **Rationale:** tonic is the de-facto Rust gRPC framework with async/await support.

## ADR-004: Python Codegen via grpcio

- **Status:** Accepted
- **Context:** Python services (agents, MCP) need gRPC stubs.
- **Decision:** Use grpcio with buf-generated Python stubs in `python/` package.
- **Rationale:** grpcio is mature and well-supported; buf generates clean Python code.

## ADR-005: Three-Service Architecture

- **Status:** Accepted
- **Context:** Need separation of concerns for feature lifecycle, agent management, and external integrations.
- **Decision:** Three gRPC services: AgilePlusCoreService, AgentDispatchService, IntegrationsService.
- **Rationale:** Each service has distinct scaling requirements and failure domains.

## ADR-006: Breaking Change Policy

- **Status:** Accepted
- **Context:** Downstream consumers must not be broken by proto changes.
- **Decision:** Breaking changes require version bump (v1 -> v2) and coordination with all consumers.
- **Rationale:** Protobuf wire format guarantees backward compat for additive changes only.
