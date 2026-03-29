"""WP-10003: Dispatch graph implementation.

Provides deterministic resolution of operations through a policy-aware dispatch graph.
"""

from typing import Any

from thegent.contracts.capability_registry import CapabilityRegistry


class DispatchResolver:
    """Resolves an OperationEnvelopeV2 to a specific execution path."""

    def __init__(self, registry: CapabilityRegistry | None = None) -> None:
        self.registry = registry or CapabilityRegistry()
        self._aliases: dict[str, str] = {}  # WP-10005: Aliases

    def add_alias(self, alias: str, target_command: str) -> None:
        """Register an alias for a command (WP-10005)."""
        self._aliases[alias] = target_command

    def resolve(self, envelope: Any) -> dict[str, Any]:
        """Resolve the operation to a dispatch path.

        Returns:
            Dict with 'dispatch_path', 'resolved_command', 'status'.
        """
        command = envelope.command
        # Resolve alias
        resolved_command = self._aliases.get(command, command)

        # Check registry
        cap_id = f"{envelope.operation_type}.{resolved_command}"
        if not self.registry.is_supported(cap_id):
            # WP-10006: Suggestion engine
            all_caps = self.registry.list_capabilities()
            suggestions = [c.id for c in all_caps if c.id and resolved_command and resolved_command in c.id]

            return {
                "status": "unsupported",
                "reason": f"Capability {cap_id} not registered.",
                "resolved_command": resolved_command,
                "suggested_alternatives": suggestions,
                "migration_guide": "Refer to docs/guides/PHASE_7_9_GUIDE.md for command changes.",
            }

        # Deterministic path (simplified)
        path = ["validate", "authorize", "execute", "audit"]

        # WP-10007: Traceability context
        cap = self.registry.get_capability(cap_id)

        registry_version = getattr(cap, "version", None)

        return {
            "status": "resolved",
            "dispatch_path": " -> ".join(path),
            "resolved_command": resolved_command,
            "registry_version": registry_version,
            "dispatch_trace": {
                "rule_reason": "direct_match",
                "policy_version": "v2.0",
                "route_hash": "hash_val",
            },
        }
