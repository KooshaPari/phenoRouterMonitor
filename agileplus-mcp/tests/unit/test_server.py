"""Smoke tests for MCP server initialization."""

from __future__ import annotations

import pytest


def test_server_creates() -> None:
    """Verify the FastMCP server can be instantiated with the expected name."""
    from agileplus_mcp.server import mcp

    assert mcp.name == "agileplus"


@pytest.mark.asyncio
async def test_tools_registered() -> None:
    """Verify all required MCP tools are registered with the server.

    This test acts as a contract: if someone removes a tool, this test fails.
    """
    from agileplus_mcp.server import mcp

    tool_names = [t.name for t in await mcp.list_tools()]
    # Feature tools
    assert "get_feature" in tool_names
    assert "list_features" in tool_names
    assert "get_work_packages" in tool_names
    # Governance tools
    assert "check_governance" in tool_names
    assert "get_audit_trail" in tool_names
    assert "verify_audit_chain" in tool_names
    # Status tools
    assert "get_dashboard" in tool_names
    assert "health_check" in tool_names


def test_grpc_client_target() -> None:
    """Verify gRPC client can be instantiated with the correct default target."""
    from agileplus_mcp.grpc_client import AgilePlusCoreClient

    client = AgilePlusCoreClient()
    assert client.target == "localhost:50051"


def test_grpc_client_custom_target() -> None:
    """Verify gRPC client accepts custom host and port."""
    from agileplus_mcp.grpc_client import AgilePlusCoreClient

    client = AgilePlusCoreClient(host="grpc-core", port=9090)
    assert client.target == "grpc-core:9090"


@pytest.mark.asyncio
async def test_grpc_client_stubs_raise_not_implemented() -> None:
    """Verify stub methods raise NotImplementedError as expected."""
    from agileplus_mcp.grpc_client import AgilePlusCoreClient

    client = AgilePlusCoreClient()
    with pytest.raises(NotImplementedError):
        await client.get_feature("test-slug")
