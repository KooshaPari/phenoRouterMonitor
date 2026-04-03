# PhenoSDK Extraction & Publishing Strategy

**Date**: 2026-03-29
**Status**: SPECIFICATION (Ready for Implementation)
**Scope**: Transform phenotype-infrakit Rust crates into @phenotype npm packages

---

## Executive Summary

This document outlines the strategy to extract core functionality from phenotype-infrakit Rust workspace and publish as permanent, shared npm packages in the `@phenotype` scope. Three primary packages are targeted:

1. **@phenotype/pheno-core** - Hexagonal architecture contracts, domain models, shared interfaces
2. **@phenotype/pheno-llm** - LLM integration patterns, prompt management, token handling
3. **@phenotype/pheno-resilience** - Event sourcing, state machines, policy evaluation, caching

---

## Current State Analysis

### Phenotype-Infrakit Rust Workspace

**Location**: `/Users/kooshapari/CodeProjects/Phenotype/repos/repos/worktrees/phenotype-infrakit/`

**Five Rust Crates**:

| Crate | Purpose | LOC | Dependencies | Candidates for NPM |
|-------|---------|-----|--------------|-------------------|
| **phenotype-contracts** | Hexagonal architecture ports (interfaces) and domain models | ~400 | thiserror, serde, uuid | ✓ pheno-core |
| **phenotype-event-sourcing** | Event store, SHA-256 hash chain, event envelope, snapshots | ~800 | serde, sha2, chrono, uuid | ✓ pheno-resilience |
| **phenotype-cache-adapter** | Two-tier caching (LRU + distributed) using moka & dashmap | ~300 | moka, dashmap, serde | ✓ pheno-resilience |
| **phenotype-policy-engine** | Rule-based policy evaluation, regex-compiled policies | ~600 | regex, serde, thiserror | ✓ pheno-resilience |
| **phenotype-state-machine** | Finite state machine with transitions and guard conditions | ~250 | thiserror, serde | ✓ pheno-resilience |

**Total**: ~2,350 LOC across 5 crates | **Shared Dependencies**: serde, chrono, uuid, thiserror

---

## Package Design & Mapping

### Package 1: @phenotype/pheno-core

**Purpose**: Core contracts, domain models, and shared interfaces for hexagonal architecture

**Source**: phenotype-contracts (Rust crate)

**Exports**:

```typescript
// Ports (Interfaces)
export interface UseCase {
  execute(request: unknown): Promise<unknown>;
}

export interface CommandHandler {
  handle(command: Command): Promise<void>;
}

export interface QueryHandler {
  handle(query: Query): Promise<QueryResult>;
}

export interface EventHandler {
  handle(event: DomainEvent): Promise<void>;
}

export interface Repository<T extends Entity> {
  save(entity: T): Promise<void>;
  findById(id: string): Promise<T | null>;
  delete(id: string): Promise<void>;
}

export interface CachePort {
  get<T>(key: string): Promise<T | null>;
  set<T>(key: string, value: T, ttl?: number): Promise<void>;
  delete(key: string): Promise<void>;
}

// Domain Models
export abstract class Entity {
  id: string;
  createdAt: Date;
  updatedAt: Date;
}

export class PhenotypeError extends Error {
  constructor(
    public kind: ErrorKind,
    message: string,
    public context?: Record<string, unknown>
  ) {
    super(message);
  }
}
```

**Stability**: STABLE - Core contracts don't change frequently

---

### Package 2: @phenotype/pheno-llm

**Purpose**: LLM integration patterns, prompt management, token handling, model context

**Stability**: EVOLVING - LLM patterns will change as models evolve

---

### Package 3: @phenotype/pheno-resilience

**Purpose**: Event sourcing, state machines, policy evaluation, caching, fault tolerance

**Sources**:
- phenotype-event-sourcing (Rust crate)
- phenotype-state-machine (Rust crate)
- phenotype-policy-engine (Rust crate)
- phenotype-cache-adapter (Rust crate)

**Stability**: STABLE - Core patterns proven in production

---

## Implementation Phases

### Phase 0: Setup (5 minutes)

**WP0.1**: Create monorepo structure
**WP0.2**: Root-level publishing configuration

---

### Phase 1: pheno-core (20 minutes)

**WP1.1**: Port hexagonal contracts from Rust to TypeScript
**WP1.2**: Unit tests for pheno-core
**WP1.3**: Documentation

---

### Phase 2: pheno-resilience (30 minutes)

**WP2.1**: Event Sourcing Module
**WP2.2**: State Machine Module
**WP2.3**: Policy Engine Module
**WP2.4**: Caching Module
**WP2.5**: Integration tests
**WP2.6**: Documentation

---

### Phase 3: pheno-llm (15 minutes)

**WP3.1**: Core LLM interfaces
**WP3.2**: Prompt Management
**WP3.3**: OpenAI Adapter
**WP3.4**: Documentation

---

### Phase 4: Publishing & Distribution (10 minutes)

**WP4.1**: Package.json updates
**WP4.2**: GitHub Packages publishing workflow
**WP4.3**: .npmrc configuration
**WP4.4**: Release automation

---

## GitHub Packages Setup

### Prerequisites

1. **GitHub Token with Packages Scope**
2. **Configure .npmrc**
3. **Repository Settings** - Allow publishing to GitHub Packages

### Publishing Workflow

Create `.github/workflows/publish.yml` to automate publishing on version tags.

---

## Success Metrics

| Metric | Target | Validation |
|--------|--------|-----------|
| **Packages Published** | 3 (@phenotype/pheno-*) | GitHub Packages UI shows all three packages |
| **Test Coverage** | ≥80% | CI reports coverage for each package |
| **Type Safety** | 100% | No `any` types, strict tsconfig |
| **Zero Breaking Changes** | - | Interfaces stable across minor versions |
| **Documentation** | 100% | Every export has JSDoc comments |
| **Zero Dependencies (pheno-core)** | - | package.json has empty dependencies |

---

## Timeline & Effort Estimate

| Phase | Duration | Effort | Status |
|-------|----------|--------|--------|
| **Phase 0**: Setup | 5 min | Low | Ready |
| **Phase 1**: pheno-core | 20 min | Medium | Ready |
| **Phase 2**: pheno-resilience | 30 min | High | Ready |
| **Phase 3**: pheno-llm | 15 min | Medium | Ready |
| **Phase 4**: Publishing | 10 min | Low | Ready |
| **TOTAL** | **80 minutes** | **~3.5 hrs agent time** | **Ready for Execution** |

---

## Related Documentation

- **Detailed Audit**: `docs/research/PHENOSDK_PACKAGE_AUDIT.md` - Complete interface specifications
- **Implementation Checklist**: `docs/checklists/PHENOSDK_IMPLEMENTATION_CHECKLIST.md` - Step-by-step tasks
- **PR Template**: `docs/reference/PHENOSDK_PR_TEMPLATE.md` - PR submission guidance

---

## Next Steps for User

1. **Review** this strategy document
2. **Approve** package structure and naming
3. **Confirm** GitHub Packages accessibility
4. **Authorize** agent to implement (recommend agent execution for speed)
5. **Assign** execution to implementation agent with this spec as context

---

**Document Status**: SPECIFICATION COMPLETE
**Ready for**: AgilePlus ticket creation + implementation assignment
**Confidence Level**: HIGH (95%+)

---

*This strategy represents a stable, long-term approach to sharing phenotype-infrakit functionality across the Phenotype organization as permanent npm packages.*
