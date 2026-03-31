"""MCP feature state query integration tests."""

from __future__ import annotations

import pytest


@pytest.mark.asyncio
async def test_feature_state_query(client):
    """get_feature_state should return current state and next command."""
    slug = "mcp-state-query-test"

    await client.run_command(
        "specify",
        feature_slug=slug,
        from_content="# FR-001\nState query test.",
    )

    state_info = await client.get_feature_state(slug)
    assert "state" in state_info
    assert "next_command" in state_info
    assert state_info["state"] == "specified"
    assert state_info["next_command"] != "", "Expected a suggested next command"
