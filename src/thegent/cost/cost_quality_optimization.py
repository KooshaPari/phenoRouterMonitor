"""Cost-quality optimization (RouteLLM-style)."""

import logging
from typing import Any

logger = logging.getLogger(__name__)


class CostQualityOptimizer:
    """Optimize cost vs quality trade-offs."""

    def __init__(self) -> None:
        """Initialize cost-quality optimizer."""
        self.models: dict[str, dict[str, Any]] = {}
        self.routing_history: list[dict[str, Any]] = []

    def register_model(
        self,
        model_id: str,
        cost_per_token: float,
        quality_score: float,
    ) -> None:
        """Register a model with cost and quality metrics.

        Args:
            model_id: Model identifier
            cost_per_token: Cost per token
            quality_score: Quality score (0.0-1.0)
        """
        self.models[model_id] = {
            "cost_per_token": cost_per_token,
            "quality_score": quality_score,
        }
        logger.info(f"Registered model: {model_id} (cost: ${cost_per_token:.6f}, quality: {quality_score:.2f})")

    def route_request(
        self,
        task_complexity: float,
        quality_threshold: float = 0.7,
        max_cost: float | None = None,
    ) -> str:
        """Route request to optimal model.

        Args:
            task_complexity: Task complexity (0.0-1.0)
            quality_threshold: Minimum quality threshold
            max_cost: Maximum cost constraint

        Returns:
            Selected model ID
        """
        # Filter models by quality threshold
        candidates = {mid: model for mid, model in self.models.items() if model["quality_score"] >= quality_threshold}

        if not candidates:
            # Fall back to highest quality
            candidates = self.models

        # If cost constraint, filter further
        if max_cost:
            # Estimate tokens (simplified)
            estimated_tokens = int(task_complexity * 1000)
            candidates = {
                mid: model
                for mid, model in candidates.items()
                if model["cost_per_token"] * estimated_tokens <= max_cost
            }

        # Select model with best cost/quality ratio
        best_model = None
        best_ratio = float("inf")

        for mid, model in candidates.items():
            ratio = model["cost_per_token"] / model["quality_score"]
            if ratio < best_ratio:
                best_ratio = ratio
                best_model = mid

        if best_model:
            self.routing_history.append(
                {
                    "model": best_model,
                    "task_complexity": task_complexity,
                    "quality_threshold": quality_threshold,
                }
            )
            logger.info(f"Routed to model: {best_model}")
            return best_model

        # Fallback to first available
        return next(iter(self.models.keys())) if self.models else "default"

    def get_routing_stats(self) -> dict[str, Any]:
        """Get routing statistics.

        Returns:
            Routing statistics
        """
        if not self.routing_history:
            return {}

        model_counts = {}
        for entry in self.routing_history:
            model = entry["model"]
            model_counts[model] = model_counts.get(model, 0) + 1

        return {
            "total_routes": len(self.routing_history),
            "model_distribution": model_counts,
        }
