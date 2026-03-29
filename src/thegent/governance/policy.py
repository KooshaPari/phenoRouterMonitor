"""Core policy management and evaluation (WP-3001, WP-3002)."""

import logging
from typing import Any

logger = logging.getLogger(__name__)


class PolicyManager:
    """Manages system-wide policies and their evaluation."""

    def __init__(self, initial_policies: dict[str, Any] | None = None) -> None:
        self._policies = initial_policies or {}

    def update(self, new_policies: dict[str, Any]):
        """Update policies."""
        self._policies.update(new_policies)
        logger.info("Policies updated: %s", list(new_policies.keys()))

    def get_policy(self, key: str) -> Any | None:
        """Get a policy value."""
        return self._policies.get(key)


class LearningSession:
    """Represents an autonomous learning session bounded by policy."""

    def __init__(self, policy_manager: PolicyManager) -> None:
        self.policy_manager = policy_manager
        self.cost_cap = policy_manager.get_policy("cost_cap") or 10.0
        self._active = False

    def start(self):
        """Start the learning session."""
        self._active = True

    def is_valid(self) -> bool:
        """Verify session is still valid against current policy."""
        # Refresh from policy manager
        new_cap = self.policy_manager.get_policy("cost_cap")
        if new_cap is not None:
            self.cost_cap = new_cap

        return self._active
