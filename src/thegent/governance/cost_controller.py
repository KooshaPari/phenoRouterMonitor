"""Call-count budget management for AgilePlus autonomous governance.

Tracks daily agent trigger counts against a configurable budget (default 20/day)
and enforces tiered throttling as utilization increases. Budget tiers and limits
are loaded from contracts/health-targets.json.
"""

import json
import logging
from datetime import UTC, datetime
from enum import StrEnum
from pathlib import Path

from pydantic import BaseModel, Field

_log = logging.getLogger(__name__)


class BudgetTier(StrEnum):
    """Throttle tier based on daily budget utilization percentage."""

    NORMAL = "normal"
    CAUTIOUS = "cautious"
    RESTRICTED = "restricted"
    HALTED = "halted"


class DailyUsage(BaseModel):
    """Snapshot of agent call consumption for a single calendar day."""

    date: str = Field(description="YYYY-MM-DD")
    calls_used: int = 0
    calls_limit: int = 20
    per_dimension: dict[str, int] = Field(default_factory=dict)
    per_agent: dict[str, int] = Field(default_factory=dict)


class CostController:
    """Manages daily agent-call budgets with tiered throttling.

    Budget is measured in agent trigger count (not dollars). When utilization
    crosses tier thresholds the controller progressively restricts which agent
    types may be spawned, ultimately halting all spawns at 95%+ utilization.
    """

    def __init__(self, session_dir: Path, health_targets_path: Path) -> None:
        self._session_dir = session_dir
        self._usage_dir = session_dir / "agileplus"
        self._usage_path = self._usage_dir / "daily_usage.jsonl"

        with open(health_targets_path) as fh:
            data = json.load(fh)

        budget = data["budget"]
        self._daily_limit: int = budget["daily_agent_calls"]

        # Build ordered tier list from config (ascending by max_utilization_pct)
        raw_tiers = budget["tiers"]
        self._tier_thresholds: list[tuple[BudgetTier, float]] = sorted(
            [(BudgetTier(name), cfg["max_utilization_pct"] / 100.0) for name, cfg in raw_tiers.items()],
            key=lambda t: t[1],
        )

        _log.debug(
            "cost controller: limit=%d/day, tiers=%s",
            self._daily_limit,
            [(t.value, pct) for t, pct in self._tier_thresholds],
        )

    @property
    def usage_path(self) -> Path:
        return self._usage_path

    def record_call(
        self,
        dimension: str,
        agent: str,
        *,
        cost_usd: float | None = None,
    ) -> None:
        """Record one agent trigger against today's budget."""
        usage = self.get_today_usage()
        usage.calls_used += 1
        usage.per_dimension[dimension] = usage.per_dimension.get(dimension, 0) + 1
        usage.per_agent[agent] = usage.per_agent.get(agent, 0) + 1
        self._persist(usage)
        _log.debug(
            "recorded call: dim=%s agent=%s total=%d/%d%s",
            dimension,
            agent,
            usage.calls_used,
            usage.calls_limit,
            f" cost=${cost_usd:.4f}" if cost_usd is not None else "",
        )

    def get_today_usage(self) -> DailyUsage:
        """Load or create today's usage record from the JSONL ledger."""
        today = datetime.now(UTC).strftime("%Y-%m-%d")

        if self._usage_path.exists():
            # Scan for today's record (last matching line wins)
            latest: DailyUsage | None = None
            for line in self._usage_path.read_text().splitlines():
                stripped = line.strip()
                if not stripped:
                    continue
                record = json.loads(stripped)
                if record.get("date") == today:
                    latest = DailyUsage(**record)

            if latest is not None:
                return latest

        return DailyUsage(date=today, calls_limit=self._daily_limit)

    def get_tier(self) -> BudgetTier:
        """Determine the current budget tier from today's utilization."""
        usage = self.get_today_usage()
        if usage.calls_limit <= 0:
            return BudgetTier.HALTED

        utilization = usage.calls_used / usage.calls_limit

        # Walk tiers in ascending order; return the highest tier whose
        # threshold has NOT been exceeded. Once utilization >= a tier's
        # max_utilization_pct we move to the next (stricter) tier.
        current = BudgetTier.NORMAL
        for tier, max_pct in self._tier_thresholds:
            if utilization < max_pct:
                return tier
            current = tier

        # Utilization >= highest threshold
        return current

    def can_spawn(self, estimated_calls: int = 1) -> bool:
        """Return False when budget exhausted or insufficient for estimated_calls."""
        if self.get_tier() == BudgetTier.HALTED:
            return False
        return self.calls_remaining() >= estimated_calls

    def calls_remaining(self) -> int:
        """Number of agent calls remaining in today's budget."""
        usage = self.get_today_usage()
        return max(0, usage.calls_limit - usage.calls_used)

    def _persist(self, usage: DailyUsage) -> None:
        """Append or update today's record in the JSONL ledger."""
        self._usage_dir.mkdir(parents=True, exist_ok=True)

        # Rewrite the file: keep all lines for other days, replace today's line
        today = usage.date
        other_lines: list[str] = []

        if self._usage_path.exists():
            for line in self._usage_path.read_text().splitlines():
                stripped = line.strip()
                if not stripped:
                    continue
                record = json.loads(stripped)
                if record.get("date") != today:
                    other_lines.append(stripped)

        other_lines.append(usage.model_dump_json())
        self._usage_path.write_text("\n".join(other_lines) + "\n")
