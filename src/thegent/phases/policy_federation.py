"""Policy federation surface map (FederatedPolicyEngine)."""

from typing import Any, ClassVar


class FederatedPolicyEngine:
    """Federated policy engine for multi-tenant coordination."""

    # Static registry to simulate shared policy storage across engine instances
    _global_policies: ClassVar[dict[str, dict[str, Any]]] = {}

    def __init__(self, namespace: str) -> None:
        """Initialize federated policy engine for a specific namespace."""
        self.namespace = namespace

    def set_policy(self, key: str, value: Any) -> None:
        """Set a policy for the current namespace."""
        if self.namespace not in self._global_policies:
            self._global_policies[self.namespace] = {}
        self._global_policies[self.namespace][key] = value

    def get_policy(self, key: str) -> Any | None:
        """Get a policy, resolving through namespace hierarchy."""
        # Resolve namespace hierarchy (specific -> parent -> global)
        parts = self.namespace.split(".")
        namespaces_to_check = []

        # Build hierarchy: acme.payments.production -> acme.payments -> acme
        for i in range(len(parts), 0, -1):
            namespaces_to_check.append(".".join(parts[:i]))

        # Check each namespace level
        for ns in namespaces_to_check:
            ns_policies = self._global_policies.get(ns, {})
            if key in ns_policies:
                return ns_policies[key]

        return None

    def is_allowed(self, action: str, context: dict[str, Any]) -> bool:
        """Evaluate if an action is allowed based on resolved policies."""
        if action == "model":
            max_cost = self.get_policy("max_model_cost")
            if max_cost is not None:
                current_cost = context.get("cost", 0.0)
                return current_cost <= max_cost

        # Default allow
        return True

    def register_tenant(self, tenant_id: str, policy: dict[str, Any]) -> None:
        """Register a tenant with its policy."""
        self._global_policies[tenant_id] = policy

    def evaluate(self, tenant_id: str, action: str, context: dict[str, Any]) -> bool:
        """Evaluate policy for an action."""
        # Temporary instance for evaluation
        engine = FederatedPolicyEngine(namespace=tenant_id)
        return engine.is_allowed(action, context)

    def resolve_policy(self, namespace: str, policy_key: str) -> dict[str, Any]:
        """Resolve policy through namespace hierarchy."""
        engine = FederatedPolicyEngine(namespace=namespace)
        val = engine.get_policy(policy_key)
        if val is not None:
            return {"value": val}
        return {"allow": True}

    def get_federation_status(self) -> dict[str, Any]:
        """Get federation status."""
        return {
            "namespaces": len(self._global_policies),
            "status": "active",
        }


class PolicyConflictResolver:
    """Resolves conflicts between multiple applicable policies."""

    def resolve(self, policies: list[dict[str, Any]], target_namespace: str) -> dict[str, Any]:
        """Resolve conflicting policies via precedence (most specific wins)."""

        # Sort policies by namespace specificity (number of dots)
        def specificity(p: dict[str, Any]) -> int:
            return p.get("namespace", "").count(".")

        sorted_policies = sorted(policies, key=specificity, reverse=True)

        resolved: dict[str, Any] = {}
        for p in sorted_policies:
            rules = p.get("rules", {})
            for key, val in rules.items():
                if key not in resolved:
                    resolved[key] = val

        return resolved
