"""Connector SLA tracking and breach detection.

Tracks connector SLA targets and actual performance, emits alerts on breaches.

FR traceability: WL-233 (Connector SLA Tracking)
"""

from __future__ import annotations

import logging
from dataclasses import dataclass

logger = logging.getLogger(__name__)


@dataclass
class SLARecord:
    """Represents a connector SLA record.

    # @trace WL-233
    """

    connector_id: str
    """Unique connector identifier."""

    target_ms: float
    """Target latency in milliseconds."""

    actual_ms: float | None = None
    """Actual measured latency in milliseconds, None if not yet recorded."""


class ConnectorSLATracker:
    """Tracks connector SLA targets and actual performance.

    # @trace WL-233
    """

    def __init__(self) -> None:
        """Initialize the tracker with empty records."""
        self._records: dict[str, SLARecord] = {}

    def set_target(self, connector_id: str, target_ms: float) -> SLARecord:
        """Set or update the SLA target for a connector.

        Args:
            connector_id: Unique connector identifier.
            target_ms: Target latency in milliseconds.

        Returns:
            The SLARecord with the target set.

        Raises:
            ValueError: If target_ms is not positive.
        """
        if target_ms <= 0:
            raise ValueError(f"Target SLA must be positive, got {target_ms}")

        record = SLARecord(connector_id=connector_id, target_ms=target_ms)
        self._records[connector_id] = record
        logger.debug(f"Set SLA target for {connector_id}: {target_ms}ms")
        return record

    def record_actual(self, connector_id: str, actual_ms: float) -> None:
        """Record actual measured latency for a connector.

        Args:
            connector_id: Unique connector identifier.
            actual_ms: Actual measured latency in milliseconds.

        Raises:
            ValueError: If connector not registered or actual_ms is negative.
            KeyError: If connector has not had a target set yet.
        """
        if connector_id not in self._records:
            raise KeyError(f"Connector {connector_id} has no SLA target set")

        if actual_ms < 0:
            raise ValueError(f"Actual latency cannot be negative, got {actual_ms}")

        record = self._records[connector_id]
        record.actual_ms = actual_ms
        logger.debug(f"Recorded actual latency for {connector_id}: {actual_ms}ms")

    def is_breached(self, connector_id: str) -> bool:
        """Check if a connector's SLA is currently breached.

        Args:
            connector_id: Unique connector identifier.

        Returns:
            True if actual > target, False if actual <= target or not recorded yet.

        Raises:
            KeyError: If connector is not registered.
        """
        if connector_id not in self._records:
            raise KeyError(f"Connector {connector_id} not found")

        record = self._records[connector_id]
        if record.actual_ms is None:
            return False

        return record.actual_ms > record.target_ms

    def breached(self) -> list[SLARecord]:
        """Get all currently breached SLA records.

        Returns:
            List of SLARecord entries where actual > target.
        """
        return [r for r in self._records.values() if self.is_breached(r.connector_id)]

    def all_records(self) -> list[SLARecord]:
        """Get all SLA records.

        Returns:
            List of all SLARecord entries in the tracker.
        """
        return list(self._records.values())
