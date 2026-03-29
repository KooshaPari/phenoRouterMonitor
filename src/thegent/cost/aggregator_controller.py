"""Budget tier management and cost controller for thegent."""

from __future__ import annotations

import orjson as json
import logging
from dataclasses import dataclass, field

from enum import Enum
from pathlib import Path  # noqa: TC003 -- Path used at runtime
from typing import Any

_log = logging.getLogger(__name__)


class BudgetTier(Enum):
    """Budget utilization tiers (FR-GOV-002)."""

    NORMAL = "normal"
    CAUTIOUS = "cautious"
    RESTRICTED = "restricted"
    HALTED = "halted"


@dataclass
class UsageSnapshot:
    """Current usage state."""

    calls_used: int = 0
    calls_limit: int = 20
    per_dimension: dict[str, int] = field(default_factory=dict)
    per_agent: dict[str, int] = field(default_factory=dict)

    @property
    def utilization_pct(self) -> float:
        if self.calls_limit <= 0:
            return 100.0
        return (self.calls_used / self.calls_limit) * 100.0


class CostController:
    """Manages daily agent call budgets and tier enforcement (FR-GOV-002)."""

    def __init__(self, session_dir: Path, health_targets_path: Path | None = None) -> None:
        self.session_dir = session_dir
        self._calls_used = 0
        self._per_dimension: dict[str, int] = {}
        self._per_agent: dict[str, int] = {}
        self._calls_limit = 20
        self._tiers: dict[str, dict[str, Any]] = {}

        if health_targets_path and health_targets_path.exists():
            targets = json.loads(health_targets_path.read_text())
            budget = targets.get("budget", {})
            self._calls_limit = budget.get("daily_agent_calls", 20)
            self._tiers = budget.get("tiers", {})

    def record_call(self, dimension: str, agent: str, *, cost_usd: float | None = None) -> None:
        """Record a single agent call."""
        self._calls_used += 1
        self._per_dimension[dimension] = self._per_dimension.get(dimension, 0) + 1
        self._per_agent[agent] = self._per_agent.get(agent, 0) + 1

    def get_today_usage(self) -> UsageSnapshot:
        """Return current usage snapshot."""
        return UsageSnapshot(
            calls_used=self._calls_used,
            calls_limit=self._calls_limit,
            per_dimension=dict(self._per_dimension),
            per_agent=dict(self._per_agent),
        )

    def get_tier(self) -> BudgetTier:
        """Determine current budget tier based on utilization."""
        pct = self.get_today_usage().utilization_pct
        if pct >= 95:
            return BudgetTier.HALTED
        if pct >= 80:
            return BudgetTier.RESTRICTED
        if pct >= 50:
            return BudgetTier.CAUTIOUS
        return BudgetTier.NORMAL

    def calls_remaining(self) -> int:
        """Return number of agent calls remaining in today's budget."""
        return max(0, self._calls_limit - self._calls_used)

    def can_spawn(self, estimated_calls: int = 1) -> bool:
        """Check if new agent spawns are allowed."""
        return self.get_tier() != BudgetTier.HALTED
