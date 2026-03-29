"""Governance-specific policy federation implementation.

Implements multi-tenant policy federation for governance system, building on
the base FederatedPolicyEngine with governance-specific integration.
"""

import logging
from typing import Any

from cachetools import TTLCache

from thegent.governance.escalation import EscalationQueue
from thegent.phases.policy_federation import FederatedPolicyEngine

_log = logging.getLogger(__name__)


class PolicyCache:
    """TTL-based policy cache for federation."""

    def __init__(self, ttl_seconds: int = 300) -> None:
        """Initialize policy cache.

        Args:
            ttl_seconds: Cache TTL in seconds (default: 300 = 5 minutes)
        """
        self.cache: TTLCache[str, dict[str, Any]] = TTLCache(maxsize=1000, ttl=ttl_seconds)

    def get(self, namespace: str, policy_key: str) -> dict[str, Any] | None:
        """Get cached policy.

        Args:
            namespace: Policy namespace
            policy_key: Policy key

        Returns:
            Cached policy dict or None if not cached
        """
        cache_key = f"{namespace}:{policy_key}"
        return self.cache.get(cache_key)

    def set(self, namespace: str, policy_key: str, policy: dict[str, Any]) -> None:
        """Cache policy.

        Args:
            namespace: Policy namespace
            policy_key: Policy key
            policy: Policy dict to cache
        """
        cache_key = f"{namespace}:{policy_key}"
        self.cache[cache_key] = policy

    def invalidate(self, namespace: str, policy_key: str | None = None) -> None:
        """Invalidate cache for namespace or specific policy.

        Args:
            namespace: Policy namespace
            policy_key: Optional specific policy key to invalidate
        """
        if policy_key:
            cache_key = f"{namespace}:{policy_key}"
            self.cache.pop(cache_key, None)
        else:
            # Invalidate all policies for namespace
            keys_to_remove = [k for k in self.cache if k.startswith(f"{namespace}:")]
            for key in keys_to_remove:
                self.cache.pop(key, None)


class GovernanceConflictResolver:
    """Governance-specific conflict resolution."""

    def resolve_governance_conflict(
        self,
        policies: list[dict[str, Any]],
        namespace: str,
    ) -> dict[str, Any]:
        """Resolve governance policy conflicts.

        Precedence rules:
        1. Project-level overrides org-level
        2. Environment-level overrides project-level
        3. Explicit deny overrides allow
        4. More restrictive wins (lower cost cap, stricter SLA)

        Args:
            policies: List of policy dicts to resolve
            namespace: Target namespace

        Returns:
            Resolved policy dict
        """
        # Sort by namespace depth (deeper = higher precedence)
        sorted_policies = sorted(
            policies,
            key=lambda p: len(p.get("namespace", "").split(".")),
            reverse=True,
        )

        resolved: dict[str, Any] = {}
        for policy in sorted_policies:
            # Merge with precedence
            rules = policy.get("rules", {})
            for key, value in rules.items():
                if key not in resolved or self._is_more_restrictive(key, value, resolved[key]):
                    resolved[key] = value

        return resolved

    def _is_more_restrictive(self, key: str, new_value: Any, current_value: Any) -> bool:
        """Check if new value is more restrictive.

        Args:
            key: Policy key
            new_value: New policy value
            current_value: Current policy value

        Returns:
            True if new value is more restrictive
        """
        if key in {"cost_cap", "sla_minutes"}:
            return (
                isinstance(new_value, (int | float))
                and isinstance(current_value, (int | float))
                and new_value < current_value
            )
        if key == "allow":
            return not new_value  # Deny is more restrictive
        return False


class GovernancePolicyFederation:
    """Governance-specific policy federation.

    Integrates FederatedPolicyEngine with governance components (escalation queue,
    policy cache, conflict resolution) for multi-tenant policy coordination.
    """

    def __init__(
        self,
        federated_engine: FederatedPolicyEngine | None = None,
        cache_ttl: int = 300,
    ) -> None:
        """Initialize governance policy federation.

        Args:
            federated_engine: Federated policy engine (creates new if None)
            cache_ttl: Cache TTL in seconds (default: 300)
        """
        self.federated_engine = federated_engine or FederatedPolicyEngine(namespace="default")
        self.escalation_queue = EscalationQueue()
        self.cache = PolicyCache(ttl_seconds=cache_ttl)
        self.conflict_resolver = GovernanceConflictResolver()

    def evaluate_governance_policy(
        self,
        namespace: str,
        action: str,
        context: dict[str, Any],
    ) -> bool:
        """Evaluate governance policy with federation support.

        Args:
            namespace: Policy namespace (e.g., "acme.payments.production")
            action: Action to evaluate (e.g., "cost_exceeded", "sla_violation")
            context: Action context (must include "run_id" for escalation)

        Returns:
            True if action is allowed, False if denied
        """
        policy_key = f"governance.{action}"

        # Check cache first
        cached_policy = self.cache.get(namespace, policy_key)
        if cached_policy:
            _log.debug("Using cached policy for %s:%s", namespace, policy_key)
            policy = cached_policy
        else:
            # Resolve policy through federation hierarchy
            try:
                policy = self.federated_engine.resolve_policy(
                    namespace=namespace,
                    policy_key=policy_key,
                )
                # Cache resolved policy
                self.cache.set(namespace, policy_key, policy)
            except Exception as e:
                _log.warning("Failed to resolve policy %s:%s: %s", namespace, policy_key, e)
                # Default: allow if policy resolution fails (fail-open)
                return True

        # Evaluate policy
        if not self._policy_allows(policy, context):
            # Add to escalation queue if blocked
            run_id = context.get("run_id")
            if run_id:
                sla_minutes = policy.get("sla_minutes", 60)
                # Map SLA minutes to priority (lower SLA = higher priority)
                if sla_minutes <= 15:
                    priority = 4  # URGENT
                elif sla_minutes <= 60:
                    priority = 3  # HIGH
                elif sla_minutes <= 240:
                    priority = 2  # NORMAL
                else:
                    priority = 1  # LOW
                self.escalation_queue.add(
                    run_id=run_id,
                    reason=f"Policy denied: {action}",
                    priority=priority,
                )
            _log.info("Policy denied for %s:%s (run_id: %s)", namespace, action, run_id)
            return False

        return True

    def _policy_allows(self, policy: dict[str, Any], context: dict[str, Any]) -> bool:
        """Check if policy allows action.

        Args:
            policy: Policy dict
            context: Action context

        Returns:
            True if allowed
        """
        # Simple policy evaluation: check "allow" key
        if "allow" in policy:
            return bool(policy["allow"])

        # Default: allow if no explicit deny
        return True

    def invalidate_cache(self, namespace: str, policy_key: str | None = None) -> None:
        """Invalidate policy cache.

        Args:
            namespace: Policy namespace
            policy_key: Optional specific policy key
        """
        self.cache.invalidate(namespace, policy_key)
        _log.info("Cache invalidated for %s:%s", namespace, policy_key or "*")
