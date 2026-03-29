"""Budget alerts and cost-overage gates for the orchestration layer."""

from __future__ import annotations

import logging
from dataclasses import dataclass
from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from pathlib import Path

logger = logging.getLogger(__name__)


@dataclass
class BudgetConfig:
    """Budget configuration."""

    hourly_limit_usd: float = 10.0
    daily_limit_usd: float = 100.0
    run_limit_usd: float = 5.0
    warning_threshold: float = 0.8


class BudgetAlertSystem:
    """Check budgets and emit alerts."""

    def __init__(self, cost_dir: Path | None = None, config: BudgetConfig | None = None) -> None:
        """Initialize budget alert system.

        Args:
            cost_dir: Directory where cost summaries are stored.
            config: Budget configuration.
        """
        from thegent.cost.tracker import DEFAULT_COST_DIR

        self.cost_dir = cost_dir or DEFAULT_COST_DIR
        self.config = config or BudgetConfig()

    @classmethod
    def from_settings(cls, settings: Any) -> BudgetAlertSystem:
        """Create budget alert system from settings."""
        config = BudgetConfig(
            hourly_limit_usd=getattr(settings, "budget_hourly_limit", 10.0),
            daily_limit_usd=getattr(settings, "budget_daily_limit", 100.0),
            run_limit_usd=getattr(settings, "budget_run_limit", 5.0),
            warning_threshold=getattr(settings, "budget_warning_threshold", 0.8),
        )
        # Ensure they are floats (in case of mocks returning MagicMocks)
        try:
            config.hourly_limit_usd = float(config.hourly_limit_usd)
            config.daily_limit_usd = float(config.daily_limit_usd)
            config.run_limit_usd = float(config.run_limit_usd)
            config.warning_threshold = float(config.warning_threshold)
        except (TypeError, ValueError):
            # Fallback for mocks
            config = BudgetConfig()

        return cls(config=config)

    def check_budget(self, current_cost: float, context: str = "run") -> tuple[str, bool]:
        """Check if cost exceeds budget.

        Args:
            current_cost: Current cost in USD.
            context: Context for limit ("run", "hourly", "daily").

        Returns:
            Tuple of (alert_level, is_blocking).
            alert_level is one of "OK", "WARN", "BLOCK".
        """
        limit = getattr(self.config, f"{context}_limit_usd", self.config.run_limit_usd)

        if current_cost >= limit:
            logger.error(
                "Budget EXCEEDED for %s: $%.4f >= $%.4f",
                context,
                current_cost,
                limit,
            )
            return ("BLOCK", True)
        if current_cost >= limit * self.config.warning_threshold:
            logger.warning(
                "Budget Warning for %s: $%.4f (%.0f%% of $%.4f)",
                context,
                current_cost,
                (current_cost / limit) * 100,
                limit,
            )
            return ("WARN", False)

        return ("OK", False)

    def get_hourly_spend(self) -> float:
        """Get total spend in the current hour.

        Calculated by scanning the aggregate.jsonl log.
        """
        aggregate_file = self.cost_dir / "aggregate.jsonl"
        if not aggregate_file.exists():
            return 0.0

        from datetime import UTC, datetime, timedelta

        one_hour_ago = datetime.now(UTC) - timedelta(hours=1)
        total = 0.0

        import json

        try:
            with open(aggregate_file, encoding="utf-8") as f:
                for line in f:
                    if not line.strip():
                        continue
                    try:
                        data = json.loads(line)
                        ts_str = data.get("timestamp")
                        if ts_str:
                            ts = datetime.fromisoformat(ts_str)
                            if ts >= one_hour_ago:
                                total += data.get("total_cost", 0.0)
                    except (ValueError, json.JSONDecodeError):
                        continue
        except Exception as exc:
            logger.debug("Error reading aggregate.jsonl: %s", exc)

        return total

    def get_daily_spend(self) -> float:
        """Get total spend in the current day (UTC)."""
        aggregate_file = self.cost_dir / "aggregate.jsonl"
        if not aggregate_file.exists():
            return 0.0

        from datetime import UTC, datetime

        today_start = datetime.now(UTC).replace(hour=0, minute=0, second=0, microsecond=0)
        total = 0.0

        import json

        try:
            with open(aggregate_file, encoding="utf-8") as f:
                for line in f:
                    if not line.strip():
                        continue
                    try:
                        data = json.loads(line)
                        ts_str = data.get("timestamp")
                        if ts_str:
                            ts = datetime.fromisoformat(ts_str)
                            if ts >= today_start:
                                total += data.get("total_cost", 0.0)
                    except (ValueError, json.JSONDecodeError):
                        continue
        except Exception as exc:
            logger.debug("Error reading aggregate.jsonl: %s", exc)

        return total


# Backward compatibility alias
BudgetAlerts = BudgetAlertSystem
