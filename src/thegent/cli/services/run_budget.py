"""Budget checking helpers for run execution.

Extracted from run_execution_core_helpers.py for maintainability.
"""

from __future__ import annotations

import logging
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from thegent.config import ThegentSettings

logger = logging.getLogger(__name__)


def _get_run_spend(alert_system: object) -> float:
    """Return current run spend when the budget system exposes it."""
    get_run_spend = getattr(alert_system, "get_run_spend", None)
    if callable(get_run_spend):
        value = get_run_spend()
        if isinstance(value, (int, float, str)):
            try:
                return float(value)
            except ValueError:
                return 0.0
    return 0.0


def check_budget_limits(settings: "ThegentSettings") -> tuple[bool, str | None]:
    """Check if budget limits have been exceeded.

    Args:
        settings: Thegent settings with budget configuration

    Returns:
        Tuple of (blocked, error_message)
    """
    from thegent.cost import BudgetAlertSystem

    alert_system = BudgetAlertSystem.from_settings(settings)
    hourly_spend = alert_system.get_hourly_spend()
    daily_spend = alert_system.get_daily_spend()

    # Check hourly budget
    _alert, block = alert_system.check_budget(hourly_spend, context="hourly")
    if block:
        return True, f"Hourly budget EXCEEDED: ${hourly_spend:.2f} >= ${settings.budget_hourly_limit:.2f}"

    # Check daily budget
    _alert, block = alert_system.check_budget(daily_spend, context="daily")
    if block:
        return True, f"Daily budget EXCEEDED: ${daily_spend:.2f} >= ${settings.budget_daily_limit:.2f}"

    # Check run budget
    run_spend = _get_run_spend(alert_system)
    _alert, block = alert_system.check_budget(run_spend, context="run")
    if block:
        return True, f"Run budget EXCEEDED: ${run_spend:.2f} >= ${settings.budget_run_limit:.2f}"

    return False, None


def check_budget_warning(settings: "ThegentSettings") -> tuple[bool, str | None]:
    """Check if budget is approaching limits (warning).

    Args:
        settings: Thegent settings with budget configuration

    Returns:
        Tuple of (warning, warning_message)
    """
    from thegent.cost import BudgetAlertSystem

    alert_system = BudgetAlertSystem.from_settings(settings)
    hourly_spend = alert_system.get_hourly_spend()
    daily_spend = alert_system.get_daily_spend()

    threshold = settings.budget_warning_threshold

    # Check hourly warning
    if hourly_spend >= settings.budget_hourly_limit * threshold:
        return True, f"Hourly budget warning: ${hourly_spend:.2f} / ${settings.budget_hourly_limit:.2f}"

    # Check daily warning
    if daily_spend >= settings.budget_daily_limit * threshold:
        return True, f"Daily budget warning: ${daily_spend:.2f} / ${settings.budget_daily_limit:.2f}"

    return False, None


__all__ = [
    "check_budget_limits",
    "check_budget_warning",
]
