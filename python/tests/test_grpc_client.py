"""Unit tests for AgilePlusCoreClient with mock gRPC stubs.

Traceability: WP14-T081
"""

from __future__ import annotations

import asyncio
from unittest.mock import AsyncMock, MagicMock, patch
import pytest

from agileplus_mcp.grpc_client import AgilePlusCoreClient, GrpcConnectionError


# ---------------------------------------------------------------------------
# Fixtures
# ---------------------------------------------------------------------------

def _make_mock_feature(slug: str = "test-feat") -> MagicMock:
    f = MagicMock()
    f.id = 1
    f.slug = slug
    f.friendly_name = "Test Feature"
    f.state = "created"
    f.target_branch = "main"
    f.created_at = "2026-01-01T00:00:00Z"
    f.updated_at = "2026-01-01T00:00:00Z"
    f.wp_count = 0
    f.wp_done = 0
    return f


def _make_mock_stub() -> MagicMock:
    stub = MagicMock()
    # Feature RPCs
    feature_response = MagicMock()
    feature_response.feature = _make_mock_feature()
    stub.GetFeature = AsyncMock(return_value=feature_response)

    list_response = MagicMock()
    list_response.features = [_make_mock_feature("feat-a"), _make_mock_feature("feat-b")]
    stub.ListFeatures = AsyncMock(return_value=list_response)

    state_response = MagicMock()
    state_response.feature_state = MagicMock(
        state="created", next_command="specify", blockers=[]
    )
    stub.GetFeatureState = AsyncMock(return_value=state_response)

    # Command dispatch
    cmd_response = MagicMock()
    cmd_response.result = MagicMock(success=True, message="ok", outputs={})
    stub.DispatchCommand = AsyncMock(return_value=cmd_response)

    # Governance
    gate_response = MagicMock()
    gate_response.passed = True
    gate_response.violations = []
    stub.CheckGovernanceGate = AsyncMock(return_value=gate_response)

    # Audit
    verify_response = MagicMock()
    verify_response.valid = True
    verify_response.entries_verified = 3
    verify_response.first_invalid_id = ""
    verify_response.error_message = ""
    stub.VerifyAuditChain = AsyncMock(return_value=verify_response)

    return stub


# ---------------------------------------------------------------------------
# Tests
# ---------------------------------------------------------------------------

@pytest.fixture
def client_with_stub():
    client = AgilePlusCoreClient("localhost:50051")
    client._stub = _make_mock_stub()
    return client


@pytest.mark.asyncio
async def test_get_feature_returns_dict(client_with_stub):
    result = await client_with_stub.get_feature("test-feat")
    assert result["slug"] == "test-feat"
    assert result["friendly_name"] == "Test Feature"
    assert result["state"] == "created"


@pytest.mark.asyncio
async def test_list_features_returns_list(client_with_stub):
    results = await client_with_stub.list_features()
    assert len(results) == 2
    slugs = {r["slug"] for r in results}
    assert "feat-a" in slugs
    assert "feat-b" in slugs


@pytest.mark.asyncio
async def test_list_features_with_state_filter(client_with_stub):
    # Just verify the request is built with the filter
    results = await client_with_stub.list_features(state="created")
    assert isinstance(results, list)


@pytest.mark.asyncio
async def test_get_feature_state(client_with_stub):
    result = await client_with_stub.get_feature_state("test-feat")
    assert result["state"] == "created"
    assert result["next_command"] == "specify"


@pytest.mark.asyncio
async def test_run_command_returns_dict(client_with_stub):
    result = await client_with_stub.run_command("specify", feature_slug="test-feat")
    assert result["success"] is True
    assert result["message"] == "ok"


@pytest.mark.asyncio
async def test_check_governance_gate_passed(client_with_stub):
    result = await client_with_stub.check_governance_gate(
        "test-feat", "specified->planned"
    )
    assert result["passed"] is True
    assert result["violations"] == []


@pytest.mark.asyncio
async def test_verify_audit_chain(client_with_stub):
    result = await client_with_stub.verify_audit_chain("test-feat")
    assert result["valid"] is True
    assert result["entries_verified"] == 3


@pytest.mark.asyncio
async def test_require_stub_raises_when_not_connected():
    client = AgilePlusCoreClient("localhost:50051")
    with pytest.raises(GrpcConnectionError, match="Not connected"):
        client._require_stub()


@pytest.mark.asyncio
async def test_retry_on_unavailable(client_with_stub):
    """Verify that transient UNAVAILABLE errors are retried."""
    try:
        import grpc
    except ImportError:
        pytest.skip("grpcio not installed")

    call_count = 0
    original = client_with_stub._stub.GetFeature

    async def flaky(*args, **kwargs):
        nonlocal call_count
        call_count += 1
        if call_count < 2:
            exc = grpc.aio.AioRpcError(grpc.StatusCode.UNAVAILABLE, "try again")
            raise exc
        return await original(*args, **kwargs)

    client_with_stub._stub.GetFeature = flaky
    client_with_stub._retry_delay = 0.01  # Speed up test
    result = await client_with_stub.get_feature("test-feat")
    assert result["slug"] == "test-feat"
    assert call_count == 2
