"""Cost aggregation primitives for thegent."""

from __future__ import annotations


class CostCap:
    """Enforces a maximum cost limit."""

    def __init__(self, max_cost: float) -> None:
        self.max_cost = max_cost

    def check(self, cost: float) -> bool:
        """Return True if the cost is within the cap."""
        return cost <= self.max_cost


class CostTracker:
    """Tracks costs per session in real time."""

    def __init__(self) -> None:
        self._sessions: dict[str, float] = {}

    def start_session(self, session_id: str) -> None:
        """Start tracking a new session."""
        self._sessions[session_id] = 0.0

    def record_cost(self, session_id: str, amount: float) -> None:
        """Record a cost for a session."""
        self._sessions[session_id] = self._sessions.get(session_id, 0.0) + amount

    def get_session_cost(self, session_id: str) -> float:
        """Get total cost for a session."""
        return self._sessions.get(session_id, 0.0)

    def is_within_budget(self, session_id: str, budget: float) -> bool:
        """Check if session cost is within budget."""
        return self.get_session_cost(session_id) <= budget


class BudgetAlert:
    """Alerts when cost exceeds a percentage threshold of budget."""

    def __init__(self, threshold: float = 0.8) -> None:
        self.threshold = threshold
        self._budget = 0.0

    def set_budget(self, budget: float) -> None:
        """Set the budget amount."""
        self._budget = budget

    def should_alert(self, current_cost: float) -> bool:
        """Return True if current cost exceeds the threshold percentage of budget."""
        if self._budget <= 0:
            return False
        return (current_cost / self._budget) >= self.threshold
