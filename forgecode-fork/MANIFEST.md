# forgecode-fork — Repository Initialization Manifest

**Date**: 2026-03-30
**Status**: Repository structure prepared (ready for git clone)
**Location**: `/Users/kooshapari/CodeProjects/Phenotype/repos/forgecode-fork`

## Directory Structure Created

```
forgecode-fork/
├── README.md                           # Project overview (purpose, extensions, status)
├── MANIFEST.md                         # This file
├── .gitignore                          # Node/build/cache/IDE exclusions
├── .agileplus/
│   └── config.toml                     # AgilePlus configuration (specs location, providers, CI/CD)
└── docs/
    ├── FORK_INTENT.md                  # Custom providers & subagent extensions (5 sections)
    └── ARCHITECTURE.md                 # System design, hexagonal model, integration points (9 sections)
```

## File Descriptions

### Root Files

| File | Purpose | Status |
|------|---------|--------|
| `README.md` | Project overview, structure, next steps | ✅ Created |
| `MANIFEST.md` | This initialization manifest | ✅ Created |
| `.gitignore` | Node, build, dist, cache, IDE, OS exclusions | ✅ Created |

### Documentation

| File | Sections | Status |
|------|----------|--------|
| `docs/FORK_INTENT.md` | 1. Custom Providers (3), 2. Subagent Extensions (3), 3. AgilePlus Integration, 4. Implementation Roadmap, 5. Design Principles, 6. Questions, 7. References | ✅ Created (6.2 KB) |
| `docs/ARCHITECTURE.md` | 1. System Architecture (diagram), 2. Hexagonal Design, 3. Integration Points (4), 4. Modification Strategy, 5. Development Workflow, 6. Testing Strategy, 7. Config Management, 8. CI/CD Integration, 9. References | ✅ Created (8.4 KB) |

### Configuration

| File | Purpose | Status |
|------|---------|--------|
| `.agileplus/config.toml` | Fork-level AgilePlus config with providers, subagent settings, CI/CD, testing, git sync | ✅ Created |

## Content Summary

### Custom Providers (3 planned)

1. **Phenotype Rust Crate Generator**
   - Hexagonal architecture scaffolding
   - SOLID principle templates
   - Test harness generation
   - Workspace integration

2. **AgilePlus Work Package Generator**
   - YAML spec scaffolding
   - FR/ADR cross-references
   - Effort estimation templates
   - Dependency DAG placeholders

3. **xDD Test Generator**
   - Test-first scaffolds (Rust, TypeScript, Python)
   - FR annotation generation
   - BDD/ATDD template support
   - Mutation testing hooks

### Subagent Extensions (3 planned)

1. **Multi-Agent Coordinator**
   - WP distribution across agents
   - Task dependency resolution
   - Merge coordination
   - Conflict detection

2. **Evidence & Audit Trail Generator**
   - Test execution logs
   - Code coverage reports
   - Linter/type check summaries
   - Git commit & CI/CD link embeddings

3. **Schema Versioning & Migration Generator**
   - JSON Schema / Protobuf / GraphQL SDL generation
   - Compatibility matrices
   - Migration script generation
   - Breaking change documentation

### Integration Points

- **Provider Plugin Interface** (Provider trait, ProviderConfig)
- **Subagent Coordination API** (REST endpoints for agent assignment, merge, status)
- **Evidence & Audit Trail** (EvidenceBundle data model, evidence collection)
- **Dashboard REST API** (provider listing, job management, evidence retrieval, agent status)

## AgilePlus Spec Location (Decision)

**Location**: `../kitty-specs/forgecode-fork/`
**Rationale**: forgecode fork is shared infrastructure; specs belong in central kitty-specs location

**Directory Structure** (to be created):
```
kitty-specs/
└── forgecode-fork/
    ├── 001-custom-providers.md
    ├── 002-subagent-extensions.md
    ├── 003-agileplus-dashboard-integration.md
    └── plan.md
```

**Initialization Command**:
```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos/forgecode-fork
agileplus spec --source ../kitty-specs/forgecode-fork/
```

## Next Steps

### Immediate (Ready to Execute)

1. **Clone upstream forgecode**
   ```bash
   cd /Users/kooshapari/CodeProjects/Phenotype/repos/forgecode-fork
   git clone https://github.com/forgecode/forgecode . --depth 1
   ```

2. **Initialize AgilePlus specs** (after upstream clone)
   ```bash
   # Create kitty-specs/forgecode-fork/ directory
   mkdir -p ../kitty-specs/forgecode-fork/

   # Initialize AgilePlus link
   agileplus spec --source ../kitty-specs/forgecode-fork/
   ```

3. **Create initial provider specs** in `../kitty-specs/forgecode-fork/`
   - `001-custom-providers.md` (Provider implementations, integration)
   - `002-subagent-extensions.md` (Coordinator, evidence, schema migration)
   - `003-agileplus-dashboard-integration.md` (Dashboard widgets, REST API)

### Phase 1 (Weeks 1-2)

- [ ] Clone and integrate upstream forgecode
- [ ] Set up development environment (TypeScript, Node, test framework)
- [ ] Implement `PhenotypeRustCrateProvider` (Provider 1)
- [ ] Create Provider Gallery widget mockup for AgilePlus
- [ ] Initialize AgilePlus specs in kitty-specs/

### Phase 2 (Weeks 3-4)

- [ ] Implement `AgilePlusWorkPackageProvider` (Provider 2)
- [ ] Implement `XDDTestGenerator` (Provider 3)
- [ ] Integration testing for all providers
- [ ] Evidence Timeline widget mockup

### Phase 3 (Weeks 5-6)

- [ ] Implement `MultiAgentCoordinator` extension
- [ ] Implement `EvidenceGenerator` extension
- [ ] Subagent registry integration
- [ ] Conflict resolution strategy

### Phase 4 (Weeks 7-8)

- [ ] AgilePlus dashboard REST API integration
- [ ] Provider Gallery widget (production)
- [ ] Evidence Timeline widget (production)
- [ ] Agent Work Distribution widget

### Phase 5 (Week 9+)

- [ ] Documentation and runbooks
- [ ] CI/CD pipeline setup (GitHub Actions)
- [ ] Public release preparation
- [ ] Community feedback integration

## Configuration Files (Ready)

### `.agileplus/config.toml`

Defines:
- Fork metadata (name, version, upstream URL)
- AgilePlus integration (specs location, tracker settings)
- Enabled providers and extensions
- Subagent coordinator settings
- Dashboard API endpoints
- CI/CD pipeline configuration
- Testing frameworks (pytest, coverage, clippy)
- Git upstream sync strategy
- Workspace paths and artifact directories

### `docs/FORK_INTENT.md`

Documents:
- Purpose and scope of custom providers
- Detailed provider implementations (with code examples)
- Subagent extension architecture
- AgilePlus integration (dashboard widgets, spec location)
- 5-phase implementation roadmap
- Design principles (SOLID, DRY, KISS, xDD)
- Open questions and decisions

### `docs/ARCHITECTURE.md`

Documents:
- System architecture (ASCII diagram)
- Hexagonal design model
- 4 core integration points (Provider API, Subagent API, Evidence, Dashboard)
- Fork maintenance strategy (upstream sync, cherry-pick workflow)
- Development workflow (adding providers, extensions)
- Testing strategy (unit, integration, evidence collection)
- Configuration management (fork-level, provider-level)
- CI/CD integration (GitHub Actions workflow)

## Key Decisions

| Decision | Value | Rationale |
|----------|-------|-----------|
| Spec Location | `../kitty-specs/forgecode-fork/` | Shared infrastructure; centralized spec management |
| Versioning | CalVer (YYYY.MONTH.PATCH) | Follows thegent pattern; predictable releases |
| Test Framework | pytest + cargo test + Jest | Multi-language support for all modules |
| Git Strategy | Monthly upstream sync + cherry-pick | Maintain custom extensions while staying current |
| Architecture | Hexagonal with plugin system | Clear separation; extensible design |
| Provider Model | Interface-based, single-responsibility | SOLID principles; independent providers |

## Files Checklist

- ✅ `.gitignore` — Node, build, IDE, OS exclusions
- ✅ `README.md` — Project overview, next steps
- ✅ `MANIFEST.md` — This initialization manifest
- ✅ `.agileplus/config.toml` — AgilePlus configuration
- ✅ `docs/FORK_INTENT.md` — Custom providers & extensions (6.2 KB)
- ✅ `docs/ARCHITECTURE.md` — System design & integration (8.4 KB)

## Pending

- ⏳ `kitty-specs/forgecode-fork/` — Specs directory (create after upstream clone)
- ⏳ Upstream forgecode clone (ready for execution)
- ⏳ Provider implementations (Phase 1)
- ⏳ Subagent extensions (Phase 3)
- ⏳ Dashboard integration (Phase 4)

## References

- **Repository**: Ready at `/Users/kooshapari/CodeProjects/Phenotype/repos/forgecode-fork`
- **Upstream**: https://github.com/forgecode/forgecode
- **AgilePlus**: /Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus
- **Phenotype Governance**: /Users/kooshapari/CodeProjects/Phenotype/CLAUDE.md
- **Fork Intent**: `docs/FORK_INTENT.md` (custom providers, subagent extensions, integration plan)
- **Architecture**: `docs/ARCHITECTURE.md` (system design, modification strategy, integration points)

---

**Status Summary**: Directory structure prepared. Ready for upstream clone and AgilePlus initialization. All documentation generated and in place.

**Manifest generated**: 2026-03-30 by Claude Code
