"""AgilePlus gRPC client — connects to the Rust agileplus-core server.

Traceability: WP14-T081

Uses grpcio (not grpclib) for compatibility with FastMCP's async model.
Generated stubs from agileplus_proto are imported conditionally so the module
can be imported even before buf generate has run (e.g. during unit tests with
mocks).
"""

from __future__ import annotations

import asyncio
import logging
from collections.abc import AsyncIterator
from contextlib import asynccontextmanager
from typing import Any

logger = logging.getLogger(__name__)


class GrpcConnectionError(Exception):
    """Raised when the gRPC channel cannot be established."""


class GrpcCallError(Exception):
    """Raised when a gRPC call fails with a non-retryable status."""

    def __init__(self, code: Any, message: str) -> None:
        self.code = code
        super().__init__(f"gRPC error {code}: {message}")


class AgilePlusCoreClient:
    """Async gRPC client for AgilePlusCoreService.

    Wraps the generated grpcio stubs with a Pythonic async interface.
    Connection management, retries, and error mapping are handled here.
    """

    def __init__(self, address: str = "localhost:50051") -> None:
        self._address = address
        self._channel: Any | None = None
        self._stub: Any | None = None
        self._max_retries = 3
        self._retry_delay = 0.5  # seconds, doubles on each attempt

    # ------------------------------------------------------------------
    # Connection lifecycle
    # ------------------------------------------------------------------

    async def connect(self) -> None:
        """Open the gRPC channel and create the service stub."""
        try:
            import grpc
            from agileplus_proto.gen.agileplus.v1 import core_pb2_grpc  # type: ignore[import]

            self._channel = grpc.aio.insecure_channel(self._address)
            await asyncio.wait_for(self._channel.channel_ready(), timeout=5.0)
            self._stub = core_pb2_grpc.AgilePlusCoreServiceStub(self._channel)
            logger.info("Connected to AgilePlus gRPC server at %s", self._address)
        except (ImportError, ModuleNotFoundError) as exc:
            raise GrpcConnectionError(
                f"agileplus_proto stubs not found — run `buf generate` first: {exc}"
            ) from exc
        except Exception as exc:
            raise GrpcConnectionError(
                f"Failed to connect to {self._address}: {exc}"
            ) from exc

    async def close(self) -> None:
        """Close the gRPC channel gracefully."""
        if self._channel is not None:
            await self._channel.close()
            self._channel = None
            self._stub = None
            logger.info("gRPC channel closed")

    def _require_stub(self) -> Any:
        if self._stub is None:
            raise GrpcConnectionError("Not connected — call connect() first")
        return self._stub

    # ------------------------------------------------------------------
    # Retry helper
    # ------------------------------------------------------------------

    async def _call_with_retry(self, coro_factory: Any) -> Any:
        """Execute a gRPC call, retrying transient errors with backoff."""
        try:
            import grpc
        except ImportError:
            import grpc  # type: ignore

        delay = self._retry_delay
        last_exc: Exception | None = None

        for attempt in range(self._max_retries):
            try:
                return await coro_factory()
            except grpc.aio.AioRpcError as exc:
                code = exc.code()
                # UNAVAILABLE and DEADLINE_EXCEEDED are transient
                if code in (grpc.StatusCode.UNAVAILABLE, grpc.StatusCode.DEADLINE_EXCEEDED):
                    logger.warning(
                        "Transient gRPC error (attempt %d/%d): %s",
                        attempt + 1,
                        self._max_retries,
                        exc.details(),
                    )
                    last_exc = exc
                    await asyncio.sleep(delay)
                    delay *= 2
                else:
                    raise GrpcCallError(code, exc.details()) from exc

        raise GrpcConnectionError(
            f"gRPC call failed after {self._max_retries} retries"
        ) from last_exc

    # ------------------------------------------------------------------
    # Feature RPCs
    # ------------------------------------------------------------------

    async def get_feature(self, slug: str) -> dict[str, Any]:
        """Retrieve a single feature by slug."""
        from agileplus_proto.gen.agileplus.v1 import core_pb2  # type: ignore[import]

        stub = self._require_stub()
        request = core_pb2.GetFeatureRequest(slug=slug)
        response = await self._call_with_retry(lambda: stub.GetFeature(request))
        return self._feature_to_dict(response.feature)

    async def list_features(self, state: str | None = None) -> list[dict[str, Any]]:
        """List all features, optionally filtered by state."""
        from agileplus_proto.gen.agileplus.v1 import core_pb2  # type: ignore[import]

        stub = self._require_stub()
        request = core_pb2.ListFeaturesRequest(state_filter=state or "")
        response = await self._call_with_retry(lambda: stub.ListFeatures(request))
        return [self._feature_to_dict(f) for f in response.features]

    async def get_feature_state(self, slug: str) -> dict[str, Any]:
        """Get state and next suggested command for a feature."""
        from agileplus_proto.gen.agileplus.v1 import core_pb2  # type: ignore[import]

        stub = self._require_stub()
        request = core_pb2.GetFeatureStateRequest(slug=slug)
        response = await self._call_with_retry(lambda: stub.GetFeatureState(request))
        fs = response.feature_state
        return {
            "state": fs.state,
            "next_command": fs.next_command,
            "blockers": list(fs.blockers),
        }

    # ------------------------------------------------------------------
    # Work Package RPCs
    # ------------------------------------------------------------------

    async def list_work_packages(
        self, feature_slug: str, state: str | None = None
    ) -> list[dict[str, Any]]:
        """List work packages for a feature."""
        from agileplus_proto.gen.agileplus.v1 import core_pb2  # type: ignore[import]

        stub = self._require_stub()
        request = core_pb2.ListWorkPackagesRequest(
            feature_slug=feature_slug, state_filter=state or ""
        )
        response = await self._call_with_retry(lambda: stub.ListWorkPackages(request))
        return [self._wp_to_dict(wp) for wp in response.packages]

    async def get_work_package_status(
        self, feature_slug: str, wp_sequence: int
    ) -> dict[str, Any]:
        """Get status of a specific work package."""
        from agileplus_proto.gen.agileplus.v1 import core_pb2  # type: ignore[import]

        stub = self._require_stub()
        request = core_pb2.GetWorkPackageStatusRequest(
            feature_slug=feature_slug, wp_sequence=wp_sequence
        )
        response = await self._call_with_retry(
            lambda: stub.GetWorkPackageStatus(request)
        )
        return self._wp_to_dict(response.work_package_status)

    # ------------------------------------------------------------------
    # Governance RPCs
    # ------------------------------------------------------------------

    async def check_governance_gate(
        self, feature_slug: str, transition: str
    ) -> dict[str, Any]:
        """Check whether a governance gate passes for a state transition."""
        from agileplus_proto.gen.agileplus.v1 import core_pb2  # type: ignore[import]

        stub = self._require_stub()
        request = core_pb2.CheckGovernanceGateRequest(
            feature_slug=feature_slug, transition=transition
        )
        response = await self._call_with_retry(lambda: stub.CheckGovernanceGate(request))
        return {
            "passed": response.passed,
            "violations": [
                {
                    "fr_id": v.fr_id,
                    "rule_id": v.rule_id,
                    "message": v.message,
                    "remediation": v.remediation,
                }
                for v in response.violations
            ],
        }

    async def get_audit_trail(self, feature_slug: str, after_id: int = 0) -> list[dict[str, Any]]:
        """Retrieve audit trail entries for a feature."""
        from agileplus_proto.gen.agileplus.v1 import core_pb2  # type: ignore[import]

        stub = self._require_stub()
        request = core_pb2.GetAuditTrailRequest(
            feature_slug=feature_slug, after_id=after_id
        )
        entries = []
        async for response in stub.GetAuditTrail(request):
            entries.append(self._audit_entry_to_dict(response.audit_entry))
        return entries

    async def verify_audit_chain(self, feature_slug: str) -> dict[str, Any]:
        """Verify the integrity of the audit hash chain."""
        from agileplus_proto.gen.agileplus.v1 import core_pb2  # type: ignore[import]

        stub = self._require_stub()
        request = core_pb2.VerifyAuditChainRequest(feature_slug=feature_slug)
        response = await self._call_with_retry(lambda: stub.VerifyAuditChain(request))
        return {
            "valid": response.valid,
            "entries_verified": response.entries_verified,
            "first_invalid_id": response.first_invalid_id,
            "error_message": response.error_message,
        }

    # ------------------------------------------------------------------
    # Command dispatch RPC
    # ------------------------------------------------------------------

    async def run_command(self, command: str, feature_slug: str = "", **kwargs: Any) -> dict[str, Any]:
        """Dispatch a named command to the Rust core.

        Args:
            command: One of specify, research, plan, implement, validate, ship,
                     retrospective.
            feature_slug: Target feature slug.
            **kwargs: Additional string-valued arguments passed in the args map.
        """
        from agileplus_proto.gen.agileplus.v1 import common_pb2, core_pb2  # type: ignore[import]

        stub = self._require_stub()
        cmd_request = common_pb2.CommandRequest(
            command=command,
            feature_slug=feature_slug,
            args={k: str(v) for k, v in kwargs.items()},
        )
        request = core_pb2.DispatchCommandRequest(command=cmd_request)
        response = await self._call_with_retry(lambda: stub.DispatchCommand(request))
        result = response.result
        return {
            "success": result.success,
            "message": result.message,
            "outputs": dict(result.outputs),
        }

    # ------------------------------------------------------------------
    # Streaming RPC
    # ------------------------------------------------------------------

    async def stream_agent_events(self, feature_slug: str) -> AsyncIterator[dict[str, Any]]:
        """Stream agent status events from the Rust core.

        Yields dicts with keys: event_type, feature_slug, wp_sequence,
        agent_id, payload, timestamp.

        Reconnects automatically on UNAVAILABLE errors.
        """
        from agileplus_proto.gen.agileplus.v1 import core_pb2  # type: ignore[import]
        import grpc

        stub = self._require_stub()
        request = core_pb2.StreamAgentEventsRequest(feature_slug=feature_slug)

        while True:
            try:
                async for event in stub.StreamAgentEvents(request):
                    yield {
                        "event_type": event.event_type,
                        "feature_slug": event.feature_slug,
                        "wp_sequence": event.wp_sequence,
                        "agent_id": event.agent_id,
                        "payload": event.payload,
                        "timestamp": event.timestamp,
                    }
                break  # Stream ended cleanly
            except grpc.aio.AioRpcError as exc:
                if exc.code() == grpc.StatusCode.UNAVAILABLE:
                    logger.warning("Stream disconnected, reconnecting in 2s...")
                    await asyncio.sleep(2.0)
                    # Re-acquire stub after reconnect attempt
                    try:
                        await self.connect()
                        stub = self._require_stub()
                    except GrpcConnectionError:
                        logger.error("Reconnect failed, giving up")
                        return
                else:
                    raise GrpcCallError(exc.code(), exc.details()) from exc

    # ------------------------------------------------------------------
    # Serialization helpers
    # ------------------------------------------------------------------

    @staticmethod
    def _feature_to_dict(f: Any) -> dict[str, Any]:
        return {
            "id": f.id,
            "slug": f.slug,
            "friendly_name": f.friendly_name,
            "state": f.state,
            "target_branch": f.target_branch,
            "created_at": f.created_at,
            "updated_at": f.updated_at,
            "wp_count": f.wp_count,
            "wp_done": f.wp_done,
        }

    @staticmethod
    def _wp_to_dict(wp: Any) -> dict[str, Any]:
        return {
            "id": wp.id,
            "title": wp.title,
            "state": wp.state,
            "sequence": wp.sequence,
            "agent_id": wp.agent_id,
            "pr_url": wp.pr_url,
            "pr_state": wp.pr_state,
            "depends_on": list(wp.depends_on),
            "file_scope": list(wp.file_scope),
        }

    @staticmethod
    def _audit_entry_to_dict(e: Any) -> dict[str, Any]:
        return {
            "id": e.id,
            "feature_slug": e.feature_slug,
            "wp_sequence": e.wp_sequence,
            "timestamp": e.timestamp,
            "actor": e.actor,
            "transition": e.transition,
            "evidence_refs": list(e.evidence_refs),
            "prev_hash": bytes(e.prev_hash).hex(),
            "hash": bytes(e.hash).hex(),
        }


@asynccontextmanager
async def connect_client(
    address: str = "localhost:50051",
) -> AsyncIterator[AgilePlusCoreClient]:
    """Async context manager for a connected gRPC client.

    Usage::

        async with connect_client() as client:
            feature = await client.get_feature("my-feature")
    """
    client = AgilePlusCoreClient(address)
    await client.connect()
    try:
        yield client
    finally:
        await client.close()
