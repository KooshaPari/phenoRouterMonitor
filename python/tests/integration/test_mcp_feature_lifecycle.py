"""MCP feature lifecycle integration tests."""

from __future__ import annotations

import pytest


@pytest.mark.asyncio
async def test_mcp_feature_lifecycle(client):
    """Full feature lifecycle via gRPC client: specify -> research -> plan."""
    slug = "mcp-integration-test"

    result = await client.run_command(
        "specify",
        feature_slug=slug,
        from_content="# FR-001\nThe system shall accept test input.",
    )
    assert result["success"], f"specify failed: {result['message']}"

    feature = await client.get_feature(slug)
    assert feature["state"] == "specified", f"Expected specified, got {feature['state']}"

    result = await client.run_command("research", feature_slug=slug)
    assert result["success"], f"research failed: {result['message']}"

    result = await client.run_command("plan", feature_slug=slug)
    assert result["success"], f"plan failed: {result['message']}"

    feature = await client.get_feature(slug)
    assert feature["state"] == "planned"

    trail = await client.get_audit_trail(slug)
    assert len(trail) >= 1, "Expected at least one audit entry"
