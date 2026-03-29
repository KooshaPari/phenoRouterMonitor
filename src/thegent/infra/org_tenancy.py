# @trace WL-051
"""WL-051: Org-level namespace hierarchy above ProjectTenancy.

Implements:
  - OrgNamespace: org_id, org_name, tenants list
  - OrgRegistry: singleton backed by ~/.thegent/orgs/registry.json
  - Module-level helpers: org_create, org_get, org_list, org_add_tenant
"""

from __future__ import annotations

import json
import uuid
from datetime import UTC, datetime
from pathlib import Path

from pydantic import BaseModel, ConfigDict, Field

_DEFAULT_ORG_REGISTRY_PATH = Path.home() / ".thegent" / "orgs" / "registry.json"


def _utc_now() -> str:
    return datetime.now(UTC).isoformat()


def _new_org_id() -> str:
    return "org_" + uuid.uuid4().hex[:12]


# ---------------------------------------------------------------------------
# Pydantic models
# ---------------------------------------------------------------------------


class OrgNamespace(BaseModel):
    """Org-level namespace that groups multiple tenant IDs (WL-051)."""

    model_config = ConfigDict(extra="forbid", frozen=True)

    org_id: str = Field(min_length=1)
    org_name: str = Field(min_length=1)
    tenants: list[str] = Field(default_factory=list)
    created_at: str = Field(min_length=1)
    updated_at: str = Field(min_length=1)


class OrgRegistryPayload(BaseModel):
    """On-disk registry payload schema."""

    model_config = ConfigDict(extra="forbid")

    orgs: list[OrgNamespace] = Field(default_factory=list)


# ---------------------------------------------------------------------------
# OrgRegistry
# ---------------------------------------------------------------------------


class OrgRegistry:
    """Manages org-level namespaces backed by a strict JSON registry (WL-051).

    Thread-safety: not guaranteed. The registry is a single JSON file; callers
    must serialise concurrent writes externally if needed.
    """

    def __init__(self, registry_path: Path | str = _DEFAULT_ORG_REGISTRY_PATH) -> None:
        self._registry_path = Path(registry_path).expanduser()

    # -- Create / read -------------------------------------------------------

    def create_org(
        self,
        *,
        org_name: str,
        org_id: str | None = None,
        initial_tenants: list[str] | None = None,
    ) -> OrgNamespace:
        """Create a new org namespace and persist it.

        Raises ValueError on duplicate org_id or org_name.
        """
        payload = self._load()
        effective_id = org_id or _new_org_id()

        for existing in payload.orgs:
            if existing.org_id == effective_id:
                raise ValueError(f"org_id conflict: {effective_id}")
            if existing.org_name == org_name.strip():
                raise ValueError(f"org_name conflict: {org_name!r}")

        now = _utc_now()
        org = OrgNamespace(
            org_id=effective_id,
            org_name=org_name.strip(),
            tenants=list(initial_tenants or []),
            created_at=now,
            updated_at=now,
        )
        payload.orgs.append(org)
        self._save(payload)
        return org

    def list_orgs(self) -> list[OrgNamespace]:
        """Return all org namespaces sorted by creation time."""
        payload = self._load()
        return sorted(payload.orgs, key=lambda o: o.created_at)

    def get_org(self, *, org_id: str | None = None, org_name: str | None = None) -> OrgNamespace:
        """Return an org by id or name.

        Raises KeyError if not found.  Raises ValueError if selector is ambiguous.
        """
        if org_id is None and org_name is None:
            raise ValueError("At least one selector required: org_id or org_name.")

        orgs = self._load().orgs
        matches = orgs

        if org_id is not None:
            matches = [o for o in matches if o.org_id == org_id]
        if org_name is not None:
            matches = [o for o in matches if o.org_name == org_name]

        if not matches:
            selector = org_id or org_name
            raise KeyError(f"OrgNamespace not found: {selector!r}")
        if len(matches) > 1:
            raise ValueError("OrgNamespace selector is ambiguous; use both org_id and org_name.")
        return matches[0]

    # -- Tenant management ---------------------------------------------------

    def add_tenant(self, org_id: str, tenant_id: str) -> OrgNamespace:
        """Add tenant_id to an org. Raises ValueError if already present."""
        payload = self._load()
        updated_orgs: list[OrgNamespace] = []
        found = False

        for org in payload.orgs:
            if org.org_id == org_id:
                if tenant_id in org.tenants:
                    raise ValueError(f"tenant_id {tenant_id!r} already in org {org_id!r}")
                new_tenants = list(org.tenants) + [tenant_id]
                org = OrgNamespace(
                    org_id=org.org_id,
                    org_name=org.org_name,
                    tenants=new_tenants,
                    created_at=org.created_at,
                    updated_at=_utc_now(),
                )
                found = True
            updated_orgs.append(org)

        if not found:
            raise KeyError(f"OrgNamespace not found: {org_id!r}")

        payload = OrgRegistryPayload(orgs=updated_orgs)
        self._save(payload)
        return self.get_org(org_id=org_id)

    def remove_tenant(self, org_id: str, tenant_id: str) -> OrgNamespace:
        """Remove tenant_id from an org. Raises ValueError if not present."""
        payload = self._load()
        updated_orgs: list[OrgNamespace] = []
        found = False

        for org in payload.orgs:
            if org.org_id == org_id:
                if tenant_id not in org.tenants:
                    raise ValueError(f"tenant_id {tenant_id!r} not in org {org_id!r}")
                new_tenants = [t for t in org.tenants if t != tenant_id]
                org = OrgNamespace(
                    org_id=org.org_id,
                    org_name=org.org_name,
                    tenants=new_tenants,
                    created_at=org.created_at,
                    updated_at=_utc_now(),
                )
                found = True
            updated_orgs.append(org)

        if not found:
            raise KeyError(f"OrgNamespace not found: {org_id!r}")

        payload = OrgRegistryPayload(orgs=updated_orgs)
        self._save(payload)
        return self.get_org(org_id=org_id)

    # -- Persistence ---------------------------------------------------------

    def _load(self) -> OrgRegistryPayload:
        if not self._registry_path.exists():
            return OrgRegistryPayload()
        raw = json.loads(self._registry_path.read_text(encoding="utf-8"))
        return OrgRegistryPayload.model_validate(raw)

    def _save(self, payload: OrgRegistryPayload) -> None:
        self._registry_path.parent.mkdir(parents=True, exist_ok=True)
        tmp = self._registry_path.with_suffix(".json.tmp")
        content = json.dumps(payload.model_dump(mode="json"), indent=2, sort_keys=True)
        tmp.write_text(f"{content}\n", encoding="utf-8")
        tmp.replace(self._registry_path)


# ---------------------------------------------------------------------------
# Module-level default instance + helpers
# ---------------------------------------------------------------------------

_DEFAULT_ORG_REGISTRY = OrgRegistry()


def org_create(
    *,
    org_name: str,
    org_id: str | None = None,
    initial_tenants: list[str] | None = None,
) -> OrgNamespace:
    return _DEFAULT_ORG_REGISTRY.create_org(
        org_name=org_name,
        org_id=org_id,
        initial_tenants=initial_tenants,
    )


def org_get(*, org_id: str | None = None, org_name: str | None = None) -> OrgNamespace:
    return _DEFAULT_ORG_REGISTRY.get_org(org_id=org_id, org_name=org_name)


def org_list() -> list[OrgNamespace]:
    return _DEFAULT_ORG_REGISTRY.list_orgs()


def org_add_tenant(org_id: str, tenant_id: str) -> OrgNamespace:
    return _DEFAULT_ORG_REGISTRY.add_tenant(org_id, tenant_id)
