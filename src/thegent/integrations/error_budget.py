"""Error Budget and Escalation Thresholds for autosync reliability.

# @trace WL-170
"""

from __future__ import annotations

from dataclasses import dataclass


@dataclass
class ErrorBudgetConfig:
    """Configuration for error budget tracking and escalation."""

    max_consecutive_failures: int = 3
    max_failure_rate: float = 0.5
    escalation_after: int = 5


class ErrorBudgetTracker:
    """Track error budget and determine escalation/hard-fail behavior."""

    def __init__(self, config: ErrorBudgetConfig | None = None) -> None:
        """Initialize the error budget tracker.

        Args:
            config: ErrorBudgetConfig instance. Uses defaults if None.
        """
        self.config = config or ErrorBudgetConfig()
        self._success_count: int = 0
        self._failure_count: int = 0
        self._consecutive_failures: int = 0

    def record_success(self) -> None:
        """Record a successful operation."""
        self._success_count += 1
        self._consecutive_failures = 0

    def record_failure(self) -> None:
        """Record a failed operation."""
        self._failure_count += 1
        self._consecutive_failures += 1

    def should_escalate(self) -> bool:
        """Determine if escalation is needed.

        Escalation happens when total failures exceed escalation_after threshold.

        Returns:
            True if escalation is recommended, False otherwise.
        """
        return self._failure_count >= self.config.escalation_after

    def should_hard_fail(self) -> bool:
        """Determine if hard failure should occur.

        Hard fail happens when:
        - Consecutive failures exceed max_consecutive_failures, OR
        - Failure rate exceeds max_failure_rate

        Returns:
            True if hard failure should occur, False otherwise.
        """
        # Check consecutive failures threshold
        if self._consecutive_failures > self.config.max_consecutive_failures:
            return True

        # Check failure rate threshold
        total_operations = self._success_count + self._failure_count
        if total_operations > 0:
            failure_rate = self._failure_count / total_operations
            if failure_rate > self.config.max_failure_rate:
                return True

        return False

    def reset(self) -> None:
        """Reset all counters."""
        self._success_count = 0
        self._failure_count = 0
        self._consecutive_failures = 0

    def get_stats(self) -> dict[str, int | float]:
        """Get current tracking statistics.

        Returns:
            Dictionary with success_count, failure_count, consecutive_failures,
            total_operations, and current_failure_rate.
        """
        total_operations = self._success_count + self._failure_count
        failure_rate = self._failure_count / total_operations if total_operations > 0 else 0.0

        return {
            "success_count": self._success_count,
            "failure_count": self._failure_count,
            "consecutive_failures": self._consecutive_failures,
            "total_operations": total_operations,
            "current_failure_rate": failure_rate,
        }
