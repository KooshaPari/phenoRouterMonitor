"""Pipeline stage percentile tracking for observability.

Tracks execution duration of pipeline stages and computes percentiles (p50, p95, p99)
for performance analysis.

FR traceability: WL-303 (Pipeline Stage Percentiles)
"""

from __future__ import annotations

import logging
import statistics
from dataclasses import dataclass
from datetime import datetime, UTC

logger = logging.getLogger(__name__)


@dataclass
class StageTimer:
    """Record of a single pipeline stage execution.

    Attributes:
        stage: Name of the pipeline stage.
        duration_ms: Execution duration in milliseconds.
        cycle_id: Associated cycle identifier.
        timestamp: When the execution was recorded.
    """

    stage: str
    duration_ms: float
    cycle_id: str
    timestamp: datetime


class PipelinePercentileTracker:
    """Tracks and computes percentiles of pipeline stage durations."""

    def __init__(self) -> None:
        """Initialize the tracker with empty recording list."""
        self._records: list[StageTimer] = []

    def record(self, stage: str, duration_ms: float, cycle_id: str) -> None:
        """Record a pipeline stage execution.

        Args:
            stage: Name of the pipeline stage.
            duration_ms: Execution duration in milliseconds.
            cycle_id: Associated cycle identifier.

        Raises:
            ValueError: If duration_ms is negative.
        """
        if duration_ms < 0:
            raise ValueError(f"duration_ms must be non-negative, got {duration_ms}")

        record = StageTimer(
            stage=stage,
            duration_ms=duration_ms,
            cycle_id=cycle_id,
            timestamp=datetime.now(UTC),
        )
        self._records.append(record)
        logger.debug(f"Recorded stage {stage} with duration {duration_ms}ms for cycle {cycle_id}")

    def percentile(self, stage: str, p: float) -> float | None:
        """Get the p-th percentile of durations for a stage.

        Args:
            stage: Name of the pipeline stage.
            p: Percentile value (0-100).

        Returns:
            The p-th percentile in milliseconds, or None if no data exists for the stage.

        Raises:
            ValueError: If p is not in range [0, 100].
        """
        if not (0 <= p <= 100):
            raise ValueError(f"percentile must be in range [0, 100], got {p}")

        durations = [r.duration_ms for r in self._records if r.stage == stage]

        if not durations:
            return None

        # Handle single value case
        if len(durations) == 1:
            return durations[0]

        # Handle p0 (minimum)
        if p == 0:
            return min(durations)

        # Handle p100 (maximum)
        if p == 100:
            return max(durations)

        # Use quantiles to compute percentile for p in (0, 100)
        quantiles_list = statistics.quantiles(durations, n=100)
        return quantiles_list[int(p) - 1]

    def summary(self, stage: str) -> dict:
        """Get summary statistics for a stage.

        Args:
            stage: Name of the pipeline stage.

        Returns:
            Dictionary with keys: stage, count, p50, p95, p99.
            p50, p95, p99 are None if no data exists.
        """
        durations = [r.duration_ms for r in self._records if r.stage == stage]

        return {
            "stage": stage,
            "count": len(durations),
            "p50": self.percentile(stage, 50),
            "p95": self.percentile(stage, 95),
            "p99": self.percentile(stage, 99),
        }

    def all_stages(self) -> list[str]:
        """Get sorted list of all unique stages recorded.

        Returns:
            Sorted list of unique stage names.
        """
        stages = {r.stage for r in self._records}
        return sorted(stages)
