"""Status and shipping MCP tools.

Maps to contracts/mcp-tools.json entries:
- agileplus_status
- agileplus_ship
- agileplus_retrospective
- agileplus_stream_status

Traceability: FR-010, FR-012, FR-013 / WP14-T082, T083
"""

from __future__ import annotations

from collections.abc import AsyncIterator
from typing import Any

from fastmcp import FastMCP

from agileplus_mcp.grpc_client import AgilePlusCoreClient


def register_tools(mcp: FastMCP, client: AgilePlusCoreClient) -> None:
    """Register all status and shipping tools onto *mcp*."""

    @mcp.tool(name="agileplus_status")
    async def status(feature_slug: str = "", wp_sequence: int = 0) -> dict[str, Any]:
        """Get the status of features and work packages.

        If *feature_slug* is provided, returns that feature with its work
        packages. If *wp_sequence* is also provided, returns that specific
        work package. If neither is provided, lists all features.

        Args:
            feature_slug: Optional kebab-case feature identifier.
            wp_sequence: Optional work package sequence number.

        Returns:
            Status dict; shape depends on which arguments were provided.
        """
        if feature_slug and wp_sequence:
            wp = await client.get_work_package_status(feature_slug, wp_sequence)
            return {"work_package": wp}
        if feature_slug:
            feature = await client.get_feature(feature_slug)
            wps = await client.list_work_packages(feature_slug)
            state = await client.get_feature_state(feature_slug)
            return {"feature": feature, "state": state, "work_packages": wps}
        features = await client.list_features()
        return {"features": features}

    @mcp.tool(name="agileplus_ship")
    async def ship(feature_slug: str, target_branch: str = "main") -> dict[str, Any]:
        """Ship a validated feature by merging all WP branches.

        Transitions the feature from Validated to Shipped.

        Args:
            feature_slug: Kebab-case feature identifier.
            target_branch: Target branch to merge into (default: main).

        Returns:
            dict with keys ``status`` and ``message``.
        """
        result = await client.run_command(
            "ship", feature_slug=feature_slug, target_branch=target_branch
        )
        return {
            "status": "success" if result["success"] else "error",
            "message": result["message"],
        }

    @mcp.tool(name="agileplus_retrospective")
    async def retrospective(feature_slug: str) -> dict[str, Any]:
        """Generate a retrospective report for a shipped feature.

        Transitions the feature from Shipped to Retrospected.

        Args:
            feature_slug: Kebab-case feature identifier.

        Returns:
            dict with keys ``status`` and ``message``.
        """
        result = await client.run_command("retrospective", feature_slug=feature_slug)
        return {
            "status": "success" if result["success"] else "error",
            "message": result["message"],
        }

    @mcp.tool(name="agileplus_stream_status")
    async def stream_status(feature_slug: str) -> AsyncIterator[dict[str, Any]]:
        """Stream real-time agent status events for a feature.

        Yields event dicts as agents progress through work packages. The
        stream continues until the server ends it or the connection is lost.

        Args:
            feature_slug: Kebab-case feature identifier to filter events.

        Yields:
            Event dicts with keys: event_type, feature_slug, wp_sequence,
            agent_id, payload, timestamp.
        """
        async for event in client.stream_agent_events(feature_slug):
            yield event
