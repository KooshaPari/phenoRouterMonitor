"""SLA and error budget adapter for workstream autosync.

Handles SLA evaluation, error budget tracking, and connector health monitoring.
"""

from typing import Any

from thegent.integrations.capability_alerts import ConnectorSLAEvaluator, ConnectorSLAThresholds
from thegent.integrations.error_budget import ErrorBudgetConfig, ErrorBudgetTracker
from thegent.integrations.pipeline_percentiles import PipelinePercentileTracker


class SLAAdapter:
    """Adapter for SLA evaluation and error budget operations."""

    def __init__(self, config: Any, error_budget: ErrorBudgetTracker):
        self.config = config
        self._error_budget = error_budget
        self._connector_sla_thresholds: dict[str, ConnectorSLAThresholds] = dict(config.connector_sla_thresholds)
        self._connector_sla_evaluator = ConnectorSLAEvaluator()
        self._connector_latency_tracker = PipelinePercentileTracker()
        self._connector_error_budgets: dict[str, ErrorBudgetTracker] = {}

    def evaluate_slo_state(self, snapshot_age_seconds: int | None) -> list[str]:
        """Evaluate current SLO state and return alerts."""
        alerts = []

        # Check snapshot staleness
        if snapshot_age_seconds is not None and snapshot_age_seconds > self.config.autosync_stale_snapshot_seconds:
            alerts.append(f"autosync snapshot stale for {snapshot_age_seconds}s")

        # Check global error budget
        if self._error_budget.should_escalate():
            alerts.append("autosync error budget escalation threshold reached")
        if self._error_budget.should_hard_fail():
            alerts.append("autosync error budget hard-fail threshold reached")

        # Check connector SLAs
        for connector, thresholds in sorted(self._connector_sla_thresholds.items()):
            latency_summary = self._connector_latency_tracker.summary(connector)
            if latency_summary.get("count", 0) == 0:
                continue
            error_budget = self.get_connector_error_budget(connector)
            result = self._connector_sla_evaluator.evaluate(
                connector_name=connector,
                latency_summary=latency_summary,
                error_budget_stats=error_budget.get_stats(),
                thresholds=thresholds,
            )
            for breach in result.get("breaches", []):
                alerts.append(f"connector {connector} SLA breach: {breach}")

        return alerts

    def get_connector_error_budget(self, connector: str) -> ErrorBudgetTracker:
        """Get or create error budget for connector."""
        normalized = connector.lower()
        if normalized not in self._connector_error_budgets:
            self._connector_error_budgets[normalized] = ErrorBudgetTracker(
                ErrorBudgetConfig(
                    max_consecutive_failures=self.config.error_budget_max_consecutive_failures,
                    max_failure_rate=self.config.error_budget_max_failure_rate,
                    escalation_after=self.config.error_budget_escalation_after,
                ),
            )
        return self._connector_error_budgets[normalized]

    def record_connector_latency(self, connector: str, duration_seconds: float) -> None:
        """Record connector latency metric."""
        # Implementation would use actual pipeline tracker


__all__ = ["SLAAdapter"]
