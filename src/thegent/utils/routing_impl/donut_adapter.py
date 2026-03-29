"""Donut Architecture adapter for LiteLLM routing integration.

Integrates the routing layer with the Donut shared layer, enabling:
- Shared router instances across teammates
- Model preference propagation from queue
- Routing stats harvesting on session stop
- Team configuration export for multi-agent coordination

The Donut architecture provides a shared layer (queue, harvest, rules sync)
that is platform-agnostic, used by Claude Code, Codex, Cursor, Factory Droid,
and Augment Code.
"""

from __future__ import annotations

import orjson as json
import logging
from dataclasses import dataclass, field
from datetime import UTC, datetime
from pathlib import Path
from typing import Any

logger = logging.getLogger(__name__)

# Default paths for Donut shared layer
DEFAULT_QUEUE_PATH = Path.home() / ".thegent" / "prompt_queue.jsonl"
DEFAULT_HARVEST_PATH = Path.home() / ".thegent" / "routing_harvest.jsonl"


@dataclass
class RoutingStats:
    """Routing statistics for harvest export."""

    total_requests: int = 0
    requests_by_model: dict[str, int] = field(default_factory=dict)
    requests_by_provider: dict[str, int] = field(default_factory=dict)
    requests_by_category: dict[str, int] = field(default_factory=dict)
    total_cost_usd: float = 0.0
    total_tokens: int = 0
    errors: int = 0
    fallback_count: int = 0


class RoutingDonutAdapter:
    """Adapter integrating routing with Donut shared layer.

    Provides:
    - Shared router instance management (singleton per policy)
    - Model preference reading from prompt queue
    - Routing stats harvesting on session stop
    - Team router config export for multi-agent coordination

    The adapter follows the Donut Architecture pattern where shared components
    (queue, harvest, rules) are platform-agnostic and used across all agents.
    """

    def __init__(
        self,
        queue_path: Path | str | None = None,
        harvest_path: Path | str | None = None,
    ) -> None:
        """Initialize the Donut adapter.

        Args:
            queue_path: Path to prompt queue JSONL file.
                Defaults to ~/.thegent/prompt_queue.jsonl
            harvest_path: Path to routing harvest JSONL file.
                Defaults to ~/.thegent/routing_harvest.jsonl
        """
        self._queue_path = Path(queue_path) if queue_path else DEFAULT_QUEUE_PATH
        self._harvest_path = Path(harvest_path) if harvest_path else DEFAULT_HARVEST_PATH
        self._routers: dict[str, Any] = {}  # Router instances by policy
        self._stats = RoutingStats()

    @property
    def queue_path(self) -> Path:
        """Path to the prompt queue file."""
        return self._queue_path

    @property
    def harvest_path(self) -> Path:
        """Path to the routing harvest file."""
        return self._harvest_path

    def get_router(self, policy: str = "cheapest") -> Any:
        """Get or create a shared LiteLLM router for the given policy.

        Routers are cached by policy, enabling reuse across teammates
        and multiple calls within a session.

        Args:
            policy: Routing policy (cheapest, fastest, round_robin)

        Returns:
            Configured LiteLLM Router instance
        """
        if policy not in self._routers:
            from thegent.utils.routing_impl.litellm_router import get_litellm_router

            logger.debug("Creating new LiteLLM router with policy=%s", policy)
            self._routers[policy] = get_litellm_router(policy)
        return self._routers[policy]

    def read_model_preference_from_queue(self) -> str | None:
        """Read preferred_model from the first unclaimed queue item.

        The prompt queue stores pending items that may include a preferred_model
        field indicating which model should handle the task. This method reads
        from the Donut shared queue to extract that preference.

        Queue item format:
            {"ts": "ISO8601", "prompt": "...", "preferred_model": "...",
             "claimed_by": null, "lease_expires_at": null}

        Returns:
            The preferred_model string if found in an unclaimed item,
            otherwise None.
        """
        if not self._queue_path.exists():
            return None

        try:
            lines = self._queue_path.read_text(encoding="utf-8").splitlines()
        except OSError as e:
            logger.warning("Failed to read queue file %s: %s", self._queue_path, e)
            return None

        for line in lines:
            line = line.strip()
            if not line:
                continue

            try:
                item = json.loads(line)
            except json.JSONDecodeError:
                continue

            # Check if item is unclaimed
            if item.get("claimed_by") is None:
                preferred_model = item.get("preferred_model")
                if preferred_model:
                    logger.debug("Found preferred_model in queue: %s", preferred_model)
                    return preferred_model

        return None

    def harvest_on_stop(self) -> dict[str, Any]:
        """Export routing stats for harvest on session stop.

        Creates a harvest entry with routing statistics and appends it
        to the routing harvest JSONL file. This is called at session end
        to capture routing metrics for analysis and cost tracking.

        Returns:
            The harvest entry dictionary that was written
        """
        # Get cost tracker stats if available
        cost_stats = self._get_cost_stats()

        entry: dict[str, Any] = {
            "type": "routing_harvest",
            "ts": datetime.now(UTC).isoformat(),
            "stats": {
                "total_requests": self._stats.total_requests,
                "requests_by_model": dict(self._stats.requests_by_model),
                "requests_by_provider": dict(self._stats.requests_by_provider),
                "requests_by_category": dict(self._stats.requests_by_category),
                "total_cost_usd": self._stats.total_cost_usd,
                "total_tokens": self._stats.total_tokens,
                "errors": self._stats.errors,
                "fallback_count": self._stats.fallback_count,
            },
            "cost": cost_stats,
        }

        # Ensure directory exists
        self._harvest_path.parent.mkdir(parents=True, exist_ok=True)

        # Append to harvest file
        try:
            with self._harvest_path.open("a", encoding="utf-8") as f:
                f.write(json.dumps(entry).decode() + "\n")
            logger.debug("Wrote routing harvest to %s", self._harvest_path)
        except OSError as e:
            logger.warning("Failed to write harvest file %s: %s", self._harvest_path, e)

        return entry

    def get_team_router_config(self) -> dict[str, Any]:
        """Get router config dict for sharing across teammates.

        Exports configuration that can be used by teammate agents
        to instantiate compatible routers. This enables coordinated
        routing decisions across a team of agents.

        Returns:
            Dictionary containing:
            - policies: List of available routing policies
            - default_policy: The default routing policy
            - queue_path: Path to the shared queue
            - harvest_path: Path to the harvest file
            - stats_summary: Current routing statistics summary
        """
        return {
            "policies": ["cheapest", "fastest", "round_robin", "latency-based-routing"],
            "default_policy": "cheapest",
            "queue_path": str(self._queue_path),
            "harvest_path": str(self._harvest_path),
            "stats_summary": {
                "total_requests": self._stats.total_requests,
                "total_cost_usd": self._stats.total_cost_usd,
                "errors": self._stats.errors,
            },
        }

    def record_request(
        self,
        model: str,
        provider: str,
        category: str = "normal",
        tokens: int = 0,
        cost_usd: float = 0.0,
        is_fallback: bool = False,
        is_error: bool = False,
    ) -> None:
        """Record a routing request for stats tracking.

        Args:
            model: The model that handled the request
            provider: The provider used
            category: Task category (fast, normal, complex, high_complex)
            tokens: Total tokens used
            cost_usd: Cost in USD
            is_fallback: Whether this was a fallback routing
            is_error: Whether the request resulted in an error
        """
        self._stats.total_requests += 1
        self._stats.requests_by_model[model] = self._stats.requests_by_model.get(model, 0) + 1
        self._stats.requests_by_provider[provider] = self._stats.requests_by_provider.get(provider, 0) + 1
        self._stats.requests_by_category[category] = self._stats.requests_by_category.get(category, 0) + 1
        self._stats.total_tokens += tokens
        self._stats.total_cost_usd += cost_usd

        if is_fallback:
            self._stats.fallback_count += 1
        if is_error:
            self._stats.errors += 1

    def _get_cost_stats(self) -> dict[str, Any]:
        """Get cost tracking stats from cost tracker if available."""
        cost_data: dict[str, Any] = {
            "mtd_total": 0.0,
            "daily_total": 0.0,
        }

        try:
            from thegent.utils.routing_impl.cost_tracker import get_cost_tracker

            tracker = get_cost_tracker()
            stats = tracker.get_stats()
            cost_data["mtd_total"] = stats.total_cost_usd
            cost_data["daily_total"] = stats.daily_spend_usd
        except Exception as e:
            logger.debug("Could not get cost stats: %s", e)

        return cost_data

    def clear_stats(self) -> None:
        """Reset routing statistics."""
        self._stats = RoutingStats()

    def get_stats(self) -> RoutingStats:
        """Get current routing statistics."""
        return RoutingStats(
            total_requests=self._stats.total_requests,
            requests_by_model=dict(self._stats.requests_by_model),
            requests_by_provider=dict(self._stats.requests_by_provider),
            requests_by_category=dict(self._stats.requests_by_category),
            total_cost_usd=self._stats.total_cost_usd,
            total_tokens=self._stats.total_tokens,
            errors=self._stats.errors,
            fallback_count=self._stats.fallback_count,
        )


# Global adapter instance
_adapter: RoutingDonutAdapter | None = None


def get_donut_adapter() -> RoutingDonutAdapter:
    """Get global Donut adapter instance."""
    global _adapter
    if _adapter is None:
        _adapter = RoutingDonutAdapter()
    return _adapter
