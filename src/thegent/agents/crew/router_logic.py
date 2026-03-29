"""Pure Python JIT-friendly routing logic for PyPy and non-native runtimes."""

from dataclasses import dataclass
from enum import StrEnum
from typing import Any


class RoutingStrategy(StrEnum):
    COST_OPTIMIZED = "cost_optimized"
    PERFORMANCE_OPTIMIZED = "performance_optimized"
    BALANCED = "balanced"


@dataclass
class RouteMetrics:
    cost_per_token: float = 0.0
    latency_ms: float = 0.0
    success_rate: float = 1.0
    availability: float = 1.0


class PurePythonRouter:
    """JIT-friendly implementation of RouterManager."""

    def __init__(self, strategy: RoutingStrategy = RoutingStrategy.BALANCED) -> None:
        self.strategy = strategy
        self.route_cache: dict[str, str] = {}
        self.agent_metrics: dict[str, RouteMetrics] = {}

    def select_agent(self, task_description: str, available_agents: list) -> Any:
        # Optimized loop for JIT
        if not available_agents:
            return None

        best_agent = None
        best_score = -1.0

        for agent in available_agents:
            metrics = self.agent_metrics.get(agent.id, RouteMetrics())
            # Simplified but fast math
            score = (metrics.success_rate * metrics.availability) / (
                max(metrics.cost_per_token, 0.001) + max(metrics.latency_ms / 1000, 0.001)
            )
            if score > best_score:
                best_score = score
                best_agent = agent

        return best_agent or available_agents[0]
