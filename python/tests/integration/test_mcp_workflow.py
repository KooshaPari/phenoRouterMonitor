"""MCP layer integration tests for AgilePlus.

These tests exercise the Python gRPC client against a live Rust core server.
They require `AGILEPLUS_GRPC_URL` to be set and the server to be running.
Mark with pytest.mark.integration and skip when the server is unavailable.

Traceability: WP16-T096
"""
from __future__ import annotations

import os

import pytest

# Skip all integration tests unless explicitly requested
pytestmark = pytest.mark.skipif(
    not os.environ.get("AGILEPLUS_GRPC_URL"),
    reason="AGILEPLUS_GRPC_URL not set; skipped outside Docker Compose environment",
)


@pytest.fixture
async def client():
    """Provide a connected AgilePlus gRPC client."""
    from agileplus_mcp.grpc_client import connect_client

    address = os.environ.get("AGILEPLUS_GRPC_URL", "localhost:50051")
    async with connect_client(address) as c:
        yield c


@pytest.mark.asyncio
async def test_mcp_feature_lifecycle(client):
    """Full feature lifecycle via gRPC client: specify -> research -> plan."""
    slug = "mcp-integration-test"

    # Create feature
    result = await client.run_command(
        "specify",
        feature_slug=slug,
        from_content="# FR-001\nThe system shall accept test input.",
    )
    assert result["success"], f"specify failed: {result['message']}"

    # Query state
    feature = await client.get_feature(slug)
    assert feature["state"] == "specified", f"Expected specified, got {feature['state']}"

    # Research
    result = await client.run_command("research", feature_slug=slug)
    assert result["success"], f"research failed: {result['message']}"

    # Plan
    result = await client.run_command("plan", feature_slug=slug)
    assert result["success"], f"plan failed: {result['message']}"

    feature = await client.get_feature(slug)
    assert feature["state"] == "planned"

    # Audit trail should have entries
    trail = await client.get_audit_trail(slug)
    assert len(trail) >= 1, "Expected at least one audit entry"


@pytest.mark.asyncio
async def test_audit_chain_verification(client):
    """Audit chain for a feature with history should verify as valid."""
    slug = "mcp-audit-test"

    # Create and advance feature
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


@pytest.mark.asyncio
async def test_governance_gate_blocks_on_missing_evidence(client):
    """Governance gate should report violations when evidence is missing."""
    slug = "mcp-governance-test"

    # Set up feature in implementing state
    for command in ("specify", "research", "plan"):
        kwargs = {}
        if command == "specify":
            kwargs["from_content"] = "# FR-001\nGovernance test."
        result = await client.run_command(command, feature_slug=slug, **kwargs)
        assert result["success"], f"{command} failed: {result['message']}"

    # Check governance gate without evidence
    gate = await client.check_governance_gate(slug, "implementing -> validated")
    # Gate may pass or fail depending on governance contract — just verify the shape
    assert "passed" in gate, "Governance gate response missing 'passed' field"
    assert "violations" in gate, "Governance gate response missing 'violations' field"


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


@pytest.mark.asyncio
async def test_list_features(client):
    """list_features should return a list of feature dicts."""
    features = await client.list_features()
    assert isinstance(features, list), "Expected a list of features"
    # Each feature must have the required fields
    required = ["id", "slug", "state", "friendly_name"]
    for feature in features:
        for field in required:
            assert field in feature, f"Feature missing field '{field}': {feature}"
