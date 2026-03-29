"""WP-11004: Preemption and saturation avoidance policies.

Defines rules for preempting non-critical tasks to avoid service saturation.
"""

from typing import Any


class PreemptionPolicy:
    """Policy engine for task preemption and saturation control."""

    def evaluate_preemption(self, system_load: float, task_lane: str) -> dict[str, Any]:
        """Determine if a task should be preempted based on load and lane."""
        # System saturation threshold (e.g. 90% load)
        preempt = False
        reason = "System load normal"

        if system_load > 0.9:
            if task_lane != "critical":
                preempt = True
                reason = f"System saturated ({system_load:.1%}). Preempting non-critical task."

        return {
            "preempt": preempt,
            "reason": reason,
            "rollback_assumption": "Task will be requeued in standard lane when load drops.",
        }
