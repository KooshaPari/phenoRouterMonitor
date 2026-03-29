"""Public dict-based ParetoRouter shim.

Wraps the internal RouteCandidate-based ParetoRouter from pareto_router.py
with a simple dict API suitable for property-based testing and external callers.

Provider dict schema::

    {
        "model":       str,    # model identifier (must be unique per list)
        "cost":        float,  # cost per call (>= 0)
        "latency_ms":  int,    # expected latency in milliseconds
        "quality":     float,  # quality score in [0, 1]
    }
"""

from __future__ import annotations

from thegent.utils.routing_impl.pareto_router import ParetoRouter as _CoreRouter
from thegent.utils.routing_impl.pareto_router import RouteCandidate


class ParetoRouter:
    """Select Pareto-optimal provider from a list of provider dicts.

    Args:
        providers: List of provider dicts with keys ``model``, ``cost``,
                   ``latency_ms``, and ``quality``.
    """

    def __init__(self, providers: list[dict]) -> None:
        self._providers = providers
        self._router = _CoreRouter()

    def select(self, max_cost_per_call: float = float("inf")) -> dict | None:
        """Return the Pareto-optimal provider dict, or None if no candidates pass constraints.

        A provider is feasible when its ``cost`` <= *max_cost_per_call*.
        Among feasible providers the non-dominated set (Pareto frontier on
        cost and quality) is computed, then the candidate with the highest
        quality/cost ratio is returned (highest quality when cost == 0).

        Duplicate model names are deduplicated: the first occurrence is used.

        Args:
            max_cost_per_call: Hard cost ceiling; providers above this are excluded.

        Returns:
            The selected provider dict, or None when no provider passes the
            cost constraint.
        """
        feasible = [p for p in self._providers if p["cost"] <= max_cost_per_call]
        if not feasible:
            return None

        # Deduplicate by model name: keep first occurrence so that the
        # RouteCandidate and the result dict are always in sync.
        seen: set[str] = set()
        unique_feasible: list[dict] = []
        for p in feasible:
            if p["model"] not in seen:
                seen.add(p["model"])
                unique_feasible.append(p)

        candidates = [
            RouteCandidate(
                model=p["model"],
                provider=p.get("provider", ""),
                cost_per_1k=p["cost"],
                quality_score=p["quality"],
            )
            for p in unique_feasible
        ]

        # Build a model->provider dict mapping for result lookup
        model_map: dict[str, dict] = {p["model"]: p for p in unique_feasible}

        selected = self._router.select(candidates)
        return model_map.get(selected.model)
