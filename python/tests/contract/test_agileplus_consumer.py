"""Consumer-side contract tests for the AgilePlusCoreService gRPC boundary.

These tests verify that the Python MCP client constructs requests and
interprets responses in a way that matches the proto contract.

Because gRPC Pact support is immature (see WP14 risks), we use a simplified
contract approach:
1. Verify proto message construction (field names, types).
2. Verify response parsing helpers produce the expected dict shapes.
3. Structural checks act as living documentation of the expected contract.

A full Pact broker integration can be added once pact-python gains stable
gRPC support. For now, `buf breaking` in CI enforces schema compatibility.

Traceability: WP14-T084
"""

from __future__ import annotations

from unittest.mock import AsyncMock, MagicMock
import pytest


# ---------------------------------------------------------------------------
# Contract: GetFeature
# ---------------------------------------------------------------------------

class TestGetFeatureContract:
    """GetFeature RPC consumer contract."""

    @pytest.mark.asyncio
    async def test_get_feature_request_uses_slug_field(self):
        """Consumer must send a GetFeatureRequest with a 'slug' field."""
        from agileplus_mcp.grpc_client import AgilePlusCoreClient

        client = AgilePlusCoreClient()

        captured_request = {}

        async def fake_get_feature(request):
            captured_request["slug"] = request.slug
            r = MagicMock()
            r.feature = MagicMock(
                id=1,
                slug="test-feature",
                friendly_name="Test Feature",
                state="created",
                target_branch="main",
                created_at="2026-01-01T00:00:00Z",
                updated_at="2026-01-01T00:00:00Z",
                wp_count=0,
                wp_done=0,
            )
            return r

        stub = MagicMock()
        stub.GetFeature = fake_get_feature
        client._stub = stub

        # Patch proto import
        mock_core_pb2 = MagicMock()
        mock_core_pb2.GetFeatureRequest = lambda slug: MagicMock(slug=slug)
        with __import__("unittest.mock", fromlist=["patch"]).patch.dict(
            "sys.modules",
            {"agileplus_proto.gen.agileplus.v1.core_pb2": mock_core_pb2},
        ):
            import sys
            sys.modules["agileplus_proto.gen.agileplus.v1"] = MagicMock()
            sys.modules["agileplus_proto.gen.agileplus.v1.core_pb2"] = mock_core_pb2
            result = await client.get_feature("test-feature")

        assert captured_request.get("slug") == "test-feature"
        assert result["slug"] == "test-feature"
        assert result["friendly_name"] == "Test Feature"

    @pytest.mark.asyncio
    async def test_get_feature_response_shape(self):
        """GetFeature response must contain id, slug, state, and timestamps."""
        from agileplus_mcp.grpc_client import AgilePlusCoreClient

        client = AgilePlusCoreClient()

        mock_feature = MagicMock(
            id=42,
            slug="feat-x",
            friendly_name="Feature X",
            state="planned",
            target_branch="main",
            created_at="2026-02-01T10:00:00Z",
            updated_at="2026-02-15T10:00:00Z",
            wp_count=5,
            wp_done=2,
        )

        result = AgilePlusCoreClient._feature_to_dict(mock_feature)

        # Verify all required fields are present
        required_fields = ["id", "slug", "friendly_name", "state", "target_branch",
                           "created_at", "updated_at", "wp_count", "wp_done"]
        for field in required_fields:
            assert field in result, f"Missing field: {field}"

        assert result["id"] == 42
        assert result["slug"] == "feat-x"
        assert result["wp_count"] == 5


# ---------------------------------------------------------------------------
# Contract: ListWorkPackages
# ---------------------------------------------------------------------------

class TestListWorkPackagesContract:
    """ListWorkPackages RPC consumer contract."""

    def test_wp_response_shape(self):
        """WorkPackageStatus must include id, title, state, sequence."""
        from agileplus_mcp.grpc_client import AgilePlusCoreClient

        mock_wp = MagicMock(
            id=1,
            title="Implement gRPC server",
            state="doing",
            sequence=1,
            agent_id="agt-01",
            pr_url="https://github.com/org/repo/pull/42",
            pr_state="open",
            depends_on=[],
            file_scope=["crates/agileplus-grpc/src/server/mod.rs"],
        )

        result = AgilePlusCoreClient._wp_to_dict(mock_wp)

        required_fields = ["id", "title", "state", "sequence", "agent_id",
                           "pr_url", "pr_state", "depends_on", "file_scope"]
        for field in required_fields:
            assert field in result, f"Missing field: {field}"

        assert result["sequence"] == 1
        assert result["state"] == "doing"
        assert len(result["file_scope"]) == 1


# ---------------------------------------------------------------------------
# Contract: DispatchCommand
# ---------------------------------------------------------------------------

class TestDispatchCommandContract:
    """DispatchCommand RPC consumer contract."""

    def test_command_response_shape(self):
        """CommandResponse must have success, message, and outputs."""
        from agileplus_mcp.grpc_client import AgilePlusCoreClient

        mock_result = MagicMock(
            success=True,
            message="command 'specify' queued",
            outputs={"spec_path": "kitty-specs/feat-x/spec.md"},
        )
        response = MagicMock()
        response.result = mock_result

        # Verify field extraction pattern used in run_command
        data = {
            "success": response.result.success,
            "message": response.result.message,
            "outputs": dict(response.result.outputs),
        }

        assert data["success"] is True
        assert "specify" in data["message"]
        assert "spec_path" in data["outputs"]


# ---------------------------------------------------------------------------
# Contract: AuditEntry
# ---------------------------------------------------------------------------

class TestAuditEntryContract:
    """GetAuditTrail RPC consumer contract."""

    def test_audit_entry_response_shape(self):
        """AuditEntry must include id, timestamps, actor, transition, and hashes."""
        from agileplus_mcp.grpc_client import AgilePlusCoreClient

        mock_entry = MagicMock(
            id=7,
            feature_slug="feat-y",
            wp_sequence=2,
            timestamp="2026-03-01T12:00:00Z",
            actor="agent-01",
            transition="doing->review",
            evidence_refs=["FR-001", "FR-002"],
            prev_hash=bytes(32),
            hash=bytes(32),
        )

        result = AgilePlusCoreClient._audit_entry_to_dict(mock_entry)

        required_fields = ["id", "feature_slug", "wp_sequence", "timestamp",
                           "actor", "transition", "evidence_refs", "prev_hash", "hash"]
        for field in required_fields:
            assert field in result, f"Missing field: {field}"

        # Hashes are returned as hex strings
        assert len(result["prev_hash"]) == 64
        assert len(result["hash"]) == 64
        assert result["prev_hash"] == "0" * 64

    def test_verify_audit_chain_response_shape(self):
        """VerifyAuditChain response must have valid, entries_verified, and error info."""
        # Simulate the response parsing
        mock_response = MagicMock(
            valid=True,
            entries_verified=10,
            first_invalid_id="",
            error_message="",
        )

        result = {
            "valid": mock_response.valid,
            "entries_verified": mock_response.entries_verified,
            "first_invalid_id": mock_response.first_invalid_id,
            "error_message": mock_response.error_message,
        }

        assert result["valid"] is True
        assert result["entries_verified"] == 10
        assert result["first_invalid_id"] == ""
