# Architecture Decision Records - Architecture

## ADR-007: Hexagonal Architecture with Clean Architecture Layers

- **Status:** Accepted
- **Date:** 2026-03-25
- **Context:** Need clear separation of domain logic from infrastructure concerns to enable testability, maintainability, and extensibility.

### Decision

Implement **Hexagonal Architecture** (Ports & Adapters) combined with **Clean Architecture** principles:

```
┌─────────────────────────────────────────────────────────────┐
│                    Infrastructure Layer                      │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌────────────────┐ │
│  │   CLI   │  │   API   │  │  gRPC  │  │   Git/VCS     │ │
│  └────┬────┘  └────┬────┘  └────┬────┘  └───────┬────────┘ │
└───────┼────────────┼────────────┼───────────────┼───────────┘
        │            │            │               │
┌───────┴────────────┴────────────┴───────────────┴───────────┐
│                    Primary (Driving) Ports                  │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌───────────┐  │
│  │  Agent   │  │  Review  │  │ Content  │  │  VCS     │  │
│  │   Port   │  │   Port   │  │   Port   │  │   Port   │  │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └─────┬─────┘  │
└───────┼─────────────┼─────────────┼───────────────┼────────┘
        │             │             │               │
┌───────┴─────────────┴─────────────┴───────────────┴──────────┐
│                    Application Layer (Use Cases)              │
│  ┌─────────────────────────────────────────────────────┐    │
│  │  BacklogService  │  FeatureService  │  CycleService │    │
│  └─────────────────────────────────────────────────────┘    │
└────────────────────────────┬───────────────────────────────┘
                             │
┌────────────────────────────┴───────────────────────────────┐
│                      Domain Layer                            │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌────────────────┐  │
│  │Feature  │  │ Backlog │  │ Cycle   │  │    Module      │  │
│  │ Entity  │  │ Entity  │  │ Entity  │  │    Entity      │  │
│  └─────────┘  └─────────┘  └─────────┘  └────────────────┘  │
└────────────────────────────┬───────────────────────────────┘
                             │
┌────────────────────────────┴───────────────────────────────┐
│                 Secondary (Driven) Ports                     │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌───────────┐  │
│  │ Storage  │  │  Event   │  │   Telem  │  │ Observab  │  │
│  │   Port   │  │   Port   │  │   Port   │  │   Port   │  │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └─────┬─────┘  │
└───────┼─────────────┼─────────────┼───────────────┼────────┘
        │             │             │               │
┌───────┴─────────────┴─────────────┴───────────────┴──────────┐
│                 Infrastructure Adapters                       │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌────────────────┐  │
│  │ SQLite  │  │  NATS   │  │ Open    │  │   Metrics      │  │
│  │Adapter  │  │ Adapter │  │Telemetry│  │   Adapter      │  │
│  └─────────┘  └─────────┘  └─────────┘  └────────────────┘  │
└──────────────────────────────────────────────────────────────┘
```

### Directory Structure

```
crates/agileplus-domain/
├── src/
│   ├── lib.rs                 # Public API (barrel export)
│   ├── error.rs               # Domain errors
│   ├── domain/                # Domain entities
│   │   ├── mod.rs
│   │   ├── feature.rs
│   │   ├── backlog.rs
│   │   ├── cycle.rs
│   │   ├── module.rs
│   │   └── work_package/       # DDD: Bounded contexts
│   ├── ports/                  # Hexagonal: Interfaces
│   │   ├── mod.rs
│   │   ├── agent.rs           # Primary: Agent operations
│   │   ├── review.rs          # Primary: Code review
│   │   ├── content.rs         # Primary: Content management
│   │   ├── vcs.rs             # Primary: VCS operations
│   │   ├── storage.rs         # Secondary: Persistence
│   │   └── observability.rs   # Secondary: Telemetry
│   └── config/                # Configuration domain
└── tests/                     # Domain tests
```

### Port Interfaces

#### Primary (Driving) Ports
- `AgentPort` - Agent lifecycle and task delegation
- `ReviewPort` - Code review operations
- `ContentPort` - Content/document management
- `VcsPort` - Version control operations

#### Secondary (Driven) Ports
- `StoragePort` - Persistence abstraction
- `EventPort` - Event publishing
- `ObservabilityPort` - Metrics, traces, logs

### Application Services

Use cases orchestrate domain logic through ports:

```rust
pub struct BacklogService<P: StoragePort, E: EventPort> {
    storage: Arc<P>,
    events: Arc<E>,
}

impl<P: StoragePort, E: EventPort> BacklogService<P, E> {
    pub async fn add_feature(&self, feature: Feature) -> Result<FeatureId> {
        // Domain logic
        let id = self.storage.save_feature(feature).await?;
        self.events.publish(FeatureAdded { id: id.clone() }).await?;
        Ok(id)
    }
}
```

### Rationale

1. **Testability** - Domain logic has no infrastructure dependencies
2. **Maintainability** - Clear separation of concerns
3. **Extensibility** - New adapters can be added without modifying domain
4. **SOLID Compliance** - Interface segregation via ports
5. **DDD Alignment** - Work packages as bounded contexts

### Alternatives Considered

| Alternative | Reason for Rejection |
|-------------|---------------------|
| Pure layered | Less explicit port abstraction |
| Feature-oriented | Less mature tooling |
| ECS (Entity-Component-System) | Overkill for this domain |

### Consequences

- **Positive:** Domain can be tested without mocks for infrastructure
- **Positive:** Multiple UI adapters (CLI, API, gRPC) can share domain
- **Negative:** Slight indirection overhead
- **Negative:** More boilerplate for small features

---

## ADR-008: SOLID Principles Enforcement

- **Status:** Accepted
- **Date:** 2026-03-25

### Single Responsibility Principle (SRP)

Each module has one reason to change:
- `BacklogService` - only backlog management
- `FeatureService` - only feature operations
- `CycleService` - only cycle/iteration management

### Open/Closed Principle (OCP)

Domain entities open for extension, closed for modification:
- Use traits for new behavior
- Value objects immutable

### Liskov Substitution Principle (LSP)

All port implementations must satisfy port contracts.

### Interface Segregation Principle (ISP)

Fine-grained ports over monolithic interfaces:
- `AgentPort` vs generic `ServicePort`
- `StoragePort` vs `PersistencePort`

### Dependency Inversion Principle (DIP)

High-level modules depend on abstractions (ports), not concretions.

### Enforcement

- Clippy linting in CI
- Code review checklist
- Integration tests for each adapter

---

## ADR-009: DDD Bounded Contexts

- **Status:** Accepted
- **Date:** 2026-03-25

### Contexts Identified

| Context | Entities | Ubiquitous Language |
|---------|----------|-------------------|
| **Feature Management** | Feature, BacklogItem | story points, acceptance criteria |
| **Cycle Management** | Cycle, Sprint, Milestone | velocity, burndown |
| **Module Organization** | Module, Dependency | coupling, cohesion |
| **Agent Coordination** | Agent, Task, Delegation | capability, routing |
| **Governance** | Policy, Audit, Compliance | enforcement, reporting |

### Integration Points

- Events for cross-context communication
- Shared kernel for common concepts (dates, IDs)

---

## ADR-010: TDD/BDD Testing Strategy

- **Status:** Accepted
- **Date:** 2026-03-25

### Test Pyramid

```
         ┌─────────┐
         │   E2E   │  ← BDD scenarios (Gherkin)
         ├─────────┤
         │Integration│ ← Adapter tests with real infra
         ├─────────┤
         │  Unit   │  ← Domain logic, services
         └─────────┘
```

### Test Locations

| Type | Location |
|------|----------|
| Unit | `crates/*/src/**/*.rs` (inline `#[cfg(test)]`) |
| Integration | `crates/*/tests/*.rs` |
| Contract | `crates/agileplus-contract-tests/` |
| BDD | `tests/bdd/` |

### Naming Conventions

- Unit: `#[test] fn feature_described()`
- Integration: `tests/integration/test_*.rs`
- BDD: `tests/bdd/features/*.feature`

---

## ADR-011: Specification-Driven Development (SpecDD)

- **Status:** Accepted
- **Date:** 2026-03-25

### Spec-First Workflow

```
1. Write/Update spec.md
        ↓
2. Generate test stubs from spec
        ↓
3. Implement until tests pass
        ↓
4. Spec validated by working code
        ↓
5. Repeat
```

### Spec Structure

```markdown
# Feature: Backlog Management

## Acceptance Criteria
- [ ] Can create backlog items
- [ ] Can assign to cycles
- [ ] Progress tracked automatically

## Edge Cases
- Empty backlog: Show empty state
- Duplicate names: Reject with error
```

### Tooling

- `kitty-specs/` - Specification management
- BDD scenarios as living documentation
- Spec as contract between stakeholders and devs

---

## ADR-012: Error Handling Strategy

- **Status:** Accepted
- **Date:** 2026-03-25

### Error Types

| Type | Use Case | Example |
|------|----------|---------|
| `DomainError` | Business rule violations | Invalid state transition |
| `PortError` | Infrastructure failures | DB connection lost |
| `ApplicationError` | Use case failures | Feature not found |

### Error Propagation

```
PortError ──wrap──→ ApplicationError ──wrap──→ DomainError
```

### Result Type Convention

All port and service methods return `Result<T, Error>`:
- `Ok(T)` - Success with value
- `Err(DomainError)` - Business rule violation
- `Err(ApplicationError)` - Operational failure

---

## ADR-013: Observability Stack

- **Status:** Accepted
- **Date:** 2026-03-25

### Three Pillars

| Pillar | Implementation |
|--------|---------------|
| **Metrics** | OpenTelemetry + Prometheus |
| **Logging** | `tracing` with structured fields |
| **Traces** | OpenTelemetry with W3C context |

### Log Format

```json
{
  "level": "INFO",
  "timestamp": "2026-03-25T12:00:00Z",
  "message": "Feature created",
  "feature_id": "feat_123",
  "trace_id": "abc123"
}
```

### SLOs to Track

- Feature creation latency < 100ms p99
- API availability > 99.9%
- Sync operation success rate > 99%

---

## ADR-014: Plugin Architecture Pattern

- **Status:** Proposed
- **Date:** 2026-03-25

### Plugin Boundaries

```
┌──────────────────────────────────────────┐
│              Core (Domain)                │
│  Ports + Entities + Application Services │
└──────────────────────────────────────────┘
        ↑                           ↑
   Plugin API                 Plugin API
        ↑                           ↑
┌──────────────────────────────────────────┐
│         Plugin A (e.g., GitHub)          │
└──────────────────────────────────────────┘
┌──────────────────────────────────────────┐
│         Plugin B (e.g., GitLab)          │
└──────────────────────────────────────────┘
```

### Plugin Traits

```rust
#[trait_variant::make(VcsPlugin: Send + Sync)]
pub trait VcsPort {
    async fn clone_repo(&self, url: &Url) -> Result<PathBuf>;
    async fn create_branch(&self, name: &str) -> Result<Branch>;
    async fn open_pr(&self, title: &str) -> Result<PullRequest>;
}
```

### Registration

```rust
// In core
pub fn register_vcs_plugin<P: VcsPort + 'static>(plugin: P) {
    // Compile-time plugin registry
}
```

---

## ADR-015: Monorepo Workspace Structure

- **Status:** Accepted
- **Date:** 2026-03-25

### Workspace Layout

```
AgilePlus/
├── Cargo.toml          # Workspace root
├── Cargo.lock
├── crates/             # Rust crates
│   ├── agileplus-domain/          # Core domain (hexagonal)
│   ├── agileplus-api/             # REST API adapter
│   ├── agileplus-grpc/             # gRPC adapter
│   ├── agileplus-cli/              # CLI adapter
│   ├── agileplus-git/              # Git VCS adapter
│   └── ... (15+ more crates)
├── proto/              # Protocol Buffer definitions
├── tests/              # BDD, contract, integration
├── docs/               # Documentation
└── scripts/            # Build/dev scripts
```

### Crate Naming Convention

| Prefix | Purpose |
|--------|---------|
| `agileplus-` | Core crates |
| `agileplus-*-tests` | Test-only crates |
| `agileplus-*-bench` | Benchmark crates |

### Dependency Rules

- `agileplus-domain` has **zero** external dependencies (except std)
- Adapter crates depend on `agileplus-domain`
- No circular dependencies

---

## ADR-016: Code Quality Gates

- **Status:** Accepted
- **Date:** 2026-03-25

### CI Gates (in order)

1. **Format Check** - `cargo fmt --check`
2. **Clippy Lints** - `cargo clippy --all-targets`
3. **Typos Check** - `typos --format brief`
4. **Tests** - `cargo test --workspace`
5. **Security Audit** - `cargo audit`
6. **Miri** (unsafe) - `cargo miri test` (nightly)
7. **Fuzz** - `cargo fuzz` (periodic)

### Pre-commit Hooks

```yaml
- repo: https://github.com/rust-lang/rustfmt
  hooks: [rustfmt]
- repo: https://github.com/rust-lang/rust-clippy
  hooks: [clippy]
- repo: crate-ci/typos
  hooks: [typos]
```

### Code Coverage

- Target: 80% line coverage for domain
- Enforcement: CI fails below threshold
- Tool: `cargo-llvm-cov`

---

*This document is the authoritative source for architecture decisions. Update via PR with ADR template.*
