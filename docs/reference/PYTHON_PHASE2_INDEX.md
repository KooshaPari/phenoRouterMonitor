# Python Phenosdk Phase 2 Decomposition: Document Index

**Created:** 2026-03-30
**Status:** Complete Design Documentation
**Purpose:** Enable independent deployment and distribution of phenosdk components

---

## Quick Navigation

### Executive Level
- **[PYTHON_DECOMPOSITION_SUMMARY.md](./PYTHON_DECOMPOSITION_SUMMARY.md)** (5 min read)
  - High-level overview of Phase 2 goals, benefits, and success criteria
  - Best starting point for understanding the initiative
  - Use this for steering committee presentations

### Implementation Level
- **[PYTHON_DECOMPOSITION_PLAN.md](./PYTHON_DECOMPOSITION_PLAN.md)** (20 min read, 250 lines)
  - Detailed 4-phase decomposition strategy
  - Architecture decisions and trade-offs
  - Work breakdown structure (WI-2.1.1 through WI-2.4.4)
  - Acceptance criteria for each work item
  - Dependency validation and risk mitigation

- **[PYTHON_PHASE2_EXTRACTION_ROADMAP.md](./PYTHON_PHASE2_EXTRACTION_ROADMAP.md)** (30 min read, 400+ lines)
  - Week-by-week timeline and effort estimates
  - Detailed steps for each work item
  - Code examples and command reference
  - CI/CD publishing strategy
  - Migration path and backward compatibility

### Technical Design
- **[PYTHON_DEPENDENCY_GRAPH.md](./PYTHON_DEPENDENCY_GRAPH.md)** (15 min read)
  - Current state vs. target state dependency graphs
  - Module relationships (ASCII diagrams)
  - Current coupling analysis and duplication
  - Import order validation
  - Circular dependency detection

### Testing & QA
- **[PYTHON_TESTING_STRATEGY.md](./PYTHON_TESTING_STRATEGY.md)** (25 min read)
  - Pytest markers by phase
  - Test pyramid and coverage targets
  - Phase-specific test structures with examples
  - CI/CD test execution strategy
  - Coverage configuration

---

## Document Map by Role

### For Project Manager
1. **PYTHON_DECOMPOSITION_SUMMARY.md** — Overview & timeline
2. **PYTHON_PHASE2_EXTRACTION_ROADMAP.md** — Effort estimates & milestones
3. **PYTHON_DECOMPOSITION_PLAN.md** — Risk mitigation & success criteria

### For Architect
1. **PYTHON_DECOMPOSITION_PLAN.md** — Architecture decisions
2. **PYTHON_DEPENDENCY_GRAPH.md** — Dependency design
3. **PYTHON_TESTING_STRATEGY.md** — Test architecture

### For Developer/Agent
1. **PYTHON_PHASE2_EXTRACTION_ROADMAP.md** — Work items & steps
2. **PYTHON_DECOMPOSITION_PLAN.md** — Acceptance criteria
3. **PYTHON_TESTING_STRATEGY.md** — Test execution

### For QA/Tester
1. **PYTHON_TESTING_STRATEGY.md** — Complete testing strategy
2. **PYTHON_PHASE2_EXTRACTION_ROADMAP.md** — Test validation steps
3. **PYTHON_DECOMPOSITION_PLAN.md** — Success metrics

---

## Phase 2 at a Glance

```
PHASE 2.1 (Weeks 1-2)
Extract MCP Core
  Deliverable: pheno-mcp-core v0.1.0
  Work Items: 6 (WI-2.1.1 through WI-2.1.6)
  Effort: 6-8 hours

PHASE 2.2 (Weeks 2-3)
Extract Agent Orchestration
  Deliverable: pheno-agents v0.1.0
  Work Items: 4 (WI-2.2.1 through WI-2.2.4)
  Effort: 4-6 hours

PHASE 2.3 (Weeks 3-4)
Consolidate Adapters
  Deliverable: pheno-atoms v0.1.0
  Work Items: 4 (WI-2.3.1 through WI-2.3.4)
  Effort: 5-7 hours

PHASE 2.4 (Weeks 4-5)
SDK Orchestration & Release
  Deliverable: phenotype-sdk + phenosdk-legacy v0.1.0
  Work Items: 4 (WI-2.4.1 through WI-2.4.4)
  Effort: 4-6 hours

TOTAL: 6-8 weeks, 19-27 hours, 45-62 tool calls
```

---

## Key Decisions

| Decision | Rationale | Location |
|----------|-----------|----------|
| **5 packages** (core → mcp-core → agents → atoms → sdk) | Clean layering, no circular deps, independent versioning | DECOMPOSITION_PLAN.md |
| **Single pheno-mcp-core** (not pheno-mcp + phenosdk duplicates) | Single source of truth, easier to maintain | DEPENDENCY_GRAPH.md |
| **Separate pheno-agents** (not embedded in pheno-mcp) | Agents are orchestration patterns, not MCP-specific | DECOMPOSITION_PLAN.md |
| **phenosdk-legacy alias** (not hard break) | Backward compatibility, gradual migration path | EXTRACTION_ROADMAP.md |
| **Optional adapter dependencies** | Reduce bloat, users install only what they need | DECOMPOSITION_PLAN.md |
| **Phase-based test markers** | Enable independent test execution, CI/CD optimization | TESTING_STRATEGY.md |

---

## File Changes Summary

```
CREATE:
  python/pheno-mcp-core/
  python/pheno-agents/
  python/pheno-atoms/
  python/phenotype-sdk/
  python/phenosdk-legacy/  (optional)

MOVE (from pheno-mcp → pheno-mcp-core):
  pheno_mcp/mcp/entry_points.py
  pheno_mcp/tools/tool_registry.py
  pheno_mcp/tools/decorators.py

MOVE (from pheno-mcp → pheno-agents):
  pheno_mcp/agents/orchestration.py → models + orchestrator

MOVE (from phenosdk → pheno-atoms):
  pheno/auth/
  pheno/adapters/persistence/
  pheno/vector/

DELETE:
  phenosdk/src/pheno/mcp/entry_points.py (duplicate)
  phenosdk/src/pheno/shared/mcp_entry_points.py (stub)

UPDATE:
  pheno-mcp/pyproject.toml (add pheno-mcp-core dep)
  phenosdk/pyproject.toml (add pheno-mcp-core, pheno-atoms deps)
  All test imports (point to new canonical locations)
```

---

## Testing Overview

| Phase | Package | Tests | Coverage | Status |
|-------|---------|-------|----------|--------|
| 1 | pheno-core | 40+ | 85% | ✓ Existing |
| 2.1 | pheno-mcp-core | 35+ | 80% | New |
| 2.2 | pheno-agents | 30+ | 80% | New |
| 2.3 | pheno-atoms | 35+ | 75% | New |
| 2.4 | phenotype-sdk | 50+ | 70% | New (integration) |
| **Total** | **All** | **155+** | **78-82%** | **7 marker-based** |

---

## Dependency Chain (Target)

```
pheno-core (v0.1.0)
    ↑ (required)
    │
    ├─ pheno-mcp-core (v0.1.0)
    ├─ pheno-agents (v0.1.0)
    └─ pheno-atoms (v0.1.0)
         │
         └─ phenotype-sdk (v0.1.0, meta-package)

PROPERTIES:
✓ No circular dependencies
✓ Clear dependency hierarchy
✓ Each package independent
✓ Each package independently versionable
```

---

## Success Metrics

**Phase 2.1 Complete:**
- [ ] pheno-mcp-core published to PyPI
- [ ] 0 duplicate MCPEntryPoint definitions
- [ ] All tests passing (80%+ coverage)
- [ ] Zero circular dependencies

**Phase 2.2 Complete:**
- [ ] pheno-agents published to PyPI
- [ ] Agent classes fully extracted & independent
- [ ] All tests passing (80%+ coverage)
- [ ] Backward compat in pheno-mcp maintained

**Phase 2.3 Complete:**
- [ ] pheno-atoms published to PyPI
- [ ] Auth, persistence, vector adapters consolidated
- [ ] Optional dependencies working correctly
- [ ] All tests passing (75%+ coverage)

**Phase 2.4 Complete:**
- [ ] phenotype-sdk published to PyPI (meta-package)
- [ ] phenosdk-legacy alias working (backward compat)
- [ ] All 5 packages available on PyPI & GitHub Packages
- [ ] Integration tests 100% passing (70%+ coverage)
- [ ] Documentation complete & examples working

---

## Getting Started

1. **Read This File** (you are here) ← Quick overview
2. **Read SUMMARY** (5 min) ← High-level goals
3. **Read PLAN** (20 min) ← Detailed strategy
4. **Read ROADMAP** (30 min) ← Implementation steps
5. **Read TESTING** (25 min) ← Test strategy
6. **Review GRAPHS** (15 min) ← Visual dependencies

**Total Onboarding Time:** ~90 minutes

---

## Document Statistics

| Document | Lines | Size | Focus |
|----------|-------|------|-------|
| PYTHON_DECOMPOSITION_SUMMARY.md | 450 | 18 KB | Executive summary |
| PYTHON_DECOMPOSITION_PLAN.md | 650 | 27 KB | Detailed plan |
| PYTHON_DEPENDENCY_GRAPH.md | 550 | 22 KB | Architecture |
| PYTHON_PHASE2_EXTRACTION_ROADMAP.md | 750 | 30 KB | Implementation |
| PYTHON_TESTING_STRATEGY.md | 650 | 26 KB | QA/Testing |
| **TOTAL** | **3,050** | **123 KB** | Complete design |

---

## Quick Reference Commands

```bash
# Create pheno-mcp-core package
mkdir -p python/pheno-mcp-core/src/pheno_mcp_core

# Run phase 2.1 tests
pytest python/pheno-mcp-core/tests/ -m phase2_1 --cov

# Check for circular dependencies
pipdeptree --warn fail | grep pheno

# Validate all imports
python -c "from phenotype_sdk import *; print('OK')"

# Build and publish
cd python/pheno-mcp-core && python -m build
twine upload dist/* --repository pypi
```

---

## FAQ

**Q: How long will this take?**
A: 6-8 weeks total (4-5 weeks development + 1-3 weeks review/testing). See EXTRACTION_ROADMAP.md for detailed timeline.

**Q: Will this break existing code?**
A: No. Backward compat maintained via phenosdk-legacy alias. See DECOMPOSITION_PLAN.md Migration Path section.

**Q: Which document should I start with?**
A: PYTHON_DECOMPOSITION_SUMMARY.md (5 min overview), then PYTHON_PHASE2_EXTRACTION_ROADMAP.md (if implementing).

**Q: How many tests will be affected?**
A: ~155+ new/moved tests. Current tests continue to work. See TESTING_STRATEGY.md for details.

**Q: Can phases run in parallel?**
A: No. Phase 2.2+ require 2.1 to be done first (dependency order). See EXTRACTION_ROADMAP.md timeline.

**Q: What's the risk?**
A: Low. Detailed planning + testing strategy mitigate. See DECOMPOSITION_PLAN.md Risk Mitigation section.

---

## Contact & Version Control

**Author:** Claude Code (AI Agent)
**Created:** 2026-03-30
**Last Updated:** 2026-03-30
**Status:** Ready for Review & Kickoff
**Version:** 1.0 (Design Complete)

---

## Related Initiatives

- **OSS Wrapping Initiative** (WS4: Python httpx consolidation) — Complements Phase 2
- **Phase 1 Shared Libs** (phenotype-error-core, etc.) — Foundation for Phase 2
- **Libification Audit 2026-03-29** — Identified decomposition opportunities
- **Versioning Strategy** (cliff.toml, SemVer + CalVer) — Release automation

---

**Ready to Kickoff Phase 2?** Start with PYTHON_DECOMPOSITION_SUMMARY.md
**Need Implementation Details?** See PYTHON_PHASE2_EXTRACTION_ROADMAP.md
**Reviewing Architecture?** See PYTHON_DECOMPOSITION_PLAN.md
**Planning Tests?** See PYTHON_TESTING_STRATEGY.md
