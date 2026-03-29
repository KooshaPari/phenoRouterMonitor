"""MonitoringEngine for health, performance, and cost tracking."""

from dataclasses import dataclass, field
from datetime import UTC, datetime
from typing import Any

from .crew import Crew
from .executor import ExecutionResult


@dataclass
class HealthStatus:
    """Health status of a crew or agent."""

    healthy: bool = True
    message: str = ""
    last_check: datetime = field(default_factory=lambda: datetime.now(UTC))
    details: dict[str, Any] = field(default_factory=dict)


@dataclass
class PerformanceMetrics:
    """Performance metrics."""

    total_tasks: int = 0
    completed_tasks: int = 0
    failed_tasks: int = 0
    avg_duration_seconds: float = 0.0
    total_duration_seconds: float = 0.0
    throughput_tasks_per_minute: float = 0.0


@dataclass
class CostMetrics:
    """Cost tracking metrics."""

    total_tokens: int = 0
    total_cost_usd: float = 0.0
    cost_per_task: float = 0.0
    cost_per_token: float = 0.0
    by_agent: dict[str, float] = field(default_factory=dict)


class MonitoringEngine:
    """
    Monitoring engine for crew execution.

    Tracks:
    - Health status
    - Performance metrics
    - Cost metrics
    - Agent utilization
    """

    def __init__(self) -> None:
        """Initialize MonitoringEngine."""
        self.health_statuses: dict[str, HealthStatus] = {}
        self.performance_metrics: dict[str, PerformanceMetrics] = {}
        self.cost_metrics: dict[str, CostMetrics] = {}
        self.execution_history: list[dict[str, Any]] = []

    def check_health(self, crew: Crew) -> HealthStatus:
        """
        Check health of a crew.

        Args:
            crew: Crew to check

        Returns:
            HealthStatus
        """
        status = HealthStatus()
        status.last_check = datetime.now(UTC)

        # Basic health checks
        if not crew.agents:
            status.healthy = False
            status.message = "No agents in crew"
            return status

        if not crew.tasks:
            status.healthy = False
            status.message = "No tasks in crew"
            return status

        # Check for failed tasks
        failed_tasks = [t for t in crew.tasks if t.status.value == "failed"]
        if failed_tasks:
            status.healthy = False
            status.message = f"{len(failed_tasks)} failed tasks"
            status.details["failed_tasks"] = [t.id for t in failed_tasks]

        self.health_statuses[crew.id] = status
        return status

    def track_performance(self, crew_id: str, results: dict[str, ExecutionResult]) -> PerformanceMetrics:
        """
        Track performance metrics for crew execution.

        Args:
            crew_id: Crew identifier
            results: Execution results

        Returns:
            PerformanceMetrics
        """
        metrics = PerformanceMetrics()
        metrics.total_tasks = len(results)

        completed = [r for r in results.values() if r.success]
        failed = [r for r in results.values() if not r.success]

        metrics.completed_tasks = len(completed)
        metrics.failed_tasks = len(failed)

        if completed:
            total_duration = sum(r.duration_seconds for r in completed)
            metrics.total_duration_seconds = total_duration
            metrics.avg_duration_seconds = total_duration / len(completed)

            # Calculate throughput
            if metrics.total_duration_seconds > 0:
                metrics.throughput_tasks_per_minute = (len(completed) / metrics.total_duration_seconds) * 60

        self.performance_metrics[crew_id] = metrics
        return metrics

    def track_costs(self, crew_id: str, results: dict[str, ExecutionResult]) -> CostMetrics:
        """
        Track cost metrics for crew execution.

        Args:
            crew_id: Crew identifier
            results: Execution results

        Returns:
            CostMetrics
        """
        metrics = CostMetrics()

        total_tokens = sum(r.tokens_used for r in results.values())
        total_cost = sum(r.cost_usd for r in results.values())

        metrics.total_tokens = total_tokens
        metrics.total_cost_usd = total_cost

        if results:
            metrics.cost_per_task = total_cost / len(results)

        if total_tokens > 0:
            metrics.cost_per_token = total_cost / total_tokens

        # Track costs by agent (would need agent_id in ExecutionResult)
        # For now, aggregate all

        self.cost_metrics[crew_id] = metrics
        return metrics

    def record_execution(
        self,
        crew_id: str,
        results: dict[str, ExecutionResult],
        metadata: dict[str, Any] | None = None,
    ) -> None:
        """
        Record execution for history tracking.

        Args:
            crew_id: Crew identifier
            results: Execution results
            metadata: Optional metadata
        """
        record = {
            "crew_id": crew_id,
            "timestamp": datetime.now(UTC).isoformat(),
            "num_tasks": len(results),
            "num_completed": len([r for r in results.values() if r.success]),
            "num_failed": len([r for r in results.values() if not r.success]),
            "metadata": metadata or {},
        }
        self.execution_history.append(record)

        # Track performance and costs
        self.track_performance(crew_id, results)
        self.track_costs(crew_id, results)

    def get_summary(self, crew_id: str) -> dict[str, Any]:
        """
        Get monitoring summary for a crew.

        Args:
            crew_id: Crew identifier

        Returns:
            Summary dictionary
        """
        health = self.health_statuses.get(crew_id)
        performance = self.performance_metrics.get(crew_id)
        cost = self.cost_metrics.get(crew_id)

        return {
            "crew_id": crew_id,
            "health": health.__dict__ if health else None,
            "performance": performance.__dict__ if performance else None,
            "cost": cost.__dict__ if cost else None,
        }
