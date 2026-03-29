"""WP-3007: Trust boundary checks.

OPT-008: LRU cache for policy evaluation results (with TTL) - <50ms repeated evaluations.
"""

import enum
import hashlib
import logging
from typing import Any

from cachetools import TTLCache

from thegent.config import ThegentSettings

_log = logging.getLogger(__name__)


class TrustLevel(enum.IntEnum):
    """Trust levels for agents and domains."""

    EXTERNAL = 0
    PARTNER = 1
    INTERNAL = 2
    STRICT = 3


class TrustBoundaryChecker:
    """Enforces trust boundaries between agents and tasks.

    OPT-008: Uses LRU cache with TTL for repeated policy evaluations.
    """

    def __init__(self, settings: ThegentSettings, cache_ttl_sec: int = 300) -> None:
        """Initialize trust boundary checker.

        Args:
            settings: Thegent settings
            cache_ttl_sec: Cache TTL in seconds (default: 5 minutes)
        """
        self.settings = settings
        self.agent_trust_map = {
            "interactive_agent": TrustLevel.INTERNAL,
            "headless_agent": TrustLevel.INTERNAL,
            "cursor": TrustLevel.INTERNAL,
            "copilot": TrustLevel.EXTERNAL,
            "gemini": TrustLevel.EXTERNAL,
            "quality-agent": TrustLevel.INTERNAL,
        }
        # Add teammate agents if needed
        # OPT-008: LRU cache for routing evaluation results (max 1000 entries, TTL-based)
        self._cache: TTLCache[str, dict[str, Any]] = TTLCache(maxsize=1000, ttl=cache_ttl_sec)

    def get_agent_trust(self, agent_name: str) -> TrustLevel:
        """Return trust level for an agent."""
        return self.agent_trust_map.get(agent_name, TrustLevel.EXTERNAL)

    def evaluate_routing(self, task_prompt: str, target_agent: str) -> dict[str, Any]:
        """
        Evaluate if routing a task to an agent violates trust boundaries.
        Checks for sensitive keywords in prompt vs agent trust level.

        OPT-008: Caches results for repeated evaluations (<50ms for cached lookups).

        Args:
            task_prompt: Task prompt text
            target_agent: Target agent name

        Returns:
            Evaluation result dict with "allowed", "reason", "agent_trust", "risk_score"
        """
        # OPT-008: Create cache key from prompt hash and agent (prompt may be long)
        prompt_hash = hashlib.sha256(task_prompt.encode()).hexdigest()[:16]
        cache_key = f"{target_agent}:{prompt_hash}"

        # OPT-008: Check cache first
        if cache_key in self._cache:
            return self._cache[cache_key]

        # Evaluate policy
        result = {
            "allowed": True,
            "reason": None,
            "agent_trust": self.get_agent_trust(target_agent).name,
            "risk_score": 0,
        }

        # Sensitive keywords that require INTERNAL+ trust
        sensitive_keywords = ["password", "secret", "private_key", "token", "credential", "api_key"]

        agent_level = self.get_agent_trust(target_agent)

        found_sensitive = [kw for kw in sensitive_keywords if kw in task_prompt.lower()]

        if found_sensitive and agent_level < TrustLevel.INTERNAL:
            result["allowed"] = False
            result["reason"] = f"Sensitive data ({found_sensitive[0]}) cannot be sent to EXTERNAL agent {target_agent}"
            result["risk_score"] = 10
            _log.warning("Trust boundary violation: %s", result["reason"])

        # OPT-008: Cache result (TTL handled automatically)
        self._cache[cache_key] = result
        return result

    def check_data_flow(self, source_agent: str, dest_agent: str) -> bool:
        """Verify data flow from source to destination is allowed."""
        source_level = self.get_agent_trust(source_agent)
        dest_level = self.get_agent_trust(dest_agent)

        # Generally, data can flow to same or higher trust
        # Flowing from higher to lower trust might need careful auditing
        if source_level > dest_level:
            _log.info(
                "Cross-boundary data flow: %s (%s) -> %s (%s)",
                source_agent,
                source_level.name,
                dest_agent,
                dest_level.name,
            )

        return True  # Default allow, but logged
