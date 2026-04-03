"""Shared test fixtures for AgilePlus MCP tests."""

import pytest

from agileplus_mcp.grpc_client import AgilePlusCoreClient


@pytest.fixture
def grpc_client() -> AgilePlusCoreClient:
    """Create a gRPC client for testing (not connected)."""
    return AgilePlusCoreClient(host="localhost", port=50051)
