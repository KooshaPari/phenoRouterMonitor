# forgecode Fork Intent & Custom Extensions

## Overview

This fork extends forgecode with Phenotype-specific capabilities for subagent coordination, custom code generation providers, and rich UI integration with AgilePlus.

## Custom Providers (Planned)

### Provider 1: Phenotype Rust Crate Generator

**Purpose**: Generate boilerplate for phenotype-infrakit crate patterns

**Scope**:
- Hexagonal architecture scaffolding (ports, adapters, domain layers)
- SOLID principle enforcement
- Test harness generation (inline #[cfg(test)] modules)
- serde/thiserror integration
- Workspace integration (add to Cargo.toml members)

**Example Output**:
```
phenotype-<feature>/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── ports/
│   │   └── <port_name>.rs
│   ├── adapters/
│   │   └── <adapter_name>.rs
│   └── domain/
│       └── <entity_name>.rs
└── tests/
    └── integration_tests.rs
```

**Integration Points**:
- CLI: `forgecode generate:crate phenotype-<feature> --template hexagonal`
- Dashboard: AgilePlus "New Crate" wizard → triggers provider
- Subagent API: `POST /api/providers/phenotype-crate-gen`

### Provider 2: AgilePlus Work Package Template Generator

**Purpose**: Generate WP spec templates with traceability markers

**Scope**:
- WP YAML scaffolding (title, description, deliverables, acceptance criteria)
- FR/ADR cross-references
- Effort estimation templates
- Dependency DAG placeholders

**Example Output**:
```yaml
workPackage: WP-NNN
title: Feature Implementation
epic: E-N
dependencies:
  - WP-NNN-1
  - WP-NNN-2
fraceability:
  - FR-PHENO-NNN
deliverables:
  - Code: src/...
  - Tests: tests/...
  - Docs: docs/...
effortEstimate: "3-5 days (agent-driven)"
acceptanceCriteria:
  - All tests pass
  - Code review approved
  - Documentation updated
```

**Integration Points**:
- CLI: `forgecode generate:wp --epic E-N --effort 3d`
- AgilePlus dashboard: "Create WP" → auto-generates template
- Subagent API: `POST /api/providers/agileplus-wp-gen`

### Provider 3: xDD Test Generator

**Purpose**: Generate test scaffolds from requirements with traceability

**Scope**:
- Test-first templates (Rust unit/integration, TypeScript Jest, Python pytest)
- FR annotation generation (`// Traces to: FR-PHENO-NNN`)
- BDD/ATDD template support (Gherkin for feature files)
- Mutation testing hooks

**Example Output** (Rust):
```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Traces to: FR-PHENO-NNN
    #[test]
    fn test_feature_behavior() {
        // Arrange
        let input = ...;

        // Act
        let result = function_under_test(input);

        // Assert
        assert_eq!(result, expected);
    }
}
```

**Integration Points**:
- CLI: `forgecode generate:test --fr FR-PHENO-NNN --lang rust`
- Subagent API: `POST /api/providers/xdd-test-gen`

## Subagent Extensions (Planned)

### Extension 1: Multi-Agent Coordination Layer

**Purpose**: Enable multiple subagents to work on forgecode-generated scaffolds simultaneously

**Scope**:
- Work package distribution (split WPs across agents)
- Task dependency resolution (topo sort of WP DAG)
- Merge coordination (combine outputs from parallel agents)
- Conflict detection (overlapping file modifications)

**Architecture**:
```
forgeCodeAgent (orchestrator)
├── Task Split: WP-001, WP-002, WP-003 (parallel)
├── Agent-1: implements WP-001
├── Agent-2: implements WP-002
├── Agent-3: implements WP-003
└── Task Merge: verify no conflicts, combine, test
```

**Integration Points**:
- Subagent Registry: `forgecode/extensions/subagent-coordinator`
- API: `POST /api/extensions/coordinate-agents` (accepts WPs, returns assignment strategy)

### Extension 2: Evidence & Audit Trail Generation

**Purpose**: Auto-generate compliance artifacts and evidence bundles

**Scope**:
- Test execution logs (capture output, screenshots, timing)
- Code coverage reports (line coverage, branch coverage)
- Linter/type check summaries
- Git commit history snapshots
- CI/CD link embeddings

**Example Output**:
```
evidence/
├── test-execution-2026-03-30.log
├── coverage-report.html
├── clippy-audit-2026-03-30.json
├── git-log-PR-NNN.txt
└── ci-links.json
```

**Integration Points**:
- CLI: `forgecode audit:generate --wp WP-NNN --include tests,coverage,git`
- Dashboard: Evidence Gallery (clickable timeline, CI links, test results)
- Subagent API: `POST /api/extensions/generate-evidence`

### Extension 3: Schema Versioning & Migration Generator

**Purpose**: Generate versioned schema definitions with forward/backward compatibility

**Scope**:
- JSON Schema / Protobuf / GraphQL SDL generation
- Compatibility matrices (what version pairs are compatible)
- Migration script generation (data transformation for schema updates)
- Documentation of breaking changes

**Example Output**:
```
schemas/
├── v1/
│   ├── user.proto
│   └── post.proto
├── v2/
│   ├── user.proto (added: email_verified)
│   └── post.proto
├── migrations/
│   └── v1_to_v2.sql
└── SCHEMA_VERSION_MATRIX.md
```

**Integration Points**:
- CLI: `forgecode generate:schema --entity User --version 2 --lang protobuf`
- Subagent API: `POST /api/extensions/generate-migration`

## AgilePlus Integration

### Dashboard Widgets

1. **forgecode Provider Gallery**
   - Browse available providers
   - Trigger generation with GUI
   - Preview outputs before commit

2. **Evidence & Audit Timeline**
   - Clickable test results, CI links, git commits
   - Gallery view of generated artifacts
   - Hover-to-expand rich previews

3. **Subagent Work Distribution**
   - Visual DAG of WP dependencies
   - Real-time agent progress (running, idle, blocked)
   - Conflict detection alerts

### Spec Location

**Decision**: Store forgecode fork specs in **kitty-specs/** (shared across Phenotype)

**Rationale**:
- forgecode fork is shared infrastructure (not repo-specific)
- AgilePlus extends it, so specs belong in central location
- Allows cross-repo reuse of generated artifacts

**Structure**:
```
kitty-specs/
├── forgecode-fork/
│   ├── 001-custom-providers.md
│   ├── 002-subagent-extensions.md
│   ├── 003-agileplus-dashboard-integration.md
│   └── plan.md
```

**Initialization**:
```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos/forgecode-fork
agileplus spec --source ../kitty-specs/forgecode-fork/
```

## Implementation Roadmap

### Phase 1: Foundation (Weeks 1-2)
- [ ] Git clone forgecode upstream
- [ ] Set up development environment
- [ ] Implement Provider 1: Phenotype Rust Crate Generator
- [ ] Create initial AgilePlus specs in kitty-specs/

### Phase 2: Advanced Providers (Weeks 3-4)
- [ ] Implement Provider 2: AgilePlus WP Generator
- [ ] Implement Provider 3: xDD Test Generator
- [ ] Integration testing for all providers

### Phase 3: Subagent Extensions (Weeks 5-6)
- [ ] Implement Extension 1: Multi-Agent Coordinator
- [ ] Implement Extension 2: Evidence Generator
- [ ] Subagent Registry integration

### Phase 4: Dashboard Integration (Weeks 7-8)
- [ ] AgilePlus widget for Provider Gallery
- [ ] Evidence Timeline widget
- [ ] Agent Work Distribution widget

### Phase 5: Release & Stabilization (Week 9+)
- [ ] Documentation and runbooks
- [ ] Public release (if applicable)
- [ ] Community feedback loop

## Design Principles

| Principle | Application |
|-----------|-------------|
| **SOLID** | Providers implement Provider interface; each provider is single-responsibility |
| **DRY** | Shared scaffolding logic in base templates; reuse across providers |
| **KISS** | Each provider solves one problem; compose for complex generation |
| **xDD** | All generated code includes test scaffolds and traceability markers |
| **Hexagonal Architecture** | Providers are adapters; forgecode core is domain; gallery UI is outbound port |

## Questions & Open Items

1. **Upstream synchronization**: How often to sync with official forgecode repo?
   - Proposed: Monthly, cherry-pick custom providers

2. **CI/CD pipeline**: Run provider tests on every PR?
   - Proposed: Yes, with evidence generation as part of CI

3. **Versioning**: Semver or CalVer?
   - Proposed: CalVer (YYYY.MONTH.PATCH) following thegent pattern

4. **Multi-language support**: Generate for languages beyond Rust/TypeScript?
   - Proposed: Phase 2+ (Python, Go, Java)

## References

- **Repository**: https://github.com/KooshaPari/forgecode-fork (pending creation)
- **AgilePlus**: /Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus
- **Phenotype Governance**: /Users/kooshapari/CodeProjects/Phenotype/CLAUDE.md
- **xDD Methodologies**: https://en.wikipedia.org/wiki/Behavior-driven_development
