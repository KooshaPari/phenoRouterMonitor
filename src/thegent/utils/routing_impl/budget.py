"""GW-29/GW-30/GW-31: Budget hierarchy with reset periods and soft alerts.

Implements Team -> User -> Key spending hierarchy.
Budget reset periods: daily, weekly, monthly.
Soft alert at 80% spend, hard block at 100%.

# @trace FR-BUDGET-029 FR-BUDGET-030 FR-BUDGET-031
"""

from __future__ import annotations

import logging
import threading
import time
from dataclasses import dataclass, field
from enum import Enum
from typing import ClassVar

_log = logging.getLogger(__name__)


class BudgetPeriod(str, Enum):
    """Supported budget reset period lengths."""

    DAILY = "daily"
    WEEKLY = "weekly"
    MONTHLY = "monthly"


@dataclass
class BudgetRecord:
    """Budget state for a single entity (team, user, or key).

    Tracks cumulative spend within the current period. The period auto-resets
    when BudgetResetChecker.maybe_reset() is called and the period has elapsed.
    """

    entity_id: str  # team_id, user_id, or key_id
    entity_type: str  # "team" | "user" | "key"
    budget_usd: float  # hard limit; 0 = unlimited
    period: BudgetPeriod = BudgetPeriod.MONTHLY
    spent_usd: float = 0.0
    period_start: float = field(default_factory=time.time)
    alert_threshold: float = 0.80  # fraction at which to emit soft alert

    @property
    def is_exhausted(self) -> bool:
        """True when spend >= budget (hard block).

        When budget_usd is 0 the budget is unlimited and this always returns False.
        """
        return self.budget_usd > 0 and self.spent_usd >= self.budget_usd

    @property
    def is_soft_alert(self) -> bool:
        """True when spend >= alert_threshold * budget but not yet exhausted.

        When budget_usd is 0 there is no limit so this always returns False.
        """
        return (
            self.budget_usd > 0 and self.spent_usd >= self.alert_threshold * self.budget_usd and not self.is_exhausted
        )

    @property
    def fraction_used(self) -> float:
        """Fraction of budget used (0.0-1.0+). Returns 0.0 when budget_usd=0."""
        if self.budget_usd <= 0:
            return 0.0
        return self.spent_usd / self.budget_usd


class BudgetResetChecker:
    """Checks if a BudgetRecord's period has elapsed and resets it if so."""

    PERIOD_SECONDS: ClassVar[dict[BudgetPeriod, float]] = {
        BudgetPeriod.DAILY: 86_400.0,
        BudgetPeriod.WEEKLY: 604_800.0,
        BudgetPeriod.MONTHLY: 2_592_000.0,  # 30 days
    }

    @classmethod
    def maybe_reset(cls, record: BudgetRecord) -> bool:
        """Reset record.spent_usd to 0 and update period_start if period elapsed.

        Args:
            record: The BudgetRecord to inspect and potentially reset.

        Returns:
            True if a reset occurred, False otherwise.
        """
        elapsed = time.time() - record.period_start
        period_sec = cls.PERIOD_SECONDS[record.period]
        if elapsed >= period_sec:
            record.spent_usd = 0.0
            record.period_start = time.time()
            _log.info(
                "Budget period reset entity_id=%s period=%s",
                record.entity_id,
                record.period.value,
            )
            return True
        return False


@dataclass
class BudgetCheckResult:
    """Result of checking the budget hierarchy for a request."""

    allowed: bool
    soft_alert: bool = False
    blocking_entity: str | None = None
    alert_entities: list[str] = field(default_factory=list)


class BudgetHierarchy:
    """Team -> User -> Key budget hierarchy.

    Records spending at all levels simultaneously.
    Hard-blocks a request if ANY level is exhausted.
    Emits a soft alert (via BudgetCheckResult) if ANY level is at threshold.
    """

    def __init__(self) -> None:
        self._records: dict[str, BudgetRecord] = {}
        self._lock = threading.Lock()

    def register(self, record: BudgetRecord) -> None:
        """Register or replace a budget record."""
        with self._lock:
            self._records[record.entity_id] = record
            _log.debug(
                "Registered budget record entity_id=%s entity_type=%s budget_usd=%.4f period=%s",
                record.entity_id,
                record.entity_type,
                record.budget_usd,
                record.period.value,
            )

    def get(self, entity_id: str) -> BudgetRecord | None:
        """Return the budget record for entity_id, or None if not registered."""
        with self._lock:
            return self._records.get(entity_id)

    def record_spend(self, entity_ids: list[str], cost_usd: float) -> None:
        """Record cost_usd against all given entity_ids (team, user, key chain).

        Automatically resets periods that have elapsed before adding spend.

        Args:
            entity_ids: Ordered list of entity identifiers (e.g. [team_id, user_id, key_id]).
            cost_usd: Amount spent in USD to add to each entity's record.
        """
        with self._lock:
            for eid in entity_ids:
                record = self._records.get(eid)
                if record is None:
                    continue
                BudgetResetChecker.maybe_reset(record)
                record.spent_usd += cost_usd
                _log.debug(
                    "Recorded spend entity_id=%s cost_usd=%.6f total_spent=%.6f budget=%.4f",
                    eid,
                    cost_usd,
                    record.spent_usd,
                    record.budget_usd,
                )

    def check_budget(self, entity_ids: list[str]) -> BudgetCheckResult:
        """Check if any entity in the hierarchy has exhausted its budget.

        Iterates entity_ids in order (team -> user -> key). Returns on the
        first exhausted entity. Collects all entities at the soft-alert threshold.

        Args:
            entity_ids: Ordered list of entity identifiers to check.

        Returns:
            BudgetCheckResult with allowed, soft_alert, blocking_entity, and
            alert_entities populated.
        """
        with self._lock:
            blocking: str | None = None
            alerts: list[str] = []
            for eid in entity_ids:
                record = self._records.get(eid)
                if record is None:
                    continue
                BudgetResetChecker.maybe_reset(record)
                if record.is_exhausted:
                    blocking = eid
                    _log.warning(
                        "Budget exhausted — hard block entity_id=%s spent=%.4f budget=%.4f",
                        eid,
                        record.spent_usd,
                        record.budget_usd,
                    )
                    break
                if record.is_soft_alert:
                    alerts.append(eid)
                    _log.info(
                        "Budget soft alert entity_id=%s fraction_used=%.2f threshold=%.2f",
                        eid,
                        record.fraction_used,
                        record.alert_threshold,
                    )
            return BudgetCheckResult(
                allowed=blocking is None,
                soft_alert=len(alerts) > 0,
                blocking_entity=blocking,
                alert_entities=alerts,
            )


# ---------------------------------------------------------------------------
# Module-level singleton
# ---------------------------------------------------------------------------

_budget_hierarchy: BudgetHierarchy | None = None
_budget_lock = threading.Lock()


def get_budget_hierarchy() -> BudgetHierarchy:
    """Return the process-global BudgetHierarchy singleton."""
    global _budget_hierarchy
    if _budget_hierarchy is None:
        with _budget_lock:
            if _budget_hierarchy is None:
                _budget_hierarchy = BudgetHierarchy()
                _log.debug("Created global BudgetHierarchy singleton")
    return _budget_hierarchy


def reset_budget_hierarchy() -> None:
    """Reset the singleton (for testing only)."""
    global _budget_hierarchy
    with _budget_lock:
        _budget_hierarchy = None
