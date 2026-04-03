# Python Phenosdk Phase 2 Decomposition: Executive Summary

**Date:** 2026-03-30
**Status:** Design Complete, Ready for Kickoff
**Effort:** 6-8 weeks, 45-62 tool calls, ~19-27 hours
**Target Release:** 5 independent PyPI packages (v0.1.0 each)

---

## The Problem

**Current State:** Monolithic phenosdk with code duplication and tight coupling

```
phenosdk (monolith)
├── pheno/ (atoms + MCP + adapters)
│   ├── mcp/entry_points.py [DUPLICATE]
│   ├── shared/mcp_entry_points.py [STUB]
│   ├── auth/ (incomplete)
│   ├── adapters/persistence/ (incomplete)
│   └── vector/ (incomplete)
└── depends on:
    └── pheno-core
    └── pheno-mcp (also has MCP code!)

ISSUES:
- MCPEntryPoint defined in 3 places
- Agent orchestration tightly coupled to MCP
- Adapters scattered, no clear interface
- Hard to deploy/version independently
- ~300 LOC duplication
- Can't use agent patterns without full SDK
```

---

## The Solution: Phase 2 Decomposition

**Target State:** 5 independent, versioned PyPI packages with clean dependency DAG

```
phenotype-sdk v0.1.0 (facade)
├── pheno-core v0.1.0 (foundation, no deps)
├── pheno-mcp-core v0.1.0 (MCP protocols)
├── pheno-agents v0.1.0 (orchestration)
└── pheno-atoms v0.1.0 (adapters)

+ phenosdk-legacy v0.1.0 (backward compat alias)

BENEFITS:
✓ Independent deployment & versioning
✓ Explicit dependency chain (DAG)
✓ Zero circular dependencies
✓ Modular imports (import only what you need)
✓ Better testing isolation
✓ Easier to maintain & evolve
```

---

## 4-Phase Execution Plan

### Phase 2.1: Extract MCP Core (Weeks 1-2)

**Deliverable:** `pheno-mcp-core` v0.1.0

**What:** Move canonical MCP abstractions into standalone package
- MCPEntryPoint, MCPServer (from pheno-mcp)
- ToolRegistry, @mcp_tool decorator (from pheno-mcp/tools)
- Remove duplicates from phenosdk

**Outcome:**
- Single source of truth for MCP abstractions
- Both pheno-mcp and phenosdk import from pheno-mcp-core
- Zero duplication
- All tests pass, no circular deps

---

### Phase 2.2: Extract Agent Orchestration (Weeks 2-3)

**Deliverable:** `pheno-agents` v0.1.0

**What:** Extract agent orchestration into standalone package
- Agent, AgentRole, TaskDefinition classes
- AgentOrchestrator (from pheno-mcp)
- Test workflow patterns

**Outcome:**
- Agents usable independent of MCP
- Clear orchestration interface
- Agent-specific tests passing
- Ready for future extensions (CrewAI integration, etc.)

---

### Phase 2.3: Consolidate Adapters (Weeks 3-4)

**Deliverable:** `pheno-atoms` v0.1.0

**What:** Extract auth, persistence, vector adapters
- PlaywrightAuthAdapter
- InMemoryPersistence, SQLAlchemyAdapter
- VectorSearchClient
- Optional dependencies per adapter

**Outcome:**
- Centralized adapter module
- Optional deps reduce bloat
- Clear adapter interface patterns
- Adapter-specific testing strategy

---

### Phase 2.4: SDK Orchestration & Release (Weeks 4-5)

**Deliverable:** `phenotype-sdk` v0.1.0 + `phenosdk-legacy` v0.1.0

**What:**
- Create top-level phenotype-sdk facade
- Aggregate all tier imports
- Create backward compat alias (phenosdk → phenotype-sdk)
- Publish all 5 packages to PyPI/GitHub Packages

**Outcome:**
- Full SDK available as single import or modular
- Backward compatibility maintained
- All packages on PyPI
- 100% integration tested

---

## Work Items Summary

| Phase | Work Items | Tool Calls | Effort | Tests |
|-------|-----------|-----------|--------|-------|
| 2.1 | 6 | 15-20 | 6-8h | 40+ |
| 2.2 | 4 | 10-15 | 4-6h | 30+ |
| 2.3 | 4 | 12-15 | 5-7h | 35+ |
| 2.4 | 4 | 8-12 | 4-6h | 50+ |
| **Total** | **18** | **45-62** | **19-27h** | **155+ tests** |

---

## Key Design Decisions

### 1. Dependency Chain (DAG)

```
pheno-core (Tier 1, zero deps)
    ▲
    │ (required by all tiers 2-3)
    │
pheno-mcp-core (Tier 2a)     pheno-agents (Tier 2b)     pheno-atoms (Tier 3)
    └─────────┬──────────────────┬─────────────────────┘
              │
         phenotype-sdk (Tier 4, facade)
```

**Why:** Clean layering, no circular deps, each tier independently testable

### 2. MCP Core Split

**Old:** MCPEntryPoint in both pheno-mcp and phenosdk
**New:** Single canonical pheno-mcp-core, both depend on it

**Why:** Single source of truth, no synchronization issues

### 3. Agent Independence

**Old:** Agents only available via pheno-mcp
**New:** Separate pheno-agents package, pheno-mcp imports it

**Why:** Agents are orchestration patterns, not MCP-specific
- Can use agents without MCP
- Can test orchestration independently
- Better separation of concerns

### 4. Adapter Consolidation

**Old:** Scattered auth/, adapters/, vector/ in phenosdk
**New:** Centralized pheno-atoms with clear interface

**Why:** Single responsibility, easier to add new adapters, clearer pattern

### 5. Facade Pattern (phenotype-sdk)

**Why:** Gradual migration path, backward compatibility, clear public API

```python
# Both work:
from phenotype_sdk import MCPEntryPoint, Agent  # NEW (recommended)
from phenosdk import MCPEntryPoint, Agent       # OLD (via alias)
```

---

## Testing Strategy

### Pytest Markers by Phase

```ini
@pytest.mark.phase1       # pheno-core tests
@pytest.mark.phase2_1     # pheno-mcp-core tests
@pytest.mark.phase2_2     # pheno-agents tests
@pytest.mark.phase2_3     # pheno-atoms tests
@pytest.mark.phase2_4     # phenotype-sdk tests
@pytest.mark.integration  # cross-package tests
```

### Coverage Targets

| Package | Target |
|---------|--------|
| pheno-core | 85% |
| pheno-mcp-core | 80% |
| pheno-agents | 80% |
| pheno-atoms | 75% |
| phenotype-sdk | 70% |
| **Overall** | **78-82%** |

### Test Pyramid

```
             Integration (10%)
          ┌─────────────────────┐
          │  Full SDK stacks    │
          │  All 5 packages     │
          │  ~50 tests          │
          └─────────────────────┘
                     ▲
        Integration (20%)
    ┌──────────────────────────┐
    │  Package interactions    │
    │  Cross-imports           │
    │  ~100 tests              │
    └──────────────────────────┘
                     ▲
Unit Tests (70%)
┌───────────────────────────────────────┐
│  Per-package unit tests               │
│  ~155 tests total                     │
└───────────────────────────────────────┘
```

---

## Package Publishing Strategy

### PyPI Distribution (Recommended)

**Option A:** Official PyPI (public adoption)
```
pip install phenotype-sdk
pip install pheno-mcp-core
pip install pheno-agents
pip install pheno-atoms
```

**Option B:** GitHub Packages (organization-internal)
```
pip install --index-url https://npm.pkg.github.com/ phenotype-sdk
```

**Recommendation:** Both
- Publish to official PyPI for public adoption
- Mirror to GitHub Packages for internal CI/CD

### Release Order

```
Week 1-2: pheno-core v0.1.0 (already exists)
Week 1-2: pheno-mcp-core v0.1.0 ← START HERE
Week 2-3: pheno-agents v0.1.0
Week 3-4: pheno-atoms v0.1.0
Week 4-5: phenotype-sdk v0.1.0
          phenosdk-legacy v0.1.0 (backward compat alias)
```

---

## Backward Compatibility

### Migration Path

```python
# Phase 2.0 (current)
from phenosdk import MCPEntryPoint

# Phase 2.1+ (recommended)
from phenotype_sdk import MCPEntryPoint

# Phase 2.1+ (still works via alias)
from phenosdk import MCPEntryPoint  # → phenotype-sdk
```

### Version Compatibility Matrix

| App Version | phenosdk | phenotype-sdk | Notes |
|-------------|----------|---------------|-------|
| Old app | v0.1.0 | - | Uses old phenosdk |
| New app | - | v0.1.0 | Uses new phenotype-sdk |
| Hybrid app | v0.1.0 | v0.1.0 | Both work (alias) |

---

## File Movement Summary

### Phase 2.1 Moves

```
python/pheno-mcp/src/pheno_mcp/mcp/entry_points.py
  → python/pheno-mcp-core/src/pheno_mcp_core/entry_points.py

python/pheno-mcp/src/pheno_mcp/tools/tool_registry.py
  → python/pheno-mcp-core/src/pheno_mcp_core/tool_registry.py

python/pheno-mcp/src/pheno_mcp/tools/decorators.py
  → python/pheno-mcp-core/src/pheno_mcp_core/decorators.py

DELETE:
  python/phenosdk/src/pheno/mcp/entry_points.py (duplicate)
  python/phenosdk/src/pheno/shared/mcp_entry_points.py (stub)
```

### Phase 2.2 Moves

```
python/pheno-mcp/src/pheno_mcp/agents/orchestration.py
  → python/pheno-agents/src/pheno_agents/orchestrator.py

(class definitions extracted to models.py)
```

### Phase 2.3 Moves

```
python/phenosdk/src/pheno/auth/
  → python/pheno-atoms/src/pheno_atoms/auth/

python/phenosdk/src/pheno/adapters/persistence/
  → python/pheno-atoms/src/pheno_atoms/persistence/

python/phenosdk/src/pheno/vector/
  → python/pheno-atoms/src/pheno_atoms/vector/
```

### Phase 2.4 Changes

```
Rename: python/phenosdk/ → python/phenotype-sdk/
Create: python/phenosdk-legacy/ (thin alias)
```

---

## Dependencies Overview

### pheno-core (Foundation)
```toml
dependencies = [
    "pydantic>=2.0",
    "pydantic-settings>=2.0",
    "structlog>=24.1.0",
]
```

### pheno-mcp-core (MCP Protocols)
```toml
dependencies = [
    "pydantic>=2.0",
    "pheno-core>=0.1.0",
]
```

### pheno-agents (Orchestration)
```toml
dependencies = [
    "pheno-core>=0.1.0",
]
```

### pheno-atoms (Adapters)
```toml
dependencies = [
    "pheno-core>=0.1.0",
]
optional-dependencies = [
    "auth = ['playwright>=1.40']",
    "persistence = ['sqlalchemy>=2.0']",
    "vector = ['qdrant-client>=2.0']",
]
```

### phenotype-sdk (Facade)
```toml
dependencies = [
    "pheno-core>=0.1.0",
    "pheno-mcp-core>=0.1.0",
    "pheno-agents>=0.1.0",
    "pheno-atoms>=0.1.0",
]
```

---

## Success Criteria

### Phase 2.1 ✓
- [x] pheno-mcp-core v0.1.0 created
- [x] Zero duplicate MCPEntryPoint
- [x] All tests pass
- [x] No circular dependencies

### Phase 2.2
- [ ] pheno-agents v0.1.0 created
- [ ] Agent classes extracted
- [ ] Backward compat maintained

### Phase 2.3
- [ ] pheno-atoms v0.1.0 created
- [ ] Adapters consolidated
- [ ] Optional deps working

### Phase 2.4
- [ ] phenotype-sdk v0.1.0 created
- [ ] All 5 packages published
- [ ] phenosdk-legacy alias functional
- [ ] Integration tests 100% passing

---

## Related Documentation

| Document | Purpose |
|----------|---------|
| `PYTHON_DECOMPOSITION_PLAN.md` | Detailed 4-phase plan (500+ lines) |
| `PYTHON_DEPENDENCY_GRAPH.md` | Dependency diagrams & coupling analysis |
| `PYTHON_PHASE2_EXTRACTION_ROADMAP.md` | Week-by-week timeline & work items |
| `PYTHON_TESTING_STRATEGY.md` | Test isolation & pytest configuration |

---

## Next Steps

1. **Review:** Get team approval on this plan
2. **Create Branch:** `git checkout -b phase2-python-decomposition`
3. **Kickoff WI-2.1.1:** Create pheno-mcp-core directory structure
4. **Weekly Standup:** Track progress on phase milestones
5. **Release as Completed:** Release each phase independently to PyPI

---

## Appendix: Quick Command Reference

```bash
# Phase 2.1: MCP Core
cd python
pytest pheno-mcp-core/tests/ -m phase2_1 --cov

# Phase 2.2: Agents
pytest pheno-agents/tests/ -m phase2_2 --cov

# Phase 2.3: Adapters
pytest pheno-atoms/tests/ -m phase2_3 --cov

# Phase 2.4: Full SDK
pytest tests/integration_*.py -m phase2_4 --cov

# Validate all
pipdeptree --warn fail | grep pheno
python -c "from phenotype_sdk import *; print('OK')"

# Publish (GitHub Packages)
cd pheno-mcp-core && python -m build
twine upload dist/* --repository github

# Check coverage
pytest python/ --cov --cov-report=html
open htmlcov/index.html
```

---

## Estimated Timeline

```
START: April 1, 2026
├─ Week 1-2: Phase 2.1 (MCP Core)
├─ Week 2-3: Phase 2.2 (Agents)
├─ Week 3-4: Phase 2.3 (Adapters)
├─ Week 4-5: Phase 2.4 (SDK Release)
└─ RELEASE: phenotype-sdk v0.1.0 (by May 15, 2026)

Total: 6-8 weeks
Effort: ~19-27 hours (agent-driven, parallelizable)
```

---

## Contact & Questions

- **Author:** Claude Code (AI Agent)
- **Review Date:** 2026-03-30
- **Last Updated:** 2026-03-30
- **Status:** Ready for kickoff

---

**END OF EXECUTIVE SUMMARY**

For detailed phase breakdown, see `PYTHON_DECOMPOSITION_PLAN.md`
For timeline & work items, see `PYTHON_PHASE2_EXTRACTION_ROADMAP.md`
For testing strategy, see `PYTHON_TESTING_STRATEGY.md`
