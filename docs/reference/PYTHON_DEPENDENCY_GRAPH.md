# Python Phenosdk Dependency Architecture

**Last Updated:** 2026-03-30
**Status:** Phase 2 Design
**Focus:** Module dependency graphs and coupling analysis

---

## Current State Dependency Graph

```
                    CURRENT (COUPLED)
┌──────────────────────────────────────────────────────────────┐
│                                                              │
│  phenosdk (atoms + MCP + adapters)                          │
│  ┌────────────────────────────────────────────┐             │
│  │ ├── pheno/auth/                           │             │
│  │ │   └── playwright_adapter.py             │             │
│  │ ├── pheno/mcp/                            │ ← PROBLEM   │
│  │ │   └── entry_points.py [DUPLICATE]      │ (also in    │
│  │ ├── pheno/shared/                         │  pheno-mcp) │
│  │ │   └── mcp_entry_points.py               │             │
│  │ ├── pheno/adapters/                       │             │
│  │ │   ├── persistence/in_memory.py          │             │
│  │ │   └── persistence/sqlalchemy_adapter.py │             │
│  │ └── pheno/vector/                         │             │
│  │     └── client.py                         │             │
│  └────────────────────────────────────────────┘             │
│              ▲                                               │
│              │ (implicit dependency on pheno-core)         │
│              │ (shared MCP abstraction)                     │
│              │                                               │
│  pheno-core  │          pheno-mcp [ALSO HAS MCP]           │
│  ┌──────────┐          ┌──────────────────────┐            │
│  │ errors   │          │ mcp/                 │ ← PROBLEM  │
│  │ config   ├──────────→ entry_points.py      │ (duplicate │
│  │ logging  │          │ [DUPLICATE] ┌────┐  │  entry     │
│  │ observ.  │          │             │ x2 │  │  points)   │
│  └──────────┘          │ tools/      └────┘  │            │
│                        │ ├── decorators.py    │            │
│                        │ ├── registry.py      │            │
│                        │ agents/              │            │
│                        │ ├── orchestration.py │            │
│                        │ └── __init__.py      │            │
│                        └──────────────────────┘            │
│                                                              │
└──────────────────────────────────────────────────────────────┘

ISSUES:
  1. MCPEntryPoint defined in TWO places
     - phenosdk/src/pheno/mcp/entry_points.py
     - pheno-mcp/src/pheno_mcp/mcp/entry_points.py
  2. BaseMCPEntryPoint also duplicated
     - phenosdk/src/pheno/shared/mcp_entry_points.py
  3. Agent/AgentOrchestrator tightly coupled to pheno-mcp
     - Hard to test independently
     - Hard to use agent patterns without MCP
  4. Adapters scattered across pheno-atoms responsibilities
     - No clear interface
     - No consistent error handling
```

---

## Phase 2 Target Dependency Graph

```
                    PHASE 2 FINAL (CLEAN SEPARATION)

TIER 1: FOUNDATION
┌────────────────────────────────────────────────────────────┐
│                                                            │
│  pheno-core v0.1.0                                         │
│  ├── errors.py: ZenMCPError, ConfigurationError          │
│  ├── config.py: BaseConfig, from_env()                    │
│  ├── logging.py: structlog integration                     │
│  └── observability.py: health checks, metrics             │
│                                                            │
│  Dependencies: [pydantic>=2.0, structlog>=24.1.0]         │
│  Dependents: pheno-mcp-core, pheno-agents, pheno-atoms   │
│                                                            │
└────────────────────────────────────────────────────────────┘
                           ▲
                           │
                      (required)
                           │
┌────────────────────────────────────────────────────────────┐
│                                                            │
│  TIER 2a: PROTOCOL ABSTRACTION                            │
│                                                            │
│  pheno-mcp-core v0.1.0 [NEW]                              │
│  ├── entry_points.py: MCPEntryPoint, MCPServer           │
│  ├── tool_registry.py: ToolRegistry                       │
│  └── decorators.py: @mcp_tool, ToolMetadata              │
│                                                            │
│  Dependencies: [pydantic>=2.0, pheno-core]                │
│  Dependents: pheno-mcp (backward compat), phenotype-sdk   │
│                                                            │
└────────────────────────────────────────────────────────────┘
                           ▲
                           │
                      (required)
                           │
┌────────────────────────────────────────────────────────────┐
│                                                            │
│  TIER 2b: AGENT ORCHESTRATION                             │
│                                                            │
│  pheno-agents v0.1.0 [NEW]                                │
│  ├── models.py:                                           │
│  │   ├── Agent, AgentRole                                │
│  │   └── TaskDefinition                                   │
│  └── orchestrator.py: AgentOrchestrator                   │
│                                                            │
│  Dependencies: [dataclasses, pheno-core]                  │
│  Dependents: pheno-mcp (re-export), phenotype-sdk         │
│                                                            │
└────────────────────────────────────────────────────────────┘
                           ▲
                           │
                      (required)
                           │
┌────────────────────────────────────────────────────────────┐
│                                                            │
│  TIER 3: ADAPTER IMPLEMENTATIONS                          │
│                                                            │
│  pheno-atoms v0.1.0 [NEW]                                 │
│  ├── auth/                                                │
│  │   └── playwright_adapter.py: PlaywrightAuthAdapter    │
│  ├── persistence/                                         │
│  │   ├── in_memory.py: InMemoryPersistence               │
│  │   └── sqlalchemy_adapter.py: SQLAlchemyAdapter        │
│  └── vector/                                              │
│      └── client.py: VectorSearchClient                    │
│                                                            │
│  Dependencies: [pheno-core]                               │
│  Optional: [playwright>=1.40, sqlalchemy>=2.0, ...]       │
│  Dependents: phenotype-sdk                                │
│                                                            │
└────────────────────────────────────────────────────────────┘
                           ▲
                           │
                      (required)
                           │
┌────────────────────────────────────────────────────────────┐
│                                                            │
│  TIER 4: PUBLIC SDK (ORCHESTRATION)                       │
│                                                            │
│  phenotype-sdk v0.1.0 [NEW, replaces phenosdk]           │
│  ├── __init__.py: Aggregated facade                       │
│  ├── Re-exports from pheno-core                           │
│  ├── Re-exports from pheno-mcp-core                       │
│  ├── Re-exports from pheno-agents                         │
│  └── Re-exports from pheno-atoms                          │
│                                                            │
│  Dependencies: [pheno-core, pheno-mcp-core, pheno-agents, │
│                 pheno-atoms]                              │
│  Dependents: End users, applications                      │
│                                                            │
└────────────────────────────────────────────────────────────┘

BACKWARD COMPAT:
┌────────────────────────────────────────────────────────────┐
│  phenosdk-legacy v0.1.0 [DEPRECATED ALIAS]               │
│  └── Depends on: phenotype-sdk>=0.1.0                    │
│  └── From: from phenosdk import * → works via alias      │
└────────────────────────────────────────────────────────────┘
```

---

## Import Dependency Matrix (Phase 2)

| From Package | To Package | Module | What's Imported |
|--------------|-----------|--------|-----------------|
| pheno-mcp-core | pheno-core | (all) | errors, config, logging, observability |
| pheno-agents | pheno-core | models, orchestrator | errors, config, logging |
| pheno-atoms | pheno-core | (all adapters) | errors, config, logging |
| pheno-mcp | pheno-mcp-core | tools, mcp | MCPEntryPoint, ToolRegistry, @mcp_tool |
| pheno-mcp | pheno-agents | agents | Agent, AgentRole, TaskDefinition |
| phenotype-sdk | pheno-core | (__init__) | All core exports |
| phenotype-sdk | pheno-mcp-core | (__init__) | All MCP core exports |
| phenotype-sdk | pheno-agents | (__init__) | All agent exports |
| phenotype-sdk | pheno-atoms | (__init__) | All adapter exports |

**Circular Dependencies:** NONE (DAG maintained)

---

## Current Code Duplication (Before Phase 2)

### Entry Point Duplication

**Location 1:** `python/pheno-mcp/src/pheno_mcp/mcp/entry_points.py`
```
Lines: 1-171
Classes: MCPEntryPoint, MCPServer
```

**Location 2:** `python/phenosdk/src/pheno/mcp/entry_points.py`
```
Lines: 1-67 (incomplete stub)
Classes: MCPEntryPoint (incomplete)
```

**Shared Base:** `python/phenosdk/src/pheno/shared/mcp_entry_points.py`
```
Lines: 1-50 (stub)
Classes: BaseMCPEntryPoint, MCPConfiguration, MCPEntryPointRegistry
```

**Impact:**
- Two competing implementations of MCPEntryPoint
- No shared interface/protocol
- Confusion about which to use
- 300+ LOC of duplication

**Resolution (Phase 2.1):**
- Canonical: `python/pheno-mcp-core/src/pheno_mcp_core/entry_points.py`
- Both pheno-mcp and phenosdk import from pheno-mcp-core
- Delete duplicates

---

### Agent Orchestration Coupling

**Current Location:** `python/pheno-mcp/src/pheno_mcp/agents/orchestration.py`
```
Classes: Agent, AgentRole, TaskDefinition, AgentOrchestrator
Tightly coupled to MCP tooling layer
~330 LOC
```

**Issues:**
- Agent patterns useful independent of MCP
- Hard to test agent orchestration in isolation
- Can't use AgentOrchestrator without pheno-mcp dependency

**Resolution (Phase 2.2):**
- Extract to: `python/pheno-agents/src/pheno_agents/`
- Create:
  - `models.py`: Agent, AgentRole, TaskDefinition
  - `orchestrator.py`: AgentOrchestrator
- pheno-agents zero app dependencies (only dataclasses + pheno-core)
- pheno-mcp re-exports for backward compatibility

---

### Adapter Fragmentation

**Current Locations:**
1. `python/phenosdk/src/pheno/auth/playwright_adapter.py` (130 LOC stub)
2. `python/phenosdk/src/pheno/adapters/persistence/in_memory.py` (27 LOC)
3. `python/phenosdk/src/pheno/adapters/persistence/sqlalchemy_adapter.py` (80 LOC stub)
4. `python/phenosdk/src/pheno/vector/client.py` (27 LOC stub)

**Issues:**
- No clear adapter pattern interface
- Inconsistent error handling
- No centralized adapter registry
- ~260 LOC scattered across modules

**Resolution (Phase 2.3):**
- Consolidate to: `python/pheno-atoms/`
- Create clear adapter interfaces
- Each adapter in dedicated module
- Consistent error handling (via pheno-core)

---

## Module Relationship Diagram (Current → Target)

### Current Relationships (Coupling)

```
Current State: Coupled & Duplicated

    ┌─────────────────┐
    │  phenosdk       │
    │  (monolith)     │
    └────────┬────────┘
             │
      ┌──────┴──────┐
      │             │
      ▼             ▼
  pheno-core    pheno-mcp
      │             │
      └─────┬───────┘
            │
      (many adapter
       stubs, incomplete
       implementations)

PROBLEMS:
- phenosdk imports from pheno-mcp (shared MCP code)
- pheno-mcp has its own adapters (duplication)
- No clear separation of concerns
- Cyclic implicit dependencies
```

### Target Relationships (Clean DAG)

```
Target State: Clean Separation (Phase 2 Final)

End Users
    │
    ▼
┌─────────────────────┐
│ phenotype-sdk v0.1  │ (Facade layer)
└──────┬──────────────┘
       │
       ├──────┬────────┬──────────┐
       ▼      ▼        ▼          ▼
    pheno- pheno-   pheno-    pheno-
    core   mcp-core agents    atoms
      ▲       │       │          │
      │       ▼       │          │
      │      core ◄───┘          │
      │                          │
      └──────────────────────────┘

PROPERTIES:
- DAG: No cycles
- Three dependency layers:
  1. Core (no deps)
  2. Abstractions (→ core)
  3. Implementations (→ core/abstractions)
- Each package independently versioned
- Each package independently testable
- Each package independently deployable
```

---

## Dependency Depth Analysis

### Current (Monolithic)

```
Dependency Depth: 1 level
- phenosdk (top-level, brings in everything)
  └── pheno-core (foundation)

Problem: All-or-nothing dependency
```

### Phase 2 Target (Modular)

```
Dependency Depth: 4 levels (clean hierarchy)

Level 4 (End User):
  └── phenotype-sdk

Level 3 (Orchestration):
  ├── pheno-mcp
  └── (legacy apps import pheno-mcp directly)

Level 2 (Abstractions):
  ├── pheno-mcp-core
  ├── pheno-agents
  └── pheno-atoms

Level 1 (Foundation):
  └── pheno-core

Benefit: Users can import only what they need
- Want just config? → from pheno_core import BaseConfig
- Want MCP? → from pheno_mcp_core import MCPEntryPoint
- Want agents? → from pheno_agents import Agent
- Want all? → from phenotype_sdk import *
```

---

## Circular Dependency Detection

### Current State Check

```bash
$ pipdeptree
pheno-core
pheno-mcp
  └── pydantic>=2.0  # OK
phenosdk (standalone, no declared deps)
```

**Result:** No direct cycles (monolith avoids them by being one package)

### Phase 2 State Check

```bash
$ pipdeptree
pheno-core
pheno-mcp-core
  └── pheno-core
pheno-agents
  └── pheno-core
pheno-atoms
  └── pheno-core
pheno-mcp
  ├── pheno-mcp-core
  └── pheno-agents
phenotype-sdk
  ├── pheno-core
  ├── pheno-mcp-core
  ├── pheno-agents
  └── pheno-atoms
```

**Validation Script:**

```python
# python/tests/test_no_circular_deps.py
import subprocess
import re

def test_no_circular_dependencies():
    """Verify no circular dependencies in decomposed SDK."""
    result = subprocess.run(
        ["pipdeptree", "--warn", "fail"],
        capture_output=True,
        text=True
    )

    # Should exit with code 0 (no warnings)
    assert result.returncode == 0, f"Circular deps found:\n{result.stdout}"

    # Verify expected packages exist
    output = result.stdout
    assert "pheno-core" in output
    assert "pheno-mcp-core" in output
    assert "pheno-agents" in output
    assert "pheno-atoms" in output
    assert "phenotype-sdk" in output
```

---

## Import Order Validation

### Pre-Phase-2.1 (Current Works)

```python
# This works because everything is one package
from phenosdk import *  # OK
```

### Post-Phase-2 (Proper Layering)

```python
# Layer 1: Foundation
from pheno_core import BaseConfig, ConfigurationError, ZenMCPError

# Layer 2: Abstractions
from pheno_mcp_core import MCPEntryPoint, ToolRegistry
from pheno_agents import Agent, AgentOrchestrator

# Layer 3: Implementations
from pheno_atoms import PlaywrightAuthAdapter, InMemoryPersistence

# Layer 4: Full SDK
from phenotype_sdk import *  # Comprehensive facade

# Backward Compat
from phenosdk import *  # Still works (via alias)
```

---

## Missing Interfaces (Post-Phase-2 Recommendations)

### Adapter Interface Pattern

**Recommendation:** Add abstract base classes in each adapter module

```python
# pheno-atoms/src/pheno_atoms/auth/base.py
from abc import ABC, abstractmethod

class AuthAdapter(ABC):
    """Abstract base class for authentication adapters."""

    @abstractmethod
    def authenticate(self, credentials: dict) -> str:
        """Authenticate and return token."""

    @abstractmethod
    def refresh_token(self, token: str) -> str:
        """Refresh authentication token."""

    @abstractmethod
    def validate_token(self, token: str) -> bool:
        """Validate token validity."""

# pheno-atoms/src/pheno_atoms/auth/playwright_adapter.py
class PlaywrightAuthAdapter(AuthAdapter):
    """Playwright-based authentication adapter."""
    # ... implementation ...
```

**Benefit:**
- Type hints for adapter implementations
- Clear contract for custom adapters
- Dependency inversion (depend on interfaces, not implementations)

---

## Phase 2 Delivery Checklist

- [ ] **Phase 2.1: MCP Core**
  - [ ] Package created & builds
  - [ ] Zero duplicate MCPEntryPoint
  - [ ] Backward compat in pheno-mcp
  - [ ] All tests pass
  - [ ] pipdeptree clean

- [ ] **Phase 2.2: Agent Orchestration**
  - [ ] Package created & builds
  - [ ] Agent classes extracted
  - [ ] Backward compat in pheno-mcp
  - [ ] All tests pass

- [ ] **Phase 2.3: Adapters**
  - [ ] Package created & builds
  - [ ] Adapters extracted & consolidated
  - [ ] Optional deps configured
  - [ ] All tests pass

- [ ] **Phase 2.4: SDK Orchestration**
  - [ ] phenotype-sdk facade created
  - [ ] All 5 packages published
  - [ ] Integration tests pass
  - [ ] phenosdk-legacy alias works

---

## References

- **Dependency Analysis:** Current code at `/Users/kooshapari/CodeProjects/Phenotype/repos/python/`
- **Related Docs:**
  - PYTHON_DECOMPOSITION_PLAN.md (detailed work items)
  - OSS_WRAPPING_AUDIT_2026-03-29.md (libification context)
