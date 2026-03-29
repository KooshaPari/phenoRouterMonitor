# Federated Hybrid Architecture Patterns — Application to Phenotype

**Date**: 2026-03-29
**Context**: User explored 4 standard patterns + 4 new unique hybrid patterns for expanding current repo integration strategy.

---

## Current State: Isolated Repos with Fork Strategy

**Today's Architecture**:
- 20+ independent repositories under KooshaPari GitHub org
- Fork strategy: hyper-modify/extend upstream (aizen, vibeproxy, zen, etc.)
- Worktree-based development: `repos/<project>-wtrees/<topic>`
- Canonical repos on `main` only; feature work in worktrees
- Cross-repo integration: submodules (@phenotype/docs), npm packages (@phenotype scope), shared modules (phenotype-shared, phenotype-go-kit)
- **Problem**: Repos are physically separate but logically isolated — no unified runtime experience

---

## 4 Standard Patterns: Which Apply Now?

| Pattern | Current Use | Expansion Opportunity |
|---------|-------------|----------------------|
| **Module Federation** (runtime) | Not in use | **AgilePlus + heliosApp**: Front-end micro-frontends; agents load domain-specific dashboards dynamically |
| **Micro-Frontends** (domains) | Emerging | **Yes**: AgilePlus (core), heliosApp (mobile), each with own stacks; merge into "unified shell" |
| **Git Submodules + Monorepo** (build-time) | **In use** (phenotype-docs) | **Expand**: use Nx/Turborepo for core monorepo boundary + submodules for third-party/experimental |
| **Sidecar Pattern** (services) | **Partially in use** (MCP servers) | **Expand**: Logging/auth sidecars for independent service stacks; ensure consistency standalone vs. integrated |

---

## 4 New Unique Patterns: How They Apply

### 1. Platform-Centric "Chassis"

**What it is**: Core app provides shared identity, design system, and data layer. Remote repos inherit capabilities via standardized interfaces.

**Applies to Phenotype**:
- **@phenotype/docs** becomes the "Chassis" for design language/theming
- **AgilePlus** becomes the "Chassis" for spec-driven delivery (FR traceability, worklog structure, versioning)
- **Remote repos** (heliosApp, agent-wave, etc.) adopt @phenotype/docs theme + AgilePlus spec structure
- **Benefit**: Sub-apps stay small, focused on business logic. No design/spec reinvention.

**Implementation**:
```
@phenotype/docs (Chassis)
├── Design tokens, theme
├── VitePress config builder (createPhenotypeConfig)
└── Interfaces: UI components, color system, typography

AgilePlus (Meta-Chassis for governance)
├── Spec framework (PRD, ADR, FR, PLAN)
├── Worklog structure
└── Versioning strategy

All other repos
├── Import @phenotype/docs theme
├── Follow AgilePlus spec pattern
└── Inherit governance from CLAUDE.md
```

---

### 2. Intent-Driven Orchestration

**What it is**: Core app uses an orchestrator that loads modules based on user intent. Queries a registry instead of hardcoding which repo goes where.

**Applies to Phenotype**:
- **Agent Registry**: Instead of hardcoding "agent-wave handles forecasting," system queries registry: "who handles forecasting intent?"
- **Service Discovery**: heliosApp requests "authentication" from registry → loaded from auth-sidecar or phenotype-shared
- **Dynamic Dashboard**: AgilePlus orchestrates "show me all work items" → queries phenotype-shared, cliproxyapi-plusplus, agentapi-plusplus for data
- **Benefit**: Core app infinitely extensible without central config file changes. Add new service, register it, system uses it.

**Implementation**:
```
Registry (service-mesh or YAML config)
├── "forecasting" → agent-wave service + MCP
├── "auth" → phenotype-shared:auth module
├── "work-item-persistence" → AgilePlus:database
└── "ui-rendering" → @phenotype/docs:renderer

AgilePlus Orchestrator
├── Queries registry by intent
├── Loads service dynamically at runtime
└── Falls back if service unavailable
```

---

### 3. Domain-Aligned Modular Monoliths

**What it is**: Monorepo tooling (Nx/Turborepo) enforces strict boundaries in core, while experimental/third-party features stay in separate repos and are pulled in via federation.

**Applies to Phenotype**:
- **Core Monorepo Boundary** (phenotype-infrakit): AgilePlus + phenotype-shared + governance docs + spec framework (strict layer boundaries enforced by linter)
- **Domain Repos** (stay separate): heliosApp (mobile), agent-wave (forecasting), civ (compliance), aizen (identity) — independent development, pulled in via Module Federation
- **Benefit**: Consistency and testability where you need it (core); speed and autonomy where you don't (domains)

**Implementation**:
```
phenotype-infrakit (Monorepo with Nx)
├── Layer 1 (Platform): AgilePlus, governance
├── Layer 2 (Shared): phenotype-shared, @phenotype/docs
├── Layer 3 (Infra): phenotype-go-kit, phenotype-go-auth
└── Enforced: import-linter rules (Layer 3 cannot import Layer 1)

Domain Repos (stay separate, loaded via Module Federation)
├── heliosApp (mobile stacks, own CI/CD)
├── agent-wave (agent runtime, own tooling)
└── civ (compliance engine, own versioning)
```

---

### 4. Feedback-Loop / AI-Native Architecture

**What it is**: System monitors federated modules, detects failures/slowness, dynamically swaps versions or replicas for "lite" versions or alternative implementations. Self-healing.

**Applies to Phenotype**:
- **Module Health Monitoring**: AgilePlus monitors heliosApp + agent-wave response times. If heliosApp > 2s latency, swaps for "lite" mode (reduced features, static data)
- **Automated Failover**: If phenotype-shared auth service fails, fallback to local JWT cache + retry queue
- **AI-Driven Optimization**: Claude-as-observer watches which modules are slow/failing, suggests: "consolidate heliosApp + agent-wave into phenotype-shared features for faster boot"
- **Benefit**: Self-healing, high availability, continuous optimization without manual intervention

**Implementation**:
```
AgilePlus Health Monitor (runs every 30s)
├── Ping heliosApp, agent-wave, phenotype-shared
├── Measure latency, error rate
├── If heliosApp > threshold:
│   └── Swap module for lite-heliosapp (cached data, static UI)
└── Log event → feed to optimizer agent

Optimizer Agent (runs weekly)
├── Analyzes health logs
├── Suggests: "Consider consolidating X + Y"
├── Proposes PR to improve failing module
```

---

## All 4 Hybridized: The "Federated Hybrid" Model

**Combining all patterns creates the ultimate model**:

1. **Build-time** (Git Submodules): Central phenotype-infrakit shell references versioned child repos (via `.gitmodules`)
2. **Runtime** (Module Federation): heliosApp, agent-wave, civ remain standalone but plug into AgilePlus dynamically
3. **Organization** (Micro-Frontends): Each repo has its own team, tech stack, CI/CD; boundaries enforced
4. **Infrastructure** (Sidecar): Auth, logging, tracing sidecars run alongside each app, ensuring consistency
5. **Governance** (Chassis): @phenotype/docs + AgilePlus define shared identity; all repos inherit
6. **Discovery** (Intent-Driven): Services register capabilities; AgilePlus orchestrates by intent
7. **Resilience** (AI-Native): Health monitor swaps modules, optimizer suggests improvements

**Result**: Repos are physically separate (KISS decomposition) but logically unified (seamless experience).

---

## Recommended Implementation Roadmap

### Phase 1: Establish Chassis (Weeks 1-2)
- [ ] Finalize @phenotype/docs theme and config builder
- [ ] Document AgilePlus governance as the meta-chassis
- [ ] Create INTERFACE contracts (how repos inherit from Chassis)
- [ ] Update all repos to use @phenotype/docs + AgilePlus spec structure

### Phase 2: Module Federation (Weeks 3-5)
- [ ] Implement Module Federation in AgilePlus host
- [ ] Configure heliosApp as first remote module
- [ ] Test dynamic loading and standalone execution
- [ ] Add agent-wave as second remote module

### Phase 3: Intent-Driven Registry (Weeks 6-8)
- [ ] Design service registry schema (YAML or database)
- [ ] Implement registry querying in AgilePlus orchestrator
- [ ] Move auth, logging, persistence to registry-based discovery
- [ ] Update documentation with registry patterns

### Phase 4: Health Monitoring + AI-Native (Weeks 9-12)
- [ ] Build AgilePlus health monitor
- [ ] Integrate lite-mode fallbacks for each module
- [ ] Deploy optimizer agent (weekly analysis)
- [ ] Test failover scenarios

---

## Current Blockers & Next Steps

1. **@phenotype/docs Publishing**: Awaiting phenodocs PR #91 merge → GitHub Packages registry setup
2. **Module Federation Setup**: Need to install @module-federation/enhanced in AgilePlus
3. **Service Registry**: Design schema (should registry be Nx.json, separate .registry.yml, or database?)
4. **Agent Registry**: Define how agent-wave, civ, aizen register themselves (MCP server registry? AgilePlus database?)

---

## Decision: What to Prioritize First?

**Option A (Foundation First)**:
Focus on Chassis + Intent-Driven Registry. Gives governance structure and extensibility foundation. Slower but more stable.

**Option B (UI Experience First)**:
Focus on Module Federation + AI-Native Health. Gives users unified seamless experience quickly. Requires more refactoring later.

**Option C (Fork Strategy Expansion)**:
Enhance current fork hyper-modification approach with Sidecar pattern for auth/logging consistency. Lower risk, incremental.

**Recommendation**: **Option A → Option B → Option C** (sequential).

---

## See Also
- `docs/governance/plugin_architecture_governance.md` — Existing plugin patterns
- `docs/reference/SOFTWARE_ARCHITECTURE_REFERENCE.md` — Architecture decision records
- `PLAN.md` in AgilePlus — Spec-driven delivery framework
- `@phenotype/docs` package docs — Theme and governance chassis

