<!-- Base: platforms/thegent/dotfiles/governance/CLAUDE.base.md -->
<!-- Last synced: 2026-03-29 -->

# phenotype-infrakit — CLAUDE.md

Extends thegent governance base. See `platforms/thegent/dotfiles/governance/CLAUDE.base.md` for canonical definitions.

## Project Overview

- **Name**: phenotype-infrakit
- **Description**: Rust workspace containing generic infrastructure crates extracted from the Phenotype ecosystem
- **Location**: `/Users/kooshapari/CodeProjects/Phenotype/repos/` (monorepo)
- **Language Stack**: Rust (edition 2021)
- **Published**: Internal (shared across Phenotype org)

## AgilePlus Mandate

All work MUST be tracked in AgilePlus:
- Reference: `/Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus`
- CLI: `cd /Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus && agileplus <command>`

## Work Requirements

1. **Check for AgilePlus spec before implementing**
2. **Create spec for new work**: `agileplus specify --title "<feature>" --description "<desc>"`
3. **Update work package status**: `agileplus status <feature-id> --wp <wp-id> --state <state>`
4. **No code without corresponding AgilePlus spec**

## Branch Discipline

- Feature branches in `repos/worktrees/<project>/<category>/<branch>`
- Canonical repository tracks `main` only
- Return to `main` for merge/integration checkpoints

## UTF-8 Encoding

All markdown files must use UTF-8.

---

## Local Quality Checks

From this repository root:

```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --check
```

## Testing & Specification Traceability

All tests MUST reference a Functional Requirement (FR):

```rust
// Traces to: FR-PHENO-NNN
#[test]
fn test_feature_name() {
    // Test body
}
```

**Verification**:
- Every FR in FUNCTIONAL_REQUIREMENTS.md MUST have >=1 test
- Every test MUST reference >=1 FR
- Run: `cargo test` to verify

---

## Project-Specific Configuration

This monorepo consists of domain-agnostic, independently consumable Rust crates:

### Crate Structure

```
crates/
  phenotype-event-sourcing/   # Append-only event store with SHA-256 hash chains
  phenotype-cache-adapter/    # Two-tier LRU + DashMap cache with TTL
  phenotype-policy-engine/    # Rule-based policy evaluation with TOML config
  phenotype-state-machine/    # Generic FSM with transition guards
  phenotype-contracts/        # Shared traits and types
  phenotype-error-core/       # Canonical error types
  phenotype-health/           # Health check abstraction
  phenotype-config-core/      # Configuration management
```

### Conventions

- All public types implement `Debug`, `Clone` where possible
- Error types use `thiserror` with `#[from]` for conversions
- Serialization via `serde` with `Serialize`/`Deserialize` derives
- No inter-crate dependencies; each crate stands alone
- Workspace-level dependency versions in root `Cargo.toml`
- Tests are inline (`#[cfg(test)]` modules) within each source file

### Adding a New Crate

1. Create `crates/<name>/` with `Cargo.toml` and `src/lib.rs`
2. Add to `members` in root `Cargo.toml`
3. Use `workspace = true` for shared dependencies
4. Include inline tests with `#[cfg(test)]`
5. Update `README.md` crate table

---

## Architecture

### Hexagonal Architecture (Ports & Adapters)

This project follows Hexagonal Architecture with clear separation of concerns:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         Hexagonal Architecture                                 │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   ┌──────────────┐     ┌──────────────────┐     ┌──────────────────┐        │
│   │    Ports    │     │      Domain      │     │    Adapters     │        │
│   │(Interfaces) │◄────▶│     (Core)       │◄────▶│(Implementations)│        │
│   │             │     │                  │     │                  │        │
│   │  Inbound:   │     │   Business       │     │  Outbound:      │        │
│   │  - UseCase │     │   Logic          │     │  - Repository   │        │
│   │  - Command │     │                  │     │  - CachePort    │        │
│   │  - Query   │     │                  │     │  - SecretPort   │        │
│   │  - Event   │     │                  │     │  - EventBus     │        │
│   └──────────────┘     └──────────────────┘     └──────────────────┘        │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Design Principles

| Principle | Description | Application |
|-----------|-------------|-------------|
| **SOLID** | Single Responsibility, Open/Closed, Liskov Substitution, Interface Segregation, Dependency Inversion | Ports define minimal interfaces; Domain depends on abstractions |
| **GRASP** | General Responsibility Assignment Software Patterns | Low Coupling, High Cohesion, Information Expert |
| **Law of Demeter** | Talk only to immediate friends | Adapters only access ports they implement |
| **DRY** | Don't Repeat Yourself | Shared contracts in `phenotype-contracts` |
| **KISS** | Keep It Simple, Stupid | Minimal interfaces, focused crates |
| **YAGNI** | You Aren't Gonna Need It | Build features as needed |

### xDD Methodologies Applied

| Category | Methodologies |
|----------|--------------|
| **Development** | TDD, BDD, DDD, CQRS, ATDD, SDD |
| **Design** | SOLID, GRASP, DRY, KISS, YAGNI, LoD, SoC |
| **Architecture** | Clean, Hexagonal, Onion, EDA, Event Sourcing |
| **Quality** | Property-Based Testing, Mutation Testing, Contract Testing |
| **Process** | CI/CD, Agile, Scrum, Kanban, GitOps |
| **Documentation** | ADRs, RFC, Runbooks, SpecDD |

### ADRs (Architecture Decision Records)

See `docs/adr/` for architecture decisions.

---

## TypeScript Validation Standard

All TypeScript projects use **Zod** (^3.24.2+) for runtime validation:

- **React forms**: Use `@hookform/resolvers` for react-hook-form + Zod integration
- **APIs**: Use `zod-to-json-schema` for JSON Schema generation from Zod schemas
- **Schema location**: Define all validation schemas in `src/schemas/` directory
- **Runtime validation**: Always use `.safeParse()` for validation with proper error handling
- **No custom validators**: Use Zod's built-in methods and composable validators
- **Type safety**: Leverage Zod's `z.infer<typeof schema>` for TypeScript type inference

See `docs/reference/VALIDATION_STANDARDS.md` for detailed patterns and examples.

---

## Python Configuration Standard

All Python projects use **pydantic-settings v2.x** for configuration management:

- **Base pattern**: Composite `BaseSettings` with `SettingsConfigDict`
- **Environment loading**: Auto `.env` loading via `env_file='.env'` in Config
- **Field validators**: Use type hints, descriptions, and range validation on all config fields
- **Consistent env prefix**: Define per-project prefix (e.g., `THGENT_*`, `AGILEPLUS_*`)
- **Type safety**: All fields typed; use `Annotated` for validation metadata
- **Immutability**: Configure `ConfigDict(frozen=True)` for production configs

See `docs/reference/CONFIGURATION_STANDARDS.md` for detailed patterns and examples.