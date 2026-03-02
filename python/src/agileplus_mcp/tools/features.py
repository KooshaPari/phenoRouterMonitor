"""Feature-management MCP tools.

Maps to contracts/mcp-tools.json entries:
- agileplus_specify
- agileplus_research
- agileplus_plan
- agileplus_implement

Traceability: FR-010, FR-011 / WP14-T082
"""

from __future__ import annotations

from typing import Any

from fastmcp import FastMCP

from agileplus_mcp.grpc_client import AgilePlusCoreClient

# Tools are registered onto the shared FastMCP instance passed in at startup.
# Module-level registration happens via register_tools().


def register_tools(mcp: FastMCP, client: AgilePlusCoreClient) -> None:  # noqa: C901
    """Register all feature-management tools onto *mcp*."""

    @mcp.tool(name="agileplus_specify")
    async def specify(feature_slug: str, from_file: str = "", target_branch: str = "main") -> dict[str, Any]:
        """Create or revise a feature specification.

        Runs the `specify` command on the Rust core, which prompts for or reads
        a feature specification and stores it with a governance contract.

        Args:
            feature_slug: Kebab-case feature identifier.
            from_file: Optional path to a pre-written spec file.
            target_branch: Target branch for eventual merge (default: main).

        Returns:
            dict with keys ``status`` and ``message``.
        """
        kwargs: dict[str, str] = {"target_branch": target_branch}
        if from_file:
            kwargs["from_file"] = from_file
        result = await client.run_command("specify", feature_slug=feature_slug, **kwargs)
        return {
            "status": "success" if result["success"] else "error",
            "message": result["message"],
        }

    @mcp.tool(name="agileplus_research")
    async def research(feature_slug: str) -> dict[str, Any]:
        """Research a feature — codebase scan and feasibility analysis.

        Transitions the feature from Specified to Researched.

        Args:
            feature_slug: Kebab-case feature identifier.

        Returns:
            dict with keys ``status`` and ``message``.
        """
        result = await client.run_command("research", feature_slug=feature_slug)
        return {
            "status": "success" if result["success"] else "error",
            "message": result["message"],
        }

    @mcp.tool(name="agileplus_plan")
    async def plan(feature_slug: str) -> dict[str, Any]:
        """Generate a plan (work packages) for a researched feature.

        Transitions the feature from Researched to Planned and creates
        work packages in the database.

        Args:
            feature_slug: Kebab-case feature identifier.

        Returns:
            dict with keys ``status`` and ``message``.
        """
        result = await client.run_command("plan", feature_slug=feature_slug)
        return {
            "status": "success" if result["success"] else "error",
            "message": result["message"],
        }

    @mcp.tool(name="agileplus_implement")
    async def implement(feature_slug: str, wp_id: str = "") -> dict[str, Any]:
        """Dispatch agents to implement work packages.

        Transitions the feature to Implementing and assigns agents to
        ready work packages.

        Args:
            feature_slug: Kebab-case feature identifier.
            wp_id: Optional specific work package ID (e.g. ``WP01``).

        Returns:
            dict with keys ``status`` and ``message``.
        """
        kwargs: dict[str, str] = {}
        if wp_id:
            kwargs["wp"] = wp_id
        result = await client.run_command("implement", feature_slug=feature_slug, **kwargs)
        return {
            "status": "success" if result["success"] else "error",
            "message": result["message"],
        }
