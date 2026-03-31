"""MCP audit chain integration tests."""

from __future__ import annotations

import pytest


@pytest.mark.asyncio
async def test_audit_chain_verification(client):
    """Audit chain for a feature with history should verify as valid."""
    slug = "mcp-audit-test"

    await client.run_command(
        "specify",
        feature_slug=slug,
        from_content="# FR-016\nAudit chain test.",
    )

    verification = await client.verify_audit_chain(slug)
    assert verification["valid"], (
        f"Audit chain should be valid: {verification.get('error_message')}"
    )
    assert verification["entries_verified"] >= 1
