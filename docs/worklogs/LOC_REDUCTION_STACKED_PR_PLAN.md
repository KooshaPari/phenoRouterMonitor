# LOC Reduction & Stacked PR Plan - thegent

**Date**: 2026-03-30
**Status**: Planning

## Overview

thegent has several files exceeding the 500 LOC hard limit from ADR-015. This document outlines a stacked PR decomposition strategy.

## Target Files for Decomposition

| File | Current LOC | Target | Delta | Category |
|------|-------------|--------|-------|----------|
| phench/service.py | 2,423 | 500 | -1,923 | Core |
| cli/services/run_execution_core_helpers.py | 1,670 | 500 | -1,170 | CLI |
| integrations/workstream_autosync_shared.py | 1,380 | 500 | -880 | Integration |
| cliproxy_adapter.py | 1,267 | 500 | -767 | Adapter |
| agents/codex_proxy.py | 1,264 | 500 | -764 | Agent |
| agents/plangent.py | 1,044 | 500 | -544 | Agent |
| config/settings.py | 1,034 | 500 | -534 | Config |
| utils/routing_impl/litellm_router.py | 1,017 | 500 | -517 | Routing |

## Stacked PR Decomposition Strategy

```
origin/main
├── refactor/phench-service-decompose
│   ├── phench/service.py → modules/
│   ├── phench/models.py (new)
│   ├── phench/store.py (existing)
│   └── phench/runner.py (existing)
│
├── refactor/cli-execution-decompose
│   ├── cli/services/run_execution_core_helpers.py → modules/
│   ├── cli/services/build_runner.py (new)
│   └── cli/services/execute_run.py (new)
│
├── refactor/workstream-sync-decompose
│   ├── integrations/workstream_autosync_shared.py → adapters/
│   ├── integrations/adapters/sync_github.py (new)
│   └── integrations/adapters/sync_gitlab.py (new)
│
├── refactor/cliproxy-adapter-decompose
│   ├── cliproxy_adapter.py → protocols/ + implementations/
│   ├── agents/protocols/adapter.py (new - AgentAdapterProtocol)
│   ├── agents/protocols/router.py (new - RouterProtocol)
│   └── agents/implementations/*.py (new)
│
└── refactor/codex-proxy-decompose
    ├── agents/codex_proxy.py → agents/implementations/
    └── agents/crew/router.py (enhance)
```

## Protocol Definitions

### AgentAdapterProtocol
```python
# src/thegent/agents/protocols/adapter.py
from typing import Protocol, Any
from dataclasses import dataclass

@dataclass
class AgentResponse:
    content: str
    metadata: dict[str, Any]

class AgentAdapterProtocol(Protocol):
    """Swappable adapter for agent providers."""
    
    name: str
    provider: str
    
    async def execute(self, prompt: str, **kwargs) -> AgentResponse:
        """Execute prompt with the agent provider."""
        ...
    
    async def stream(self, prompt: str, **kwargs) -> AgentResponse:
        """Stream response from agent provider."""
        ...
    
    def supports_model(self, model: str) -> bool:
        """Check if adapter supports given model."""
        ...
```

### RouterProtocol
```python
# src/thegent/agents/protocols/router.py
from typing import Protocol, TYPE_CHECKING

if TYPE_CHECKING:
    from .adapter import AgentAdapterProtocol

@dataclass
class RouteDecision:
    adapter: AgentAdapterProtocol
    model: str
    cost: float
    latency_estimate: float

class RouterProtocol(Protocol):
    """Swappable router for agent selection."""
    
    async def route(self, request: Request) -> RouteDecision:
        """Select best adapter for request."""
        ...
    
    def register(self, adapter: AgentAdapterProtocol) -> None:
        """Register adapter for routing."""
        ...
```

## Implementation Order

### Phase 1: Protocol Foundation (Low Risk)
1. Create `agents/protocols/` directory
2. Add `__init__.py`
3. Define `AgentAdapterProtocol`
4. Define `RouterProtocol`
5. Create `AdapterRegistry` helper class
6. **PR**: `refactor/agent-protocols`

### Phase 2: Adapter Extraction (Medium Risk)
1. Extract `cliproxy_adapter.py` logic into implementations
2. Create `CodexAdapter`, `ClaudeAdapter`, `CursorAdapter`
3. Update imports in dependent files
4. **PR**: `refactor/extract-agent-adapters`

### Phase 3: Crew Router (Medium Risk)
1. Move crew routing logic to `agents/crew/router.py`
2. Add crew-specific protocol
3. **PR**: `feat/crew-router-protocol`

### Phase 4: Phench Service Decompose (High Risk)
1. Identify functional boundaries in `service.py`
2. Create module structure
3. Extract incrementally
4. **PR**: `refactor/phench-service-decompose`

### Phase 5: CLI Execution Decompose (High Risk)
1. Same approach as phench
2. Focus on testable units
3. **PR**: `refactor/cli-execution-decompose`

## Success Metrics

| Metric | Target | Current |
|--------|--------|---------|
| Max file LOC | ≤500 | 2,423 |
| Avg file LOC | ≤200 | ~150 |
| Protocol coverage | ≥80% adapters | 0% |
| Router swappability | Yes | No |

## Action Items

- [ ] Create agents/protocols/ directory
- [ ] Define AgentAdapterProtocol
- [ ] Define RouterProtocol
- [ ] Create AdapterRegistry
- [ ] Extract first adapter implementation
- [ ] Add tests for protocol contracts
