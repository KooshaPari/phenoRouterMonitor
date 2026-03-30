"""gRPC client for communication with Rust AgilePlus core."""

from __future__ import annotations

import logging
from dataclasses import dataclass, field

import grpc

logger = logging.getLogger(__name__)

DEFAULT_HOST = "localhost"
DEFAULT_PORT = 50051


@dataclass
class AgilePlusCoreClient:
    """Client for the AgilePlus Rust core gRPC service."""

    host: str = DEFAULT_HOST
    port: int = DEFAULT_PORT
    _channel: grpc.Channel | None = field(default=None, init=False, repr=False)

    @property
    def target(self) -> str:
        """Return the gRPC target address as host:port."""
        return f"{self.host}:{self.port}"

    def connect(self) -> None:
        """Establish gRPC channel to the Rust core."""
        logger.info("Connecting to AgilePlus core at %s", self.target)
        self._channel = grpc.insecure_channel(self.target)

    def close(self) -> None:
        """Close the gRPC channel."""
        if self._channel is not None:
            self._channel.close()
            self._channel = None

    async def get_feature(self, slug: str) -> dict:  # type: ignore[empty-body]
        """Stub: Get feature by slug via gRPC.

        Args:
            slug: The kebab-case feature slug (e.g., '001-spec-engine')

        Returns:
            Feature details including state, spec hash, and timestamps.

        Raises:
            NotImplementedError: Until gRPC stubs are generated from proto in WP14.
        """
        raise NotImplementedError("gRPC stubs not yet generated")

    async def list_features(self) -> list[dict]:  # type: ignore[empty-body]
        """Stub: List all features via gRPC.

        Returns:
            List of feature summaries.

        Raises:
            NotImplementedError: Until gRPC stubs are generated from proto in WP14.
        """
        raise NotImplementedError("gRPC stubs not yet generated")

    async def check_governance(self, feature_id: int) -> dict:  # type: ignore[empty-body]
        """Stub: Check governance status via gRPC.

        Args:
            feature_id: The numeric feature ID.

        Returns:
            Governance check result with pass/fail and violations.

        Raises:
            NotImplementedError: Until gRPC stubs are generated from proto in WP14.
        """
        raise NotImplementedError("gRPC stubs not yet generated")

    async def get_audit_trail(self, feature_id: int) -> list[dict]:  # type: ignore[empty-body]
        """Stub: Get audit trail via gRPC.

        Args:
            feature_id: The numeric feature ID.

        Returns:
            List of audit entries with hash chain integrity status.

        Raises:
            NotImplementedError: Until gRPC stubs are generated from proto in WP14.
        """
        raise NotImplementedError("gRPC stubs not yet generated")
