"""Governance and audit MCP tools.

Maps to contracts/mcp-tools.json entries:
- agileplus_validate
- agileplus_get_governance_gate
- agileplus_get_audit_trail
- agileplus_verify_audit_chain

Traceability: FR-010 / WP14-T082
"""

from __future__ import annotations

from typing import Any

from fastmcp import FastMCP

from agileplus_mcp.grpc_client import AgilePlusCoreClient


def register_tools(mcp: FastMCP, client: AgilePlusCoreClient) -> None:
    """Register all governance and audit tools onto *mcp*."""

    @mcp.tool(name="agileplus_validate")
    async def validate(feature_slug: str, skip_policies: bool = False) -> dict[str, Any]:
        """Validate governance compliance for a feature in Implementing state.

        Checks evidence collection, policy rules, and transitions the feature
        to Validated on success.

        Args:
            feature_slug: Kebab-case feature identifier.
            skip_policies: If True, evaluate evidence only (skip policy rules).

        Returns:
            dict with keys ``status`` and ``message``.
        """
        kwargs: dict[str, str] = {}
        if skip_policies:
            kwargs["skip_policies"] = "true"
        result = await client.run_command("validate", feature_slug=feature_slug, **kwargs)
        return {
            "status": "success" if result["success"] else "error",
            "message": result["message"],
        }

    @mcp.tool(name="agileplus_check_governance_gate")
    async def check_governance_gate(
        feature_slug: str, transition: str
    ) -> dict[str, Any]:
        """Check whether a governance gate passes for a planned state transition.

        Args:
            feature_slug: Kebab-case feature identifier.
            transition: The state transition to evaluate (e.g. ``specified->planned``).

        Returns:
            dict with keys ``passed`` (bool) and ``violations`` (list of dicts).
        """
        return await client.check_governance_gate(feature_slug, transition)

    @mcp.tool(name="agileplus_get_audit_trail")
    async def get_audit_trail(
        feature_slug: str, verify: bool = False, after_id: int = 0
    ) -> dict[str, Any]:
        """Retrieve and optionally verify the audit trail for a feature.

        Args:
            feature_slug: Kebab-case feature identifier.
            verify: If True, also verify the hash-chain integrity.
            after_id: Only return entries with id > after_id (for pagination).

        Returns:
            dict with keys ``entries`` (list) and optionally ``verification`` (dict).
        """
        trail = await client.get_audit_trail(feature_slug, after_id=after_id)
        result: dict[str, Any] = {"entries": trail}
        if verify:
            verification = await client.verify_audit_chain(feature_slug)
            result["verification"] = verification
        return result

    @mcp.tool(name="agileplus_verify_audit_chain")
    async def verify_audit_chain(feature_slug: str) -> dict[str, Any]:
        """Verify the SHA-256 hash-chain integrity of the audit trail.

        Args:
            feature_slug: Kebab-case feature identifier.

        Returns:
            dict with keys ``valid``, ``entries_verified``, ``first_invalid_id``,
            and ``error_message``.
        """
        return await client.verify_audit_chain(feature_slug)
