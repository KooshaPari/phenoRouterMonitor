# Python Phenosdk Decomposition Plan: Phase 2

**Status:** Phase 2 Design Document
**Date:** 2026-03-30
**Target:** Enable independent deployment and distribution of phenosdk components
**Scope:** phenosdk (atoms), pheno-core (foundation), pheno-mcp (MCP tools)

---

## Executive Summary

The current Python ecosystem has three packages with emerging coupling:

1. **pheno-core** (foundation): Config, logging, errors, observability
2. **pheno-mcp** (MCP abstraction): Tools, agents, entry points, orchestration
3. **phenosdk** (atoms): Auth, MCP, adapters (persistence, vector), shared utilities

**Goal:** Extract and decouple into **5 independent PyPI packages** with explicit dependency chains:
- `pheno-core` (standalone, zero app deps)
- `pheno-mcp-core` (new: MCP protocols, server-agnostic)
- `pheno-agents` (new: agent orchestration, orchestration interfaces)
- `pheno-atoms` (refactored phenosdk: auth, persistence, vector adapters)
- `phenotype-sdk` (top-level orchestration, wraps all)

**Target Outcome:**
- Separate PyPI packages with SemVer releases
- Zero circular dependencies
- Clear dependency chain: core → mcp → agents → atoms → orchestration
- Independent testing and deployment per package
- GitHub Packages publishing support

---

## Current Structure Analysis

### Module Map

```
python/
├── pheno-core/
│   ├── src/pheno_core/
│   │   ├── errors.py          # Error hierarchy (ZenMCPError, ConfigurationError)
│   │   ├── config.py          # Pydantic BaseConfig, from_env()
│   │   ├── logging.py         # structlog integration
│   │   └── observability.py   # Health checks, metrics
│   └── tests/
│
├── pheno-mcp/
│   ├── src/pheno_mcp/
│   │   ├── tools/
│   │   │   ├── tool_registry.py      # Tool registry abstraction
│   │   │   ├── decorators.py         # @mcp_tool, ToolMetadata
│   │   │   └── __init__.py
│   │   ├── mcp/
│   │   │   ├── entry_points.py       # MCPEntryPoint, MCPServer
│   │   │   └── __init__.py
│   │   ├── agents/
│   │   │   ├── orchestration.py      # AgentOrchestrator, Agent, TaskDefinition
│   │   │   └── __init__.py
│   │   └── __init__.py
│   └── tests/
│
└── phenosdk/
    ├── src/pheno/
    │   ├── auth/
    │   │   ├── playwright_adapter.py  # PlaywrightAuthAdapter
    │   │   └── __init__.py
    │   ├── mcp/
    │   │   ├── entry_points.py        # MCPEntryPoint (DUPLICATE: pheno-mcp)
    │   │   └── __init__.py
    │   ├── shared/
    │   │   ├── mcp_entry_points.py    # BaseMCPEntryPoint, MCPConfiguration
    │   │   └── __init__.py
    │   ├── adapters/
    │   │   ├── persistence/
    │   │   │   ├── in_memory.py       # InMemoryPersistence
    │   │   │   ├── sqlalchemy_adapter.py  # SQLAlchemy adapter
    │   │   │   └── __init__.py
    │   │   └── __init__.py
    │   ├── vector/
    │   │   ├── client.py              # VectorSearchClient
    │   │   └── __init__.py
    │   └── __init__.py
    └── README.md
```

### Current Dependencies

```
pheno-core
  ├── pydantic>=2.0
  ├── pydantic-settings>=2.0
  └── structlog>=24.1.0

pheno-mcp
  └── pydantic>=2.0
  └── typing-extensions>=4.0

phenosdk (standalone, currently)
  └── (no explicit dependencies; adapters have implicit deps)
```

### Coupling Analysis

| Module | Coupled To | Issue | Severity |
|--------|-----------|-------|----------|
| phenosdk.mcp | pheno-mcp.mcp | **Duplicate MCPEntryPoint** | High |
| phenosdk.shared | pheno-mcp.mcp | Base class duplication (BaseMCPEntryPoint) | High |
| phenosdk auth | pheno-core | Implicit (auth needs config/logging from core) | Medium |
| phenosdk.adapters | pheno-core | Config management dependency | Medium |
| phenosdk.vector | (no explicit) | Should depend on pheno-core for logging | Low |

---

## Decomposition Strategy

### 3 Key Insights

1. **Duplication in MCP Layer**: `phenosdk.mcp` and `pheno-mcp.mcp` both define MCPEntryPoint
   - **Solution:** Extract single `pheno-mcp-core` package with canonical MCPEntryPoint, MCPServer
   - Both phenosdk and pheno-mcp import from pheno-mcp-core

2. **Agent Orchestration Isolation**: AgentOrchestrator, Agent, TaskDefinition are self-contained
   - **Solution:** Create separate `pheno-agents` package
   - pheno-mcp depends on pheno-agents for orchestration

3. **Atoms Layer Consolidation**: Auth, persistence, vector adapters are implementation-specific
   - **Solution:** Extract into `pheno-atoms` (replaces old phenosdk)
   - All adapters depend on pheno-core only
   - Orchestration layer (phenotype-sdk) wraps pheno-atoms + pheno-mcp

### Proposed Dependency Tree

```
┌─────────────────────────────────────────────────────────────────────┐
│                    PHENOTYPE SDK ECOSYSTEM                          │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  TIER 1: Foundation (No app dependencies)                          │
│  ┌──────────────────────────────────────┐                          │
│  │ pheno-core v0.1.0                    │                          │
│  │ - Errors, Config, Logging            │                          │
│  │ - Observability (Health, Metrics)    │                          │
│  │ Deps: pydantic, structlog            │                          │
│  └──────────────────────────────────────┘                          │
│                    ▲                                                │
│                    │                                                │
│  TIER 2: Protocol & Service Abstraction                            │
│  ┌──────────────────────────────────────┐                          │
│  │ pheno-mcp-core v0.1.0 [NEW]          │                          │
│  │ - MCPEntryPoint, MCPServer           │                          │
│  │ - ToolRegistry, Tool metadata        │                          │
│  │ - @mcp_tool decorator                │                          │
│  │ Deps: pydantic, pheno-core           │                          │
│  └──────────────────────────────────────┘                          │
│                    ▲                                                │
│                    │                                                │
│  ┌──────────────────────────────────────┐                          │
│  │ pheno-agents v0.1.0 [NEW]            │                          │
│  │ - Agent, AgentRole, TaskDefinition   │                          │
│  │ - AgentOrchestrator                  │                          │
│  │ - Workflow execution & validation    │                          │
│  │ Deps: dataclasses, pheno-core        │                          │
│  └──────────────────────────────────────┘                          │
│                    ▲                                                │
│                    │                                                │
│  TIER 3: Implementations (Atoms)                                   │
│  ┌──────────────────────────────────────┐                          │
│  │ pheno-atoms v0.1.0 [NEW]             │                          │
│  │ - PlaywrightAuthAdapter              │                          │
│  │ - InMemoryPersistence                │                          │
│  │ - SQLAlchemyAdapter                  │                          │
│  │ - VectorSearchClient                 │                          │
│  │ Deps: pheno-core (optional: sqlalchemy, playwright) │
│  └──────────────────────────────────────┘                          │
│                    ▲                                                │
│                    │                                                │
│  TIER 4: Orchestration (Public SDK)                                │
│  ┌──────────────────────────────────────┐                          │
│  │ phenotype-sdk v0.1.0 [REPLACES phenosdk] │                      │
│  │ - Wraps: pheno-mcp-core, pheno-agents  │                        │
│  │ - Wraps: pheno-atoms                 │                          │
│  │ - Convenience imports & facades      │                          │
│  │ Deps: all above                      │                          │
│  └──────────────────────────────────────┘                          │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘

NOTES:
- pheno-core: 0 app dependencies (Foundation)
- pheno-mcp-core: depends on pheno-core only
- pheno-agents: depends on pheno-core only
- pheno-atoms: depends on pheno-core only
- phenotype-sdk: aggregate of Tiers 1-3
```

---

## Phase 2 Work Breakdown (4 Phases: 6-8 weeks)

### Phase 2.1: Extract MCP Core (Week 1-2)
**Deliverable:** `pheno-mcp-core` PyPI package (canonical MCP abstractions)

**Work Items:**
- WI-2.1.1: Create `crates/pheno-mcp-core/` directory structure
  - `src/pheno_mcp_core/__init__.py` (re-export canonical types)
  - `src/pheno_mcp_core/entry_points.py` (move from pheno-mcp/mcp/)
  - `src/pheno_mcp_core/tool_registry.py` (move from pheno-mcp/tools/)
  - `src/pheno_mcp_core/decorators.py` (move from pheno-mcp/tools/)
  - `pyproject.toml` with deps: [pydantic>=2.0]
  - `README.md` with usage examples

- WI-2.1.2: Update pheno-mcp to consume pheno-mcp-core
  - Remove duplicate entry_points.py
  - Remove duplicate tool_registry.py, decorators.py
  - Update imports: `from pheno_mcp_core import MCPEntryPoint, ToolRegistry`
  - Add `pheno-mcp-core` to dependencies
  - Update tests to import from pheno-mcp-core

- WI-2.1.3: Update phenosdk to consume pheno-mcp-core
  - Remove phenosdk/src/pheno/mcp/entry_points.py (duplicate)
  - Remove phenosdk/src/pheno/shared/mcp_entry_points.py
  - Update imports in phenosdk.auth, phenosdk.adapters
  - Add `pheno-mcp-core` to phenosdk dependencies

- WI-2.1.4: Testing & validation
  - Run: `pytest python/pheno-mcp-core/tests/`
  - Run: `pytest python/pheno-mcp/tests/` (ensure imports work)
  - Run: `pytest python/phenosdk/tests/` (ensure import resolution)
  - Validate circular deps: `pipdeptree --graph | grep pheno`

**Acceptance Criteria:**
- [ ] pheno-mcp-core package builds cleanly
- [ ] Zero duplicate code (entry_points, tool_registry, decorators)
- [ ] All tests pass in pheno-mcp, pheno-core, phenosdk
- [ ] No circular dependencies
- [ ] CI/CD: GitHub Packages publishing workflow passes

---

### Phase 2.2: Extract Agent Orchestration (Week 2-3)
**Deliverable:** `pheno-agents` PyPI package (agent orchestration abstraction)

**Work Items:**
- WI-2.2.1: Create `python/pheno-agents/` directory structure
  - `src/pheno_agents/__init__.py` (export Agent, AgentRole, etc.)
  - `src/pheno_agents/models.py` (Agent, AgentRole, TaskDefinition classes)
  - `src/pheno_agents/orchestrator.py` (AgentOrchestrator)
  - `tests/test_agent_models.py`, `test_orchestrator.py`
  - `pyproject.toml` with deps: [dataclasses, pheno-core]
  - `README.md` with agent patterns

- WI-2.2.2: Update pheno-mcp to consume pheno-agents
  - Move imports: `from pheno_agents import Agent, AgentRole, TaskDefinition`
  - Remove agent models from pheno_mcp/agents/orchestration.py
  - Keep pheno-mcp as a facade that re-exports for backward compat
  - Add `pheno-agents` to pheno-mcp dependencies

- WI-2.2.3: Refactor pheno-mcp agent tests
  - Ensure agent-specific tests use pheno-agents
  - Keep pheno-mcp integration tests (orchestrator + tools)
  - Add parametrized tests for agent role scenarios

- WI-2.2.4: Testing & integration
  - Run: `pytest python/pheno-agents/tests/`
  - Run: `pytest python/pheno-mcp/tests/test_agents_orchestration.py`
  - Validate: `pipdeptree | grep -A3 pheno-agents`

**Acceptance Criteria:**
- [ ] pheno-agents package created and builds
- [ ] Zero code duplication (Agent classes extracted)
- [ ] All pheno-mcp agent tests pass
- [ ] Backward compatibility: pheno-mcp re-exports Agent, AgentRole, etc.
- [ ] README includes agent composition patterns

---

### Phase 2.3: Consolidate Adapters → pheno-atoms (Week 3-4)
**Deliverable:** `pheno-atoms` PyPI package (implementation adapters)

**Work Items:**
- WI-2.3.1: Create `python/pheno-atoms/` directory structure
  - `src/pheno_atoms/__init__.py` (export adapter classes)
  - `src/pheno_atoms/auth/` (move from phenosdk/auth/)
    - `playwright_adapter.py`
    - `__init__.py`
  - `src/pheno_atoms/persistence/` (move from phenosdk/adapters/persistence/)
    - `in_memory.py`
    - `sqlalchemy_adapter.py`
    - `__init__.py`
  - `src/pheno_atoms/vector/` (move from phenosdk/vector/)
    - `client.py`
    - `__init__.py`
  - `pyproject.toml` with optional deps:
    - `base: [pheno-core]`
    - `auth: [playwright>=1.40]`
    - `persistence: [sqlalchemy>=2.0]`
    - `vector: [qdrant-client>=2.0]`
  - `README.md` with adapter usage

- WI-2.3.2: Update phenosdk imports to consume pheno-atoms
  - Remove duplicate auth/, adapters/, vector/ modules
  - Update: `from pheno_atoms import PlaywrightAuthAdapter, ...`
  - Keep phenosdk as a facade for backward compatibility

- WI-2.3.3: Add optional dependency groups to pheno-atoms/pyproject.toml
  ```toml
  [project.optional-dependencies]
  auth = ["playwright>=1.40"]
  persistence = ["sqlalchemy>=2.0"]
  vector = ["qdrant-client>=2.0"]
  all = ["playwright>=1.40", "sqlalchemy>=2.0", "qdrant-client>=2.0"]
  ```

- WI-2.3.4: Testing & validation
  - Run: `pytest python/pheno-atoms/tests/`
  - Test optional deps: `pytest -m "auth or persistence or vector"`
  - Validate adapter isolation: each adapter should import only needed deps

**Acceptance Criteria:**
- [ ] pheno-atoms package created and builds
- [ ] Auth, persistence, vector adapters cleanly extracted
- [ ] Optional dependencies correctly configured
- [ ] All adapter tests pass
- [ ] No unexpected dependencies in pheno-atoms/base

---

### Phase 2.4: Orchestration Layer & Publishing (Week 4-5)
**Deliverable:** `phenotype-sdk` as top-level orchestration wrapper

**Work Items:**
- WI-2.4.1: Refactor phenosdk → phenotype-sdk (top-level orchestration)
  - Keep phenosdk as backward-compat alias (empty package that depends on phenotype-sdk)
  - Create `phenotype-sdk/` directory
  - `src/phenotype_sdk/__init__.py` (aggregate facade)
    ```python
    # Tier 1: Core
    from pheno_core import *

    # Tier 2: Protocols
    from pheno_mcp_core import MCPEntryPoint, MCPServer, ToolRegistry, mcp_tool
    from pheno_agents import Agent, AgentRole, TaskDefinition, AgentOrchestrator

    # Tier 3: Adapters
    from pheno_atoms import (
        PlaywrightAuthAdapter,
        InMemoryPersistence,
        SQLAlchemyAdapter,
        VectorSearchClient
    )
    ```
  - Dependencies: [pheno-core, pheno-mcp-core, pheno-agents, pheno-atoms]

- WI-2.4.2: Create phenosdk-compat package (backward compat)
  - Empty package with alias: `setup.py` points to phenotype-sdk
  - Or: `phenosdk/pyproject.toml` = `dependencies = ["phenotype-sdk>=0.1.0"]`
  - Deprecation warning in __init__.py

- WI-2.4.3: Update GitHub Packages publishing
  - Add `.github/workflows/publish-phenotype-sdk.yml`
  - Matrix strategy: publish all 5 packages in order
    - pheno-core → pheno-mcp-core → pheno-agents → pheno-atoms → phenotype-sdk
  - Use: `gh release create v0.1.0 --generate-notes` for version tags
  - Upload to GitHub Packages (PyPI requires HTTPS auth)

- WI-2.4.4: Integration tests & validation
  - Create `python/tests/integration_decomposed_sdk.py`
    ```python
    import pytest
    from phenotype_sdk import (
        MCPEntryPoint, Agent, AgentRole, PlaywrightAuthAdapter
    )

    def test_sdk_imports():
        """Verify all tier facades are accessible."""
        assert MCPEntryPoint is not None
        assert Agent is not None
        assert PlaywrightAuthAdapter is not None
    ```
  - Run full test suite: `pytest python/tests/`
  - Validate no circular imports

**Acceptance Criteria:**
- [ ] phenotype-sdk package created and builds
- [ ] All 5 packages (pheno-core through phenotype-sdk) publish to GitHub Packages
- [ ] Backward compat maintained (phenosdk alias works)
- [ ] Integration tests pass
- [ ] README documents the full SDK decomposition

---

## Phase 2 Testing Strategy

### Pytest Markers by Phase

Add to `python/pyproject.toml` (shared root config):

```ini
[tool.pytest.ini_options]
markers = [
    "phase1: Core foundation (pheno-core only)",
    "phase2_1: MCP core extraction (pheno-mcp-core)",
    "phase2_2: Agent orchestration (pheno-agents)",
    "phase2_3: Adapters consolidation (pheno-atoms)",
    "phase2_4: Integration (phenotype-sdk)",
    "integration: Full SDK integration tests",
    "slow: Slow tests (skip with -m 'not slow')",
]
```

### Test Isolation Matrix

| Phase | Package | Test Path | Pytest Marks | CI Gate |
|-------|---------|-----------|--------------|---------|
| 1 | pheno-core | `python/pheno-core/tests/` | `@pytest.mark.phase1` | Always run |
| 2.1 | pheno-mcp-core | `python/pheno-mcp-core/tests/` | `@pytest.mark.phase2_1` | Always run |
| 2.2 | pheno-agents | `python/pheno-agents/tests/` | `@pytest.mark.phase2_2` | Always run |
| 2.3 | pheno-atoms | `python/pheno-atoms/tests/` | `@pytest.mark.phase2_3` | Always run |
| 2.4 | phenotype-sdk | `python/tests/integration_*.py` | `@pytest.mark.phase2_4` | Always run |

### Coverage Targets

- **pheno-core**: 85% (foundation critical)
- **pheno-mcp-core**: 80% (protocol stable)
- **pheno-agents**: 80% (orchestration patterns)
- **pheno-atoms**: 75% (adapters may skip NotImplementedError paths)
- **phenotype-sdk**: 70% (facade, integration tested)

---

## Packaging Strategy

### PyPI Distribution

**Option A (Recommended): Separate PyPI Packages**

Publish to official PyPI:
- `pheno-core`
- `pheno-mcp-core`
- `pheno-agents`
- `pheno-atoms`
- `phenotype-sdk` (meta package, depends on all)

**Option B (GitHub Packages): Organization-Internal**

Publish to GitHub Packages (requires GitHub auth for installation):
- All 5 packages published to github.com/KooshaPari/phenotype-infrakit
- Installation: `pip install --index-url https://npm.pkg.github.com/ phenotype-sdk`

**Recommended Approach:** Option A (official PyPI) for public adoption + Option B (GitHub Packages) for internal CI/CD

---

## Dependency Validation

### Circular Dependency Detection

```bash
# Run after each phase
pipdeptree --warn fail | grep "pheno"

# Expected output (no cycles):
# pheno-core
# pheno-mcp-core
#   └── pheno-core
# pheno-agents
#   └── pheno-core
# pheno-atoms
#   └── pheno-core
# phenotype-sdk
#   ├── pheno-core
#   ├── pheno-mcp-core
#   ├── pheno-agents
#   └── pheno-atoms
```

### Import Order Validation

```python
# No circular imports test
import pheno_core
import pheno_mcp_core  # Should not fail
import pheno_agents
import pheno_atoms
from phenotype_sdk import *  # Should not fail
```

---

## Risk Mitigation

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|-----------|
| Circular dependency in refactoring | Medium | Critical | Use pipdeptree validation in CI; manual review of imports |
| Duplication remains after extraction | Low | High | Code review checklist; grep for duplicate class names |
| Breaking changes for phenosdk users | Low | Medium | Maintain backward compat alias (phenosdk → phenotype-sdk) |
| Test coverage gaps | Medium | Medium | Coverage threshold 80% per package; fail if drops |
| CI/CD publish failures | Low | Low | Dry-run publish to GitHub Packages first; manual trigger if needed |

---

## Success Metrics

**Phase 2.1 Complete:**
- [ ] pheno-mcp-core v0.1.0 published and installable
- [ ] 0 duplicate MCPEntryPoint definitions
- [ ] All pheno-mcp tests pass
- [ ] pipdeptree shows no cycles

**Phase 2.2 Complete:**
- [ ] pheno-agents v0.1.0 published
- [ ] Agent orchestration 100% extracted
- [ ] pheno-mcp backward-compat maintained
- [ ] Agent-specific tests pass

**Phase 2.3 Complete:**
- [ ] pheno-atoms v0.1.0 published
- [ ] Auth, persistence, vector adapters extracted
- [ ] Optional dependencies correctly configured
- [ ] Adapter tests pass with minimal deps

**Phase 2.4 Complete:**
- [ ] phenotype-sdk v0.1.0 published
- [ ] Full integration test suite passes
- [ ] All 5 packages available on PyPI + GitHub Packages
- [ ] README documents decomposition strategy
- [ ] Zero breaking changes (backward compat maintained)

---

## Migration Path for Existing Code

```python
# OLD (phenosdk direct)
from pheno.mcp import MCPEntryPoint
from pheno.auth import PlaywrightAuthAdapter

# NEW (recommended, post-Phase 2.4)
from phenotype_sdk import MCPEntryPoint, PlaywrightAuthAdapter

# COMPAT (still works via alias)
from phenosdk import MCPEntryPoint, PlaywrightAuthAdapter  # → phenotype-sdk
```

---

## Next Steps

1. **Review & Approval**: Review this plan with team
2. **Create worktree**: `git checkout -b phase2-python-decomposition`
3. **Phase 2.1 kickoff**: Assign WI-2.1.1 through WI-2.1.4
4. **Parallel work**: Phases 2.2-2.4 can begin after 2.1 merges to main
5. **Release cadence**: Release each package as it completes (v0.1.0 → v0.2.0 path)

---

## Appendix: File Move Matrix

### Phase 2.1: MCP Core Extraction

| Old Path | New Path | Status |
|----------|----------|--------|
| `python/pheno-mcp/src/pheno_mcp/mcp/entry_points.py` | `python/pheno-mcp-core/src/pheno_mcp_core/entry_points.py` | Move |
| `python/pheno-mcp/src/pheno_mcp/tools/tool_registry.py` | `python/pheno-mcp-core/src/pheno_mcp_core/tool_registry.py` | Move |
| `python/pheno-mcp/src/pheno_mcp/tools/decorators.py` | `python/pheno-mcp-core/src/pheno_mcp_core/decorators.py` | Move |
| `python/phenosdk/src/pheno/mcp/entry_points.py` | (removed, import from pheno-mcp-core) | Delete |
| `python/phenosdk/src/pheno/shared/mcp_entry_points.py` | (removed, import from pheno-mcp-core) | Delete |

### Phase 2.2: Agent Orchestration Extraction

| Old Path | New Path | Status |
|----------|----------|--------|
| `python/pheno-mcp/src/pheno_mcp/agents/orchestration.py` | `python/pheno-agents/src/pheno_agents/orchestrator.py` | Move |
| (embedded Agent, AgentRole, TaskDefinition classes) | `python/pheno-agents/src/pheno_agents/models.py` | Extract |

### Phase 2.3: Adapters Consolidation

| Old Path | New Path | Status |
|----------|----------|--------|
| `python/phenosdk/src/pheno/auth/` | `python/pheno-atoms/src/pheno_atoms/auth/` | Move |
| `python/phenosdk/src/pheno/adapters/persistence/` | `python/pheno-atoms/src/pheno_atoms/persistence/` | Move |
| `python/phenosdk/src/pheno/vector/` | `python/pheno-atoms/src/pheno_atoms/vector/` | Move |

### Phase 2.4: Orchestration Wrapper

| Old Path | New Path | Status |
|----------|----------|--------|
| `python/phenosdk/` | `python/phenotype-sdk/` | Rename/Refactor |
| `python/phenosdk-legacy/` | (new) | Compat alias |

---

## References

- **Current Structure**: `/Users/kooshapari/CodeProjects/Phenotype/repos/python/`
- **Specs**: `docs/reference/` (coordinating docs)
- **CI/CD**: `.github/workflows/` (publishing workflows)
- **Related Docs**:
  - OSS_WRAPPING_AUDIT_2026-03-29.md (libification initiative)
  - PHASE1_COMPLETION_SUMMARY.md (prior phases)
