"""Status drift severity classification for sync operations.

Classifies status drift into severity tiers and defines escalation thresholds
based on age and status changes.

FR traceability: WL-181 (Status Drift Severity Classification)
"""

from __future__ import annotations

import logging
from dataclasses import dataclass
from enum import Enum

logger = logging.getLogger(__name__)


class DriftSeverity(str, Enum):
    """Severity tiers for status drift."""

    LOW = "low"
    MEDIUM = "medium"
    HIGH = "high"
    CRITICAL = "critical"


@dataclass
class DriftEscalationThresholds:
    """Configurable escalation thresholds for drift severity.

    Attributes:
        medium_age_hours: Age (in hours) at which drift becomes MEDIUM (default: 6).
        high_age_hours: Age (in hours) at which drift becomes HIGH (default: 24).
        critical_age_hours: Age (in hours) at which drift becomes CRITICAL (default: 72).
    """

    medium_age_hours: int = 6
    high_age_hours: int = 24
    critical_age_hours: int = 72

    def validate(self) -> bool:
        """Validate threshold ordering.

        Returns:
            True if thresholds are in ascending order.

        Raises:
            ValueError: If thresholds are not in ascending order.
        """
        if not (self.medium_age_hours <= self.high_age_hours <= self.critical_age_hours):
            raise ValueError(
                "Thresholds must be in ascending order: "
                f"medium({self.medium_age_hours}) <= high({self.high_age_hours}) "
                f"<= critical({self.critical_age_hours})"
            )
        return True


def classify_drift(
    local_status: str,
    remote_status: str,
    age_hours: float,
    thresholds: DriftEscalationThresholds | None = None,
) -> DriftSeverity:
    """Classify status drift by age and severity.

    Classification logic:
    - If status differs, escalate by age: age > critical → CRITICAL,
      age > high → HIGH, age > medium → MEDIUM, else LOW.
    - If status matches, return LOW (no drift).

    Args:
        local_status: The local status value.
        remote_status: The remote status value.
        age_hours: Time elapsed since drift occurred (in hours).
        thresholds: Escalation thresholds. Uses defaults if None.

    Returns:
        The severity classification.

    Raises:
        ValueError: If age_hours is negative.
    """
    if age_hours < 0:
        raise ValueError("age_hours must be non-negative")

    if thresholds is None:
        thresholds = DriftEscalationThresholds()

    # Validate thresholds
    thresholds.validate()

    # No drift if statuses match
    if local_status == remote_status:
        return DriftSeverity.LOW

    # Drift exists; classify by age
    if age_hours > thresholds.critical_age_hours:
        return DriftSeverity.CRITICAL
    if age_hours > thresholds.high_age_hours:
        return DriftSeverity.HIGH
    if age_hours > thresholds.medium_age_hours:
        return DriftSeverity.MEDIUM

    return DriftSeverity.LOW


def get_default_thresholds() -> DriftEscalationThresholds:
    """Get default escalation thresholds.

    Returns:
        Default thresholds: 6h (MEDIUM), 24h (HIGH), 72h (CRITICAL).
    """
    return DriftEscalationThresholds(medium_age_hours=6, high_age_hours=24, critical_age_hours=72)
