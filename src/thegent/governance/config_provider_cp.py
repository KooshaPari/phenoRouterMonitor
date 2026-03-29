"""WP-10002: ControlPlaneConfigProvider implementation for Control Plane Phase 2."""

import logging
from typing import Any

import httpx

logger = logging.getLogger(__name__)


class ControlPlaneConfigProvider:
    """Connects to the long-running control-plane service for configuration.

    Applies full resolution order (request -> session -> tenant -> stamp -> global)
    server-side to ensure multi-tenant isolation.
    """

    def __init__(self, url: str, timeout: float = 2.0) -> None:
        self.url = url.rstrip("/")
        self.timeout = timeout

    def resolve(
        self,
        tenant_id: str | None = None,
        session_id: str | None = None,
        request_overrides: dict[str, Any] | None = None,
        keys: list[str] | None = None,
    ) -> dict[str, Any]:
        """Resolve config via Control Plane API."""
        try:
            with httpx.Client(timeout=self.timeout) as client:
                response = self._post_resolve(client, tenant_id, session_id, request_overrides, keys)
                if response.status_code == 200:
                    return response.json()

                logger.error(f"CP resolution failed: {response.status_code} {response.text}")
        except Exception as e:
            logger.error(f"CP connection error: {e}")

        # Fallback to local env if CP is down (Circuit Breaker logic would go here)
        from thegent.governance.config_provider import EnvConfigProvider

        return EnvConfigProvider().resolve(tenant_id, session_id, request_overrides, keys)

    def _post_resolve(
        self,
        client: httpx.Client,
        tenant_id: str | None,
        session_id: str | None,
        overrides: dict[str, Any] | None,
        keys: list[str] | None,
    ) -> httpx.Response:
        """Helper for resolution request."""
        payload = {
            "tenant_id": tenant_id,
            "session_id": session_id,
            "overrides": overrides,
            "keys": keys,
        }
        return client.post(f"{self.url}/v1/config/resolve", json=payload)

    def get_tenant_config(self, tenant_id: str) -> dict[str, Any] | None:
        """Fetch raw tenant config via CP API."""
        try:
            with httpx.Client(timeout=self.timeout) as client:
                response = client.get(f"{self.url}/v1/tenants/{tenant_id}/config")
                if response.status_code == 200:
                    return response.json()
        except Exception as e:
            logger.error(f"CP connection error (tenant_config): {e}")
        return None
