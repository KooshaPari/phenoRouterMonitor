"""Budget-aware routing and economic governance for thegent.

Implements the Economic Governance Framework (WP-5003).
Also implements:
- GW-17: Provider budget routing (spend-cap based provider exclusion)
- GW-18: Deployment pool concept (N backends behind one logical model name)
- GW-21: Session stickiness (consistent-hash routing for multi-turn conversations)
"""

from __future__ import annotations

import logging
import os
from dataclasses import dataclass, field
from datetime import UTC, datetime
from enum import Enum
from typing import Any, Literal

from thegent.utils.routing_impl.pareto_router import ParetoRouter
from thegent.utils.routing_impl.pareto_router import RouteCandidate as _ParetoRouteCandidate

_log = logging.getLogger(__name__)


# ---------------------------------------------------------------------------
# Data types
# ---------------------------------------------------------------------------


class BudgetType(Enum):
    """Supported budget scopes (SCLI-P9.1)."""

    SESSION = "session"
    DAILY = "daily"
    MONTHLY = "monthly"
    QUARTERLY = "quarterly"
    PER_MODEL = "per_model"
    EMERGENCY = "emergency"


@dataclass
class Budget:
    """A budget allocation for a project or session (FR-COST-001)."""

    id: str
    project_id: str
    budget_type: BudgetType
    amount: float
    spent: float = 0.0
    start_date: datetime = field(default_factory=lambda: datetime.now(UTC))
    end_date: datetime | None = None
    renews: bool = False

    @property
    def remaining(self) -> float:
        """Return remaining budget in USD."""
        return max(0.0, self.amount - self.spent)

    @property
    def utilization(self) -> float:
        """Return fraction of budget spent."""
        if self.amount <= 0:
            return 1.0
        return self.spent / self.amount


@dataclass
class BudgetStatus:
    """Status of a budget check."""

    can_proceed: bool
    remaining_budget: float
    utilization: float
    reason: str | None = None


class BudgetExceededError(Exception):
    """Raised when spend has exceeded a configured budget limit (FR-COST-002)."""

    def __init__(self, budget_type: str, limit: float, current: float) -> None:
        self.budget_type = budget_type
        self.limit = limit
        self.current = current
        super().__init__(f"{budget_type} budget exceeded: current=${current:.4f} > limit=${limit:.4f}")


# ---------------------------------------------------------------------------
# Managers and Meters
# ---------------------------------------------------------------------------


class CostMeter:
    """Real-time cost metering for projects and models (FR-COST-001)."""

    def __init__(self) -> None:
        self.current_costs: dict[str, float] = {}
        self.cost_history: list[dict[str, Any]] = []

    async def record_cost(
        self, project_id: str, model: str, input_tokens: int, output_tokens: int, cost: float
    ) -> None:
        """Record cost for a single request (FR-COST-001)."""
        key = f"{project_id}:{model}"
        self.current_costs[key] = self.current_costs.get(key, 0.0) + cost

        self.cost_history.append(
            {
                "timestamp": datetime.now(UTC).isoformat(),
                "project_id": project_id,
                "model": model,
                "input_tokens": input_tokens,
                "output_tokens": output_tokens,
                "cost": cost,
            }
        )

    def get_project_cost(self, project_id: str) -> float:
        """Get total cost for a project across all models."""
        return sum(cost for key, cost in self.current_costs.items() if key.startswith(f"{project_id}:"))


class BudgetManager:
    """Manages budget allocations and enforcement (FR-COST-003)."""

    def __init__(self) -> None:
        self.budgets: dict[str, Budget] = {}

    def add_budget(self, budget: Budget) -> None:
        """Add a budget allocation."""
        self.budgets[budget.id] = budget

    def check_budget(self, project_id: str, requested_cost: float = 0.0) -> BudgetStatus:
        """Check if any budget for project_id is exceeded."""
        relevant_budgets = [b for b in self.budgets.values() if b.project_id == project_id]

        if not relevant_budgets:
            # If no budget is defined, we allow by default but warn
            return BudgetStatus(can_proceed=True, remaining_budget=float("inf"), utilization=0.0)

        for budget in relevant_budgets:
            if budget.spent + requested_cost > budget.amount:
                return BudgetStatus(
                    can_proceed=False,
                    remaining_budget=budget.remaining,
                    utilization=budget.utilization,
                    reason=f"Budget {budget.id} ({budget.budget_type.value}) exceeded",
                )

        # Return the most constrained budget's status
        most_constrained = min(relevant_budgets, key=lambda b: b.remaining)
        return BudgetStatus(
            can_proceed=True, remaining_budget=most_constrained.remaining, utilization=most_constrained.utilization
        )

    def record_spend(self, project_id: str, cost: float) -> None:
        """Update all relevant budgets for project_id with recorded cost."""
        for budget in self.budgets.values():
            if budget.project_id == project_id:
                budget.spent += cost


# ---------------------------------------------------------------------------
# Router
# ---------------------------------------------------------------------------


class BudgetAwareRouter:
    """Routes to optimal candidates while respecting cost budgets (FR-COST-003, WP-1004)."""

    def __init__(
        self,
        budget_manager: BudgetManager,
        pareto_router: ParetoRouter | None = None,
        warn_at_pct: float = 0.8,
        degraded_at_pct: float = 0.9,
    ) -> None:
        self.budget_manager = budget_manager
        self.router = pareto_router or ParetoRouter()
        self.warn_at_pct = warn_at_pct
        self.degraded_at_pct = degraded_at_pct

    def route(
        self, project_id: str, candidates: list[_ParetoRouteCandidate], strategy: str = "balanced"
    ) -> _ParetoRouteCandidate:
        """Select the best candidate given current budget state and Pareto strategy."""
        if not candidates:
            raise ValueError("candidates list must not be empty")

        # 1. Hard Budget Check
        status = self.budget_manager.check_budget(project_id)

        if not status.can_proceed:
            # If strictly over budget, select the absolute cheapest candidate
            _log.warning("Budget exceeded for project %s. Routing to cheapest candidate.", project_id)
            return min(candidates, key=lambda c: c.cost_per_1k)  # type: ignore[attr-defined]

        # 2. Degraded Mode (90%+)
        if status.utilization >= self.degraded_at_pct:
            _log.info(
                "Degraded mode (utilization %.2f) for project %s. Forcing cost strategy.",
                status.utilization,
                project_id,
            )
            return self.router.select_by_strategy("cost", candidates)

        # 3. Warn Mode (80%+)
        if status.utilization >= self.warn_at_pct:
            # Filter to cheapest 50% before Pareto selection
            pool = self._cheapest_half(candidates)
            _log.info(
                "Warn mode (utilization %.2f) for project %s. Filtering to cheapest half.",
                status.utilization,
                project_id,
            )
            return self.router.select_by_strategy(strategy, pool)

        # 4. Normal Routing
        return self.router.select_by_strategy(strategy, candidates)

    def _cheapest_half(self, candidates: list[_ParetoRouteCandidate]) -> list[_ParetoRouteCandidate]:
        """Return the cheapest 50% (rounded up) of candidates by cost_per_1k."""
        sorted_by_cost = sorted(candidates, key=lambda c: c.cost_per_1k)  # type: ignore[attr-defined]
        cutoff = max(1, (len(sorted_by_cost) + 1) // 2)  # ceil(n/2), min 1
        return sorted_by_cost[:cutoff]


# ---------------------------------------------------------------------------
# Simple budget router API (FR-COST-001, FR-COST-002, FR-COST-003)
# ---------------------------------------------------------------------------


@dataclass
class RouteCandidate:
    """A candidate route with name, cost per call, and quality score."""

    name: str
    cost_per_call: float
    quality: float


@dataclass
class CostBudget:
    """Simple per-session and per-day budget limits."""

    daily_limit_usd: float
    session_limit_usd: float
    warn_at_pct: float = 0.8


class SimpleCostTracker:
    """Tracks session and daily spend totals."""

    def __init__(self) -> None:
        self._session: float = 0.0
        self._daily: float = 0.0

    def record(self, cost: float) -> None:
        if cost < 0:
            raise ValueError(f"Cost must be non-negative, got {cost}")
        self._session += cost
        self._daily += cost

    def session_total(self) -> float:
        return self._session

    def daily_total(self) -> float:
        return self._daily

    def reset_session(self) -> None:
        self._session = 0.0


class CostAwareRouter:
    """Routes to candidates while enforcing CostBudget limits (FR-COST-003)."""

    def __init__(self, budget: CostBudget, tracker: SimpleCostTracker) -> None:
        self._budget = budget
        self._tracker = tracker

    def select(self, candidates: list[RouteCandidate]) -> RouteCandidate:
        """Select best candidate given current spend vs budget.

        Raises:
            ValueError: If candidates is empty.
            BudgetExceededError: If any budget limit is exceeded.
        """
        if not candidates:
            raise ValueError("candidates list must not be empty")

        session = self._tracker.session_total()
        daily = self._tracker.daily_total()

        # Hard stop: exceeded
        if session >= self._budget.session_limit_usd:
            raise BudgetExceededError("session", self._budget.session_limit_usd, session)
        if daily >= self._budget.daily_limit_usd:
            raise BudgetExceededError("daily", self._budget.daily_limit_usd, daily)

        # Warn mode: near limit -> cheapest half, best quality among them
        session_pct = session / self._budget.session_limit_usd if self._budget.session_limit_usd > 0 else 0.0
        daily_pct = daily / self._budget.daily_limit_usd if self._budget.daily_limit_usd > 0 else 0.0
        if session_pct >= self._budget.warn_at_pct or daily_pct >= self._budget.warn_at_pct:
            pool = self._cheapest_half_simple(candidates)
        else:
            pool = candidates

        return max(pool, key=lambda c: c.quality)

    @staticmethod
    def _cheapest_half_simple(candidates: list[RouteCandidate]) -> list[RouteCandidate]:
        """Return cheapest ceil(n/2) candidates by cost_per_call."""
        sorted_c = sorted(candidates, key=lambda c: c.cost_per_call)
        cutoff = max(1, (len(sorted_c) + 1) // 2)
        return sorted_c[:cutoff]


# ---------------------------------------------------------------------------
# GW-17: Provider budget routing
# ---------------------------------------------------------------------------


@dataclass
class ProviderBudgetConfig:
    """Spend cap configuration for a single provider."""

    provider: str
    daily_limit_usd: float
    monthly_limit_usd: float | None = None
    # When True: hard block (raise error). When False: soft route-away (exclude from routing)
    hard_block: bool = False


class ProviderBudgetRouter:
    """Routes away from providers that have hit their spend caps.

    Tracks cumulative spend per provider and excludes over-budget providers
    from the available model list.
    """

    def __init__(self, configs: list[ProviderBudgetConfig]) -> None:
        self._configs: dict[str, ProviderBudgetConfig] = {c.provider: c for c in configs}
        self._daily_spend: dict[str, float] = {c.provider: 0.0 for c in configs}

    def record_spend(self, provider: str, cost_usd: float) -> None:
        """Record spend for a provider."""
        if provider in self._daily_spend:
            self._daily_spend[provider] += cost_usd
        else:
            self._daily_spend[provider] = cost_usd

    def is_provider_over_budget(self, provider: str) -> bool:
        """Return True if provider has exceeded its daily limit."""
        config = self._configs.get(provider)
        if config is None:
            return False
        spent = self._daily_spend.get(provider, 0.0)
        return spent >= config.daily_limit_usd

    def filter_model_list(self, model_list: list[dict]) -> list[dict]:
        """Remove over-budget providers from the model list.

        Falls back to full list if all providers are over budget.
        """
        over_budget = {p for p in self._configs if self.is_provider_over_budget(p)}
        if not over_budget:
            return model_list

        filtered = [entry for entry in model_list if not self._entry_matches_providers(entry, over_budget)]

        # If all entries were filtered out, fall back to the full list
        if not filtered:
            _log.warning(
                "All providers are over budget (%s); falling back to full model list.",
                over_budget,
            )
            return model_list

        return filtered

    @staticmethod
    def _entry_matches_providers(entry: dict, providers: set[str]) -> bool:
        """Return True if the entry's litellm provider is in the given set."""
        litellm_model: str = entry.get("litellm_params", {}).get("model", "")
        if "/" in litellm_model:
            provider_part = litellm_model.split("/", 1)[0]
            return provider_part in providers
        return False

    def get_spend_summary(self) -> dict[str, dict[str, float]]:
        """Return {provider: {spent: X, limit: Y, remaining: Z}} for all configured providers."""
        summary: dict[str, dict[str, float]] = {}
        for provider, config in self._configs.items():
            spent = self._daily_spend.get(provider, 0.0)
            limit = config.daily_limit_usd
            summary[provider] = {
                "spent": spent,
                "limit": limit,
                "remaining": max(0.0, limit - spent),
            }
        return summary

    def reset_daily(self) -> None:
        """Reset all daily spend counters (call at midnight or on test reset)."""
        for provider in self._daily_spend:
            self._daily_spend[provider] = 0.0


_default_budget_router: ProviderBudgetRouter | None = None


def get_provider_budget_router(configs: list[ProviderBudgetConfig] | None = None) -> ProviderBudgetRouter:
    """Get or create the default ProviderBudgetRouter.

    On first call, builds configs from env vars:
    - THEGENT_BUDGET_{PROVIDER}_DAILY: daily limit in USD for provider
    E.g., THEGENT_BUDGET_OPENAI_DAILY=10.0
    """
    global _default_budget_router
    if _default_budget_router is not None and configs is None:
        return _default_budget_router

    if configs is not None:
        _default_budget_router = ProviderBudgetRouter(configs)
        return _default_budget_router

    # Build configs from environment variables
    env_configs: list[ProviderBudgetConfig] = []
    prefix = "THEGENT_BUDGET_"
    suffix = "_DAILY"
    for key, value in os.environ.items():
        if key.startswith(prefix) and key.endswith(suffix):
            provider_upper = key[len(prefix) : -len(suffix)]
            provider = provider_upper.lower()
            daily_limit = float(value)
            env_configs.append(ProviderBudgetConfig(provider=provider, daily_limit_usd=daily_limit))

    _default_budget_router = ProviderBudgetRouter(env_configs)
    return _default_budget_router


# ---------------------------------------------------------------------------
# GW-18: Deployment pool concept
# ---------------------------------------------------------------------------


@dataclass
class DeploymentConfig:
    """A single backend deployment for a model."""

    provider: str  # e.g., "openai"
    model: str  # e.g., "gpt-4o"
    weight: float = 1.0  # Load balancing weight (higher = more traffic)
    api_key_env: str | None = None  # Env var name for API key (optional override)
    api_base: str | None = None  # Optional custom base URL


@dataclass
class DeploymentPool:
    """A named pool of deployments behind one logical model name."""

    name: str  # Logical model name (e.g., "gpt-4o")
    deployments: list[DeploymentConfig]
    strategy: Literal["round_robin", "weighted", "least_cost"] = "weighted"


class DeploymentPoolManager:
    """Manages N deployments per logical model name.

    Converts DeploymentPool definitions to LiteLLM model_list entries,
    enabling LiteLLM's native load balancing across deployments.
    """

    def __init__(self, pools: list[DeploymentPool]) -> None:
        self._pools: dict[str, DeploymentPool] = {p.name: p for p in pools}

    def to_litellm_model_list(self) -> list[dict]:
        """Convert all pools to LiteLLM model_list format.

        Each deployment becomes a model_list entry with model_name=pool.name,
        so LiteLLM treats them as alternatives for the same logical model.
        """
        entries: list[dict] = []
        for pool in self._pools.values():
            for deployment in pool.deployments:
                litellm_params: dict[str, Any] = {
                    "model": f"{deployment.provider}/{deployment.model}",
                    "weight": deployment.weight,
                }
                if deployment.api_key_env is not None:
                    litellm_params["api_key"] = os.environ.get(deployment.api_key_env, "dummy-key")
                if deployment.api_base is not None:
                    litellm_params["api_base"] = deployment.api_base
                entries.append(
                    {
                        "model_name": pool.name,
                        "litellm_params": litellm_params,
                    }
                )
        return entries

    def add_pool(self, pool: DeploymentPool) -> None:
        """Add a deployment pool (or replace existing pool with same name)."""
        self._pools[pool.name] = pool

    def get_pool(self, name: str) -> DeploymentPool | None:
        """Look up pool by logical model name."""
        return self._pools.get(name)


# ---------------------------------------------------------------------------
# GW-21: Session stickiness
# ---------------------------------------------------------------------------


class SessionStickyRouter:
    """Routes requests to the same deployment for a given session_id.

    Uses consistent hashing (hash(session_id) % len(deployments)) so the same
    session always hits the same deployment as long as the pool is stable.
    Falls back to first deployment if pool is empty.
    """

    def __init__(self, pool_manager: DeploymentPoolManager) -> None:
        self._pool_manager = pool_manager

    def get_deployment_for_session(
        self,
        model: str,
        session_id: str,
    ) -> DeploymentConfig | None:
        """Return the sticky deployment for this session+model combination.

        Returns None if no pool exists for the model.
        """
        pool = self._pool_manager.get_pool(model)
        if pool is None or not pool.deployments:
            return None
        index = hash(session_id) % len(pool.deployments)
        return pool.deployments[index]

    def get_litellm_params_for_session(
        self,
        model: str,
        session_id: str,
    ) -> dict | None:
        """Return litellm_params dict for the sticky deployment, or None."""
        deployment = self.get_deployment_for_session(model, session_id)
        if deployment is None:
            return None
        params: dict[str, Any] = {
            "model": f"{deployment.provider}/{deployment.model}",
        }
        if deployment.api_base is not None:
            params["api_base"] = deployment.api_base
        if deployment.api_key_env is not None:
            params["api_key"] = os.environ.get(deployment.api_key_env, "dummy-key")
        return params


def get_session_sticky_extra(
    model: str,
    session_id: str | None,
    pool_manager: DeploymentPoolManager | None = None,
) -> dict:
    """Build extra kwargs for router.acompletion to implement session stickiness.

    When session_id is provided and a pool exists for the model, returns
    {'api_base': ..., 'api_key': ...} from the sticky deployment.
    Returns {} if no stickiness is applicable.
    """
    if session_id is None or pool_manager is None:
        return {}
    sticky_router = SessionStickyRouter(pool_manager)
    params = sticky_router.get_litellm_params_for_session(model, session_id)
    if params is None:
        return {}
    extra: dict[str, Any] = {}
    if "api_base" in params:
        extra["api_base"] = params["api_base"]
    if "api_key" in params:
        extra["api_key"] = params["api_key"]
    return extra
