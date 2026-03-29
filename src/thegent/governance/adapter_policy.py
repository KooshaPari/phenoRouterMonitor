"""WP-10004: Adapter admission and trust policy.

Enforces trust-based admission rules for provider adapters.

OPT-008: LRU cache for policy evaluation results (with TTL) - <50ms repeated evaluations.
"""

from typing import Any

from cachetools import TTLCache

from thegent.contracts.capability_registry import CapabilityRegistry


class AdapterAdmissionPolicy:
    """Policy engine for adapter admission control with caching.

    OPT-008: Uses LRU cache with TTL for repeated policy evaluations.
    """

    def __init__(self, registry: CapabilityRegistry, cache_ttl_sec: int = 300) -> None:
        """Initialize policy engine.

        Args:
            registry: Capability registry
            cache_ttl_sec: Cache TTL in seconds (default: 5 minutes)
        """
        self.registry = registry
        # OPT-008: LRU cache for policy evaluation results (max 1000 entries, TTL-based)
        self._cache: TTLCache[tuple[str, str], dict[str, Any]] = TTLCache(maxsize=1000, ttl=cache_ttl_sec)

    def evaluate_admission(self, adapter_id: str, lane: str) -> dict[str, Any]:
        """Evaluate if an adapter can be admitted to a specific lane.

        OPT-008: Caches results for repeated evaluations (<50ms for cached lookups).

        Args:
            adapter_id: Adapter identifier
            lane: Lane name (e.g., "critical", "default")

        Returns:
            Evaluation result dict with "allowed" and optional "reason"/"trust_level"
        """
        # OPT-008: Check cache first
        cache_key = (adapter_id, lane)
        if cache_key in self._cache:
            return self._cache[cache_key]

        # Evaluate policy
        cap = self.registry.get_capability(f"adapter.{adapter_id}")
        if not cap:
            result = {"allowed": False, "reason": "Adapter not registered."}
        elif lane == "critical" and cap.trust_level < 4:
            result = {"allowed": False, "reason": f"Trust level {cap.trust_level} insufficient for critical lane."}
        else:
            result = {"allowed": True, "trust_level": cap.trust_level}

        # OPT-008: Cache result (TTL handled automatically)
        self._cache[cache_key] = result
        return result
