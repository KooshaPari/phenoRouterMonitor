# Harbor Adapters Library Specification

**Project:** portage
**Status:** pending - requires deeper portage code discovery
**Type:** Library extraction

---

## Overview

Create a `harbor-adapters` library from portage's existing sandbox adapter patterns (Docker, E2B, Daytona, Kubernetes).

## Current State

Portage already implements swappable sandbox providers:
- `dockerfile-parse` - Dockerfile-based isolation
- `e2b` - E2B cloud sandbox
- `daytona` - Daytona cloud development environments  
- `kubernetes` - K8s pod-based isolation

## Target Architecture

```
harbor/
├── adapters/           # Concrete implementations
│   ├── docker.py
│   ├── e2b.py
│   ├── daytona.py
│   └── kubernetes.py
└── harbor-adapters/    # NEW: Shared interface library
    ├── __init__.py
    ├── base.py         # SandboxAdapter abstract base class
    ├── config.py       # AdapterConfig dataclass
    └── result.py       # ExecutionResult, SandboxMetrics
```

## Interface Design

```python
from abc import ABC, abstractmethod
from dataclasses import dataclass
from typing import Any

@dataclass
class ExecutionResult:
    """Result from sandbox execution."""
    stdout: str
    stderr: str
    exit_code: int
    duration_ms: int
    metrics: dict[str, Any]

@dataclass  
class AdapterConfig:
    """Base configuration for sandbox adapters."""
    timeout_seconds: int = 300
    memory_limit_mb: int = 2048
    cpu_limit: float = 1.0
    
class SandboxAdapter(ABC):
    """Abstract base class for sandbox adapters."""
    
    @abstractmethod
    async def execute(self, command: list[str], cwd: str | None = None) -> ExecutionResult:
        """Execute a command in the sandbox."""
        pass
    
    @abstractmethod
    async def health_check(self) -> bool:
        """Check if the sandbox service is available."""
        pass
    
    @abstractmethod
    async def cleanup(self, sandbox_id: str) -> None:
        """Clean up a sandbox instance."""
        pass
```

## Migration Path

1. **Phase 1:** Extract interface to `harbor-adapters` (1-2 hours)
2. **Phase 2:** Update adapters to inherit from base class (2 hours)
3. **Phase 3:** Add adapter registry for dynamic resolution (1 hour)

## LOC Savings

Estimated: 400+ LOC reduction through DRY adapter patterns.

---

_Last updated: 2026-04-03_