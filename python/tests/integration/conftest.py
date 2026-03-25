"""Shared fixtures and collection policy for integration tests."""

from __future__ import annotations

import os

import pytest

SKIP_REASON = "AGILEPLUS_GRPC_URL not set; skipped outside Docker Compose environment"


def pytest_collection_modifyitems(items: list[pytest.Item]) -> None:
    """Skip integration tests unless the gRPC server is available."""
    if os.environ.get("AGILEPLUS_GRPC_URL"):
        return

    skip_marker = pytest.mark.skip(reason=SKIP_REASON)
    for item in items:
        item.add_marker(skip_marker)


@pytest.fixture
async def client():
    """Provide a connected AgilePlus gRPC client."""
    from agileplus_mcp.grpc_client import connect_client

    address = os.environ.get("AGILEPLUS_GRPC_URL", "localhost:50051")
    async with connect_client(address) as c:
        yield c
