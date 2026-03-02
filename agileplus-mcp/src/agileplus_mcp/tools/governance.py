"""Governance and audit MCP tools."""

from __future__ import annotations

from fastmcp import FastMCP


def register(mcp: FastMCP) -> None:
    """Register governance tools with the MCP server."""

    @mcp.tool()
    async def check_governance(feature_slug: str) -> dict:  # type: ignore[return]
        """Check governance contract compliance for a feature.

        Validates that the feature satisfies all governance rules defined in the
        system's constitution: spec completeness, plan approval, review sign-off, etc.

        Args:
            feature_slug: Feature to check governance for (e.g., '001-spec-engine')

        Returns:
            Governance check result with pass/fail status and list of violations.
        """
        return {"error": "not_implemented"}

    @mcp.tool()
    async def get_audit_trail(feature_slug: str, limit: int = 50) -> list[dict]:  # type: ignore[return]
        """Get the hash-chained audit trail for a feature.

        Returns a tamper-evident log of all state transitions and agent actions
        for the feature, verified via SHA-256 hash chaining.

        Args:
            feature_slug: Feature to get audit trail for (e.g., '001-spec-engine')
            limit: Maximum number of entries to return (default: 50, max: 500)

        Returns:
            List of audit entries with hash chain integrity status, timestamps,
            agent identifiers, and action descriptions.
        """
        return [{"error": "not_implemented"}]

    @mcp.tool()
    async def verify_audit_chain(feature_slug: str) -> dict:  # type: ignore[return]
        """Verify integrity of a feature's audit hash chain.

        Walks the entire audit chain and checks each hash against its predecessor,
        detecting any tampering or corruption in the audit log.

        Args:
            feature_slug: Feature whose audit chain to verify

        Returns:
            Verification result: valid/invalid with first broken entry if invalid.
            Includes total entries checked and verification timestamp.
        """
        return {"error": "not_implemented"}

    @mcp.tool()
    async def get_governance_rules(feature_slug: str | None = None) -> dict:  # type: ignore[return]
        """Get the governance rules in effect, optionally for a specific feature.

        Returns the active governance constitution rules that control state
        transitions, approval requirements, and compliance checks.

        Args:
            feature_slug: Optional feature to scope rules to. If None, returns
                          global governance rules.

        Returns:
            Governance rules including required approvers, state transition guards,
            and compliance checklist items.
        """
        return {"error": "not_implemented"}
