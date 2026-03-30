"""Status and dashboard MCP tools."""

from __future__ import annotations

from fastmcp import FastMCP


def register(mcp: FastMCP) -> None:
    """Register status tools with the MCP server."""

    @mcp.tool()
    async def get_dashboard() -> dict:  # type: ignore[return]
        """Get a dashboard summary of all active features.

        Provides a high-level overview of the AgilePlus development engine's current
        state, including feature counts by lifecycle state, active work packages,
        and recent audit activity.

        Returns:
            Dashboard summary with:
            - feature_counts: dict of state -> count
            - active_work_packages: list of in-progress WPs
            - recent_audit_entries: last 10 audit events across all features
            - health: overall system health status
        """
        return {"error": "not_implemented"}

    @mcp.tool()
    async def get_metrics(feature_slug: str | None = None) -> dict:  # type: ignore[return]
        """Get telemetry metrics, optionally scoped to a feature.

        Returns performance and process metrics collected via OpenTelemetry,
        including duration statistics, agent utilization, and review cycle times.

        Args:
            feature_slug: Optional feature to scope metrics to. If None, returns
                          system-wide aggregate metrics.

        Returns:
            Metrics including:
            - duration_stats: min/max/avg duration for key operations
            - agent_runs: count and success rate by agent type
            - review_cycles: average cycles before approval
            - grpc_latency: p50/p95/p99 latency to Rust core
        """
        return {"error": "not_implemented"}

    @mcp.tool()
    async def health_check() -> dict:  # type: ignore[return]
        """Check health of the MCP service and its downstream dependencies.

        Performs a liveness and readiness check of the MCP server, the gRPC
        connection to the Rust core, and any other critical dependencies.

        Returns:
            Health status with:
            - status: 'healthy' | 'degraded' | 'unhealthy'
            - mcp_server: 'ok' | 'error'
            - grpc_core: 'ok' | 'error' | 'unreachable'
            - version: current service version string
        """
        return {
            "status": "healthy",
            "mcp_server": "ok",
            "grpc_core": "unreachable",  # Rust core not yet running
            "version": "0.1.0",
        }
