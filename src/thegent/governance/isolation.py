"""Tenant isolation and data leakage protection."""

from typing import Any


class AccessDenied(Exception):
    """Exception raised when cross-tenant access is attempted."""


class TenantSession:
    """Represents an isolated agent session for a tenant."""

    def __init__(self, tenant_id: str, session_id: str, provider: "TenantIsolationProvider") -> None:
        self.tenant_id = tenant_id
        self.session_id = session_id
        self.provider = provider

    def emit_telemetry(self, data: dict[str, Any]) -> None:
        """Emit telemetry for this session."""
        self.provider.record_telemetry(self.tenant_id, self.session_id, data)


class TenantIsolationProvider:
    """Provides isolation between federated namespaces."""

    def __init__(self) -> None:
        # tenant_id -> session_id -> list of telemetry
        self._telemetry_store: dict[str, dict[str, list[dict[str, Any]]]] = {}

    def create_session(self, tenant_id: str, session_id: str) -> TenantSession:
        """Create a new isolated session."""
        return TenantSession(tenant_id, session_id, self)

    def record_telemetry(self, tenant_id: str, session_id: str, data: dict[str, Any]) -> None:
        """Record telemetry for a tenant's session."""
        if tenant_id not in self._telemetry_store:
            self._telemetry_store[tenant_id] = {}
        if session_id not in self._telemetry_store[tenant_id]:
            self._telemetry_store[tenant_id][session_id] = []
        self._telemetry_store[tenant_id][session_id].append(data)

    def get_session_telemetry(self, tenant_id: str, session_id: str) -> list[dict[str, Any]]:
        """Get telemetry for a session, enforcing isolation."""
        # Check if session exists in any tenant first to detect cross-tenant access
        found_in_tenant = None
        for t_id, sessions in self._telemetry_store.items():
            if session_id in sessions:
                found_in_tenant = t_id
                break

        if found_in_tenant and found_in_tenant != tenant_id:
            raise AccessDenied(f"Access to session {session_id} denied for tenant {tenant_id}")

        if not found_in_tenant:
            return []

        return self._telemetry_store[tenant_id][session_id]
