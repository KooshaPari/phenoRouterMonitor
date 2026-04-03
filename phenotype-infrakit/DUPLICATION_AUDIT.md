# Duplication Audit - Phenotype Repos

## Purpose

This document tracks:
1. **Cross-repo duplication** - Same code/patterns exist in multiple repos
2. **Intra-repo duplication** - Same code exists in multiple places within a repo
3. **Libification opportunities** - Duplication that should become shared libraries
4. **Pattern consolidation** - Common patterns that should be extracted
5. **Fork/wrap candidates** - External packages that could be wrapped or forked

## Cross-Repo Duplication

### Authentication / Auth Providers

| Pattern | Repos | LOC | Opportunity |
|---------|-------|-----|-------------|
| AuthKit integration | `heliosApp`, `agileplus`, `vibe-kanban` | ~500 | Extract to `phenotype-auth` lib |
| Auth middleware | `phenotype-go-kit`, `phenotype-shared` | ~300 | Consolidate |

### API Client / Fetch Patterns

| Pattern | Repos | LOC | Opportunity |
|---------|-------|-----|-------------|
| OpenAPI client generation | Multiple | ~1000 | `phenotype-api-client` lib |
| React Query hooks | `heliosApp`, `phenotype-gauge` | ~400 | Extract to `@phenotype/react-query` |
| API error handling | `heliosCLI`, `phenotype-config` | ~200 | Standard error lib |

### Database / ORM Patterns

| Pattern | Repos | LOC | Opportunity |
|---------|-------|-----|-------------|
| Event sourcing | `phenotype-shared`, `phenotype-nexus` | ~800 | `phenotype-eventsourcing` lib |
| Repository pattern | `heliosApp`, `phenotype-infrakit` | ~600 | `phenotype-repository` lib |
| Migration scripts | `heliosApp`, `parpour` | ~300 | `phenotype-migrations` lib |

### Configuration Management

| Pattern | Repos | LOC | Opportunity |
|---------|-------|-----|-------------|
| Config loading | `phenotype-config`, `phenotype-go-kit` | ~400 | `phenotype-config` lib (already exists) |
| Environment parsing | `heliosCLI`, `phenotype-config` | ~150 | Consolidate |

### Logging / Observability

| Pattern | Repos | LOC | Opportunity |
|---------|-------|-----|-------------|
| Structured logging | `heliosApp`, `phenotype-shared` | ~300 | `phenotype-logging` lib |
| Tracing setup | `trace`, `profiler` | ~500 | Consolidate tracing |
| Metrics collection | `phenotype-gauge`, `profiler` | ~400 | `phenotype-metrics` lib |

### CLI / Command Patterns

| Pattern | Repos | LOC | Opportunity |
|---------|-------|-----|-------------|
| CLI framework | `heliosCLI`, `phench`, `trash-cli` | ~600 | `phenotype-cli` lib |
| Command registration | `heliosCLI`, `parpour` | ~200 | Extract |
| Help text generation | `heliosCLI`, `forgecode` | ~150 | Consolidate |

### Testing Patterns

| Pattern | Repos | LOC | Opportunity |
|---------|-------|-----|-------------|
| Test fixtures | `tests/`, `heliosApp` | ~500 | `phenotype-test-fixtures` lib |
| Mock factories | `heliosApp`, `profiler` | ~300 | Extract |
| E2E test harness | `heliosApp`, `vibe-kanban` | ~400 | `phenotype-e2e` lib |

### UI Component Patterns

| Pattern | Repos | LOC | Opportunity |
|---------|-------|-----|-------------|
| Component library | `heliosApp`, `vibe-kanban` | ~2000 | `@phenotype/ui` package |
| Hooks | `heliosApp`, `phenotype-design` | ~600 | `@phenotype/hooks` |
| Form validation | `heliosApp`, `agileplus` | ~300 | `phenotype-validation` lib |
| Tailwind config | `heliosApp`, `agileplus` | ~200 | `phenotype-tailwind-config` (already exists) |

## Intra-Repo Duplication

### heliosApp

| Pattern | Locations | LOC | Recommendation |
|---------|-----------|-----|---------------|
| API client | `packages/*/client.ts` | ~400 | Centralize |
| Store setup | Multiple feature packages | ~300 | Extract to `@phenotype/store` |

### phenotype-config

| Pattern | Locations | LOC | Recommendation |
|---------|-----------|-----|---------------|
| Config parsers | `src/providers/*` | ~500 | Consolidate |

## AgilePlus Intra-Repo Duplication

### Health Check Patterns (NEW - 2026-03-29)

| Pattern | Locations | LOC | Canonical Location |
|---------|-----------|-----|-------------------|
| CacheHealth enum | `crates/agileplus-cache/src/health.rs:5-8` | 42 | `agileplus-health` crate |
| GraphHealth enum | `crates/agileplus-graph/src/health.rs:5-8` | 90 | `agileplus-health` crate |
| BusHealth enum | `crates/agileplus-nats/src/health.rs:5-8` | 8 | `agileplus-health` crate |

- [ ] 🔴 CRITICAL: Create `agileplus-health` crate (saves ~80 LOC)

### Error Type Patterns (NEW - 2026-03-29)

| Pattern | Locations | LOC | Canonical Location |
|---------|-----------|-----|-------------------|
| ApiError | `crates/agileplus-api/src/error.rs:15-28` | 67 | `agileplus-error-core` |
| DomainError | `crates/agileplus-domain/src/error.rs:4-50` | 50 | `agileplus-error-core` |
| SyncError | `crates/agileplus-sync/src/error.rs:6-24` | 24 | `agileplus-error-core` |
| PortError | `libs/phenotype-shared/.../error.rs` | 51 | `agileplus-error-core` |

- [ ] 🟡 HIGH: Create `agileplus-error-core` crate (saves ~150 LOC)

### Config Loading Patterns (NEW - 2026-03-29)

| Pattern | Locations | Format | Canonical Location |
|---------|-----------|--------|-------------------|
| TOML + Env | `crates/agileplus-domain/src/config/loader.rs` | TOML | Integrate `libs/config-core` |
| YAML + Env | `crates/agileplus-telemetry/src/config.rs` | YAML | Integrate `libs/config-core` |

- [ ] 🟡 HIGH: Integrate `libs/config-core` into workspace (saves ~200 LOC)

### API Response Patterns (NEW - 2026-03-29)

| Pattern | Locations | LOC | Canonical Location |
|---------|-----------|-----|-------------------|
| HealthResponse | `crates/agileplus-api/src/responses.rs:125-224` | 100 | `agileplus-api-types` |
| ApiResponse | `platforms/heliosCLI/codex-rs/core/src/client.rs` | ~100 | `agileplus-api-types` |

- [ ] 🟠 MEDIUM: Create `agileplus-api-types` crate (saves ~50 LOC)

### Port/Trait Split (NEW - 2026-03-29)

| System | Locations | Ports | Canonical Location |
|--------|-----------|-------|-------------------|
| agileplus-domain | `crates/agileplus-domain/src/ports/` | 5 | `agileplus-domain` (existing) |
| phenotype-port-interfaces | `libs/phenotype-shared/crates/phenotype-port-interfaces/` | 8 | Consolidate |

- [ ] 🟢 LOW: Audit for consolidation opportunities

**Full audit report**: `docs/reports/AGILEPLUS_DUPLICATION_AUDIT_20260329.md`

## Libification Opportunities

### High Priority

1. **`@phenotype/react-query`** - React Query hooks for API calls
   - Currently in: `heliosApp`, `phenotype-gauge`
   - Est. LOC: ~400
   - Benefit: DRY API calls, consistent caching

2. **`phenotype-auth`** - Auth provider integration
   - Currently in: `heliosApp`, `agileplus`
   - Est. LOC: ~500
   - Benefit: Single auth integration point

3. **`phenotype-e2e`** - E2E testing harness
   - Currently in: `heliosApp`, `vibe-kanban`
   - Est. LOC: ~400
   - Benefit: Reusable test infrastructure

### Medium Priority

4. **`@phenotype/ui`** - Shared UI components
5. **`phenotype-logging`** - Structured logging
6. **`phenotype-metrics`** - Metrics collection

### Lower Priority

7. **`phenotype-cli`** - CLI framework helpers
8. **`phenotype-validation`** - Form validation

## Fork/Wrap Candidates

### Already Wrapped

| Package | Wrapper Repo | Usage |
|---------|-------------|-------|
| OpenAI SDK | `heliosApp` | AI features |
| Prisma | `heliosApp` | Database |
| Tailwind CSS | `phenotype-design` | Styling |

### Could Be Forked

| Package | Why Fork | Effort |
|---------|---------|--------|
| Plane (upstream) | `agileplus` customization | High |
| AgentAPI (coder) | `agentapi-plusplus` | Medium |
| VitePress | Custom docs theming | Low |
| `config` (config-rs) | `agileplus` config loading | Low - use `libs/config-core` instead |
| `health_check` crate | Async health checks pattern | Low - fork to add async_trait support |

### Internal Fork Candidates (AgilePlus)

| Pattern | Current | Recommendation |
|---------|---------|----------------|
| Health checks | Manual impl in 3+ crates | Fork `health_check` crate with async_trait |
| Config loading | Manual TOML/YAML parsers | Integrate `libs/config-core` |

## Pattern Consolidation

### Hexagonal Architecture

Multiple repos use hexagonal architecture:
- `template-commons/hexagonal-*` templates
- `phenotype-go-kit`
- `phenotype-shared`

**Opportunity:** Standardize on `template-commons/hexagonal-typescript` as canonical

### Event Sourcing

- `phenotype-shared`: Event sourcing core
- `phenotype-nexus`: Event processing

**Opportunity:** Merge into single `phenotype-eventsourcing` lib

### Microservice Scaffold

- `template-commons/microservice-scaffold`
- `heliosApp` (monolith)
- `phenotype-go-kit`

**Opportunity:** Create unified scaffold for all services

## Actions

### Immediate (AgilePlus)

- [ ] 🔴 CRITICAL: Create `agileplus-health` crate with unified `HealthChecker` trait
- [ ] 🟡 HIGH: Create `agileplus-error-core` crate with common `AppErrorKind` variants
- [ ] 🟡 HIGH: Integrate `libs/config-core` into workspace and create `FromEnv` derive

### Immediate

- [ ] Audit `heliosApp` for extractable patterns
- [ ] Create `phenotype-auth` lib from `agileplus` auth code
- [ ] Consolidate tailwind configs into `phenotype-tailwind-config`

### Short Term (this week)

- [ ] Extract `@phenotype/react-query` hooks
- [ ] Standardize CLI patterns in `phenotype-cli`
- [ ] Create shared test fixtures

### Medium Term (this month)

- [ ] Audit all repos for pattern duplication
- [ ] Create `phenotype-eventsourcing` lib
- [ ] Build unified microservice scaffold

## Known Duplication

### Already Identified

1. **AuthKitProvider** - Used in `heliosApp`, `agileplus`, `vibe-kanban`
2. **Tailwind config** - Multiple configs, should be single source
3. **API error handling** - Different approaches in each repo
4. **Test setup** - Each repo has own test utilities

## Notes

- Last updated: 2026-03-29
- Next audit: Weekly
- Use subagents to parallelize detailed audits per category
