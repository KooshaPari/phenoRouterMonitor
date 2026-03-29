"""FederatedPolicyEngine: scope-aware policy registry for governance.

Traces to: FR-GOV-001 (policy federation), FR-GOV-002 (scope precedence)
"""

from __future__ import annotations

import orjson as json
import logging
from dataclasses import dataclass, field
from enum import Enum
from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from pathlib import Path

_log = logging.getLogger(__name__)


class PolicyScope(Enum):
    """Hierarchy level of a policy rule. Higher numeric value = higher authority."""

    LOCAL = 1
    REGIONAL = 2
    GLOBAL = 3


@dataclass(order=True)
class PolicyRule:
    """A single governance rule with scope and evaluation metadata (FR-GOV-001)."""

    # priority is first so dataclass ordering uses it as the primary sort key
    priority: int
    rule_id: str = field(compare=False)
    scope: PolicyScope = field(compare=False)
    condition: str = field(compare=False)
    action: str = field(compare=False)
    namespace: str = field(default="global", compare=False)

    @classmethod
    def create(
        cls, rule_id: str, scope: PolicyScope, condition: str, action: str, priority: int, namespace: str = "global"
    ) -> PolicyRule:
        """Named constructor matching the canonical field order in the task spec."""
        return cls(
            priority=priority, rule_id=rule_id, scope=scope, condition=condition, action=action, namespace=namespace
        )


class FederatedPolicyEngine:
    """Registry and evaluator for scoped governance policy rules (FR-GOV-001).

    Supports multi-tenant federation via namespace hierarchy.
    """

    def __init__(self, default_namespace: str = "global") -> None:
        self.default_namespace = default_namespace
        # Map of namespace -> rule_id -> PolicyRule
        self._namespaces: dict[str, dict[str, PolicyRule]] = {}

    def register(self, rule: PolicyRule) -> None:
        """Add *rule* to the registry, replacing any existing rule with the same id."""
        ns = rule.namespace or self.default_namespace
        if ns not in self._namespaces:
            self._namespaces[ns] = {}
        self._namespaces[ns][rule.rule_id] = rule

    def load_from_file(self, path: Path, namespace: str = "global") -> None:
        """Load rules from a JSON file."""
        if not path.exists():
            _log.warning("Policy file %s not found.", path)
            return

        raw: list[dict[str, Any]] = json.loads(path.read_text(encoding="utf-8"))
        for item in raw:
            rule = PolicyRule.create(
                rule_id=str(item["rule_id"]),
                scope=PolicyScope[str(item["scope"]).upper()],
                condition=str(item["condition"]),
                action=str(item["action"]),
                priority=int(str(item.get("priority", 100))),
                namespace=item.get("namespace", namespace),
            )
            self.register(rule)

    def resolve_policies(self, namespace: str) -> list[PolicyRule]:
        """Resolve all rules for a namespace, following hierarchy (specific -> parent -> global)."""
        parts = namespace.split(".")
        namespaces_to_check = []
        for i in range(len(parts), 0, -1):
            namespaces_to_check.append(".".join(parts[:i]))

        if "global" not in namespaces_to_check:
            namespaces_to_check.append("global")

        resolved_rules: dict[str, PolicyRule] = {}

        # Check from global up to specific, so more specific rules override
        for ns in reversed(namespaces_to_check):
            ns_rules = self._namespaces.get(ns, {})
            for rid, rule in ns_rules.items():
                # Conflict resolution: higher scope wins; then more specific namespace wins
                if rid not in resolved_rules:
                    resolved_rules[rid] = rule
                else:
                    current_rule = resolved_rules[rid]
                    # If new rule has higher scope, it overrides
                    if rule.scope.value > current_rule.scope.value:
                        resolved_rules[rid] = rule
                    # If same scope, but this is a more specific namespace, it overrides?
                    # Usually specificity overrides in local, but global scope overrides local.
                    # We'll follow the rule: Higher Scope > Specifity.

        return list(resolved_rules.values())

    def evaluate(self, namespace: str, context: dict[str, Any]) -> list[PolicyRule]:
        """Evaluate matching rules for a namespace, sorted by priority."""
        rules = self.resolve_policies(namespace)
        matched = [rule for rule in rules if context.get(rule.condition)]
        return sorted(matched, key=lambda r: r.priority)

    def merge(self, other: FederatedPolicyEngine) -> FederatedPolicyEngine:
        """Combine two engines."""
        merged = FederatedPolicyEngine(self.default_namespace)

        # Combine all namespaces
        all_ns = set(self._namespaces.keys()) | set(other._namespaces.keys())

        for ns in all_ns:
            self_ns = self._namespaces.get(ns, {})
            other_ns = other._namespaces.get(ns, {})

            all_rids = set(self_ns.keys()) | set(other_ns.keys())
            for rid in all_rids:
                r1 = self_ns.get(rid)
                r2 = other_ns.get(rid)

                if r1 and not r2:
                    merged.register(r1)
                elif r2 and not r1:
                    merged.register(r2)
                elif r1 and r2:
                    # Higher scope wins
                    if r2.scope.value > r1.scope.value:
                        merged.register(r2)
                    else:
                        merged.register(r1)

        return merged
