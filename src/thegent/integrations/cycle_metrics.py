"""Cycle Metrics Emission for observability.

WL-173: Cycle Metrics Emission
Provides metrics emission and aggregation for cycle tracking.
"""

from __future__ import annotations

from dataclasses import dataclass


@dataclass
class CycleMetric:
    """A single metric data point for a cycle."""

    cycle_id: str
    metric_name: str
    value: float


class CycleMetricsEmitter:
    """Emitter for cycle metrics."""

    def __init__(self) -> None:
        """Initialize the cycle metrics emitter."""
        self._metrics: list[CycleMetric] = []

    def emit(self, cycle_id: str, metric_name: str, value: float) -> CycleMetric:
        """Emit a metric for a cycle.

        Args:
            cycle_id: Unique identifier for the cycle.
            metric_name: Name of the metric.
            value: Numeric value of the metric.

        Returns:
            The created CycleMetric.
        """
        metric = CycleMetric(cycle_id=cycle_id, metric_name=metric_name, value=value)
        self._metrics.append(metric)
        return metric

    def get_metrics(self, cycle_id: str) -> list[CycleMetric]:
        """Get all metrics for a specific cycle.

        Args:
            cycle_id: Unique identifier for the cycle.

        Returns:
            List of metrics for the cycle.
        """
        return [m for m in self._metrics if m.cycle_id == cycle_id]

    def aggregate(self, cycle_id: str, metric_name: str) -> float:
        """Aggregate (sum) all values for a metric in a cycle.

        Args:
            cycle_id: Unique identifier for the cycle.
            metric_name: Name of the metric.

        Returns:
            Sum of all values for that metric in that cycle.
        """
        return sum(m.value for m in self._metrics if m.cycle_id == cycle_id and m.metric_name == metric_name)
