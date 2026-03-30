# Master Planning Roadmap: Fork Consolidation Initiative

**Created:** 2026-03-30
**Owner:** Phenotype Organization
**Status:** Planning Phase
**Timeline:** 6+ weeks (April-May 2026)
**Related Docs:** `plans/2026-03-29-FORK_EXECUTION_PLAN-v3.md`, `PLAN.md`, `CLAUDE.md`

---

## Executive Summary

This document establishes a phased roadmap for consolidating three related projects (**forgecode-fork**, **phenotype-router-monitor**, **bifrost-routing**) into a unified Phenotype ecosystem with AgilePlus spec governance and integration across the ecosystem (AgilePlus, phenotype-docs, CI/CD).

### Key Outcomes
- Unified codebase for routing, forking, and custom provider infrastructure
- AgilePlus spec-driven development with full traceability
- Cross-project integration (dashboards, CI/CD pipelines)
- Production-ready releases with CHANGELOG automation
- Reusable Rust crates extracted to phenotype-infrakit

### Timeline & Effort
- **Phase 1:** 1 week (repo setup + AgilePlus initialization)
- **Phase 2:** 2 weeks (consolidation + dependency management)
- **Phase 3:** 2 weeks (extensions + custom providers)
- **Phase 4:** 1+ weeks (long-term integration)
- **Total:** 6+ weeks (April 1 - May 15, 2026)

---

## Phase 1: Foundation & Repository Setup (Week 1: April 1-5)

### Overview
Establish clean repository state, initialize AgilePlus specs, and prepare development environment.

### Deliverables

#### 1.1: Repository Cleanup & Preparation
**Status:** 🔴 Pending
**Owner:** TBD
**Effort:** 2-3 tool calls, ~30 min

**Tasks:**
- [ ] Return phenotype-infrakit to `main` branch (currently on `fix/prd-e5-consistency`)
  - Clean up uncommitted changes in `crates/phenotype-telemetry/` and `docs/worklogs/`
  - Archive `crates/agileplus-error-core/` if not ready
- [ ] Verify forgecode-fork directory structure
  - Confirm `.agileplus/` and `docs/` directories exist
  - Create placeholder specs if missing
- [ ] Create git worktree paths for parallel development
  - `worktrees/forgecode-fork/main-integration`
  - `worktrees/phenotype-router-monitor/setup`
  - `worktrees/bifrost-routing/scaffold` (conditional)

**Success Criteria:**
- ✅ phenotype-infrakit clean working tree on `main`
- ✅ forgecode-fork directory structure validated
- ✅ Worktree paths created and ready for feature work
- ✅ No uncommitted changes in canonical repos

**Blockers:** None (foundation phase)

---

#### 1.2: AgilePlus Initialization
**Status:** 🔴 Pending
**Owner:** TBD
**Effort:** 4-6 tool calls, ~1 hour

**Tasks:**
- [ ] Create master AgilePlus spec for consolidation initiative
  - **Title:** "Fork Consolidation: forgecode-fork + phenotype-router-monitor + bifrost-routing"
  - **Command:** `agileplus specify --title "..." --description "..."`
  - **Epic structure:**
    - E1: Repository Setup & Governance (1.1, 1.2)
    - E2: Code Consolidation (2.1, 2.2, 2.3)
    - E3: Extension & Provider Support (3.1, 3.2, 3.3)
    - E4: Production Integration (4.1, 4.2)
- [ ] Create phase-specific specs
  - Phase 1 spec: Repository setup + AgilePlus governance
  - Phase 2 spec: Code consolidation + API design
  - Phase 3 spec: Custom providers + subagent hooks
  - Phase 4 spec: Release + cross-project integration
- [ ] Define work packages (WP) for each phase
  - WP01-04 for Phase 1
  - WP05-12 for Phase 2
  - WP13-18 for Phase 3
  - WP19+ for Phase 4
- [ ] Link specs to kitty-specs registry
  - Create index entry: `kitty-specs/fork-consolidation-2026-Q2/`
  - Link master spec to AgilePlus workspace

**Acceptance Criteria:**
- ✅ Master AgilePlus spec created with 4 epics
- ✅ All phases have linked specs
- ✅ Work packages assigned with initial state (planned)
- ✅ kitty-specs registry updated
- ✅ AgilePlus CLI queries show consolidated initiative

**Blockers:** AgilePlus workspace must be accessible

---

#### 1.3: Documentation Foundation
**Status:** 🔴 Pending
**Owner:** TBD
**Effort:** 2-3 tool calls, ~45 min

**Tasks:**
- [ ] Create `FORK_CONSOLIDATION_ROADMAP.md` in forgecode-fork
  - Outline phases 1-4
  - Link to AgilePlus specs
  - Include DAG visualization
- [ ] Update forgecode-fork/README.md
  - Point to this master roadmap
  - Add phase status checklist
- [ ] Create docs/GOVERNANCE.md in forgecode-fork
  - AgilePlus spec requirements
  - Git worktree discipline
  - Commit message conventions
- [ ] Add CONTRIBUTING.md guidelines
  - How to reference specs in code
  - Quality gates (test, lint, doc requirements)

**Success Criteria:**
- ✅ Roadmap doc created and linked
- ✅ All governance docs in place
- ✅ Developer onboarding path clear

**Blockers:** None

---

#### 1.4: Environment & CI Setup
**Status:** 🔴 Pending
**Owner:** TBD
**Effort:** 3-4 tool calls, ~1 hour

**Tasks:**
- [ ] Prepare CI/CD pipeline structure
  - Create `.github/workflows/` for forgecode-fork (GitHub Actions)
  - Define build, lint, test, coverage stages
  - Configure branch protection rules (main: require status checks + review)
- [ ] Set up local dev environment checks
  - Create `Makefile` or `justfile` with common tasks
    - `make test` - Run all tests
    - `make lint` - Run linting (ESLint, Prettier, TypeScript)
    - `make build` - Build project
    - `make coverage` - Generate coverage reports
  - Document Node.js, package manager (npm/pnpm/bun) versions
- [ ] Configure secrets management
  - Document required GitHub Secrets (if any)
  - Set up `.env.example` for local development

**Acceptance Criteria:**
- ✅ CI/CD workflows scaffolded
- ✅ Local dev commands working
- ✅ All tests pass locally
- ✅ Coverage reporting configured

**Blockers:** None (can defer secrets if not needed)

---

### Phase 1 Success Metrics

| Metric | Target | Status |
|--------|--------|--------|
| AgilePlus specs created | 4 phase specs | 🔴 Pending |
| Work packages defined | 25+ WPs | 🔴 Pending |
| Repositories ready | 3 clean directories | 🔴 Pending |
| Documentation complete | Governance + roadmap | 🔴 Pending |
| CI/CD scaffolded | Build + test pipelines | 🔴 Pending |
| Local dev working | `make test` passes | 🔴 Pending |

### Phase 1 Dependency DAG

```
START
  ├─→ 1.1: Repo Cleanup (2-3 calls)
  ├─→ 1.2: AgilePlus Init (4-6 calls) ← DEPENDS ON: 1.1
  ├─→ 1.3: Docs Foundation (2-3 calls) ← DEPENDS ON: 1.2
  └─→ 1.4: CI Setup (3-4 calls)
        ↓
    Phase 1 COMPLETE (10-16 calls, ~3-4 hours wall-clock)
        ↓
    PHASE 2 UNBLOCKED ✓
```

---

## Phase 2: Code Consolidation & Integration (Weeks 2-3: April 8-19)

### Overview
Merge vibeproxy variants into phenotype-router-monitor, consolidate dependencies, and design unified API.

### Deliverables

#### 2.1: Repository Cloning & Fork Analysis
**Status:** 🔴 Pending
**Owner:** TBD
**Effort:** 6-8 tool calls, ~2 hours

**Prerequisites:** Phase 1 complete

**Tasks:**
- [ ] Clone upstream forgecode repository
  - `git clone https://github.com/KooshaPari/forgecode forgecode-fork/upstream`
  - Merge upstream code into local fork directory
  - Verify code structure matches expectations
- [ ] Analyze forgecode codebase
  - Identify extension points (providers, hooks, plugins)
  - Map custom provider interface requirements
  - Document subagent integration points
  - Estimate effort for custom provider implementation
- [ ] Scaffold phenotype-router-monitor repository
  - Create `crates/phenotype-router-monitor/` with Cargo.toml
  - Add to workspace members in root Cargo.toml
  - Create `src/lib.rs` with port trait skeleton
  - Add placeholder modules:
    - `ports/router.rs` - Router trait definition
    - `ports/metering.rs` - API metering port
    - `adapters/` - Placeholder for implementations
- [ ] Analyze existing vibeproxy implementations
  - Locate vibeproxy variants in codebase
  - Map common patterns (routing logic, metering, LB algorithms)
  - Identify duplication candidates
  - Estimate consolidation effort

**Acceptance Criteria:**
- ✅ forgecode cloned and analyzed
- ✅ phenotype-router-monitor scaffold created
- ✅ Duplication map documented
- ✅ Extension points identified

**Blockers:** GitHub access required for cloning

---

#### 2.2: API Design & Port Definitions
**Status:** 🔴 Pending
**Owner:** TBD
**Effort:** 4-6 tool calls, ~1.5 hours

**Prerequisites:** 2.1 complete

**Tasks:**
- [ ] Design Router port trait (hexagonal architecture)
  ```rust
  // phenotype-router-monitor/src/ports/router.rs
  #[async_trait]
  pub trait Router: Send + Sync {
      async fn route(&self, request: &Request) -> Result<Route>;
      async fn metrics(&self) -> Result<Metrics>;
      async fn health_check(&self) -> Result<HealthStatus>;
  }
  ```
- [ ] Design API metering port
  ```rust
  // phenotype-router-monitor/src/ports/metering.rs
  pub trait ApiMeteringEngine: Send + Sync {
      fn record_request(&self, key: &str, quota: u64) -> Result<()>;
      fn check_quota(&self, key: &str) -> Result<QuotaStatus>;
      fn reset_quota(&self, key: &str) -> Result<()>;
  }
  ```
- [ ] Design LB/routing strategy port
  ```rust
  // phenotype-router-monitor/src/ports/load_balancer.rs
  pub trait LoadBalancingStrategy: Send + Sync {
      fn select_backend(&self, backends: &[Backend]) -> Result<Backend>;
      fn record_outcome(&self, backend: &Backend, success: bool) -> Result<()>;
  }
  ```
- [ ] Document adapter implementations needed
  - List target adapters for each port
  - Map vibeproxy variants to adapter implementations
  - Plan implementation order (priority P0, P1, P2)

**Acceptance Criteria:**
- ✅ 3+ port traits defined with clear contracts
- ✅ Adapter roadmap created
- ✅ Code compiles (stub implementations ok)
- ✅ All traits reviewed and documented

**Blockers:** None (design phase)

---

#### 2.3: Dependency Consolidation
**Status:** 🔴 Pending
**Owner:** TBD
**Effort:** 3-4 tool calls, ~1 hour

**Prerequisites:** 2.1, 2.2 complete

**Tasks:**
- [ ] Audit Rust dependencies
  - Extract common deps from vibeproxy implementations
  - Run `cargo tree` analysis for duplication
  - Create consolidation plan for Cargo.toml
- [ ] Audit Node.js dependencies (forgecode)
  - Analyze package.json in upstream forgecode
  - Check for outdated packages (>1 major version behind)
  - Plan update strategy (bump cutting-edge versions)
  - Document security audit results
- [ ] Create unified dependency manifests
  - Root `Cargo.toml` with workspace members
  - `Cargo.lock` for reproducible builds
  - `package.json` + `bun.lock` (if using bun) or equivalent
- [ ] Document dependency versions
  - Create `DEPENDENCIES.md`
  - List all major crates/packages with version constraints
  - Note security considerations

**Acceptance Criteria:**
- ✅ Unified Cargo.toml created
- ✅ All deps up-to-date (cutting-edge)
- ✅ Build passes with new deps
- ✅ DEPENDENCIES.md created

**Blockers:** None

---

#### 2.4: AgilePlus Integration & Traceability
**Status:** 🔴 Pending
**Owner:** TBD
**Effort:** 2-3 tool calls, ~45 min

**Prerequisites:** 2.1, 2.2, 2.3 complete

**Tasks:**
- [ ] Create Functional Requirements (FR) for consolidation
  - FR-ROUTER-001: Router port trait implementation
  - FR-ROUTER-002: API metering engine
  - FR-ROUTER-003: Load balancer strategy
  - FR-ROUTER-004: Consolidated adapter implementations
- [ ] Map vibeproxy variants to FRs
  - Each variant becomes a test case linked to FR
  - Document migration path from old code to new
- [ ] Update AgilePlus work packages
  - Set Phase 2 WP status to "in-progress"
  - Link code commits to WP IDs (git commit message format)
  - Track test coverage per FR
- [ ] Create traceability matrix
  - `docs/TRACEABILITY_MATRIX.md`
  - Columns: FR-ID | Description | Tests | Implementation | Status

**Acceptance Criteria:**
- ✅ 4+ FRs created for Phase 2 deliverables
- ✅ Each FR has >=1 test
- ✅ Traceability matrix updated
- ✅ AgilePlus WPs linked to commits

**Blockers:** AgilePlus workspace access required

---

### Phase 2 Success Metrics

| Metric | Target | Status |
|--------|--------|--------|
| Repos cloned & scaffolded | 3/3 | 🔴 Pending |
| Port traits defined | 3+ | 🔴 Pending |
| Duplication identified | % LOC savings | 🔴 Pending |
| Dependencies consolidated | 1 unified manifest | 🔴 Pending |
| FRs created | 4+ | 🔴 Pending |
| Tests written | 1 per FR | 🔴 Pending |
| Build passes | Clean build + test | 🔴 Pending |

### Phase 2 Dependency DAG

```
PHASE 1 COMPLETE
  ↓
  2.1: Clone & Analyze (6-8 calls)
  ↓
  ├─→ 2.2: API Design (4-6 calls) ← DEPENDS ON: 2.1
  ├─→ 2.3: Dependencies (3-4 calls) ← DEPENDS ON: 2.1
  └─→ 2.4: AgilePlus (2-3 calls) ← DEPENDS ON: 2.2, 2.3
        ↓
    Phase 2 COMPLETE (15-21 calls, ~5-6 hours wall-clock)
        ↓
    PHASE 3 UNBLOCKED ✓
```

---

## Phase 3: Extensions & Custom Providers (Weeks 4-5: April 22-May 3)

### Overview
Implement custom provider framework and subagent integration hooks.

### Deliverables

#### 3.1: Custom Provider Framework
**Status:** 🔴 Pending
**Owner:** TBD
**Effort:** 8-10 tool calls, ~2.5 hours

**Prerequisites:** Phase 2 complete

**Tasks:**
- [ ] Implement custom provider port trait
  ```rust
  // forgecode-fork/src/providers/port.rs
  pub trait CustomProvider: Send + Sync {
      fn name(&self) -> &str;
      fn generate(&self, context: &CodeContext) -> Result<Code>;
      fn analyze(&self, code: &str) -> Result<Analysis>;
      fn supports(&self, language: &Language) -> bool;
  }
  ```
- [ ] Create provider registry
  ```rust
  // forgecode-fork/src/providers/registry.rs
  pub struct ProviderRegistry {
      providers: HashMap<String, Arc<dyn CustomProvider>>,
  }

  impl ProviderRegistry {
      pub fn register(&mut self, provider: Arc<dyn CustomProvider>) -> Result<()>;
      pub fn get(&self, name: &str) -> Result<Arc<dyn CustomProvider>>;
      pub fn list_providers(&self) -> Vec<&str>;
  }
  ```
- [ ] Implement example custom providers (2-3)
  - PhenotypeSpecProvider: Generates specs from code
  - TestGeneratorProvider: Auto-generates test cases
  - DocumentationProvider: Generates markdown docs
- [ ] Add configuration support
  - TOML provider configuration files
  - Provider-specific settings (language, style, etc.)
  - Environment variable overrides
- [ ] Write integration tests
  - Test provider registration
  - Test custom provider invocation
  - Test error handling (unknown provider, unsupported language)

**Acceptance Criteria:**
- ✅ CustomProvider trait defined
- ✅ ProviderRegistry implemented
- ✅ 2-3 example providers created
- ✅ Configuration system working
- ✅ Integration tests pass (>=80% coverage)

**Blockers:** None (builds on Phase 2 APIs)

---

#### 3.2: Subagent Integration Hooks
**Status:** 🔴 Pending
**Owner:** TBD
**Effort:** 6-8 tool calls, ~2 hours

**Prerequisites:** 3.1 complete

**Tasks:**
- [ ] Design subagent port trait
  ```rust
  // forgecode-fork/src/agents/port.rs
  #[async_trait]
  pub trait SubagentHook: Send + Sync {
      async fn on_provider_invoked(&self, event: &ProviderInvokedEvent) -> Result<()>;
      async fn on_generation_complete(&self, event: &GenerationCompleteEvent) -> Result<()>;
      async fn on_error(&self, event: &ErrorEvent) -> Result<()>;
  }
  ```
- [ ] Implement event system
  ```rust
  // forgecode-fork/src/agents/events.rs
  pub struct ProviderInvokedEvent {
      provider_name: String,
      context: CodeContext,
      timestamp: DateTime<Utc>,
  }

  pub struct GenerationCompleteEvent {
      provider_name: String,
      generated_code: Code,
      metrics: Metrics,
      timestamp: DateTime<Utc>,
  }
  ```
- [ ] Create hook registry
  ```rust
  pub struct SubagentHookRegistry {
      hooks: Vec<Arc<dyn SubagentHook>>,
  }
  ```
- [ ] Implement example hooks (2-3)
  - AgilePlusLoggingHook: Log events to AgilePlus
  - MetricsCollectionHook: Collect generation metrics
  - SlackNotificationHook: Send notifications to Slack
- [ ] Add async event dispatch
  - Non-blocking event delivery to hooks
  - Timeout handling for long-running hooks
  - Error suppression (hooks shouldn't crash generator)
- [ ] Write hook tests
  - Test hook invocation
  - Test event propagation
  - Test error handling

**Acceptance Criteria:**
- ✅ SubagentHook trait defined
- ✅ Event types defined (3+)
- ✅ Hook registry implemented
- ✅ 2-3 example hooks created
- ✅ Async dispatch working
- ✅ Hook tests pass (>=75% coverage)

**Blockers:** None (builds on Phase 2 APIs)

---

#### 3.3: Integration Testing & Documentation
**Status:** 🔴 Pending
**Owner:** TBD
**Effort:** 5-6 tool calls, ~1.5 hours

**Prerequisites:** 3.1, 3.2 complete

**Tasks:**
- [ ] Write end-to-end tests
  - Test custom provider with subagent hooks
  - Verify event propagation through hook system
  - Test error scenarios (provider failure, hook timeout)
  - Test concurrent provider invocations
- [ ] Create usage documentation
  - `docs/CUSTOM_PROVIDERS.md` - How to write custom providers
  - `docs/SUBAGENT_HOOKS.md` - How to write subagent hooks
  - `docs/INTEGRATION_GUIDE.md` - Integrating with AgilePlus
- [ ] Add code examples
  - Example custom provider (source code)
  - Example subagent hook (source code)
  - Example configuration file
- [ ] Generate API documentation
  - `cargo doc` for Rust crates
  - TSDoc for TypeScript (if applicable)
  - Publish to gh-pages
- [ ] Create developer onboarding guide
  - `docs/DEVELOPER_GUIDE.md` - Getting started
  - Setup instructions
  - Common tasks (add provider, add hook, run tests)
  - Troubleshooting guide

**Acceptance Criteria:**
- ✅ E2E tests pass (>=80% coverage)
- ✅ Developer documentation complete
- ✅ API docs generated
- ✅ Examples provided
- ✅ Onboarding guide clear and complete

**Blockers:** None (final integration phase)

---

### Phase 3 Success Metrics

| Metric | Target | Status |
|--------|--------|--------|
| Custom provider framework | Implemented | 🔴 Pending |
| Example providers | 2-3 | 🔴 Pending |
| Subagent hooks | Implemented | 🔴 Pending |
| Example hooks | 2-3 | 🔴 Pending |
| E2E tests | Pass (80%+ coverage) | 🔴 Pending |
| Documentation | Complete | 🔴 Pending |
| API docs | Published | 🔴 Pending |

### Phase 3 Dependency DAG

```
PHASE 2 COMPLETE
  ↓
  3.1: Custom Providers (8-10 calls)
  ↓
  3.2: Subagent Hooks (6-8 calls) ← DEPENDS ON: 3.1
  ↓
  3.3: Integration & Docs (5-6 calls) ← DEPENDS ON: 3.1, 3.2
        ↓
    Phase 3 COMPLETE (19-24 calls, ~6-7 hours wall-clock)
        ↓
    PHASE 4 UNBLOCKED ✓
```

---

## Phase 4: Production Integration & Long-Term (Week 6+: May 6+)

### Overview
Cross-project integration, CI/CD automation, release management, and long-term sustainability.

### Deliverables

#### 4.1: Cross-Project Dashboard Integration
**Status:** 🔴 Pending
**Owner:** TBD
**Effort:** 6-8 tool calls, ~2 hours

**Prerequisites:** Phase 3 complete

**Tasks:**
- [ ] AgilePlus dashboard integration
  - Create AgilePlus widgets for forgecode-fork status
  - Display custom provider metrics
  - Show subagent hook invocation history
  - Link to PR/commit details
  - Add "Live" monitoring (updated every 5s)
- [ ] Phenotype-docs integration
  - Create documentation site for forgecode-fork
  - Include API reference (auto-generated from code)
  - Include custom provider tutorials
  - Include subagent hook examples
  - Link from phenotype-docs main navigation
- [ ] CI/CD integration
  - Add build status badge to README.md
  - Create GitHub releases from git tags
  - Publish artifacts (compiled binaries, Docker images if applicable)
  - Trigger downstream CI in dependent repos

**Acceptance Criteria:**
- ✅ AgilePlus widgets created and tested
- ✅ Phenotype-docs pages published
- ✅ CI/CD pipelines integrated
- ✅ Build status visible in dashboards
- ✅ Artifact publishing automated

**Blockers:** AgilePlus & phenotype-docs repo access required

---

#### 4.2: Release Management & CHANGELOG Automation
**Status:** 🔴 Pending
**Owner:** TBD
**Effort:** 4-5 tool calls, ~1 hour

**Prerequisites:** 4.1 complete

**Tasks:**
- [ ] Set up git-cliff for CHANGELOG automation
  - Create `cliff.toml` configuration
  - Define commit message convention (Conventional Commits)
  - Generate CHANGELOG from commit history
  - Configure release tags (SemVer format: v1.0.0)
- [ ] Automate version bumping
  - Update Cargo.toml versions on release
  - Update package.json versions (forgecode fork)
  - Create git tags automatically
  - Push tags to GitHub
- [ ] Create release workflow
  - GitHub Actions workflow for release tagging
  - Auto-generate release notes from CHANGELOG
  - Build and publish artifacts
  - Notify downstream projects (if applicable)
- [ ] Document release process
  - `docs/RELEASE_PROCESS.md` - How to cut a release
  - Semantic Versioning guide
  - Emergency patch release procedure
  - Rollback procedure

**Acceptance Criteria:**
- ✅ git-cliff configured
- ✅ Version bumping automated
- ✅ Release workflow tested
- ✅ Release process documented
- ✅ First automated release successful (v1.0.0 or v0.1.0)

**Blockers:** None (final production step)

---

#### 4.3: Long-Term Monitoring & Maintenance
**Status:** 🔴 Pending
**Owner:** TBD
**Effort:** 2-3 tool calls per cycle (recurring)

**Prerequisites:** 4.1, 4.2 complete

**Tasks:**
- [ ] Set up dependency update monitoring
  - Dependabot (GitHub) for Rust + Node.js
  - Weekly dependency audit reports
  - Auto-update policy (security patches: immediate, minor: weekly, major: quarterly)
- [ ] Configure code quality tracking
  - CodeQL for security analysis
  - Coverage tracking (target: 80%+)
  - Complexity metrics (cyclomatic complexity)
  - Dead code detection
- [ ] Create health checks
  - Liveness probe for phenotype-router-monitor
  - Performance benchmarks for routing engine
  - Metrics dashboard (Grafana or AgilePlus)
- [ ] Plan next phases
  - Identify new features from user feedback
  - Plan performance optimizations
  - Design scale-out strategy (multi-region routing)
  - Evaluate bifrost-routing integration

**Acceptance Criteria:**
- ✅ Dependency updates automated
- ✅ Code quality metrics visible
- ✅ Health checks passing
- ✅ Monitoring dashboard created
- ✅ Next phase planning completed

**Blockers:** None (ongoing maintenance)

---

### Phase 4 Success Metrics

| Metric | Target | Status |
|--------|--------|--------|
| Dashboard widgets | 3+ | 🔴 Pending |
| Docs site published | Live | 🔴 Pending |
| CI/CD integrated | Auto-deploy | 🔴 Pending |
| CHANGELOG automated | Generated from commits | 🔴 Pending |
| First release | v1.0.0 or v0.1.0 | 🔴 Pending |
| Dependency updates | Automated | 🔴 Pending |
| Code quality | 80%+ coverage | 🔴 Pending |

### Phase 4 Dependency DAG

```
PHASE 3 COMPLETE
  ↓
  4.1: Dashboard Integration (6-8 calls)
  ↓
  4.2: Release Management (4-5 calls) ← DEPENDS ON: 4.1
  ↓
  4.3: Monitoring & Maintenance (2-3 calls/cycle) ← DEPENDS ON: 4.2
        ↓
    PRODUCTION READY ✓
```

---

## Master Dependency DAG

Complete dependency graph for all phases:

```
START (2026-04-01)
  ↓
  PHASE 1: Foundation (1-4 weeks)
    1.1: Repo Cleanup
    1.2: AgilePlus Init ← 1.1
    1.3: Docs Foundation ← 1.2
    1.4: CI Setup
  ├─ Completion: ~3-4 hours wall-clock
  └─→ PHASE 1 COMPLETE (2026-04-05)
        ↓
  PHASE 2: Consolidation (2-3 weeks)
    2.1: Clone & Analyze
    2.2: API Design ← 2.1
    2.3: Dependencies ← 2.1
    2.4: AgilePlus ← 2.2, 2.3
  ├─ Completion: ~5-6 hours wall-clock
  └─→ PHASE 2 COMPLETE (2026-04-19)
        ↓
  PHASE 3: Extensions (2 weeks)
    3.1: Custom Providers
    3.2: Subagent Hooks ← 3.1
    3.3: Integration & Docs ← 3.1, 3.2
  ├─ Completion: ~6-7 hours wall-clock
  └─→ PHASE 3 COMPLETE (2026-05-03)
        ↓
  PHASE 4: Production (1+ weeks)
    4.1: Dashboard Integration
    4.2: Release Management ← 4.1
    4.3: Monitoring & Maintenance ← 4.2 (ongoing)
  ├─ Completion: ~3 hours wall-clock + ongoing
  └─→ PRODUCTION READY (2026-05-15)

CRITICAL PATH: 1.1 → 1.2 → 2.1 → 2.2 → 3.1 → 3.2 → 4.1 → 4.2
CRITICAL PATH EFFORT: 15-23 calls, ~20-25 hours total wall-clock (6 weeks)
```

---

## Global Success Criteria (Exit Gates)

### Phase 1 Exit Gate
- [ ] AgilePlus specs created (4+ phases)
- [ ] Repositories ready (clean state)
- [ ] Documentation foundation in place
- [ ] CI/CD scaffolded and working
- [ ] Team onboarded and ready

### Phase 2 Exit Gate
- [ ] Repositories cloned and scaffolded
- [ ] Port traits designed and reviewed
- [ ] Dependencies consolidated
- [ ] Build passing with all tests
- [ ] Duplication analysis complete

### Phase 3 Exit Gate
- [ ] Custom provider framework working
- [ ] Subagent hooks implemented
- [ ] End-to-end tests passing (80%+ coverage)
- [ ] Developer documentation complete
- [ ] API documentation published

### Phase 4 Exit Gate
- [ ] Cross-project integration complete
- [ ] CI/CD fully automated
- [ ] Release automation working
- [ ] Monitoring and health checks active
- [ ] First production release shipped

---

## Risk Register & Mitigation

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|-----------|
| **AgilePlus unavailability** | Blocks spec tracking | Low | Fallback to GitHub Issues + manual tracking |
| **Scope creep (bifrost-routing)** | Delays Phase 1-2 | Medium | Keep bifrost conditional; prioritize forgecode-fork |
| **Dependency version conflicts** | Build failures | Medium | Early dependency audit (Phase 2.3); use lockfiles |
| **Custom provider API churn** | Requires rework | Medium | Stable API design in Phase 2; community review before Phase 3 |
| **Integration complexity (AgilePlus, docs)** | Phase 4 delays | Medium | Early spike in Phase 1; document integration points |
| **Team availability** | Extends timeline | Low | Parallel task structure; subagent delegation recommended |
| **GitHub Actions billing** | CI failures | Low | Use local runners or skip billed runners (macOS/Windows) |

---

## Resource Requirements

### Team & Skills
- **Rust developer** (2-3 agents): Port trait design, adapter implementations, testing
- **TypeScript/JavaScript developer** (1-2 agents): forgecode fork customization, provider implementation
- **DevOps/CI-CD specialist** (1 agent): GitHub Actions, release automation, deployment
- **Documentation specialist** (1 agent): API docs, guides, tutorials
- **AgilePlus admin** (0.5 agent): Spec management, WP tracking

### Tools & Infrastructure
- **GitHub**: Repository hosting, CI/CD, artifact storage
- **AgilePlus**: Spec management and traceability
- **Cargo** (Rust): Build system
- **npm/bun** (Node.js): JavaScript package manager
- **git-cliff**: CHANGELOG automation
- **Conventional Commits**: Commit message standard

### Time Budget
- **Phase 1:** 10-16 tool calls, 3-4 hours
- **Phase 2:** 15-21 tool calls, 5-6 hours
- **Phase 3:** 19-24 tool calls, 6-7 hours
- **Phase 4:** 12-16 tool calls, 3+ hours (recurring)
- **Total:** 56-77 tool calls, ~20-25 hours wall-clock

---

## Communication Plan

### Status Updates
- **Weekly** (every Friday): Phase progress summary to stakeholders
- **Per-phase completion**: Full deliverable review + sign-off
- **Blockers/risks**: Real-time notification to team lead

### Stakeholder Map
| Stakeholder | Role | Frequency |
|-------------|------|-----------|
| User/Project Owner | Approver | Weekly review |
| AgilePlus Admin | Spec Manager | Per-phase |
| Rust Team Lead | Code Review | Per-task |
| DevOps Team | CI/CD Owner | Per-phase |
| Documentation Lead | Doc Review | Per-phase |

### Integration Points
- **AgilePlus**: Daily WP status updates via CLI
- **GitHub**: PR reviews + status checks
- **phenotype-docs**: Doc site updates (end of Phase 3)
- **Downstream repos**: Notification when new crates available (Phase 2+)

---

## Appendix: Related Documentation

| Document | Location | Purpose |
|----------|----------|---------|
| Fork Execution Plan v3 | `plans/2026-03-29-FORK_EXECUTION_PLAN-v3.md` | Previous planning iteration |
| Project PLAN | `PLAN.md` | Overall project roadmap |
| CLAUDE.md | `CLAUDE.md` | Governance & project rules |
| AgilePlus Governance | `AGENTS.md` in AgilePlus repo | Spec management standards |
| Phenotype Cross-Reuse Protocol | Global CLAUDE.md | Shared code extraction guidelines |

---

## Version History

| Date | Version | Author | Status |
|------|---------|--------|--------|
| 2026-03-30 | 1.0 | Planning Agent | Draft |
| TBD | 1.1 | Team Lead | Review |
| TBD | 2.0 | Project Owner | Approved |

---

## Next Steps

1. **Immediate (Next 24 hours):**
   - Review this roadmap with stakeholders
   - Get approval to proceed with Phase 1
   - Assign team members to Phase 1 tasks

2. **Phase 1 Kickoff (April 1):**
   - Start task 1.1 (Repo cleanup)
   - Initialize AgilePlus specs (task 1.2)
   - Begin documentation (task 1.3)

3. **Phase 1 Completion (April 5):**
   - Verify all exit gates passing
   - Conduct Phase 1 review + sign-off
   - Release Phase 2 tasks

---

**End of Master Roadmap**
