"""Phase13: Cost sensitivity experiment (baseline + A/B)."""

import logging
from typing import Any

logger = logging.getLogger(__name__)


class CostSensitivityExperiment:
    """Cost sensitivity experiment framework."""

    def __init__(self) -> None:
        """Initialize cost sensitivity experiment."""
        self.baseline_costs: list[float] = []
        self.variant_costs: list[float] = []

    def record_baseline(self, cost: float) -> None:
        """Record baseline cost.

        Args:
            cost: Cost value
        """
        self.baseline_costs.append(cost)
        logger.info(f"Recorded baseline cost: ${cost:.4f}")

    def record_variant(self, cost: float) -> None:
        """Record variant cost.

        Args:
            cost: Cost value
        """
        self.variant_costs.append(cost)
        logger.info(f"Recorded variant cost: ${cost:.4f}")

    def analyze(self) -> dict[str, Any]:
        """Analyze cost sensitivity.

        Returns:
            Analysis results
        """
        if not self.baseline_costs or not self.variant_costs:
            return {"error": "Insufficient data"}

        baseline_avg = sum(self.baseline_costs) / len(self.baseline_costs)
        variant_avg = sum(self.variant_costs) / len(self.variant_costs)

        savings = baseline_avg - variant_avg
        savings_percent = (savings / baseline_avg) * 100 if baseline_avg > 0 else 0

        return {
            "baseline_avg": baseline_avg,
            "variant_avg": variant_avg,
            "savings": savings,
            "savings_percent": savings_percent,
        }
