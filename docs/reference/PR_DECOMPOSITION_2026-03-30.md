# PR Decomposition Recommendations - 2026-03-30

## Executive Summary

Code review completed for 7 PRs across thegent and phenotype-infrakit repositories. 
Found 1 critical violation (PR #882) that requires immediate decomposition.

## PR Status Overview

| PR # | Repo | Title | Files | Additions | Status | ADR-015 |
|------|------|-------|-------|-----------|--------|---------|
| 882 | thegent | pr 876 fix | 583 | 95,524 | CONFLICTING | **VIOLATION** |
| 886 | thegent | governance consolidation | 55 | 10,358 | MERGEABLE | Borderline |
| 889 | thegent | stash merge cleanup | 16 | 884 | MERGEABLE | OK |
| 880 | thegent | ADR/PLAN specs | 3 | 957 | MERGEABLE | OK |
| 891 | thegent | CodeQL workflow | 1 | 107 | MERGEABLE | OK |
| 892 | thegent | phench observability | 9 | 250 | MERGEABLE | OK |
| 483 | infrakit | ADR-015 crate org | 15 | 7,433 | CONFLICTING | Borderline |

## Critical Issue: PR #882

### Size Violation
- **Files**: 583 (XL threshold: 100)
- **Additions**: 95,524 (XL threshold: 5,000)
- **Violation Factor**: 19x over XL threshold

### Recommended Decomposition

```
origin/main
├── chore/pr882-agent-base (~200 lines)
│   └── src/thegent/agents/base.py
├── feat/pr882-crew-system (~900 lines)
│   └── src/thegent/agents/crew/*
├── feat/pr882-cliproxy-manager (~1500 lines)
│   └── src/thegent/agents/cliproxy_manager.py
│   └── src/thegent/agents/cliproxy_data/*
├── feat/pr882-codex-proxy (~1260 lines)
│   └── src/thegent/agents/codex_proxy.py
├── feat/pr882-context-compactor (~156 lines)
│   └── src/thegent/agents/context_compactor.py
└── chore/pr882-data-json (~500 lines)
    └── src/thegent/agents/cliproxy_data/*.json
```

### Decomposition Rationale
1. Each PR focuses on a single agent subsystem
2. Clear dependency chain (base → crew → cliproxy → codex)
3. Each PR < 2000 lines for efficient review
4. Data files (JSON) separated from code

## PR #886 Borderline Case

### Files Breakdown
- `CONSOLIDATION_AUDIT.md` + `CONSOLIDATION_SUMMARY.md` (~900 lines)
- `PHASE4_CONSOLIDATION_PLAN.md` (~416 lines)
- `governance/*` templates (~4000 lines)
- `dotfiles/*` templates (~1500 lines)
- `scripts/distribution/*` (~850 lines)
- `templates/*` (~3200 lines)
- Code changes (~100 lines)

### Optional Split Path
```
Stack 1: docs/consolidation-audits (MERGEABLE first)
Stack 2: docs/governance-standards
Stack 3: chore/distribution-scripts
Stack 4: refactor/phench-updates (depends on 1-3)
```

## Architectural Observations

### thegent Current State
- 4,428 files total
- Largest files:
  - `service.py`: 2,423 lines (needs decomposition)
  - `run_execution_core_helpers.py`: 1,670 lines
  - `workstream_autosync_shared.py`: 1,380 lines
  - `cliproxy_adapter.py`: 1,267 lines
  - `codex_proxy.py`: 1,264 lines

### Recommended Decomposition Patterns

#### 1. Plugin Architecture (Composio-style)
```
src/thegent/
├── agents/
│   ├── base.py              # Abstract base + Protocol
│   ├── protocols/           # Agent protocol definitions
│   │   ├── __init__.py
│   │   ├── orchestrator.py
│   │   ├── executor.py
│   │   └── router.py
│   ├── crew/               # Crew system as plugin
│   │   ├── __init__.py
│   │   └── ...
│   ├── cliproxy/            # CLIProxy as plugin
│   │   ├── __init__.py
│   │   └── ...
│   └── implementations/
│       ├── codex.py
│       ├── anthropic.py
│       └── ...
```

#### 2. Swappable Router Pattern
```python
# Current: monolithic router
from thegent.utils.routing_impl.litellm_router import LiteLLMRouter

# Target: swappable via Protocol
from thegent.agents.protocols import RouterProtocol

class RouterRegistry:
    _routers: dict[str, type[RouterProtocol]] = {}
    
    @classmethod
    def register(cls, name: str, router: type[RouterProtocol]):
        cls._routers[name] = router
    
    @classmethod
    def get(cls, name: str) -> RouterProtocol:
        return cls._routers[name]()
```

#### 3. Adapter Pattern for CLI Providers
```python
# Current: direct imports
from thegent.agents.codex_proxy import CodexProxy

# Target: Protocol-based
from thegent.agents.protocols import AgentAdapter

class AgentAdapterRegistry:
    """Swappable adapters for different agent providers."""
    
    @staticmethod
    def get(provider: str) -> AgentAdapter:
        adapters = {
            "codex": CodexAdapter,
            "cursor": CursorAdapter,
            "claude": ClaudeAdapter,
        }
        return adapters[provider]()
```

## Process Recommendations

### Before Creating PRs
1. **Size Check**: `git diff --stat | wc -l` (target < 50 files)
2. **Single Responsibility**: Each PR = one feature/fix/refactor
3. **Test Scope**: PR should be testable in isolation

### Stacked PR Workflow
```bash
# Create base PR
git checkout -b stack/feature-base
git push origin stack/feature-base
gh pr create --base main

# Create dependent PRs
git checkout -b stack/feature-part2 --track stack/feature-base
# ... make changes ...
git push origin stack/feature-part2
gh pr create --base stack/feature-base

# Update all stacked PRs
git fetch origin
git rebase origin/main
git push --force-with-lease
```

### LOC Reduction Targets
| File | Current | Target | Strategy |
|------|---------|--------|----------|
| service.py | 2,423 | 500 | Extract to modules |
| run_execution_core_helpers.py | 1,670 | 500 | Split by function |
| workstream_autosync_shared.py | 1,380 | 500 | Extract sync adapters |
| cliproxy_adapter.py | 1,267 | 500 | Protocol-based refactor |
| codex_proxy.py | 1,264 | 500 | Extract to crew/cliproxy |

## Action Items

- [ ] Close PR #882 and create 6 stacked PRs
- [ ] Merge PRs #880, #891, #892 (small, mergeable)
- [ ] Decide on PR #886 split (optional)
- [ ] Resolve PR #483 conflicts (ADR-015)
- [ ] Create decomposition spike for largest files
- [ ] Add RouterProtocol and AgentAdapterProtocol

## References

- [ADR-015: Crate Organization](../adr/ADR-015-crate-organization.md)
- [thegent Architecture Overview](../platforms/thegent/ARCHITECTURE_OVERVIEW.md)
- [Composio Plugin Architecture](https://docs.composio.dev) (external reference)
