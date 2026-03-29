"""Capability mismatch detection for connectors.

Detects when a connector lacks required capabilities for sync operations
and generates alerts.

FR traceability: WL-305 (Capability Mismatch Alerts)
"""

from __future__ import annotations

import logging
from dataclasses import dataclass
from datetime import datetime, UTC
from typing import Any, Callable

logger = logging.getLogger(__name__)


class CapabilityMismatchDetector:
    """Detects capability mismatches between connectors and sync requirements.

    Attributes:
        required_capabilities: List of capabilities required for sync.
    """

    def __init__(self, required_capabilities: list[str]) -> None:
        """Initialize the detector.

        Args:
            required_capabilities: Capabilities required by the sync.

        Raises:
            ValueError: If required_capabilities is empty or not a list.
        """
        if not isinstance(required_capabilities, list):
            raise ValueError("required_capabilities must be a list")
        if not required_capabilities:
            raise ValueError("required_capabilities cannot be empty")

        self.required_capabilities = required_capabilities

    def check_connector(
        self,
        connector_name: str,
        available_capabilities: list[str],
    ) -> list[str]:
        """Check a connector for missing capabilities.

        Args:
            connector_name: Name of the connector.
            available_capabilities: Capabilities the connector has.

        Returns:
            List of missing capability names (empty if all present).

        Raises:
            ValueError: If inputs are invalid.
        """
        if not isinstance(connector_name, str) or not connector_name:
            raise ValueError("connector_name must be a non-empty string")
        if not isinstance(available_capabilities, list):
            raise ValueError("available_capabilities must be a list")

        missing = []
        for cap in self.required_capabilities:
            if cap not in available_capabilities:
                missing.append(cap)

        return missing

    def is_compatible(
        self,
        connector_name: str,
        available_capabilities: list[str],
    ) -> bool:
        """Check if a connector has all required capabilities.

        Args:
            connector_name: Name of the connector.
            available_capabilities: Capabilities the connector has.

        Returns:
            True if all required capabilities are present.

        Raises:
            ValueError: If inputs are invalid.
        """
        missing = self.check_connector(connector_name, available_capabilities)
        return len(missing) == 0

    def generate_alert(
        self,
        connector_name: str,
        missing: list[str],
    ) -> dict:
        """Generate an alert for capability mismatch.

        Args:
            connector_name: Name of the connector.
            missing: List of missing capabilities.

        Returns:
            Alert dict with keys: connector, missing, severity, timestamp.

        Raises:
            ValueError: If inputs are invalid.
        """
        if not isinstance(connector_name, str) or not connector_name:
            raise ValueError("connector_name must be a non-empty string")
        if not isinstance(missing, list):
            raise ValueError("missing must be a list")

        severity = "critical" if missing else "ok"

        return {
            "connector": connector_name,
            "missing": missing,
            "severity": severity,
            "timestamp": datetime.now(UTC).isoformat(),
        }


@dataclass(frozen=True)
class ConnectorSLAThresholds:
    """SLA thresholds for a connector."""

    p95_latency_ms: float
    max_failure_rate: float


class ConnectorSLAEvaluator:
    """Evaluate connector latency/error budget compliance against SLA thresholds."""

    def evaluate(
        self,
        *,
        connector_name: str,
        latency_summary: dict[str, Any],
        error_budget_stats: dict[str, Any],
        thresholds: ConnectorSLAThresholds,
    ) -> dict[str, Any]:
        """Return SLA compliance payload and explicit breach reasons."""
        if not connector_name:
            raise ValueError("connector_name must be non-empty")
        p95_value = latency_summary.get("p95")
        failure_rate = error_budget_stats.get("current_failure_rate")
        if p95_value is None:
            raise ValueError("latency_summary must include p95")
        if failure_rate is None:
            raise ValueError("error_budget_stats must include current_failure_rate")

        breaches: list[str] = []
        if float(p95_value) > thresholds.p95_latency_ms:
            breaches.append(f"p95 latency breach ({float(p95_value):.3f}ms > {thresholds.p95_latency_ms:.3f}ms)")
        if float(failure_rate) > thresholds.max_failure_rate:
            breaches.append(f"failure rate breach ({float(failure_rate):.4f} > {thresholds.max_failure_rate:.4f})")

        return {
            "connector": connector_name,
            "timestamp": datetime.now(UTC).isoformat(),
            "thresholds": {
                "p95_latency_ms": thresholds.p95_latency_ms,
                "max_failure_rate": thresholds.max_failure_rate,
            },
            "observed": {
                "p95_latency_ms": float(p95_value),
                "failure_rate": float(failure_rate),
            },
            "within_sla": len(breaches) == 0,
            "breaches": breaches,
        }


class ConnectorCapabilityDiscovery:
    """Runtime connector capability discovery with explicit cache refresh control."""

    def __init__(self, probe: Callable[[str], list[str]]) -> None:
        self._probe = probe
        self._cache: dict[str, list[str]] = {}

    def discover(self, connector: str, *, refresh: bool = False) -> list[str]:
        """Return discovered capabilities for the connector."""
        normalized = connector.strip().lower()
        if not normalized:
            raise ValueError("connector must be non-empty")
        if refresh or normalized not in self._cache:
            self._cache[normalized] = list(self._probe(normalized))
        return list(self._cache[normalized])
