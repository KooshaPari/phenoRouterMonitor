"""Feature management MCP tools."""

from __future__ import annotations

from fastmcp import FastMCP


def register(mcp: FastMCP) -> None:
    """Register feature tools with the MCP server."""

    @mcp.tool()
    async def get_feature(slug: str) -> dict:  # type: ignore[return]
        """Get a feature by its slug identifier.

        Args:
            slug: The kebab-case feature slug (e.g., '001-spec-engine')

        Returns:
            Feature details including state, spec hash, and timestamps.
        """
        return {"error": "not_implemented", "message": "Feature lookup not yet wired to gRPC"}

    @mcp.tool()
    async def list_features(state: str | None = None) -> list[dict]:  # type: ignore[return]
        """List all features, optionally filtered by state.

        Args:
            state: Optional state filter (created, specified, planned, implementing, done)

        Returns:
            List of feature summaries with slug, title, and current state.
        """
        return [{"error": "not_implemented"}]

    @mcp.tool()
    async def get_work_packages(feature_slug: str) -> list[dict]:  # type: ignore[return]
        """Get all work packages for a feature.

        Args:
            feature_slug: The parent feature's slug

        Returns:
            List of work packages with states, dependencies, and assignees.
        """
        return [{"error": "not_implemented"}]

    @mcp.tool()
    async def get_work_package(feature_slug: str, wp_id: str) -> dict:  # type: ignore[return]
        """Get a specific work package by ID within a feature.

        Args:
            feature_slug: The parent feature's slug
            wp_id: The work package identifier (e.g., 'WP02')

        Returns:
            Work package details including tasks, lane, assignee, and dependencies.
        """
        return {"error": "not_implemented"}

    @mcp.tool()
    async def get_tasks(feature_slug: str, wp_id: str | None = None) -> list[dict]:  # type: ignore[return]
        """Get tasks for a feature, optionally scoped to a work package.

        Args:
            feature_slug: The parent feature's slug
            wp_id: Optional work package filter (e.g., 'WP02')

        Returns:
            List of tasks with status, type, and description.
        """
        return [{"error": "not_implemented"}]
