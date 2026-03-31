# Provider-Per-Repo Workflows in Polyrepos

## Executive Summary

This document research patterns, implementation strategies, and architectural approaches for managing **provider-per-repo** workflows in large polyrepos (30+ projects). The goal is to route different repositories to different AI providers (Claude, OpenAI, Cursor, etc.) based on configuration, while isolating token budgets and avoiding concurrent agent interference.

**Key Insight**: Multi-provider routing is not new—proven patterns exist in LiteLLM, OpenRouter, and Kubernetes. The challenge is *configuration-driven dispatch* at the polyrepo scale and *token budget isolation per-repo* to prevent waste.

---

## 1. Multi-Provider Routing Patterns

### 1.1 Conceptual Model

The fundamental pattern involves three layers:

```
┌─────────────────────────────────────┐
│ Polyrepo Router (Dispatcher)         │  ← Decision point: which provider?
├─────────────────────────────────────┤
│ Per-Repo Configuration               │  ← .ai-config.json per project
├─────────────────────────────────────┤
│ Provider Abstraction Layer           │  ← Unified interface (LiteLLM, OpenRouter)
├─────────────────────────────────────┤
│ Actual Providers                     │  ← Claude, OpenAI, Cursor, DeepSeek, etc.
└─────────────────────────────────────┘
```

### 1.2 Established Solutions (2026)

#### **Claude Code Router (CCR)**

**What it does**: Intercepts Claude Code requests before they reach Anthropic, applies routing rules, transforms payloads, forwards to target provider, and transforms responses back.

**Architecture**:
```
Claude Code CLI → CCR Proxy → Provider Selection → Transform → Target Provider
                                      ↓
                            Apply .ai-config rules
                            Check token budgets
                            Log usage metrics
```

**Advantages**:
- Works at the CLI level (transparent to code)
- Supports multiple routing strategies
- No changes required to project code
- Integrates with existing Claude Code workflows

**Reference**: [Claude Code Router on GitHub](https://github.com/musistudio/claude-code-router)

#### **LiteLLM Router (Production-Grade)**

**What it does**: Unified API gateway for 100+ LLM providers with built-in routing, load balancing, fallbacks, and retry logic.

**Routing Strategies** (from docs):
1. **Round-robin** — Distribute evenly across deployments
2. **Least busy** — Route to least-loaded provider
3. **User-defined** — Custom logic per request
4. **Fallback chain** — Primary → Secondary → Tertiary
5. **Context-window aware** — Choose provider based on token availability

**Fallback Hierarchy** (LiteLLM's multi-level approach):
```yaml
routing:
  order_1:
    - "gpt-4"
    - "gpt-4-turbo"
  order_2:
    - "claude-opus"
  order_3:
    - "gemini-pro"
    - "deepseek-v3"

  fallback_strategies:
    rate_limit: ["order_1" -> "order_2" -> "order_3"]
    token_limit: [use_larger_context_model]
    content_policy: [skip_provider]
```

**Cost-Aware Cooldown**:
- On rate limit (429), place deployment on cooldown immediately
- Retry within same order level first
- Jump to next order level only if all are on cooldown
- Exponential backoff with jitter

**Advantages**:
- Battle-tested in production
- Automatic cooldown management
- Multi-level fallback strategies
- Per-provider cost tracking

**Reference**: [LiteLLM Routing & Load Balancing](https://docs.litellm.ai/docs/routing-load-balancing)

#### **OpenRouter Unified API**

**What it does**: Proxy API that routes requests to dozens of providers (Claude, OpenAI, DeepSeek, local models, etc.) with unified authentication and billing.

**Request Flow**:
```
Client → OpenRouter Proxy → Provider Router → Selected Provider
         (single API key)    (intelligent dispatch)
```

**Advantages**:
- Single API key for all providers
- No need to manage multiple credentials
- Built-in cost tracking per provider
- Integrates with Claude Code via CCR

**Reference**: [OpenRouter Claude Code Integration](https://openrouter.ai/docs/guides/guides/claude-code-integration)

---

## 2. Configuration Patterns

### 2.1 Per-Repo Configuration File

**Location & Format**: `.ai-config.json` at project root

**Minimal Configuration**:
```json
{
  "project_id": "repo-name",
  "providers": {
    "primary": "claude-opus-4",
    "fallback": ["claude-sonnet-4", "gpt-4", "gemini-pro"]
  },
  "token_budget": {
    "monthly": 5_000_000,
    "per_session": 100_000,
    "warning_threshold": 0.8
  },
  "routing_rules": {
    "default": "primary",
    "code_review": "gpt-4",
    "documentation": "claude-sonnet-4",
    "testing": "deepseek-v3"
  },
  "cost_tracking": {
    "enabled": true,
    "track_per_file": true,
    "cost_center": "engineering/repo-name"
  }
}
```

**Extended Configuration** (with rate limits, timeouts, custom metadata):
```json
{
  "project_id": "phenotype-infrakit",
  "environment": "production",
  "providers": {
    "primary": {
      "name": "claude-opus-4",
      "timeout_seconds": 300,
      "retry_count": 3,
      "cooldown_minutes": 5,
      "cost_per_1k_input": 15.0,
      "cost_per_1k_output": 45.0
    },
    "fallback": [
      {
        "name": "claude-sonnet-4",
        "conditions": ["rate_limit_on_primary", "primary_timeout"]
      },
      {
        "name": "gpt-4",
        "conditions": ["all_claude_unavailable"]
      }
    ]
  },
  "token_budget": {
    "monthly_limit": 5_000_000,
    "daily_limit": 200_000,
    "per_session_limit": 100_000,
    "warning_at_percent": 80,
    "hard_stop_at_percent": 95
  },
  "routing_strategy": {
    "default": "round_robin",
    "overrides": {
      "code_generation": "claude-opus-4",
      "code_review": "gpt-4",
      "documentation": "claude-sonnet-4",
      "tests": "deepseek-v3"
    }
  },
  "cost_tracking": {
    "enabled": true,
    "granularity": "per_file",
    "export_format": "csv",
    "cost_center": "eng/infrakit",
    "tags": [
      "polyrepo",
      "rust-workspace",
      "critical"
    ]
  },
  "monitoring": {
    "slack_channel": "#ai-cost-alerts",
    "alert_on_budget_exceeded": true,
    "daily_report": true
  }
}
```

### 2.2 Global Routing Configuration

**Location**: `.ai-routing.yaml` at polyrepo root (or CI/CD environment)

```yaml
# Global routing rules for all repos
version: 1

# Repository-to-provider mapping
repositories:
  # High-priority, cost-critical repos → Claude (best for code)
  phenotype-infrakit:
    provider: claude-opus-4
    budget: 5M tokens/month
    priority: critical
    tier: tier-1

  AgilePlus:
    provider: claude-opus-4
    budget: 3M tokens/month
    priority: high
    tier: tier-1

  # Medium-priority → Balanced between Claude and OpenAI
  heliosCLI:
    provider: claude-sonnet-4
    fallback: gpt-4-turbo
    budget: 2M tokens/month
    priority: medium
    tier: tier-2

  agent-wave:
    provider: claude-sonnet-4
    fallback: [gpt-4, gemini-pro]
    budget: 1.5M tokens/month
    priority: medium
    tier: tier-2

  # Lower-priority or experimental → Cheaper providers
  phench:
    provider: gpt-4-turbo
    fallback: deepseek-v3
    budget: 1M tokens/month
    priority: low
    tier: tier-3

  experimental:
    provider: deepseek-v3  # Cheapest first for experiments
    fallback: [gpt-4, claude-sonnet-4]
    budget: 500K tokens/month
    priority: experimental
    tier: tier-3

# Global fallback rules
fallbacks:
  rate_limit: [primary, fallback_1, fallback_2]
  token_limit: use_larger_context_model
  timeout: [same_provider_retry, next_fallback]
  api_error: [cooldown_5min, next_fallback]

# Cost-aware routing
cost_strategy:
  mode: weighted
  weights:
    claude-opus-4: 1.0   # Baseline (most expensive, best quality)
    claude-sonnet-4: 0.5
    gpt-4-turbo: 0.6
    gemini-pro: 0.2      # Cheap and fast
    deepseek-v3: 0.1     # Cheapest

  rules:
    - condition: monthly_budget_exceeded_80%
      action: route_to_cheaper_provider
    - condition: session_duration_>_30min
      action: route_to_cheaper_model
    - condition: task_is_simple
      action: route_to_cheapest_capable

# Per-tier concurrency limits (prevent doubling/waste)
concurrency:
  tier-1:
    max_concurrent_agents: 5
    max_concurrent_sessions: 20
  tier-2:
    max_concurrent_agents: 10
    max_concurrent_sessions: 50
  tier-3:
    max_concurrent_agents: 20
    max_concurrent_sessions: 100

# Logging and monitoring
observability:
  log_level: info
  log_provider_decisions: true
  export_metrics: prometheus
  cost_tracking: enabled
  daily_cost_report: true
```

---

## 3. Implementation Approaches

### 3.1 Where Routing Logic Lives

#### **Option A: CLI/Proxy Layer (Claude Code Router Pattern)**

**Pros**:
- Transparent to application code
- Works with existing Claude Code workflows
- No code changes required
- Can be deployed independently

**Cons**:
- Extra network hop
- Latency overhead
- Difficult to trace through multiple layers

**Implementation**:
```bash
# Global routing proxy runs locally
claude-code-router start --config .ai-routing.yaml

# All Claude Code requests → Router → Provider
# Transparent to agent
```

#### **Option B: Configuration File + SDK (LiteLLM Pattern)**

**Pros**:
- No extra network hop
- Direct control over routing decisions
- Integrated cost tracking
- Better performance

**Cons**:
- Requires SDK integration in projects
- Need to update each agent/app
- More moving parts

**Implementation** (Python):
```python
from litellm import Router

router = Router(
    model_list=[
        {
            "model_name": "claude-opus-4",
            "litellm_params": {"model": "claude-3-5-opus-20241022"}
        },
        {
            "model_name": "gpt-4",
            "litellm_params": {"model": "gpt-4-turbo"}
        },
        {
            "model_name": "gemini-pro",
            "litellm_params": {"model": "gemini-1.5-pro"}
        }
    ],
    routing_strategy="least_busy",
    retry_attempts=3,
    timeout=300
)

# Router selects provider automatically
response = router.completion(
    model="claude-opus-4",  # Preferred model
    messages=[...],
    fallbacks=["gpt-4", "gemini-pro"]
)
```

#### **Option C: CI/CD Dispatch (GitHub Actions Pattern)**

**Pros**:
- Works at workflow level
- No runtime overhead
- Clear audit trail
- Can enforce policies (no simultaneous jobs)

**Cons**:
- Only works for CI tasks
- Not suitable for interactive agents
- Requires workflow configuration per repo

**Implementation** (GitHub Actions):
```yaml
name: Task Dispatch
on: workflow_dispatch

jobs:
  detect-provider:
    runs-on: ubuntu-latest
    outputs:
      provider: ${{ steps.route.outputs.provider }}
      model: ${{ steps.route.outputs.model }}
    steps:
      - uses: actions/checkout@v4
      - id: route
        run: |
          REPO_NAME=$(basename ${{ github.repository }})

          case "$REPO_NAME" in
            phenotype-infrakit|AgilePlus)
              echo "provider=claude" >> $GITHUB_OUTPUT
              echo "model=opus-4" >> $GITHUB_OUTPUT
              ;;
            heliosCLI|agent-wave)
              echo "provider=claude" >> $GITHUB_OUTPUT
              echo "model=sonnet-4" >> $GITHUB_OUTPUT
              ;;
            *)
              echo "provider=openai" >> $GITHUB_OUTPUT
              echo "model=gpt-4" >> $GITHUB_OUTPUT
              ;;
          esac

  run-task:
    needs: detect-provider
    runs-on: ubuntu-latest
    env:
      AI_PROVIDER: ${{ needs.detect-provider.outputs.provider }}
      AI_MODEL: ${{ needs.detect-provider.outputs.model }}
    steps:
      - uses: actions/checkout@v4
      - run: |
          echo "Running with $AI_PROVIDER/$AI_MODEL"
          # Task-specific commands
```

### 3.2 Recommended Pattern for Phenotype

**For 30+ projects + concurrent agent work, use hybrid approach**:

```
┌─────────────────────────────────────────────────────┐
│ Layer 1: CI/CD Router (GitHub Actions)              │
│ - Route GitHub-triggered tasks by repo              │
│ - Enforce concurrency limits                        │
│ - Cost tracking at workflow level                   │
└─────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────┐
│ Layer 2: Local Proxy (Claude Code Router)           │
│ - Intercept local Claude Code CLI requests          │
│ - Read .ai-config.json per repo                     │
│ - Maintain session-level budget tracking            │
└─────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────┐
│ Layer 3: Unified API (OpenRouter or LiteLLM)        │
│ - Consistent API across all providers               │
│ - Handle fallbacks & retries                        │
│ - Provider-specific payload transforms              │
└─────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────┐
│ Layer 4: Actual Providers                           │
│ Claude, OpenAI, Cursor, DeepSeek, etc.              │
└─────────────────────────────────────────────────────┘
```

---

## 4. Token Budget Isolation

### 4.1 Problem: Cost Explosion in Multi-Agent Environments

**Scenario**: Running 30+ concurrent agents (one per repo or feature) without isolation:
- Each agent unaware of global budget
- No coordination on expensive operations
- Easy to waste 10M tokens in one session
- No cost attribution to responsible team/repo

### 4.2 Solution: Hierarchical Budget Management

```
Global Monthly Budget: 50M tokens
├─ Tier-1 (Critical): 25M
│  ├─ phenotype-infrakit: 5M
│  ├─ AgilePlus: 3M
│  └─ ...
├─ Tier-2 (Medium): 15M
│  ├─ heliosCLI: 2M
│  ├─ agent-wave: 1.5M
│  └─ ...
└─ Tier-3 (Experimental): 10M
   ├─ phench: 500K
   └─ ...
```

### 4.3 Implementation: Budget Guards

```python
# At agent startup, check budget
class BudgetGuard:
    def __init__(self, repo: str, monthly_limit: int):
        self.repo = repo
        self.monthly_limit = monthly_limit
        self.used_this_month = self._load_usage()

    async def check_before_call(self, estimated_tokens: int) -> bool:
        """Before making API call, verify budget."""
        if self.used_this_month + estimated_tokens > self.monthly_limit:
            if self.used_this_month / self.monthly_limit > 0.95:
                raise BudgetExhaustedError(f"{self.repo} at 95% budget")
            else:
                # Allow but warn
                logger.warning(f"Budget warning: {self.repo} near limit")
        return True

    async def record_usage(self, input_tokens: int, output_tokens: int) -> None:
        """After call, record actual usage."""
        total = input_tokens + output_tokens
        self.used_this_month += total
        self._save_usage()

        if self.used_this_month / self.monthly_limit > 0.8:
            self._alert_team()

    def _load_usage(self) -> int:
        """Load from persistent store (Supabase, S3, etc.)."""
        return get_usage_metric(self.repo, current_month())

    def _save_usage(self) -> None:
        """Persist usage metrics."""
        save_usage_metric(self.repo, self.used_this_month)

    def _alert_team(self) -> None:
        """Send Slack/email alert."""
        notify_budget_warning(self.repo, self.used_this_month, self.monthly_limit)
```

### 4.4 Cost Tracking Per-Repository

**Ideal structure** (using Supabase or similar):

```sql
CREATE TABLE token_usage (
    id UUID PRIMARY KEY,
    repo_name TEXT NOT NULL,
    provider TEXT NOT NULL,
    model TEXT NOT NULL,
    date DATE NOT NULL,
    input_tokens INT NOT NULL,
    output_tokens INT NOT NULL,
    cost_usd DECIMAL(10, 4),
    agent_id TEXT,
    task_type TEXT,
    cost_center TEXT,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_token_usage_repo_date ON token_usage(repo_name, date);
CREATE INDEX idx_token_usage_cost_center ON token_usage(cost_center);

-- Query: Cost per repo this month
SELECT repo_name, SUM(cost_usd) as total_cost, SUM(input_tokens + output_tokens) as total_tokens
FROM token_usage
WHERE date >= DATE_TRUNC('month', NOW())
GROUP BY repo_name
ORDER BY total_cost DESC;

-- Query: Cost by provider
SELECT provider, SUM(cost_usd) as total_cost, COUNT(*) as call_count
FROM token_usage
WHERE date >= DATE_TRUNC('month', NOW())
GROUP BY provider;
```

---

## 5. CI/CD Integration

### 5.1 Problem: Doubling & Waste in CI

**Scenario**: Multiple agents run the same CI job simultaneously:
- Agent 1 processes PR, runs lints, tests → cost = 50K tokens
- Agent 2 (different worktree) processes same PR → cost = 50K tokens
- Agent 3 (same repo, parallel task) → cost = 50K tokens
- Total: 150K tokens wasted (should be 50K)

### 5.2 Solution: Job-Level Coordination

**Approach 1: Workflow Concurrency Controls** (GitHub Actions)

```yaml
concurrency:
  group: ${{ github.repository }}-${{ github.event.number || github.ref }}
  cancel-in-progress: true

jobs:
  lint-and-test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: task quality:full  # Runs once, not N times
```

**Approach 2: Job Registry** (Distributed Lock)

```python
# Before starting expensive operation, acquire lock
import redis

redis_client = redis.Redis(host='localhost', port=6379)

async def run_with_lock(job_id: str, timeout: int = 1800):
    """Ensure only one agent runs this job simultaneously."""
    lock_key = f"job_lock:{job_id}"

    # Try to acquire lock (non-blocking)
    lock_acquired = redis_client.set(
        lock_key,
        agent_id,
        nx=True,  # Only set if doesn't exist
        ex=timeout  # Expire after 30 min
    )

    if not lock_acquired:
        logger.info(f"Another agent already processing {job_id}, skipping")
        return  # Let other agent handle it

    try:
        # Run the expensive operation
        result = await process_job(job_id)
        return result
    finally:
        redis_client.delete(lock_key)  # Release lock
```

**Approach 3: Dynamic Task Matrix** (Per-Repository)

```yaml
name: Matrix Dispatch

on:
  push:
    branches: [main]

jobs:
  detect-changes:
    runs-on: ubuntu-latest
    outputs:
      matrix: ${{ steps.detect.outputs.matrix }}
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
      - id: detect
        run: |
          # Detect which repos changed
          CHANGED=$(git diff origin/main --name-only | sed 's|/.*||' | sort -u)

          # Build matrix
          MATRIX=$(echo "$CHANGED" | jq -R -s -c 'split("\n")[:-1] | {repo: .}')
          echo "matrix=$MATRIX" >> $GITHUB_OUTPUT

  process:
    needs: detect-changes
    strategy:
      matrix: ${{ fromJson(needs.detect-changes.outputs.matrix) }}
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: |
          cd ${{ matrix.repo }}
          task quality:full
          # Only processes changed repos, not all 30
```

---

## 6. Avoiding Doubling & Waste

### 6.1 Waste Vectors in Polyrepos

| Waste Type | Cause | Prevention |
|-----------|-------|-----------|
| **Duplicate Processing** | Multiple agents process same PR | Job locks, concurrency controls |
| **Unnecessary Runs** | CI runs on non-code changes | Path filters in workflows |
| **Expensive Re-analysis** | Re-running costly operations | Cache results in Supabase/S3 |
| **Silent Failures** | Failed job retries silently | Explicit failure logs + alerts |
| **Oversized Context** | Sending entire codebase to LLM | Chunking + semantic search |
| **No Deduplication** | Same analysis run multiple times | Content-addressable caching |

### 6.2 Concrete Guard Rails

```python
# Guard 1: Session-level budget cap
class SessionBudgetGuard:
    def __init__(self, session_id: str, max_tokens: int = 100_000):
        self.session_id = session_id
        self.max_tokens = max_tokens
        self.used = 0

    async def track_call(self, tokens: int) -> None:
        self.used += tokens
        if self.used > self.max_tokens:
            raise SessionBudgetExceededError(
                f"Session {self.session_id} exceeded {self.max_tokens} tokens"
            )

# Guard 2: Concurrent agent limiter
class ConcurrentAgentLimiter:
    def __init__(self, max_concurrent: int = 5):
        self.semaphore = asyncio.Semaphore(max_concurrent)

    async def __aenter__(self):
        await self.semaphore.acquire()

    async def __aexit__(self, *args):
        self.semaphore.release()

# Guard 3: Duplicate detection
class DuplicateDetector:
    def __init__(self, cache):
        self.cache = cache  # Redis or similar

    async def should_process(self, task_hash: str) -> bool:
        """Check if identical task already running."""
        running_key = f"task:{task_hash}:running"
        return not await self.cache.exists(running_key)

    async def mark_running(self, task_hash: str, ttl: int = 3600) -> None:
        await self.cache.set(f"task:{task_hash}:running", True, ex=ttl)

# Guard 4: Checksum-based result caching
class ResultCache:
    async def get_or_compute(self, input_hash: str, compute_fn):
        """Return cached result if hash matches, else compute."""
        cached = await self.cache.get(f"result:{input_hash}")
        if cached:
            return cached

        result = await compute_fn()
        await self.cache.set(f"result:{input_hash}", result, ex=86400)
        return result
```

---

## 7. Multi-Provider Fallback Chain

### 7.1 Strategy Hierarchy

**Tier-based Fallback** (LiteLLM inspired):

```
Tier 1 (Primary): claude-opus-4
  ↓ [if rate-limited]
Tier 2 (Secondary): claude-sonnet-4, gpt-4-turbo
  ↓ [if context insufficient]
Tier 3 (Tertiary): gemini-pro (large context)
  ↓ [if all above failed]
Tier 4 (Fallback): deepseek-v3 (cheap + acceptable quality)
```

**Error-Specific Fallback**:

```python
async def call_with_fallback(
    prompt: str,
    primary_model: str = "claude-opus-4",
    fallback_chain: list[str] = None
) -> str:
    """Call LLM with fallback strategy."""

    if fallback_chain is None:
        fallback_chain = ["claude-sonnet-4", "gpt-4", "gemini-pro", "deepseek-v3"]

    models_to_try = [primary_model] + fallback_chain

    for model in models_to_try:
        try:
            return await provider.call(model, prompt, timeout=300)

        except RateLimitError:
            logger.warning(f"{model} rate-limited, trying next")
            await asyncio.sleep(5)  # Cooldown
            continue

        except ContextWindowExceededError:
            logger.warning(f"{model} context full, need larger model")
            # Find next model with larger context
            next_model = find_larger_context_model(model, models_to_try)
            if next_model:
                models_to_try = [next_model] + [m for m in models_to_try if m != next_model]
            continue

        except ContentPolicyViolationError as e:
            logger.error(f"{model} rejected content: {e}")
            # Try different provider (content policies vary)
            continue

        except APIError as e:
            if e.status_code >= 500:
                logger.warning(f"{model} server error, cooling down")
                await asyncio.sleep(10)
            continue

    # Exhausted all options
    raise AllProvidersExhaustedError("All fallback chains failed")
```

---

## 8. Real-World Example: Phenotype Polyrepo Setup

### 8.1 Project Tiers

```
TIER 1 (Critical, $$$):
  - phenotype-infrakit (foundation lib)
  - AgilePlus (governance core)

TIER 2 (High-Priority, $$):
  - heliosCLI (user-facing)
  - agent-wave (orchestration)
  - cliproxyapi-plusplus (API gateway)

TIER 3 (Medium, $):
  - phench (testing harness)
  - agentapi-plusplus (agent runtime)
  - platforms/thegent (monorepo infra)

TIER 4 (Low, ¢):
  - Research repos
  - Experimental projects
  - Archived code
```

### 8.2 Configuration per Tier

**For TIER-1 (phenotype-infrakit)**:

```json
{
  "project_id": "phenotype-infrakit",
  "tier": "tier-1",
  "providers": {
    "primary": "claude-opus-4",
    "fallback": ["claude-sonnet-4", "gpt-4"]
  },
  "token_budget": {
    "monthly": 5_000_000,
    "per_session": 100_000,
    "daily": 200_000
  },
  "cost_threshold": 0.8,
  "alerts": {
    "slack": "#ai-tier1-budget",
    "pagerduty": true
  }
}
```

**For TIER-3 (phench)**:

```json
{
  "project_id": "phench",
  "tier": "tier-3",
  "providers": {
    "primary": "deepseek-v3",
    "fallback": ["gpt-4-turbo", "claude-sonnet-4"]
  },
  "token_budget": {
    "monthly": 500_000,
    "per_session": 20_000,
    "daily": 30_000
  }
}
```

### 8.3 Cost-Aware Routing Rules

```yaml
# .ai-routing.yaml (polyrepo root)

routing:
  phenotype-infrakit:
    provider: claude-opus-4
    budget_tier: 1
    monthly_limit: 5M
    cost_per_token: 0.00003  # Input tokens

  AgilePlus:
    provider: claude-opus-4
    budget_tier: 1
    monthly_limit: 3M

  heliosCLI:
    provider: claude-sonnet-4
    fallback: [gpt-4, gemini-pro]
    budget_tier: 2
    monthly_limit: 2M

  phench:
    provider: deepseek-v3
    fallback: [gpt-4-turbo, claude-sonnet-4]
    budget_tier: 4
    monthly_limit: 500K

# Cost-aware rules override provider selection
cost_rules:
  - if: "monthly_budget > 80%"
    then: "use fallback provider"
  - if: "daily_budget > 80%"
    then: "reject new sessions"
  - if: "tier-1 budget exhausted"
    then: "escalate to on-call engineer"
```

---

## 9. Pseudo-Code: Complete Routing Implementation

```python
# routing.py — Main router orchestrator

class PolyrepoRouter:
    """Route tasks to appropriate provider based on config and budget."""

    def __init__(self, routing_config: Path, budget_store: BudgetStore):
        self.config = load_yaml(routing_config)
        self.budget = budget_store
        self.logger = get_logger(__name__)

    async def route_task(
        self,
        repo: str,
        task_type: str,
        context: dict
    ) -> ProviderResponse:
        """Main routing logic."""

        # Step 1: Get repo config
        repo_config = self.config.repositories[repo]
        self.logger.info(f"Routing {repo}/{task_type} -> {repo_config.provider}")

        # Step 2: Check budget
        if not await self.budget.has_capacity(repo):
            self.logger.warning(f"{repo} budget exhausted, using fallback")
            selected_provider = repo_config.fallback[0]
        else:
            selected_provider = repo_config.provider

        # Step 3: Check cost awareness
        if await self.budget.is_near_limit(repo, threshold=0.8):
            # Route to cheaper provider
            selected_provider = self._select_cheaper_provider(
                selected_provider,
                repo_config.fallback
            )

        # Step 4: Check concurrency
        async with ConcurrencyLimiter(repo, max_concurrent=5):

            # Step 5: Call with fallback chain
            response = await self._call_with_fallback(
                selected_provider,
                repo_config.fallback,
                context
            )

        # Step 6: Record usage
        await self.budget.record_usage(
            repo=repo,
            provider=response.provider_used,
            tokens=response.tokens,
            cost=response.cost
        )

        return response

    async def _call_with_fallback(
        self,
        primary: str,
        fallback_chain: list[str],
        context: dict
    ) -> ProviderResponse:
        """Call provider with fallback strategy."""

        models_to_try = [primary] + fallback_chain

        for attempt, model in enumerate(models_to_try):
            try:
                response = await self.providers[model].call(
                    prompt=context["prompt"],
                    timeout=300
                )
                self.logger.info(f"Success with {model} on attempt {attempt+1}")
                response.provider_used = model
                return response

            except RateLimitError as e:
                self.logger.warning(f"{model} rate-limited: {e}")
                await asyncio.sleep(min(2 ** attempt, 60))  # Exponential backoff
                continue

            except ContextWindowExceededError:
                self.logger.warning(f"{model} context exceeded")
                # Find model with larger context and continue
                continue

            except Exception as e:
                self.logger.error(f"{model} failed: {e}")
                if attempt < len(models_to_try) - 1:
                    continue
                else:
                    raise AllProvidersExhaustedError(f"All providers failed: {e}")

        raise AllProvidersExhaustedError("Exhausted all fallback providers")

    def _select_cheaper_provider(
        self,
        current: str,
        fallback_chain: list[str]
    ) -> str:
        """Select cheapest provider from options."""

        cost_map = {
            "claude-opus-4": 100,
            "claude-sonnet-4": 50,
            "gpt-4-turbo": 60,
            "gemini-pro": 20,
            "deepseek-v3": 10
        }

        options = [current] + fallback_chain
        return min(options, key=lambda x: cost_map.get(x, 1000))


# budget_store.py — Budget tracking

class BudgetStore:
    """Persistent budget tracking per repo."""

    def __init__(self, db: Database):
        self.db = db

    async def has_capacity(self, repo: str) -> bool:
        """Check if repo has remaining budget."""

        usage = await self.db.query("""
            SELECT SUM(input_tokens + output_tokens) as total
            FROM token_usage
            WHERE repo_name = $1
            AND date_trunc('month', created_at) = date_trunc('month', NOW())
        """, repo)

        total_used = usage[0]["total"] or 0
        config = load_config(f"repos/{repo}/.ai-config.json")

        return total_used < config["token_budget"]["monthly"]

    async def is_near_limit(self, repo: str, threshold: float = 0.8) -> bool:
        """Check if approaching budget limit."""

        usage = await self.db.query("""
            SELECT SUM(input_tokens + output_tokens) as total
            FROM token_usage
            WHERE repo_name = $1
            AND date_trunc('month', created_at) = date_trunc('month', NOW())
        """, repo)

        total_used = usage[0]["total"] or 0
        config = load_config(f"repos/{repo}/.ai-config.json")
        monthly_limit = config["token_budget"]["monthly"]

        return total_used / monthly_limit > threshold

    async def record_usage(
        self,
        repo: str,
        provider: str,
        tokens: int,
        cost: float
    ) -> None:
        """Record token usage for cost tracking."""

        await self.db.insert("token_usage", {
            "repo_name": repo,
            "provider": provider,
            "input_tokens": tokens,  # Simplified
            "cost_usd": cost,
            "created_at": datetime.now()
        })


# ci_router.py — GitHub Actions dispatcher

class CIRouter:
    """Route GitHub Actions tasks to appropriate provider."""

    async def dispatch_workflow(
        self,
        repo: str,
        workflow_type: str,
        pr_number: int = None
    ) -> None:
        """Dispatch CI task to correct provider."""

        routing_config = load_yaml(".ai-routing.yaml")
        provider = routing_config["repositories"][repo]["provider"]

        # Set environment for workflow
        os.environ["AI_PROVIDER"] = provider
        os.environ["AI_MODEL"] = routing_config["repositories"][repo]["model"]

        # Enforce concurrency (prevent duplicate runs)
        async with JobLock(f"{repo}:{pr_number}") as lock:
            if not lock.acquired:
                print(f"Another agent already processing {repo}, skipping")
                return

            # Run task
            await run_task(repo, workflow_type)
```

---

## 10. Benefits & Trade-offs

### 10.1 Benefits of Provider-Per-Repo Approach

| Benefit | Impact |
|---------|--------|
| **Cost Optimization** | Route expensive repos to Claude, cheap repos to Deepseek → 30-50% cost reduction |
| **Token Isolation** | Budget guards prevent one runaway agent from consuming global budget |
| **Fault Tolerance** | Automatic fallback chains ensure workflows complete even if primary provider fails |
| **Quality Optimization** | Match provider to task: Code→Claude, Docs→Cheaper, Complex→Opus |
| **Audit Trail** | Per-repo cost tracking identifies expensive patterns |
| **Team Autonomy** | Each team owns their repo's provider choice |
| **No Doubling** | Job locks and concurrency controls prevent waste from parallel agents |

### 10.2 Trade-offs & Complexity

| Trade-off | Cost |
|-----------|------|
| **Configuration Overhead** | 1 file per repo, 1 global routing file → ~30 files |
| **Latency** | 50-100ms per proxy layer → manageable for most tasks |
| **Operational Burden** | Monitor 4+ providers, manage credentials, track budgets → ~4 hours/week |
| **API Key Management** | 4-6 API keys vs. single key → credential rotation more complex |
| **Debugging** | Multi-layer routing makes failures harder to trace → need structured logging |

---

## 11. Recommended Architecture for Phenotype

### 11.1 Deployment Model

```
┌─────────────────────────────────────────────────────┐
│ Local Layer (Claude Code Router)                    │
│ - Runs on developer's machine or CI runner          │
│ - Intercepts Claude Code requests                   │
│ - Reads .ai-config.json per repo                    │
│ - Maintains session budget                          │
└─────────────────────────────────────────────────────┘
           ↓ HTTP/gRPC ↓
┌─────────────────────────────────────────────────────┐
│ Central Coordination Layer (Optional Sidecar)        │
│ - Tracks global budget across all repos             │
│ - Enforces concurrency limits                       │
│ - Exports metrics to Slack/PagerDuty                │
│ (Runs in CI/Cloud, or locally for single user)      │
└─────────────────────────────────────────────────────┘
           ↓ API ↓
┌─────────────────────────────────────────────────────┐
│ Unified Provider Abstraction (OpenRouter/LiteLLM)   │
│ - Single API key for all providers                  │
│ - Handle provider-specific payload transforms       │
│ - Built-in cost tracking                            │
└─────────────────────────────────────────────────────┘
           ↓ API ↓
┌─────────────────────────────────────────────────────┐
│ Actual Providers                                    │
│ Claude, OpenAI, Cursor, DeepSeek, Gemini, etc.      │
└─────────────────────────────────────────────────────┘
```

### 11.2 Configuration Files to Create

**1. `/repos/.ai-routing.yaml` (polyrepo root)**
```yaml
# Global routing rules and tier definitions
# Reference: Section 2.2 above
```

**2. `/repos/{project}/.ai-config.json` (per project)**
```json
// Individual repo provider config and budget
// Reference: Section 2.1 above
```

**3. `/repos/.env.routing` (credentials)**
```bash
OPENAI_API_KEY=sk-...
ANTHROPIC_API_KEY=sk-ant-...
OPENROUTER_API_KEY=sk-or-...
GEMINI_API_KEY=...
DEEPSEEK_API_KEY=...
```

### 11.3 CI/CD Changes

**Add to each repo's GitHub Actions**:
```yaml
jobs:
  detect-provider:
    # Detect provider from .ai-routing.yaml
    # Output provider and model as env vars

  task:
    needs: detect-provider
    env:
      AI_PROVIDER: ${{ needs.detect-provider.outputs.provider }}
      AI_MODEL: ${{ needs.detect-provider.outputs.model }}
    # Task runs with selected provider
```

---

## 12. Cost Projections

### 12.1 Scenario: 30-Project Polyrepo

**Without routing** (all repos use Claude Opus):
- 30 repos × 2M tokens/month average = **60M tokens/month**
- Claude Opus 4 pricing: $0.000015 per token → **$900/month input alone**
- Total with output: ~**$2,000-3,000/month**

**With provider-per-repo routing** (tiered):
- Tier 1 (5 repos × Claude Opus 4): 5M tokens @ $3/1K input = $150
- Tier 2 (10 repos × Claude Sonnet 4): 10M tokens @ $1.5/1K = $150
- Tier 3 (10 repos × GPT-4 Turbo): 10M tokens @ $1.8/1K = $180
- Tier 4 (5 repos × DeepSeek): 5M tokens @ $0.1/1K = $5
- **Total: ~$500-600/month** (70% cost reduction)

### 12.2 Avoiding Doubling/Waste

**Scenario: 5 concurrent agents process same PR**
- Without de-duplication: 5 × 200K tokens = **1M tokens wasted**
- Cost: 1M × $0.00003 = **$30 wasted per incident**
- With job locks: 1 × 200K tokens = **$6 cost**
- **Savings: 80% per incident**

---

## 13. Open-Source Tools Reference

| Tool | Purpose | Cost | Notes |
|------|---------|------|-------|
| [LiteLLM](https://docs.litellm.ai/) | Unified LLM routing | Free + Proxy | Battle-tested, production-grade |
| [OpenRouter](https://openrouter.ai/) | Multi-provider API proxy | Per-token | Simple, no setup |
| [Claude Code Router](https://github.com/musistudio/claude-code-router) | Claude-specific router | Free | Lightweight, CLI-level |
| [Langfuse](https://langfuse.com/) | Observability + cost tracking | $50-500/mo | Deep analytics |
| [LLMFlite](https://github.com/nimbus-ai/llmflite) | Lightweight router | Free | Minimal dependencies |

---

## 14. Next Steps for Phenotype

1. **Create `.ai-routing.yaml`** with tier definitions and fallback chains
2. **Add `.ai-config.json`** templates for each project
3. **Set up OpenRouter or LiteLLM** as unified provider abstraction
4. **Implement BudgetGuard** class for session-level budget tracking
5. **Add GitHub Actions dispatcher** to route CI tasks by provider
6. **Create cost tracking dashboard** (Supabase + simple UI)
7. **Document in `docs/PROVIDER_ROUTING_GUIDE.md`** with examples

---

## References

- [Claude Code Router - GitHub](https://github.com/musistudio/claude-code-router)
- [LiteLLM Routing & Load Balancing](https://docs.litellm.ai/docs/routing-load-balancing)
- [OpenRouter Claude Code Integration](https://openrouter.ai/docs/guides/guides/claude-code-integration)
- [Claude API Cost Tracking](https://platform.claude.com/docs/en/agent-sdk/cost-tracking)
- [Langfuse Token Tracking](https://langfuse.com/docs/observability/features/token-and-cost-tracking)
- [GitHub Actions Monorepo Patterns](https://dev.to/pockit_tools/github-actions-in-2026-the-complete-guide-to-monorepo-cicd-and-self-hosted-runners-1jop)
- [OpenClaw API Documentation](https://docs.openclaw.ai/providers/anthropic)

---

**Document Version**: 1.0
**Last Updated**: 2026-03-30
**Status**: Research Complete
