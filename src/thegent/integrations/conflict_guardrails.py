"""Conflict growth guardrails for sync operations.

Enforces hard limits and warning thresholds on conflict counts during
reconciliation and sync cycles.

FR traceability: WL-304 (Conflict Growth Guardrails)
"""

from __future__ import annotations

import logging

logger = logging.getLogger(__name__)


class ConflictLimitExceeded(Exception):
    """Raised when conflict count exceeds the hard limit."""


class ConflictGrowthGuardrail:
    """Enforces limits on conflict growth.

    Attributes:
        max_conflicts: Hard limit on concurrent conflicts (default: 50).
        warn_threshold: Soft warning threshold (default: 25).
    """

    def __init__(self, max_conflicts: int = 50, warn_threshold: int = 25) -> None:
        """Initialize the guardrail.

        Args:
            max_conflicts: Hard limit on conflicts (default: 50).
            warn_threshold: Warning threshold (default: 25).

        Raises:
            ValueError: If thresholds are invalid.
        """
        if max_conflicts <= 0:
            raise ValueError("max_conflicts must be positive")
        if warn_threshold <= 0:
            raise ValueError("warn_threshold must be positive")
        if warn_threshold > max_conflicts:
            raise ValueError("warn_threshold must be <= max_conflicts")

        self.max_conflicts = max_conflicts
        self.warn_threshold = warn_threshold

    def check(self, current_count: int) -> None:
        """Check if conflict count exceeds the hard limit.

        Args:
            current_count: Current number of conflicts.

        Raises:
            ConflictLimitExceeded: If current_count > max_conflicts.
            ValueError: If current_count is negative.
        """
        if current_count < 0:
            raise ValueError("current_count must be non-negative")

        if current_count > self.max_conflicts:
            raise ConflictLimitExceeded(f"Conflict count {current_count} exceeds hard limit {self.max_conflicts}")

    def warn_level(self, current_count: int) -> bool:
        """Check if conflict count is at warning level.

        Args:
            current_count: Current number of conflicts.

        Returns:
            True if current_count >= warn_threshold.

        Raises:
            ValueError: If current_count is negative.
        """
        if current_count < 0:
            raise ValueError("current_count must be non-negative")

        return current_count >= self.warn_threshold

    def status(self, current_count: int) -> dict:
        """Get status dict for conflict count.

        Args:
            current_count: Current number of conflicts.

        Returns:
            Dict with keys: count, warn (bool), exceeded (bool).

        Raises:
            ValueError: If current_count is negative.
        """
        if current_count < 0:
            raise ValueError("current_count must be non-negative")

        return {
            "count": current_count,
            "warn": self.warn_level(current_count),
            "exceeded": current_count > self.max_conflicts,
        }
