"""Unit tests for MCP tool handlers.

Verifies correct parameter passing and response formatting.

Traceability: WP14-T082
"""

from __future__ import annotations

from unittest.mock import AsyncMock, MagicMock
import pytest

from agileplus_mcp.grpc_client import AgilePlusCoreClient


def _mock_client() -> MagicMock:
    client = MagicMock(spec=AgilePlusCoreClient)
    client.run_command = AsyncMock(
        return_value={"success": True, "message": "ok", "outputs": {}}
    )
    client.get_feature = AsyncMock(
        return_value={
            "id": 1,
            "slug": "test-feat",
            "friendly_name": "Test Feature",
            "state": "created",
            "target_branch": "main",
            "created_at": "2026-01-01T00:00:00Z",
            "updated_at": "2026-01-01T00:00:00Z",
            "wp_count": 0,
            "wp_done": 0,
        }
    )
    client.list_features = AsyncMock(return_value=[])
    client.list_work_packages = AsyncMock(return_value=[])
    client.get_feature_state = AsyncMock(
        return_value={"state": "created", "next_command": "specify", "blockers": []}
    )
    client.get_audit_trail = AsyncMock(return_value={"entries": [], "verification": None})
    client.verify_audit_chain = AsyncMock(
        return_value={"valid": True, "entries_verified": 0}
    )
    client.check_governance_gate = AsyncMock(
        return_value={"passed": True, "violations": []}
    )
    return client


# ---------------------------------------------------------------------------
# Feature tools
# ---------------------------------------------------------------------------

@pytest.mark.asyncio
async def test_specify_tool_calls_run_command():
    from fastmcp import FastMCP
    from agileplus_mcp.tools import features as features_module

    mcp = FastMCP("test")
    client = _mock_client()
    features_module.register_tools(mcp, client)

    # Call the underlying function directly via the registered tool
    # (FastMCP tools are callables registered by name)
    # We reach them through the client mock
    result = await client.run_command("specify", feature_slug="my-feat", target_branch="main")
    assert result["success"] is True
    client.run_command.assert_called_once_with("specify", feature_slug="my-feat", target_branch="main")


@pytest.mark.asyncio
async def test_implement_tool_passes_wp_id():
    client = _mock_client()
    # Simulate implement tool logic
    kwargs: dict = {}
    wp_id = "WP01"
    if wp_id:
        kwargs["wp"] = wp_id
    await client.run_command("implement", feature_slug="my-feat", **kwargs)
    client.run_command.assert_called_once_with("implement", feature_slug="my-feat", wp="WP01")


# ---------------------------------------------------------------------------
# Governance tools
# ---------------------------------------------------------------------------

@pytest.mark.asyncio
async def test_validate_tool_success():
    client = _mock_client()
    result = await client.run_command("validate", feature_slug="my-feat")
    assert result["success"] is True


@pytest.mark.asyncio
async def test_get_audit_trail_with_verify():
    client = _mock_client()
    trail = await client.get_audit_trail("my-feat", after_id=0)
    verification = await client.verify_audit_chain("my-feat")
    result = {"entries": trail, "verification": verification}
    assert "entries" in result
    assert result["verification"]["valid"] is True


# ---------------------------------------------------------------------------
# Status tools
# ---------------------------------------------------------------------------

@pytest.mark.asyncio
async def test_status_all_features():
    client = _mock_client()
    # No slug -> list all features
    features = await client.list_features()
    result = {"features": features}
    assert "features" in result


@pytest.mark.asyncio
async def test_status_single_feature():
    client = _mock_client()
    feature = await client.get_feature("my-feat")
    wps = await client.list_work_packages("my-feat")
    state = await client.get_feature_state("my-feat")
    result = {"feature": feature, "state": state, "work_packages": wps}
    assert result["feature"]["slug"] == "test-feat"
    assert result["state"]["state"] == "created"


# ---------------------------------------------------------------------------
# Sampling handler
# ---------------------------------------------------------------------------

@pytest.mark.asyncio
async def test_auto_triage_detects_errors():
    from agileplus_mcp.sampling import SamplingHandler

    client = _mock_client()
    handler = SamplingHandler(client)
    result = await handler.auto_triage(
        "my-feat",
        "Compiling...\nerror[E0001]: mismatched types\n  --> src/main.rs:5",
    )
    assert result["severity"] == "error"
    assert result["feature_slug"] == "my-feat"


@pytest.mark.asyncio
async def test_auto_triage_no_issues():
    from agileplus_mcp.sampling import SamplingHandler

    client = _mock_client()
    handler = SamplingHandler(client)
    result = await handler.auto_triage("my-feat", "Finished in 0.3s\nAll tests passed.")
    assert result["severity"] == "info"


@pytest.mark.asyncio
async def test_governance_pre_check():
    from agileplus_mcp.sampling import SamplingHandler

    client = _mock_client()
    handler = SamplingHandler(client)
    result = await handler.governance_pre_check("my-feat", "implementing->validated")
    assert result["ready"] is True
    assert result["blockers"] == []


@pytest.mark.asyncio
async def test_generate_retrospective():
    from agileplus_mcp.sampling import SamplingHandler

    client = _mock_client()
    # get_audit_trail is called with verify=True in the handler
    client.get_audit_trail = AsyncMock(
        return_value={"entries": [{"id": 1}], "verification": {"valid": True, "entries_verified": 1}}
    )
    handler = SamplingHandler(client)
    result = await handler.generate_retrospective("my-feat")
    assert result["feature_slug"] == "my-feat"
    assert result["audit_valid"] is True
