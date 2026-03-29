"""Per-run cost aggregation."""

import logging
from datetime import UTC, datetime
from typing import Any

logger = logging.getLogger(__name__)


class CostAggregator:
    """Per-run cost aggregation."""

    def __init__(self) -> None:
        """Initialize cost aggregator."""
        self.runs: list[dict[str, Any]] = []

    def record_run_cost(self, run_id: str, cost: float, model: str, tokens: dict[str, int]) -> None:
        """Record cost for a run.

        Args:
            run_id: Run identifier
            cost: Total cost
            model: Model used
            tokens: Token counts (input, output)
        """
        self.runs.append(
            {
                "run_id": run_id,
                "cost": cost,
                "model": model,
                "tokens": tokens,
                "timestamp": datetime.now(UTC).isoformat(),
            }
        )
        logger.info(f"Recorded cost for run {run_id}: ${cost:.4f}")

    def get_total_cost(self) -> float:
        """Get total cost across all runs.

        Returns:
            Total cost
        """
        return sum(run["cost"] for run in self.runs)

    def get_cost_by_model(self) -> dict[str, float]:
        """Get cost breakdown by model.

        Returns:
            Dictionary mapping model to total cost
        """
        breakdown: dict[str, float] = {}
        for run in self.runs:
            model = run["model"]
            breakdown[model] = breakdown.get(model, 0) + run["cost"]
        return breakdown
