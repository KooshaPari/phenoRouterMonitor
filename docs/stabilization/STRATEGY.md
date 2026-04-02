# Polyrepo Stabilization & Optimization Strategy

**Date**: 2026-04-01
**Scope**: 227-repo KooshaPari GitHub portfolio → actionable stabilization plan
**Status**: STRATEGY (Ready for AgilePlus spec creation)

---

## Executive Summary

The Phenotype ecosystem spans **227 repos** across 4 languages (65 Rust, 45 Python, 30 TypeScript, 25 Go) with only **9 cloned locally** consuming **89 GB** (22 GB build artifacts). This strategy groups all repos into **6 clusters**, identifies **merge/split candidates**, defines **auxiliary infrastructure**, and lays out a **4-phase stabilization plan** targeting full ecosystem governance within one quarter.

---

## Cluster 1: Core Platform (HIGHEST PRIORITY)

### Repos

| Repo | Language | Role | Local? |
|------|----------|------|--------|
| `phenotype-infrakit` | Rust | Foundation crates (workspace, 19 members) | ✅ |
| `AgilePlus` | Python/Rust | Spec-driven work management engine | ✅ |
| `thegent` | TypeScript/Go | Agent orchestration platform | ✅ |
| `heliosCLI` | Rust/Python | Terminal agent runtime | ✅ |
| `heliosApp` | TypeScript | Desktop/web agent application | ✅ |
| `phenotype-hub` | TypeScript | Ecosystem hub/registry | ✅ |
| `cloud/` (monorepo) | TypeScript | Cloudflare Workers + services | ✅ |
| `cliproxyapi-plusplus` | Python | CLI proxy API layer | ✅ |
| `agentapi-plusplus` | Python | Agent API service | ✅ |
| `agent-wave` | TypeScript | Agent communication layer | ✅ |
| `forgecode` | TypeScript | Code generation engine | ✅ |
| `phench` | Python | Phenotype CLI tool | ✅ |
| `bifrost` | Rust | Routing/bridge layer | ✅ |

### What Needs Stabilization

1. **phenotype-infrakit**: 10 open PRs, 19 crates in workspace — needs PR triage, crate dependency graph cleanup, and removal of duplicate crates (`phenotype-contract` vs `phenotype-contracts`, `phenotype-error-core` vs `phenotype-errors` vs `phenotype-error-macros`)
2. **AgilePlus**: 26 specs, many incomplete — needs spec completion audit, FR traceability enforcement, and API stability
3. **thegent**: Governance templates done, Phase 3 (memory) in progress — needs cross-platform integration
4. **heliosCLI**: 39 GB local (bazel artifacts) — needs artifact cleanup, sandboxing Phase 2 completion
5. **cloud/**: 20+ Cloudflare services — needs service registry, shared config, deployment pipeline

### Auxiliary Infra Needed

- **Package registry**: GitHub Packages for `@phenotype/*` npm scope + PyPI for `phenotype-*` Python packages
- **CI/CD template**: Shared GitHub Actions workflows (32 already exist at shelf root — need distribution mechanism)
- **Service mesh**: Cloudflare Workers routing table for all cloud services
- **Config federation**: Shared configuration via `phenotype-shared-config` crate

---

## Cluster 2: Agent Orchestration

### Repos

| Repo | Language | Role | Local? |
|------|----------|------|--------|
| `thegent` (also in C1) | TypeScript/Go | Core agent platform | ✅ |
| `thegent-plugin-host` | TypeScript | Plugin system | ❌ |
| `agileplus-mcp` | TypeScript | MCP server for AgilePlus | ❌ |
| `agileplus-agents` | Python | Agent definitions | ❌ |
| `agentapi-plusplus` (also in C1) | Python | Agent API | ✅ |
| `agent-wave` (also in C1) | TypeScript | Agent communication | ✅ |
| `forgecode` (also in C1) | TypeScript | Code generation | ✅ |
| `forgecode-fork` | TypeScript | Fork evaluation | ❌ |

### What Needs Stabilization

1. **MCP protocol standardization**: All agent-to-agent communication should use a single MCP SDK (thegent has Go/Python/TS MCP SDKs in Phase 1)
2. **Plugin host**: `thegent-plugin-host` needs to be merged into `thegent` as a sub-package
3. **Agent registry**: Central registry of available agents, capabilities, and health status
4. **forgecode-fork**: Evaluate and either merge back or delete

### Auxiliary Infra Needed

- **Agent registry service**: HTTP service listing all available agents with capabilities
- **MCP SDK publishing**: Publish `@thegent/mcp-*` packages to npm/PyPI
- **Health monitoring**: Shared health check endpoint across all agent services

---

## Cluster 3: SDK & Developer Tools

### Repos

| Repo | Language | Role | Local? |
|------|----------|------|--------|
| `packages/pheno-core` | TypeScript | Core contracts (hexagonal) | ✅ |
| `packages/pheno-llm` | TypeScript | LLM integration | ✅ |
| `packages/pheno-resilience` | TypeScript | Event sourcing, state machines | ✅ |
| `python/phenosdk` | Python | Python SDK | ✅ |
| `python/pheno-core` | Python | Python core contracts | ✅ |
| `python/pheno-llm` | Python | Python LLM integration | ✅ |
| `python/pheno-mcp` | Python | Python MCP bindings | ✅ |
| `python/pheno-agents` | Python | Python agent framework | ✅ |
| `python/pheno-atoms` | Python | Python atomic operations | ✅ |
| `libs/phenotype-config-core` | Rust | Config core library | ✅ |
| `libs/nexus` | Rust | Nexus library | ✅ |
| `heliosHarness` | Mixed | Test harness (GitHub) | ❌ |
| `phenotype-go-sdk` | Go | Go SDK (GitHub) | ❌ |
| `phenotype-types` | TypeScript | Shared types (GitHub) | ❌ |
| `phenotype-config-ts` | TypeScript | TS config (GitHub) | ❌ |
| `phenotype-validation` | TypeScript | Validation library (GitHub) | ❌ |

### What Needs Stabilization

1. **SDK unification**: Python SDK (`python/phenosdk`) and TypeScript SDK (`packages/`) need parallel structure and feature parity
2. **Package publishing**: All SDK packages need automated publishing pipelines
3. **Version alignment**: `@phenotype/*` npm packages and `phenotype-*` Python packages should share version numbers
4. **Go SDK**: Needs consumer audit — if no consumers, fold into a unified SDK monorepo

### Auxiliary Infra Needed

- **SDK monorepo**: Consider consolidating all SDK packages into a single `phenotype-sdk` monorepo with npm/PyPI/crates.io publishing
- **API documentation**: Auto-generated docs from TypeScript/Python/Go source
- **SDK test harness**: Shared integration test suite across all language SDKs

---

## Cluster 4: Templates & Hexagonal Kits

### Repos

| Repo | Language | Role | Local? |
|------|----------|------|--------|
| `templates/` (shelf) | Mixed | Governance templates | ✅ |
| `kits/` (shelf) | Mixed | Hexagonal architecture kits | ✅ |
| `hexagon-rs` | Rust | Rust hexagonal template (GitHub) | ❌ |
| `hexagon-rust` | Rust | Duplicate of hexagon-rs (GitHub) | ❌ |
| `phenotype-governance` | Mixed | Governance docs | ✅ |
| `thegent/templates/` | Mixed | Agent templates | ✅ |
| `thegent/dotfiles/` | Mixed | Dotfile templates | ✅ |

### What Needs Stabilization

1. **Template deduplication**: `hexagon-rs` and `hexagon-rust` are duplicates — consolidate to one
2. **Template distribution**: Current templates live in `thegent/` and shelf root — need a canonical source with distribution mechanism
3. **Template versioning**: Implement the versioning scheme documented in CONSOLIDATION_AUDIT.md (1.0 → 1.1 quarterly)
4. **Kit completeness**: `kits/` directory is empty — needs population from `templates/` and `thegent/dotfiles/`

### Auxiliary Infra Needed

- **Template registry**: Service that lists available templates with version info
- **Scaffolding CLI**: `phench new <template> <project-name>` command
- **Template testing**: CI that validates all templates generate working projects

---

## Cluster 5: Peripheral / Archive Candidates

### Repos (from portfolio triage)

| Repo | Bucket | Action | Confidence |
|------|--------|--------|------------|
| `agentapi-deprec` | DELETE | Retire immediately | High |
| `tehgent` | DELETE | Typo of thegent, delete | High |
| `hexagon-rust` | DELETE | Duplicate of hexagon-rs | High |
| `odin-*` (4 repos) | ARCHIVE | Course exercises | High |
| `BytePort-TestPortfolio` | DELETE | Test artifact | High |
| `Byteport-TestZip` | DELETE | Test artifact | High |
| `P2` | DELETE | Placeholder | High |
| `Tokn` | DELETE | Truncated name | High |
| `argisexec` | DELETE | Typo/abbrev | High |
| `FixitGo`, `FixitRs` | MERGE | Fold into canonical | High |
| `router-docs` | MERGE | Merge into canonical docs | High |
| `heliosBench` | ARCHIVE | Benchmark sidecar | Medium |
| `QuadSGM` | ARCHIVE | Weak signal | Low |
| `Kogito` | ARCHIVE | Weak fit | Low |
| `Tossy` | ARCHIVE | Dormant | Low |
| `Frostify` | ARCHIVE | One-off | Low |
| `AppGen` | ARCHIVE | Prototype | Low |
| `TripleM` | ARCHIVE | Opaque purpose | Low |
| `Project-Spyn` | ARCHIVE | One-off prototype | Medium |
| `ssToCal-front` | ARCHIVE | Frontend one-off | Medium |
| `BytePortfolio` | ARCHIVE | Superseded | Medium |
| `agentapi` | ARCHIVE | Superseded by plusplus | Medium |
| `acp` | DELETE | Ambiguous | Low |

### Cleanup Strategy

1. **Immediate (Week 1)**: Delete obvious test artifacts and typos (8 repos)
2. **Short-term (Week 2)**: Archive course/exercise repos (4 odin-* repos)
3. **Medium-term (Month 1)**: Archive low-signal personal projects (11 repos)
4. **Long-term (Quarter)**: Merge language variants and doc sidecars (5 repos)

**Net result**: ~28 repos removed from active portfolio, reducing management overhead by ~12%

---

## Cluster 6: Learning / Personal

### Repos

| Repo | Type | Recommendation |
|------|------|----------------|
| `koosha-portfolio` | Personal site | Keep, but move to separate org or personal account |
| `odin-*` (4 repos) | Course exercises | Archive after extracting any reusable patterns |
| `KaskMan` | Unknown | Audit, likely archive |
| `dotfiles` | Personal config | Keep, but separate from project shelf |
| `vibeproxy` | Personal project | Audit — may have strategic value |
| `vibeproxy-monitoring-unified` | Personal project | Already archived |

### Should They Stay on This Shelf?

**No.** Personal and learning repos should be:
1. Moved to a separate `koosha-portfolio` org or personal GitHub account
2. Removed from local shelf to free disk space
3. Excluded from CI/CD, governance, and AgilePlus tracking

Exception: `dotfiles` may be useful as a governance template source — keep a read-only reference.

---

## Auxiliary Infrastructure Needed

### 1. CI/CD Pipeline Design

```
┌─────────────────────────────────────────────────────────┐
│                    Shared Actions Repo                   │
│  github.com/KooshaPari/.github (org-level workflows)    │
├─────────────────────────────────────────────────────────┤
│  Reusable Workflows:                                    │
│  ├── ci-rust.yml      (build, test, clippy, fmt)        │
│  ├── ci-python.yml    (ruff, pytest, typecheck)          │
│  ├── ci-typescript.yml (eslint, vitest, tsc)             │
│  ├── ci-go.yml        (golangci-lint, go test)           │
│  ├── security.yml     (SAST, dependency audit, SBOM)     │
│  ├── publish.yml      (npm, PyPI, crates.io)             │
│  ├── docs.yml         (VitePress build + deploy)         │
│  └── release.yml      (conventional commits, changelog)  │
└─────────────────────────────────────────────────────────┘
```

**Current state**: 32 workflow files exist at shelf root but are not distributed to repos
**Action**: Move to org-level `.github` repo as reusable workflows, reference from each repo

### 2. Documentation Federation

```
┌─────────────────────────────────────────────────────────┐
│              phenotype-docs (federation hub)             │
├─────────────────────────────────────────────────────────┤
│  Sources (git submodules or remote fetch):               │
│  ├── thegent/docs/          → /agents/thegent/           │
│  ├── AgilePlus/docs/        → /tools/agileplus/          │
│  ├── heliosCLI/docs/        → /runtime/helios/           │
│  ├── phenotype-infrakit/docs/ → /sdk/infrakit/           │
│  └── shelf docs/            → /governance/               │
│                                                          │
│  Output: Single VitePress site at docs.phenotype.dev    │
└─────────────────────────────────────────────────────────┘
```

### 3. Package Registry

| Ecosystem | Scope | Registry | Current State |
|-----------|-------|----------|---------------|
| npm | `@phenotype/*` | GitHub Packages | Not published |
| npm | `@thegent/*` | GitHub Packages | Not published |
| PyPI | `phenotype-*` | PyPI (public) | Not published |
| crates.io | `phenotype-*` | crates.io | Not published |
| Go | `github.com/KooshaPari/phenotype-*` | Go module proxy | Not published |

**Action**: Set up publishing workflows for all 4 ecosystems

### 4. Artifact Storage

| Artifact Type | Storage | Retention |
|---------------|---------|-----------|
| Build artifacts | GitHub Actions cache | 30 days |
| Release binaries | GitHub Releases | Permanent |
| Docker images | GitHub Container Registry | 90 days |
| Benchmarks | S3/GitHub Pages | Permanent |
| SBOMs | GitHub Releases | Permanent |

**Current problem**: 22 GB in build artifacts locally (`target/`, `bazel-*`, `node_modules/`)
**Action**: Implement `.gitignore` enforcement, artifact cleanup scripts, and CI caching

### 5. Monitoring & Observability

| Service | Implementation | Coverage |
|---------|---------------|----------|
| Error tracking | Sentry (already set up) | All production services |
| Metrics | Cloudflare Analytics + custom | Cloud services |
| Health checks | `/health` endpoint pattern | All services |
| Uptime | GitHub Actions scheduled checks | Core services |
| Audit logging | Evidence ledger (existing) | All governance actions |

---

## Decomposition Recommendations

### MERGE Together

| Source Repos | Target | Rationale |
|-------------|--------|-----------|
| `phenotype-contract` + `phenotype-contracts` | `phenotype-contracts` | Duplicate naming, same purpose |
| `phenotype-error-core` + `phenotype-errors` + `phenotype-error-macros` | `phenotype-error-core` | Split unnecessarily, macros belong in core |
| `phenotype-ports-canonical` + `phenotype-port-traits` | `phenotype-contracts` | Port definitions belong with contracts |
| `thegent-plugin-host` | `thegent/apps/plugin-host` | Plugin host is part of thegent |
| `forgecode-fork` | `forgecode` (or delete) | Fork should be reconciled |
| `hexagon-rust` | `hexagon-rs` | Direct duplicate |
| `agileplus-agents` | `AgilePlus/packages/agents` | Part of AgilePlus platform |
| `agileplus-mcp` | `AgilePlus/packages/mcp` | Part of AgilePlus platform |
| `router-docs` | `phenotype-hub/docs/` | Docs sidecar, merge into hub |
| `FixitGo` + `FixitRs` | Single `fixit` repo | Language variants in one repo |
| `phenotype-config-loader` | `phenotype-config-core` | Loader is implementation detail |
| `phenotype-shared-config` | `phenotype-config-core` | Duplicate config concern |
| `phenotype-async-traits` | `phenotype-contracts` | Trait definitions belong with contracts |
| `bifrost-routing` + `bifrost-routing-backup` | `bifrost` | Backup is redundant |
| `vibeproxy-monitoring-unified` | Already archived | N/A |

**Net reduction**: ~15 repos merged into 8 targets = **7 fewer repos to manage**

### SPLIT Further

| Source Repo | Split Into | Rationale |
|-------------|-----------|-----------|
| `cloud/` (20+ services) | Individual repos per service OR keep as monorepo with proper workspace | Current structure is flat — needs workspace organization |
| `AgilePlus` (already well-structured) | No split needed | Good modular structure |
| `heliosCLI` | Keep as-is | Bazel workspace is appropriate |
| `phenotype-infrakit` | Consider splitting into 3 workspaces: `core` (contracts, errors), `runtime` (event-sourcing, cache, state-machine), `tools` (policy-engine, validation) | 19 crates is large for a single workspace |

### Becomes Monorepo Subdirectories

| Current | New Location | Rationale |
|---------|-------------|-----------|
| `packages/pheno-*` (3 packages) | `phenotype-sdk/packages/pheno-*` | SDK monorepo |
| `python/pheno-*` (5 packages) | `phenotype-sdk/python/pheno-*` | SDK monorepo |
| `libs/*` (2 libs) | `phenotype-infrakit/libs/*` or keep at shelf | Shelf-level shared libs |
| `templates/*` | `thegent/templates/` | Templates belong in thegent |
| `kits/*` | `thegent/kits/` | Kits belong in thegent |
| `scripts/*` | `.github/scripts/` or `AgilePlus/scripts/` | Shared scripts need home |

---

## Stabilization Phases

### Phase 1: Immediate (This Week — Days 1-7)

**Goal**: Stop the bleeding, establish baseline governance

| Task | Effort | Owner | Dependencies |
|------|--------|-------|-------------|
| **P1.1**: Close/merge 10 open PRs in phenotype-infrakit | 4h | Forge | None |
| **P1.2**: Delete 8 obvious test/typo repos | 1h | Forge | None |
| **P1.3**: Clean 22 GB build artifacts locally | 2h | Forge | None |
| **P1.4**: Enforce `.gitignore` across 9 cloned repos | 2h | Forge | P1.3 |
| **P1.5**: Set up org-level `.github` repo with reusable workflows | 4h | Forge | None |
| **P1.6**: Complete incomplete AgilePlus specs (audit 26 specs) | 6h | Forge | None |
| **P1.7**: Establish worktree discipline — document in WORKTREES.md | 2h | Forge | None |
| **P1.8**: Run `cargo fmt && cargo clippy -- -D warnings` on phenotype-infrakit | 2h | Forge | P1.1 |

**Success criteria**:
- Zero open PRs in phenotype-infrakit
- 89 GB → ~60 GB local disk usage
- All 9 cloned repos pass quality gates
- Org-level CI workflows published

### Phase 2: Short-term (Weeks 2-3)

**Goal**: Consolidate and deduplicate

| Task | Effort | Owner | Dependencies |
|------|--------|-------|-------------|
| **P2.1**: Merge 15 duplicate repos (see decomposition table) | 12h | Forge | Phase 1 |
| **P2.2**: Archive 4 odin-* course repos | 1h | Forge | Phase 1 |
| **P2.3**: Move personal repos to separate org | 2h | Forge | Phase 1 |
| **P2.4**: Set up GitHub Packages publishing for `@phenotype/*` | 4h | Forge | P2.1 |
| **P2.5**: Set up PyPI publishing for `phenotype-*` Python packages | 4h | Forge | P2.1 |
| **P2.6**: Complete phenotype-infrakit Phase 3 (performance) | 8h | Forge | P1.8 |
| **P2.7**: Complete AgilePlus Phase 3 (governance) | 10h | Forge | P1.6 |
| **P2.8**: Distribute base templates to all active repos | 4h | Forge | P2.1 |

**Success criteria**:
- ~200 repos in portfolio (down from 227)
- All SDK packages published to registries
- phenotype-infrakit Phase 3 complete
- AgilePlus Phase 3 complete
- All active repos using base templates

### Phase 3: Medium-term (Weeks 4-6)

**Goal**: Build auxiliary infrastructure

| Task | Effort | Owner | Dependencies |
|------|--------|-------|-------------|
| **P3.1**: Create SDK monorepo (`phenotype-sdk`) | 8h | Forge | P2.4, P2.5 |
| **P3.2**: Set up docs federation (VitePress hub) | 6h | Forge | Phase 2 |
| **P3.3**: Implement health check pattern across all services | 4h | Forge | Phase 2 |
| **P3.4**: Set up Sentry for all production services | 4h | Forge | Phase 2 |
| **P3.5**: Complete thegent Phase 3 (memory) | 12h | Forge | Phase 2 |
| **P3.6**: Complete heliosCLI Phase 2 (sandboxing) | 10h | Forge | Phase 2 |
| **P3.7**: Archive 11 low-signal personal projects | 2h | Forge | P2.3 |
| **P3.8**: Split phenotype-infrakit into 3 workspaces (optional) | 8h | Forge | P2.6 |

**Success criteria**:
- SDK monorepo with all language packages
- Docs federation live at docs.phenotype.dev
- All services reporting health
- thegent Phase 3 complete
- heliosCLI Phase 2 complete

### Phase 4: Long-term (Weeks 7-12)

**Goal**: Full ecosystem stabilization

| Task | Effort | Owner | Dependencies |
|------|--------|-------|-------------|
| **P4.1**: Complete thegent Phase 4 (cross-platform integration) | 12h | Forge | P3.5 |
| **P4.2**: Complete phenotype-infrakit Phase 4 (enterprise) | 12h | Forge | P3.8 |
| **P4.3**: Set up artifact storage and retention policies | 4h | Forge | Phase 3 |
| **P4.4**: Implement template versioning and distribution | 6h | Forge | Phase 3 |
| **P4.5**: Clone and onboard remaining ~200 repos from GitHub | 8h | Forge | Phase 3 |
| **P4.6**: Full CI/CD coverage across all active repos | 10h | Forge | P4.5 |
| **P4.7**: Governance audit — verify all repos comply with baseline | 6h | Muse | P4.6 |
| **P4.8**: Performance benchmarks and optimization report | 8h | Forge | P4.2 |

**Success criteria**:
- All phases of all core repos complete
- 100% CI/CD coverage across active repos
- Full governance compliance
- Documented, stable ecosystem ready for scaling

---

## Disk Usage Optimization

### Current State
```
heliosCLI:          39 GB  (bazel artifacts dominate)
AgilePlus:          20 GB  (venv, pycache, node_modules)
thegent:             8.1 GB (node_modules, venv)
platforms/:          5.1 GB
cloud:               2.7 GB
target/ (workspace): 2.0 GB
phenotype-infrakit:  1.8 GB
phenotype-hub:       1.2 GB
```

### Target State (after cleanup)
```
heliosCLI:           8 GB  (clean bazel cache)
AgilePlus:           4 GB  (clean venv, no node_modules)
thegent:             3 GB  (clean node_modules)
platforms/:          2 GB
cloud:               1 GB
target/ (workspace): 0.5 GB
phenotype-infrakit:  0.5 GB
phenotype-hub:       0.3 GB
─────────────────────────────────
Total:              ~20 GB  (77% reduction)
```

### Cleanup Actions
1. `rm -rf heliosCLI/bazel-*` (saves ~30 GB)
2. `rm -rf */node_modules` + use `pnpm` or workspace hoisting (saves ~5 GB)
3. `rm -rf */.venv` + use `uv` for managed venvs (saves ~3 GB)
4. `cargo clean` in workspace target (saves ~1.5 GB)
5. Delete all `.log` files at shelf root (saves ~200 MB)
6. Implement `.gitignore` enforcement to prevent recurrence

---

## Worktree Management

### Current State
```
.worktrees/
├── docs/
├── feat/
├── infrastructure/
└── phenotype-errors/
```

### Recommended Structure
```
worktrees/
├── phenotype-infrakit/
│   ├── feat/crate-split/
│   ├── fix/pr-triage/
│   └── chore/perf-optimize/
├── thegent/
│   ├── feat/memory-layer/
│   └── feat/cross-platform/
├── AgilePlus/
│   └── feat/governance-completion/
├── heliosCLI/
│   └── feat/sandboxing/
└── cloud/
    └── feat/service-registry/
```

### Rules
1. One worktree per active feature branch
2. Clean up worktrees after merge (within 48h)
3. Maximum 3 concurrent worktrees per repo
4. Worktree naming: `<repo>/<type>/<description>`

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Breaking changes during crate merges | Medium | High | Full test suite before each merge |
| CI/CD disruption during workflow migration | Medium | Medium | Parallel run old + new workflows for 1 week |
| Disk cleanup breaking builds | Low | Medium | Backup before cleanup, document restore steps |
| Repo deletion removing needed history | Low | High | Full backup before any deletion, use GitHub archive |
| Spec completion blocking downstream work | Medium | High | Prioritize specs by dependency order |
| Agent context loss during long phases | Medium | Medium | Session documentation at each phase boundary |

---

## Success Metrics

| Metric | Current | Phase 1 Target | Phase 2 Target | Phase 3 Target | Phase 4 Target |
|--------|---------|---------------|---------------|---------------|---------------|
| Active repos | 227 | 227 | ~200 | ~195 | ~190 |
| Local disk | 89 GB | 60 GB | 40 GB | 25 GB | 20 GB |
| Build artifacts | 22 GB | 10 GB | 5 GB | 2 GB | 1 GB |
| Open PRs (infrakit) | 10 | 0 | 0 | 0 | 0 |
| Incomplete specs | ~15 | 5 | 0 | 0 | 0 |
| CI coverage | ~30% | 50% | 80% | 95% | 100% |
| Template adoption | ~20% | 40% | 70% | 90% | 100% |
| Published packages | 0 | 0 | 8 | 15 | 20+ |
| Docs federation | None | Planned | In progress | Live | Complete |

---

## Next Actions

1. **Create AgilePlus spec**: `agileplus specify --title "polyrepo-stabilization" --description "4-phase stabilization plan for 227-repo ecosystem"`
2. **Begin Phase 1, Day 1**: Clean build artifacts, close PRs, delete test repos
3. **Set up tracking**: Create this document as `docs/stabilization/STRATEGY.md` in the shelf
4. **Assign owners**: Forge handles implementation, Muse reviews, Sage investigates unknowns
5. **Weekly check-ins**: Update this document with progress at each phase boundary
