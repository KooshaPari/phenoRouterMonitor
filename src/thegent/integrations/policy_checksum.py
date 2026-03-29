"""Policy data integrity verification via checksumming.

Computes and tracks checksums of policy data to detect unintended drift
and changes during runtime.

FR traceability: WL-312 (Policy Checksum Drift Detection)
"""

from __future__ import annotations

import logging
import hashlib
from dataclasses import dataclass
from datetime import datetime, UTC
from typing import Any

import orjson

logger = logging.getLogger(__name__)


@dataclass
class PolicyChecksum:
    """Record of a policy checksum baseline.

    Attributes:
        policy_id: Identifier for the policy.
        checksum: SHA256 hex digest of the policy data.
        cycle_id: Associated cycle identifier.
        timestamp: When the checksum was recorded.
    """

    policy_id: str
    checksum: str
    cycle_id: str
    timestamp: datetime


class PolicyChecksumDriftDetector:
    """Detects policy data drift via checksum comparisons."""

    def __init__(self) -> None:
        """Initialize the detector with empty baseline store."""
        self._baselines: dict[str, PolicyChecksum] = {}

    def compute_checksum(self, policy_data: dict) -> str:
        """Compute SHA256 checksum of policy data.

        Args:
            policy_data: Policy data dictionary.

        Returns:
            SHA256 hex digest of the sorted JSON serialization.
        """
        serialized = orjson.dumps(policy_data, option=orjson.OPT_SORT_KEYS)
        return hashlib.sha256(serialized).hexdigest()

    def record_baseline(self, policy_id: str, policy_data: dict, cycle_id: str) -> PolicyChecksum:
        """Record a baseline checksum for a policy.

        Args:
            policy_id: Identifier for the policy.
            policy_data: Policy data dictionary.
            cycle_id: Associated cycle identifier.

        Returns:
            The PolicyChecksum baseline that was recorded.
        """
        checksum = self.compute_checksum(policy_data)
        baseline = PolicyChecksum(
            policy_id=policy_id,
            checksum=checksum,
            cycle_id=cycle_id,
            timestamp=datetime.now(UTC),
        )
        self._baselines[policy_id] = baseline
        logger.debug(f"Recorded baseline for policy {policy_id} (checksum: {checksum})")
        return baseline

    def check_drift(self, policy_id: str, current_data: dict) -> bool:
        """Check if current policy data has drifted from baseline.

        Args:
            policy_id: Identifier for the policy.
            current_data: Current policy data dictionary.

        Returns:
            True if checksum differs from baseline (drift detected),
            False if checksums match (no drift).

        Raises:
            KeyError: If no baseline exists for the policy_id.
        """
        if policy_id not in self._baselines:
            raise KeyError(f"No baseline found for policy {policy_id}")

        current_checksum = self.compute_checksum(current_data)
        baseline_checksum = self._baselines[policy_id].checksum

        has_drift = current_checksum != baseline_checksum

        if has_drift:
            logger.warning(
                f"Drift detected in policy {policy_id}: baseline {baseline_checksum} != current {current_checksum}"
            )
        else:
            logger.debug(f"No drift in policy {policy_id}")

        return has_drift

    def get_baseline(self, policy_id: str) -> PolicyChecksum:
        """Retrieve the baseline for a policy.

        Args:
            policy_id: Identifier for the policy.

        Returns:
            The PolicyChecksum baseline.

        Raises:
            KeyError: If no baseline exists for the policy_id.
        """
        if policy_id not in self._baselines:
            raise KeyError(f"No baseline found for policy {policy_id}")

        return self._baselines[policy_id]


def compute_payload_checksum(payload: Any) -> str:
    """Compute a deterministic SHA256 checksum for a JSON-compatible payload."""
    serialized = orjson.dumps(payload, option=orjson.OPT_SORT_KEYS)
    return hashlib.sha256(serialized).hexdigest()


def verify_payload_checksum(payload: Any, expected_checksum: str) -> None:
    """Validate payload checksum and fail loudly on mismatch."""
    actual_checksum = compute_payload_checksum(payload)
    if actual_checksum != expected_checksum:
        raise ValueError(
            f"Payload checksum mismatch: expected={expected_checksum} actual={actual_checksum}",
        )
