"""Connector quota budget management for sync operations.

# @trace WL-221
"""

from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime, timedelta, UTC


class QuotaExhaustedError(Exception):
    """Raised when a connector's daily quota budget is exhausted."""


@dataclass
class ConnectorQuota:
    """Represents the quota allocation for a single connector."""

    connector_name: str
    daily_limit: int
    used_today: int = 0
    reset_at: datetime | None = None

    def __post_init__(self) -> None:
        """Initialize reset_at if not provided."""
        if self.reset_at is None:
            self.reset_at = self._calculate_reset_time()

    def _calculate_reset_time(self) -> datetime:
        """Calculate the next reset time (midnight UTC tomorrow)."""
        now = datetime.now(UTC)
        next_midnight = now.replace(hour=0, minute=0, second=0, microsecond=0) + timedelta(days=1)
        return next_midnight

    def remaining(self) -> int:
        """Return the remaining quota for today."""
        return max(0, self.daily_limit - self.used_today)

    def is_exhausted(self) -> bool:
        """Check if quota is exhausted."""
        return self.used_today >= self.daily_limit


class QuotaBudgetManager:
    """Manages quota budgets for multiple connectors."""

    def __init__(self) -> None:
        """Initialize the quota budget manager."""
        self._quotas: dict[str, ConnectorQuota] = {}

    def register(self, connector_name: str, daily_limit: int) -> None:
        """Register a connector with a daily quota limit.

        Args:
            connector_name: Name of the connector.
            daily_limit: Daily quota limit (number of operations allowed).

        Raises:
            ValueError: If daily_limit is <= 0.
        """
        if daily_limit <= 0:
            raise ValueError(f"daily_limit must be > 0, got {daily_limit}")
        self._quotas[connector_name] = ConnectorQuota(
            connector_name=connector_name,
            daily_limit=daily_limit,
            used_today=0,
        )

    def check_quota(self, connector: str, n: int = 1) -> bool:
        """Check if quota is available for the given connector.

        Args:
            connector: Name of the connector.
            n: Number of operations to check (default: 1).

        Returns:
            True if quota is available, False otherwise.

        Raises:
            KeyError: If connector is not registered.
        """
        if connector not in self._quotas:
            raise KeyError(f"Connector {connector!r} not registered")
        quota = self._quotas[connector]
        return quota.remaining() >= n

    def consume(self, connector: str, n: int = 1) -> None:
        """Consume quota for a connector.

        Args:
            connector: Name of the connector.
            n: Number of operations to consume (default: 1).

        Raises:
            KeyError: If connector is not registered.
            QuotaExhaustedError: If insufficient quota is available.
        """
        if connector not in self._quotas:
            raise KeyError(f"Connector {connector!r} not registered")

        quota = self._quotas[connector]
        self._check_and_reset_if_needed(quota)

        if quota.remaining() < n:
            raise QuotaExhaustedError(f"Insufficient quota for {connector!r}: need {n}, have {quota.remaining()}")

        quota.used_today += n

    def reset_daily(self) -> None:
        """Reset all quotas if their reset time has passed."""
        now = datetime.now(UTC)
        for quota in self._quotas.values():
            if quota.reset_at is not None and now >= quota.reset_at:
                quota.used_today = 0
                quota.reset_at = quota._calculate_reset_time()

    def _check_and_reset_if_needed(self, quota: ConnectorQuota) -> None:
        """Check and reset quota if reset time has passed."""
        now = datetime.now(UTC)
        if quota.reset_at is not None and now >= quota.reset_at:
            quota.used_today = 0
            quota.reset_at = quota._calculate_reset_time()

    def get_quota(self, connector: str) -> ConnectorQuota:
        """Get the quota object for a connector.

        Args:
            connector: Name of the connector.

        Returns:
            The ConnectorQuota object.

        Raises:
            KeyError: If connector is not registered.
        """
        if connector not in self._quotas:
            raise KeyError(f"Connector {connector!r} not registered")
        return self._quotas[connector]

    def get_all_quotas(self) -> dict[str, ConnectorQuota]:
        """Get all registered quotas.

        Returns:
            Dictionary mapping connector names to their quotas.
        """
        return self._quotas.copy()
