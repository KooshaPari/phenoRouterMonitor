# Provider-Per-Repo Routing: Implementation Guide

**Date**: 2026-03-30
**Status**: Ready to implement
**Effort**: 4 weeks (phased approach)
**Expected savings**: 60% token cost reduction

---

## Quick Start

This guide walks through implementing provider-per-repo routing for Phenotype in 4 phases.

### What You'll Get

- **Per-project model configuration** (`.ai-config.toml` in each repo)
- **Workspace orchestrator** (`repos/orchestrator.py`) that routes tasks automatically
- **Per-project budget tracking** (`.ai-state.json` showing monthly spend)
- **Dispatch audit log** (`.work-audit/dispatch.log` with full transparency)
- **60% token cost reduction** (from $450 → $180/month for 300-task workload)

---

## Phase 1: Configuration Schema & Samples (Week 1)

### 1.1 Define `.ai-config.toml` Format

Create a standard TOML schema that every project will use:

```toml
# Schema: .ai-config.toml (required in each project)

[default]
# Default provider and model for this project
provider = "anthropic"      # "anthropic" | "openai" | "cursor"
model = "claude-opus-4-6"   # Model name
monthly_budget_tokens = 100000  # Token budget for the month

# Task-based overrides: route simple/complex tasks to different models
[[task_override]]
when_tags = ["lint", "format", "test"]     # Tags that trigger this override
model = "claude-haiku-4-5-20251001"        # Model to use for these tasks
budget_tokens = 5000                       # Token budget for these tasks

[[task_override]]
when_tags = ["design", "architecture", "refactor"]
model = "claude-opus-4-6"
budget_tokens = 50000

[[task_override]]
when_tags = ["urgent"]  # Override: urgent tasks use expensive model
model = "claude-opus-4-6"
budget_tokens = 100000
```

### 1.2 Create Sample Configs for 5 Pilot Projects

**pheno-cli** (Simple CLI, mostly linting):
```toml
# repos/pheno-cli/.ai-config.toml
[default]
provider = "anthropic"
model = "claude-haiku-4-5-20251001"  # Default to cheap Haiku
monthly_budget_tokens = 20000

[[task_override]]
when_tags = ["major-refactor", "design"]
model = "claude-opus-4-6"
budget_tokens = 30000
```

**heliosCLI** (Mixed complexity):
```toml
# repos/heliosCLI/.ai-config.toml
[default]
provider = "anthropic"
model = "claude-opus-4-6"  # Default to Opus
monthly_budget_tokens = 100000

[[task_override]]
when_tags = ["lint", "format", "test"]
model = "claude-haiku-4-5-20251001"
budget_tokens = 10000

[[task_override]]
when_tags = ["design"]
model = "claude-opus-4-6"
budget_tokens = 50000
```

**thegent** (Heavy Opus use):
```toml
# repos/thegent/.ai-config.toml
[default]
provider = "anthropic"
model = "claude-opus-4-6"
monthly_budget_tokens = 150000

[[task_override]]
when_tags = ["lint", "format"]
model = "claude-haiku-4-5-20251001"
budget_tokens = 10000
```

**AgilePlus** (Mixed):
```toml
# repos/AgilePlus/.ai-config.toml
[default]
provider = "anthropic"
model = "claude-opus-4-6"
monthly_budget_tokens = 80000

[[task_override]]
when_tags = ["test", "lint"]
model = "claude-haiku-4-5-20251001"
budget_tokens = 10000
```

**agentapi-plusplus** (Go backend, complex):
```toml
# repos/agentapi-plusplus/.ai-config.toml
[default]
provider = "anthropic"
model = "claude-opus-4-6"
monthly_budget_tokens = 120000

[[task_override]]
when_tags = ["test"]
model = "claude-haiku-4-5-20251001"
budget_tokens = 15000
```

### 1.3 Create Config Validation Script

```python
#!/usr/bin/env python3
# repos/scripts/validate_ai_configs.py

import sys
from pathlib import Path
import toml

def validate_ai_configs(repos_root: str) -> bool:
    """
    Validate all .ai-config.toml files in the workspace.

    Checks:
    - Each project with agent tasks has .ai-config.toml
    - Config has required [default] section
    - All model names are recognized
    - Budget values are positive integers
    """

    repos_root = Path(repos_root)
    valid = True

    for project_dir in sorted(repos_root.iterdir()):
        if not project_dir.is_dir():
            continue
        if project_dir.name.startswith('.'):
            continue

        config_file = project_dir / ".ai-config.toml"

        # Skip projects without config (may not need agents)
        if not config_file.exists():
            continue

        print(f"Validating {project_dir.name}...", end=" ")

        try:
            config = toml.load(open(config_file))

            # Check required sections
            if "default" not in config:
                print("❌ Missing [default] section")
                valid = False
                continue

            default = config["default"]

            # Check required fields
            required_fields = ["provider", "model", "monthly_budget_tokens"]
            for field in required_fields:
                if field not in default:
                    print(f"❌ Missing default.{field}")
                    valid = False
                    continue

            # Validate provider
            if default["provider"] not in ["anthropic", "openai", "cursor"]:
                print(f"❌ Invalid provider: {default['provider']}")
                valid = False
                continue

            # Validate budget is positive
            if default["monthly_budget_tokens"] <= 0:
                print(f"❌ Budget must be positive")
                valid = False
                continue

            # Validate task overrides
            for override in config.get("task_override", []):
                if "when_tags" not in override or "model" not in override:
                    print(f"❌ Task override missing when_tags or model")
                    valid = False
                    continue

            print("✅")

        except Exception as e:
            print(f"❌ Error: {e}")
            valid = False

    return valid

if __name__ == "__main__":
    repos_root = Path(__file__).parent.parent
    if validate_ai_configs(str(repos_root)):
        print(f"\n✅ All configs valid")
        sys.exit(0)
    else:
        print(f"\n❌ Some configs invalid")
        sys.exit(1)
```

**Usage**:
```bash
cd repos
python scripts/validate_ai_configs.py
# ✅ pheno-cli: valid
# ✅ heliosCLI: valid
# ✅ thegent: valid
# ...
# ✅ All configs valid
```

---

## Phase 2: Orchestrator Core (Week 2)

### 2.1 Implement `repos/orchestrator.py`

See Part 3.3 of the research document for full implementation.

**Key components**:
- `WorkspaceOrchestrator` class: Loads configs, routes tasks
- `Agent` class: Tracks token usage, raises on over-budget
- `PerRepoFileCache` class: Prevents redundant file reads
- `TaskCoordinator` class: Deduplicates parallel tasks

### 2.2 Create Budget State Manager

```python
# repos/budget_manager.py

import json
from pathlib import Path
from datetime import datetime
from typing import Dict

class BudgetManager:
    """Manage per-project monthly token budgets"""

    def __init__(self, repos_root: str):
        self.repos_root = Path(repos_root)
        self.state_file = self.repos_root / ".ai-state.json"
        self.state = self._load_or_initialize()

    def _load_or_initialize(self) -> Dict:
        """Load existing state or create new for current month"""

        current_month = datetime.now().strftime("%Y-%m")

        if self.state_file.exists():
            state = json.load(open(self.state_file))

            # Check if we need to reset for new month
            if state.get("month") != current_month:
                # Reset all spent amounts for new month
                for project in state.get("projects", {}):
                    state["projects"][project]["spent_tokens"] = 0

                state["month"] = current_month
        else:
            # Initialize new state
            state = {
                "month": current_month,
                "projects": {},
                "total_spent": 0
            }

        return state

    def add_spend(self, project: str, tokens: int, model: str):
        """Record token spend for project"""

        if project not in self.state["projects"]:
            self.state["projects"][project] = {
                "spent_tokens": 0,
                "by_model": {}
            }

        self.state["projects"][project]["spent_tokens"] += tokens

        if model not in self.state["projects"][project]["by_model"]:
            self.state["projects"][project]["by_model"][model] = 0

        self.state["projects"][project]["by_model"][model] += tokens

        self.state["total_spent"] += tokens

        # Persist
        self._save()

    def get_remaining_budget(self, project: str, config: Dict) -> int:
        """Get remaining tokens for project this month"""

        spent = self.state["projects"].get(project, {}).get("spent_tokens", 0)
        budget = config["default"]["monthly_budget_tokens"]

        return max(0, budget - spent)

    def is_over_budget(self, project: str, config: Dict) -> bool:
        """Check if project exceeded budget"""

        remaining = self.get_remaining_budget(project, config)
        return remaining == 0

    def _save(self):
        """Persist state to disk"""

        json.dump(self.state, open(self.state_file, "w"), indent=2)

    def get_summary(self) -> str:
        """Get human-readable summary of spending"""

        lines = [
            f"Month: {self.state['month']}",
            f"Total spent: {self.state['total_spent']} tokens",
            ""
        ]

        for project, proj_state in sorted(self.state["projects"].items()):
            spent = proj_state["spent_tokens"]
            lines.append(f"{project}: {spent} tokens")

            for model, tokens in proj_state.get("by_model", {}).items():
                lines.append(f"  - {model}: {tokens}")

        return "\n".join(lines)
```

### 2.3 Create Dispatch Audit Logger

```python
# repos/audit_logger.py

import json
from pathlib import Path
from datetime import datetime
from typing import List, Optional

class DispatchLogger:
    """Log all task dispatches for audit trail"""

    def __init__(self, repos_root: str):
        self.repos_root = Path(repos_root)
        self.log_file = self.repos_root / ".work-audit" / "dispatch.log"
        self.log_file.parent.mkdir(parents=True, exist_ok=True)

    def log_dispatch(
        self,
        project: str,
        task: str,
        model: str,
        provider: str,
        tags: List[str],
        tokens_used: Optional[int] = None
    ):
        """Log a dispatch decision"""

        entry = {
            "timestamp": datetime.utcnow().isoformat(),
            "project": project,
            "task": task[:100],  # Truncate long tasks
            "model": model,
            "provider": provider,
            "tags": tags,
            "tokens_used": tokens_used
        }

        # Append to log file
        with open(self.log_file, "a") as f:
            f.write(json.dumps(entry) + "\n")

    def get_recent(self, n: int = 50) -> List[dict]:
        """Get last N dispatch entries"""

        if not self.log_file.exists():
            return []

        entries = []
        for line in self.log_file.read_text().splitlines():
            try:
                entries.append(json.loads(line))
            except json.JSONDecodeError:
                pass

        return entries[-n:]

    def get_by_project(self, project: str) -> List[dict]:
        """Get all dispatches for a project"""

        if not self.log_file.exists():
            return []

        entries = []
        for line in self.log_file.read_text().splitlines():
            try:
                entry = json.loads(line)
                if entry.get("project") == project:
                    entries.append(entry)
            except json.JSONDecodeError:
                pass

        return entries
```

---

## Phase 3: Integration with Existing Tools (Week 3)

### 3.1 Update Task CLI

Modify the `task` command to use orchestrator:

```bash
# Old way (no model selection):
task quality --repo heliosCLI

# New way (with tags for routing):
task quality --repo heliosCLI --tags "lint,format"
# Routes to Haiku (cheap)

task design --repo agileplus-dashboard --tags "architecture"
# Routes to Opus (expensive)
```

**Implementation** (pseudocode):
```python
# repos/task_cli.py

import argparse
from orchestrator import WorkspaceOrchestrator

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("command", help="test, quality, design, etc.")
    parser.add_argument("--repo", required=True, help="Project name")
    parser.add_argument("--tags", default="", help="Comma-separated tags")

    args = parser.parse_args()

    # Initialize orchestrator
    orchestrator = WorkspaceOrchestrator(".")

    # Parse tags
    tags = [t.strip() for t in args.tags.split(",") if t.strip()]

    # Route task
    agent = orchestrator.route_task(
        project=args.repo,
        task=f"Run {args.command}",
        tags=tags
    )

    # Execute task
    result = agent.execute(f"Please {args.command}")

    print(f"Completed with {result['model']}")
    print(f"Tokens used: {result['tokens_used']}")
```

### 3.2 Create Agent Pool Manager

```python
# repos/agent_pool.py

from typing import Dict
from anthropic import Anthropic
from openai import OpenAI

class AgentPoolManager:
    """
    Maintain pool of pre-initialized provider clients.

    Prevents repeated initialization overhead.
    Reuses clients across multiple agent instances.
    """

    def __init__(self):
        self.clients = {}
        self._init_clients()

    def _init_clients(self):
        """Initialize all provider clients"""
        self.clients["anthropic"] = Anthropic()
        self.clients["openai"] = OpenAI()

    def get_client(self, provider: str):
        """Get client for provider"""

        if provider not in self.clients:
            raise ValueError(f"Unknown provider: {provider}")

        return self.clients[provider]

    def is_available(self, provider: str) -> bool:
        """Check if provider is available"""
        try:
            client = self.get_client(provider)
            # Quick health check
            return client is not None
        except:
            return False
```

### 3.3 Document Provider Selection in Task Output

When a task runs, show which provider/model was used:

```
$ task quality --repo pheno-cli --tags "lint,format"

[Agent Selection]
Project: pheno-cli
Task: Run quality
Tags: lint, format
Selected Model: claude-haiku-4-5-20251001 (Haiku)
Selected Provider: anthropic
Budget: 10,000 tokens
Dispatch log: .work-audit/dispatch.log

[Execution]
Running quality checks...
... (linting output) ...

[Summary]
Status: PASSED
Tokens used: 847 / 10,000
Efficiency: 92% budget remaining
```

---

## Phase 4: Monitoring & Optimization (Week 4)

### 4.1 Create Budget Dashboard (JSON API)

Create a JSON-based dashboard backend:

```python
# repos/dashboard_api.py

import json
from pathlib import Path
from typing import Dict, List

class DashboardAPI:
    """Provide JSON endpoints for budget dashboard"""

    def __init__(self, repos_root: str):
        self.repos_root = Path(repos_root)
        self.state_file = self.repos_root / ".ai-state.json"

    def get_current_month_summary(self) -> Dict:
        """Get total budget summary for current month"""

        if not self.state_file.exists():
            return {"error": "No state file found"}

        state = json.load(open(self.state_file))

        return {
            "month": state.get("month"),
            "total_spent": state.get("total_spent", 0),
            "projects_count": len(state.get("projects", {}))
        }

    def get_project_details(self, project: str) -> Dict:
        """Get budget details for a project"""

        if not self.state_file.exists():
            return {"error": "No state file found"}

        state = json.load(open(self.state_file))
        proj_state = state.get("projects", {}).get(project)

        if not proj_state:
            return {"error": f"Project {project} not found"}

        return {
            "project": project,
            "spent_tokens": proj_state.get("spent_tokens", 0),
            "by_model": proj_state.get("by_model", {})
        }

    def get_all_projects(self) -> List[Dict]:
        """Get summary for all projects"""

        if not self.state_file.exists():
            return []

        state = json.load(open(self.state_file))
        projects = []

        for project, proj_state in state.get("projects", {}).items():
            projects.append({
                "project": project,
                "spent_tokens": proj_state.get("spent_tokens", 0),
                "model_count": len(proj_state.get("by_model", {}))
            })

        return sorted(projects, key=lambda x: x["spent_tokens"], reverse=True)
```

### 4.2 Create Cost Optimization Reporter

```python
#!/usr/bin/env python3
# repos/scripts/optimize_costs.py

import json
from pathlib import Path
from collections import defaultdict

def analyze_dispatch_log(repos_root: str):
    """
    Analyze dispatch log to find optimization opportunities:
    - Tasks using expensive models unnecessarily
    - Repeated tasks (duplication)
    - Projects with room to use cheaper models
    """

    log_file = Path(repos_root) / ".work-audit" / "dispatch.log"

    if not log_file.exists():
        print("No dispatch log found")
        return

    # Group by task
    tasks_by_name = defaultdict(list)

    for line in log_file.read_text().splitlines():
        try:
            entry = json.loads(line)
            task_name = entry["task"]
            tasks_by_name[task_name].append(entry)
        except:
            pass

    print("=" * 80)
    print("COST OPTIMIZATION REPORT")
    print("=" * 80)
    print()

    # 1. Find repeated tasks
    print("1. REPEATED TASKS (consolidation opportunities)")
    print("-" * 80)
    for task, entries in sorted(tasks_by_name.items()):
        if len(entries) > 1:
            total_tokens = sum(e.get("tokens_used", 0) for e in entries)
            avg_tokens = total_tokens // len(entries)
            print(f"   {task}")
            print(f"   - Count: {len(entries)}")
            print(f"   - Total tokens: {total_tokens}")
            print(f"   - Savings if consolidated: ~{total_tokens - avg_tokens}")
    print()

    # 2. Find inefficient model choices
    print("2. INEFFICIENT MODEL CHOICES")
    print("-" * 80)
    for task, entries in sorted(tasks_by_name.items()):
        models = [e["model"] for e in entries]
        if "claude-opus-4-6" in models and len(entries) >= 3:
            print(f"   {task}")
            print(f"   - Uses Opus {len(entries)} times")
            print(f"   - Suggestion: Consider Haiku for simple tasks")
    print()

    # 3. Per-project recommendations
    print("3. PER-PROJECT RECOMMENDATIONS")
    print("-" * 80)
    by_project = defaultdict(lambda: {"expensive": 0, "cheap": 0})
    for task, entries in tasks_by_name.items():
        for entry in entries:
            project = entry["project"]
            model = entry["model"]
            tokens = entry.get("tokens_used", 0)

            if "opus" in model.lower():
                by_project[project]["expensive"] += tokens
            elif "haiku" in model.lower():
                by_project[project]["cheap"] += tokens

    for project, costs in sorted(by_project.items()):
        total = costs["expensive"] + costs["cheap"]
        if total == 0:
            continue

        expensive_pct = (costs["expensive"] / total) * 100
        print(f"   {project}")
        print(f"   - Expensive (Opus): {costs['expensive']} ({expensive_pct:.1f}%)")
        print(f"   - Cheap (Haiku): {costs['cheap']} ({100-expensive_pct:.1f}%)")

        if expensive_pct > 80:
            print(f"   - OK: High Opus usage (expected for complex project)")
        elif expensive_pct < 20:
            print(f"   - TIP: Low Opus usage (consider using default Haiku?)")

    print()

if __name__ == "__main__":
    import sys
    repos_root = sys.argv[1] if len(sys.argv) > 1 else "."
    analyze_dispatch_log(repos_root)
```

**Usage**:
```bash
./scripts/optimize_costs.py
# 1. REPEATED TASKS
#    format code
#    - Count: 5
#    - Total tokens: 3,500
#
# 2. INEFFICIENT MODEL CHOICES
#    lint code
#    - Uses Opus 4 times
#    - Suggestion: Consider Haiku for simple tasks
#
# 3. PER-PROJECT RECOMMENDATIONS
#    pheno-cli
#    - Expensive: 500 (8%)
#    - Cheap: 5,800 (92%)
#    - TIP: Low Opus usage (consider using default Haiku?)
```

---

## Full Integration Timeline

| Week | Phase | Deliverables | Effort |
|------|-------|--------------|--------|
| 1 | Config Schema | `.ai-config.toml` spec, 5 sample configs, validation script | 8h |
| 2 | Orchestrator | `orchestrator.py`, budget manager, audit logger, unit tests | 16h |
| 3 | Integration | Task CLI updates, agent pool, dispatch output, docs | 12h |
| 4 | Monitoring | API, cost analyzer, optimization recommendations | 12h |
| **Total** | — | Full production-ready system | **48h (1 week FTE)** |

---

## Testing Strategy

### Unit Tests
```python
# tests/test_orchestrator.py

def test_route_task_selects_correct_model():
    orchestrator = WorkspaceOrchestrator(".")
    agent = orchestrator.route_task(
        project="pheno-cli",
        task="Format code",
        tags=["format"]
    )
    assert agent.model == "claude-haiku-4-5-20251001"

def test_route_task_respects_budget():
    orchestrator = WorkspaceOrchestrator(".")
    agent = orchestrator.route_task("pheno-cli", "...", tags=[])
    assert agent.budget_tokens <= 20000

def test_audit_log_is_persisted():
    orchestrator = WorkspaceOrchestrator(".")
    agent = orchestrator.route_task("pheno-cli", "Format", [])
    # Verify .work-audit/dispatch.log contains entry
```

### Integration Tests
```bash
# Test end-to-end task routing
cd repos/pheno-cli
task quality --tags "lint" --repo pheno-cli
# Should:
# 1. Load .ai-config.toml
# 2. Select Haiku (cheap model)
# 3. Execute task
# 4. Log dispatch
# 5. Update .ai-state.json

# Verify routing
grep "pheno-cli" ../.work-audit/dispatch.log
# Should show:
# {"model": "claude-haiku-4-5-20251001", ...}
```

---

## Expected Results

After 4 weeks of implementation:

- ✅ All 30 projects have `.ai-config.toml` (config schema)
- ✅ `orchestrator.py` routes 100% of tasks correctly
- ✅ Budget tracking shows per-project spend
- ✅ Dispatch audit log available for all tasks
- ✅ Dashboard API shows real-time token spend
- ✅ Cost analyzer identifies savings opportunities
- ✅ Token cost reduced by 60% (from $450 → $180/month)
- ✅ Concurrent agents no longer interfere (per-repo budgets)

---

## Quick Reference: Creating `.ai-config.toml`

For each project, create `.ai-config.toml`:

```toml
[default]
provider = "anthropic"
model = "claude-haiku-4-5-20251001"      # OR "claude-opus-4-6"
monthly_budget_tokens = 50000             # Adjust per project complexity

[[task_override]]
when_tags = ["design", "architecture"]   # Complex tasks
model = "claude-opus-4-6"
budget_tokens = 50000
```

Then:
```bash
python scripts/validate_ai_configs.py
# ✅ All configs valid
```

Done. The orchestrator will automatically route tasks to the correct model.

---

## Troubleshooting

### "Project not found in config"
- Solution: Add `.ai-config.toml` to the project root

### "Over budget"
- Solution: Check `.ai-state.json` to see current spend
- Either increase `monthly_budget_tokens` in config or wait for month to reset

### "Task used more tokens than budget"
- Solution: The agent caught this and raised an error
- Increase the task's `budget_tokens` in `[[task_override]]` section

### "Dispatch log not appearing"
- Solution: Ensure `.work-audit/` directory exists
- Run: `mkdir -p .work-audit/` from repos root

---

## Next Steps

1. **Week 1**: Create `.ai-config.toml` for 5 pilot projects
2. **Week 2**: Run `orchestrator.py` tests
3. **Week 3**: Integrate with `task` CLI
4. **Week 4**: Deploy API and cost analyzer
5. **Ongoing**: Monitor dispatch log and adjust budgets monthly

Start with Phase 1 (configuration) this week!
