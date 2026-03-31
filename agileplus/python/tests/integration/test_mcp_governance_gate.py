"""MCP governance gate integration tests."""

from __future__ import annotations

import pytest


@pytest.mark.asyncio
async def test_governance_gate_blocks_on_missing_evidence(client):
    """Governance gate should report violations when evidence is missing."""
    slug = "mcp-governance-test"

    for command in ("specify", "research", "plan"):
        kwargs = {}
        if command == "specify":
            kwargs["from_content"] = "# FR-001\nGovernance test."
        result = await client.run_command(command, feature_slug=slug, **kwargs)
        assert result["success"], f"{command} failed: {result['message']}"

    gate = await client.check_governance_gate(slug, "implementing -> validated")
    assert "passed" in gate, "Governance gate response missing 'passed' field"
    assert "violations" in gate, "Governance gate response missing 'violations' field"
