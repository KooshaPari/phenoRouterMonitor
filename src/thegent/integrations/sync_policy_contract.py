"""Sync policy file contract.

Defines and validates sync policy contracts that govern connector behavior
during synchronization operations.

# @trace WL-197
"""

from __future__ import annotations

import logging
import os
from dataclasses import dataclass
from pathlib import Path

from thegent.infra.fast_yaml_parser import yaml_load

logger = logging.getLogger(__name__)


@dataclass
class ConnectorPolicy:
    """Connector-specific sync policy controls."""

    enabled: bool
    mode: str
    direction: str
    quota_daily: int
    board_id: str = ""


@dataclass
class TenantProject:
    """Tenant-aware project mapping."""

    root: str
    tenant_id: str
    autosync_enabled: bool = True


@dataclass
class TenantConfig:
    """Top-level tenancy configuration."""

    mode: str
    default_tenant: str
    projects: list[TenantProject]


@dataclass
class SyncPolicyContract:
    """Structured sync policy contract.

    Can be instantiated in two ways:
    1. Full mode with schema_version, connectors, tenancy (from YAML file)
    2. Simple mode with version, allowed_connectors, max_batch_size, dry_run (WL-197)
    """

    schema_version: str = ""
    conflict_precedence: str = ""
    strict_mode: bool = False
    connectors: dict[str, ConnectorPolicy] | None = None
    tenancy: TenantConfig | None = None

    # WL-197 simple mode fields
    version: str = ""
    allowed_connectors: list[str] | None = None
    max_batch_size: int = 100
    dry_run: bool = False


@dataclass
class SimplePolicy(SyncPolicyContract):
    """Simple policy contract for WL-197 (backward compat wrapper)."""

    def __init__(
        self,
        version: str,
        allowed_connectors: list[str],
        max_batch_size: int = 100,
        dry_run: bool = False,
    ) -> None:
        """Initialize simple policy."""
        super().__init__(
            version=version,
            allowed_connectors=allowed_connectors,
            max_batch_size=max_batch_size,
            dry_run=dry_run,
        )


class SyncPolicyValidator:
    """Validates sync policy contracts against defined rules (WL-197)."""

    def validate(self, policy: SyncPolicyContract) -> list[str]:
        """Validate a sync policy contract.

        Checks:
        - version is non-empty
        - allowed_connectors is non-empty
        - max_batch_size > 0

        Args:
            policy: The SyncPolicyContract to validate.

        Returns:
            List of validation error messages. Empty list means validation passed.
        """
        errors = []

        # Validate version (use simple mode version if available)
        version = policy.version or policy.schema_version
        if not version or not version.strip():
            errors.append("version must be non-empty")

        # Validate allowed_connectors (simple mode) or connectors (full mode)
        connectors = policy.allowed_connectors or (list(policy.connectors.keys()) if policy.connectors else [])
        if not connectors:
            errors.append("allowed_connectors must be non-empty")

        # Validate max_batch_size
        if policy.max_batch_size <= 0:
            errors.append("max_batch_size must be > 0")

        if errors:
            logger.warning(f"Validation failed for policy: {', '.join(errors)}")
        else:
            logger.debug(f"Policy validation passed: version={version}")

        return errors


def resolve_sync_policy_path(*, project_root: Path | None = None) -> Path:
    """Resolve the policy path from env override or project root."""
    root = project_root or Path.cwd()
    override = os.getenv("THGENT_SYNC_POLICY_PATH", "").strip()
    if override:
        return Path(override).expanduser().resolve()
    return (root / ".thegent" / "sync-policy.yaml").resolve()


def _parse_connectors(raw: object) -> dict[str, ConnectorPolicy]:
    if not isinstance(raw, dict) or not raw:
        raise ValueError("connectors must be a non-empty mapping")

    connectors: dict[str, ConnectorPolicy] = {}
    for connector, attrs in raw.items():
        if not isinstance(connector, str) or not connector.strip():
            raise ValueError("connector names must be non-empty strings")
        if not isinstance(attrs, dict):
            raise ValueError(f"connector '{connector}' config must be a mapping")
        try:
            connectors[connector.strip()] = ConnectorPolicy(
                enabled=bool(attrs["enabled"]),
                mode=str(attrs["mode"]).strip(),
                direction=str(attrs["direction"]).strip(),
                quota_daily=int(attrs["quota_daily"]),
                board_id=str(attrs.get("board_id", "")).strip(),
            )
        except KeyError as err:
            raise ValueError(f"connector '{connector}' missing required field: {err}") from err
        except (TypeError, ValueError) as err:
            raise ValueError(f"connector '{connector}' has invalid field types") from err

    return connectors


def _parse_tenancy(raw: object) -> TenantConfig:
    if not isinstance(raw, dict):
        raise ValueError("tenancy must be a mapping")
    mode = str(raw.get("mode", "")).strip()
    if not mode:
        raise ValueError("tenancy.mode must be set")
    default_tenant = str(raw.get("default_tenant", "")).strip()
    if not default_tenant:
        raise ValueError("tenancy.default_tenant must be set")
    projects_raw = raw.get("projects", [])
    if not isinstance(projects_raw, list):
        raise ValueError("tenancy.projects must be a list")
    projects: list[TenantProject] = []
    seen_roots: set[str] = set()
    for idx, raw_project in enumerate(projects_raw):
        if not isinstance(raw_project, dict):
            raise ValueError(f"tenancy.projects[{idx}] must be a mapping")
        root = str(raw_project.get("root", "")).strip()
        tenant_id = str(raw_project.get("tenant_id", "")).strip()
        autosync_enabled = bool(raw_project.get("autosync_enabled", False))
        if not root or not tenant_id:
            raise ValueError("tenancy.projects entries require root and tenant_id")
        if root in seen_roots:
            raise ValueError(f"duplicate tenancy project root: {root}")
        seen_roots.add(root)
        projects.append(TenantProject(root=root, tenant_id=tenant_id, autosync_enabled=autosync_enabled))
    return TenantConfig(mode=mode, default_tenant=default_tenant, projects=projects)


def load_sync_policy_contract(
    *, project_root: Path | None = None, explicit_path: Path | None = None
) -> SyncPolicyContract:
    """Load and validate a sync-policy contract."""
    path = explicit_path.resolve() if explicit_path else resolve_sync_policy_path(project_root=project_root)
    if not path.exists():
        raise FileNotFoundError(f"Sync policy file not found: {path}")

    raw = yaml_load(path) or {}
    if not isinstance(raw, dict):
        raise ValueError("sync-policy.yaml must contain a mapping")

    schema_version = str(raw.get("schema_version", "")).strip()
    if not schema_version:
        raise ValueError("schema_version is required")
    conflict_precedence = str(raw.get("conflict_precedence", "")).strip()
    if not conflict_precedence:
        raise ValueError("conflict_precedence is required")
    strict_mode = bool(raw.get("strict_mode", False))
    connectors = _parse_connectors(raw.get("connectors"))
    tenancy = _parse_tenancy(raw.get("tenancy"))

    return SyncPolicyContract(
        schema_version=schema_version,
        conflict_precedence=conflict_precedence,
        strict_mode=strict_mode,
        connectors=connectors,
        tenancy=tenancy,
    )
