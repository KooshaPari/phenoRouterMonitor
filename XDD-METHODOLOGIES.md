# x-DD Methodologies & Best Practices

This document outlines the software engineering methodologies, patterns, and best practices applied throughout this codebase.

## Table of Contents

1. [Architecture Patterns](#architecture-patterns)
2. [Design Principles](#design-principles)
3. [Domain-Driven Design](#domain-driven-design)
4. [Testing Methodologies](#testing-methodologies)
5. [Observability & Monitoring](#observability--monitoring)
6. [Security Patterns](#security-patterns)
7. [Data Management](#data-management)
8. [API Design](#api-design)
9. [DevOps & Deployment](#devops--deployment)
10. [Code Quality](#code-quality)
11. [Anti-Patterns to Avoid](#anti-patterns-to-avoid)

---

## Architecture Patterns

| Pattern | Abbr. | Description |
|---------|-------|-------------|
| Hexagonal Architecture | HXA | Ports & Adapters - isolate domain from infrastructure |
| Clean Architecture | CA | Layered architecture with dependency rules |
| Onion Architecture | OA | Inside-out dependency flow |
| Ports & Adapters | P&A | Interface-based design for testability |
| Command Query Responsibility Segregation | CQRS | Separate read/write models |
| Event Sourcing | ES | Store events, not state |
| Event-Driven Architecture | EDA | Decouple via events |
| Saga Pattern | SAGA | Distributed transactions via events |
| CQRS + ES | CQRSES | Combined command/query with event sourcing |
| API Gateway | API-GW | Central entry point for microservices |
| Backend-for-Frontend | BFF | Specialized API layer per client |
| Strangler Fig | SF | Incrementally migrate legacy systems |
| Anti-Corruption Layer | ACL | Translate between bounded contexts |
| Domain Event Bus | DEB | Internal domain event distribution |
| Service Mesh | SM | Infrastructure for service communication |
| Sidecar Pattern | SP | Co-locate helper with main service |
| Ambassador Pattern | AP | Offload client connectivity to helper |
| Adapter Pattern | ADP | Convert interface to expected format |
| Facade Pattern | FP | Simplified interface to complex subsystem |
| Decorator Pattern | DCP | Add behavior dynamically |

---

## Design Principles

### SOLID Principles

| Principle | Letter | Description |
|-----------|--------|-------------|
| Single Responsibility | **S** | One reason to change |
| Open/Closed | **O** | Open for extension, closed for modification |
| Liskov Substitution | **L** | Subtypes must be substitutable |
| Interface Segregation | **I** | Many specific interfaces over one general |
| Dependency Inversion | **D** | Depend on abstractions, not concretions |

### GRASP Principles

| Principle | Description |
|-----------|-------------|
| Controller | Assign responsibility to classes representing UI/system boundary |
| Creator | Assign to class that contains/creates related objects |
| High Cohesion | Keep related responsibilities together |
| Low Coupling | Minimize dependencies between elements |
| Indirection | Introduce intermediary to decouple |
| Information Expert | Assign responsibility to class with information |
| Polymorphism | Distribute behavior based on type |
| Protected Variations | Isolate elements from variations |

### Other Design Principles

| Principle | Description |
|-----------|-------------|
| **DRY** | Don't Repeat Yourself |
| **KISS** | Keep It Simple, Stupid |
| **YAGNI** | You Aren't Gonna Need It |
| **BDY** | Big Design Up Front - avoid |
| **POOD** | Persistent Object-Oriented Design |
| **GOLD** | Graceful Object-Oriented Design |
| **RLT** | Release Equivalence Law |
| **OCP** | Open/Closed Principle |
| **ISP** | Interface Segregation Principle |
| **CCP** | Common Closure Principle |
| **CRP** | Common Reuse Principle |
| **SAP** | Stable Abstractions Principle |
| **SDP** | Stable Dependencies Principle |
| **ADP** | Acyclic Dependencies Principle |
| **REP** | Reuse-Release Equivalence Principle |

---

## Domain-Driven Design

| Pattern | Description |
|---------|-------------|
| **Bounded Context** | Clear boundary for a domain model |
| **Aggregates** | Cluster related entities under one root |
| **Entity** | Object with identity continuity |
| **Value Object** | Immutable, defined by attributes |
| **Domain Event** | Significant occurrence in domain |
| **Domain Service** | Operation not belonging to entity |
| **Repository** | Mechanism to access aggregates |
| **Factory** | Encapsulate complex object creation |
| **Specification** | Business rule encapsulation |
| **Anti-Corruption Layer** | Translate external models |
| **Shared Kernel** | Shared code between contexts |
| **Customer/Supplier** |上下游 relationship |
| **Conformist** | Align with supplier model |
| **Published Language** | Standard exchange format |
| **Context Map** | Relationship between contexts |
| **Continuous Integration** | Merge changes frequently |
| **Seam** | Boundary where models differ |

---

## Testing Methodologies

| Methodology | Description |
|-------------|-------------|
| **TDD** | Test-Driven Development - red/green/refactor |
| **BDD** | Behavior-Driven Development - Gherkin syntax |
| **ATDD** | Acceptance Test-Driven Development |
| **FDD** | Feature-Driven Development |
| **Property-Based Testing** | Test invariants, generate cases |
| **Mutation Testing** | Verify test quality |
| **Contract Testing** | API compatibility verification |
| **Snapshot Testing** | UI regression detection |
| **Chaos Testing** | Resilience verification |
| **Performance Testing** | Load and stress testing |
| **Security Testing** | Penetration and vulnerability testing |
| **Smoke Testing** | Basic functionality check |
| **Sanity Testing** | Focused regression check |
| **Regression Testing** | Ensure no new bugs |
| **Integration Testing** | Component interaction testing |
| **Unit Testing** | Individual component testing |
| **End-to-End Testing** | Full system flow testing |

---

## Observability & Monitoring

| Pattern | Description |
|---------|-------------|
| **Structured Logging** | JSON-formatted, contextual logs |
| **Log Correlation** | Trace ID propagation |
| **Log Levels** | DEBUG/INFO/WARN/ERROR/FATAL |
| **Metrics** | Quantitative measurements |
| **RED Method** | Rate/Errors/Duration |
| **USE Method** | Utilization/Saturation/Errors |
| **Distributed Tracing** | Request flow across services |
| **Span** | Individual operation in trace |
| **Health Endpoint** | /health or /ready endpoints |
| **Graceful Degradation** | Partial functionality on failure |
| **Circuit Breaker** | Prevent cascade failures |
| **Rate Limiting** | Prevent resource exhaustion |
| **Timeout** | Prevent indefinite waiting |
| **Retry with Backoff** | Transient failure handling |
| **Dead Letter Queue** | Failed message capture |
| **Alerting** | Proactive notification |
| **SLO/SLA** | Service level objectives/agreements |

---

## Security Patterns

| Pattern | Description |
|---------|-------------|
| **Defense in Depth** | Multiple security layers |
| **Principle of Least Privilege** | Minimal access rights |
| **Zero Trust** | Never trust, always verify |
| **Secure by Default** | Safe defaults configuration |
| **Input Validation** | Sanitize all external data |
| **Output Encoding** | Prevent injection attacks |
| **Secrets Management** | Secure credential storage |
| **Key Rotation** | Periodic credential refresh |
| **OAuth 2.0** | Delegated authorization |
| **OIDC** | Identity layer on OAuth |
| **JWT** | Stateless authentication token |
| **Password Hashing** | bcrypt/argon2/scrypt |
| **SQL Injection Prevention** | Parameterized queries |
| **XSS Prevention** | Content Security Policy |
| **CSRF Prevention** | Token-based protection |
| **Rate Limiting** | Brute force protection |

---

## Data Management

| Pattern | Description |
|---------|-------------|
| **ACID** | Atomicity/Consistency/Isolation/Durability |
| **BASE** | Basically Available/Soft state/Eventually consistent |
| **CAP Theorem** | Consistency/Availability/Partition tolerance |
| **2-Phase Commit** | Distributed transaction protocol |
| **Saga Pattern** | Choreography-based distributed transactions |
| **CQRS** | Command Query Responsibility Segregation |
| **Event Sourcing** | Store events, derive state |
| **Materialized View** | Pre-computed query results |
| **Sharding** | Horizontal data partitioning |
| **Replication** | Data redundancy across nodes |
| **Read Replica** | Scale read operations |
| **Write Behind Cache** | Async cache updates |
| **Write Through Cache** | Sync cache updates |
| **Cache Invalidation** | Strategy for stale data |
| **Database Migration** | Versioned schema changes |
| **Idempotency** | Safe to retry operations |
| **Optimistic Locking** | Version-based conflict detection |
| **Pessimistic Locking** | Explicit lock acquisition |

---

## Implementation Checklist

When implementing features, ensure:

- [ ] **Hexagonal**: Domain has zero external dependencies
- [ ] **SOLID**: Each class has single responsibility
- [ ] **DRY**: No duplicated business logic
- [ ] **KISS**: Implementation is as simple as possible
- [ ] **YAGNI**: No speculative features
- [ ] **GRASP**: Responsibilities assigned per pattern
- [ ] **TDD**: Tests written before implementation
- [ ] **BDD**: Gherkin scenarios defined for features
- [ ] **Logging**: Structured logs with correlation IDs
- [ ] **Metrics**: Key operations measured
- [ ] **Health**: /health endpoint responds correctly
- [ ] **Security**: Input validated, secrets externalized
- [ ] **Idempotency**: Operations safe to retry
- [ ] **Documentation**: ADRs for architectural decisions
- [ ] **Code Review**: All changes reviewed

---

## References

- [Clean Architecture (Uncle Bob)](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html)
- [Domain-Driven Design (Evans)](https://www.domainlanguage.com/ddd/)
- [Hexagonal Architecture (Cockburn)](https://alistair.cockburn.us/hexagonal-architecture/)
- [SOLID Principles](https://en.wikipedia.org/wiki/SOLID)
