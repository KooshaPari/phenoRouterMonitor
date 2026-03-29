"""Cost capping, tracking, and budget alerts (WP-5001, WP-5003)."""

import logging
from typing import Any

logger = logging.getLogger(__name__)


class CostCap:
    """Enforces a hard limit on action or session costs."""

    def __init__(self, max_cost: float) -> None:
        self.max_cost = max_cost

    def check(self, cost: float) -> bool:
        """Check if the given cost is within the cap."""
        if cost > self.max_cost:
            logger.warning("Cost cap exceeded: %s > %s", cost, self.max_cost)
            return False
        return True


class CostTracker:
    """Tracks real-time cost accumulation across sessions."""

    def __init__(self) -> None:
        self._sessions: dict[str, float] = {}

    def start_session(self, session_id: str):
        """Initialize tracking for a new session."""
        self._sessions[session_id] = 0.0

    def record_cost(self, session_id: str, cost: float):
        """Add cost to a session's total."""
        if session_id not in self._sessions:
            self._sessions[session_id] = 0.0
        self._sessions[session_id] += cost

    def get_session_cost(self, session_id: str) -> float:
        """Get the total accumulated cost for a session."""
        return self._sessions.get(session_id, 0.0)

    def is_within_budget(self, session_id: str, budget: float) -> bool:
        """Check if a session is still within the provided budget."""
        return self.get_session_cost(session_id) <= budget


class BudgetAlert:
    """Triggers alerts when cost reaches a threshold of the budget."""

    def __init__(self, threshold: float = 0.8) -> None:
        self.threshold = threshold
        self.budget: float = 0.0

    def set_budget(self, budget: float):
        """Set the total budget."""
        self.budget = budget

    def should_alert(self, current_cost: float) -> bool:
        """Check if an alert should be triggered."""
        if self.budget <= 0:
            return False
        return (current_cost / self.budget) >= self.threshold


class CostSensing:
    """Provides cost-based feedback loops for autonomous learning."""

    def __init__(self, slo_regulator: Any) -> None:
        self.slo = slo_regulator

    def check_cost_cap(self, action_cost: float, cap: float) -> bool:
        """Check if action exceeds cost cap."""
        return action_cost <= cap

    def get_cost_feedback(self, model_id: str) -> dict[str, Any]:
        """Get cost feedback for learning system."""
        # In a real implementation, this would query historical data
        return {
            "model_id": model_id,
            "cost_status": "optimal",
            "slo_compliant": self.slo.is_compliant() if hasattr(self.slo, "is_compliant") else True,
        }
