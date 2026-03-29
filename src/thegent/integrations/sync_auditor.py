"""Sync policy auditor for runtime validation.

# @trace WL-261
"""

from __future__ import annotations

from difflib import HtmlDiff
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any

import orjson

from thegent.integrations.base import SerializableMixin
from thegent.integrations.sync_provenance import (
    SyncProvenanceStamp,
    chain_provenance_stamps,
    verify_provenance_chain,
)
from thegent.integrations.sync_policy_contract import (
    SyncPolicyContract,
    load_sync_policy_contract,
)


@dataclass
class SyncPolicyAudit:
    """Sync policy audit result."""

    enabled_connectors: list[str]
    quota_budgets: dict[str, int]
    policy_modes: dict[str, str]
    timestamp: str
    audit_status: str = "success"


@dataclass
class RemoteOrphanReport(SerializableMixin):
    """Structured report of remote items not represented locally."""

    remote_ids: list[str]
    local_ids: list[str]
    orphan_ids: list[str]


@dataclass
class LocalOrphanReport(SerializableMixin):
    """Structured report of local items without remote tracker mapping."""

    local_ids: list[str]
    mapped_remote_ids: list[str]
    local_orphan_ids: list[str]
    orphan_count: int


class SyncAuditor:
    """Auditor for sync policies."""

    def __init__(self) -> None:
        """Initialize the sync auditor."""
        self._enabled_connectors: list[str] = []
        self._quota_budgets: dict[str, int] = {}
        self._policy_modes: dict[str, str] = {}
        self._artifact_chain: list[SyncProvenanceStamp] = []

    def set_enabled_connectors(self, connectors: list[str]) -> None:
        """Set the list of enabled connectors.

        Args:
            connectors: List of enabled connector names.
        """
        self._enabled_connectors = list(connectors)

    def set_quota_budgets(self, budgets: dict[str, int]) -> None:
        """Set quota budgets for connectors.

        Args:
            budgets: Dictionary mapping connector names to daily quota limits.
        """
        self._quota_budgets = dict(budgets)

    def set_policy_modes(self, modes: dict[str, str]) -> None:
        """Set policy enforcement modes for connectors.

        Args:
            modes: Dictionary mapping connector names to policy modes
                  (e.g., 'enforce', 'warn', 'disabled').
        """
        self._policy_modes = dict(modes)

    def audit(self) -> SyncPolicyAudit:
        """Run the sync policy audit.

        Returns:
            SyncPolicyAudit with current policies.
        """
        from datetime import datetime, timezone

        now = datetime.now(timezone.utc).isoformat()

        return SyncPolicyAudit(
            enabled_connectors=self._enabled_connectors,
            quota_budgets=self._quota_budgets,
            policy_modes=self._policy_modes,
            timestamp=now,
        )

    def load_policy_contract(
        self, *, project_root: Path | None = None, explicit_path: Path | None = None
    ) -> SyncPolicyContract:
        """Load `.thegent/sync-policy.yaml` and map it into audit surfaces."""
        contract = load_sync_policy_contract(project_root=project_root, explicit_path=explicit_path)
        connector_policies = contract.connectors or {}
        enabled = [name for name, policy in connector_policies.items() if policy.enabled]
        quota_budgets = {name: policy.quota_daily for name, policy in connector_policies.items()}
        policy_modes = {name: policy.mode for name, policy in connector_policies.items()}
        self.set_enabled_connectors(enabled)
        self.set_quota_budgets(quota_budgets)
        self.set_policy_modes(policy_modes)
        return contract

    def audit_as_json(self) -> str:
        """Get audit result as JSON string.

        Returns:
            JSON representation of audit result.
        """
        audit = self.audit()
        return orjson.dumps(asdict(audit), option=orjson.OPT_INDENT_2).decode()

    def audit_as_dict(self) -> dict[str, Any]:
        """Get audit result as dictionary.

        Returns:
            Dictionary representation of audit result.
        """
        audit = self.audit()
        return asdict(audit)

    def validate_policy(self) -> tuple[bool, list[str]]:
        """Validate sync policy configuration.

        Returns:
            Tuple of (is_valid, list_of_issues).
        """
        issues: list[str] = []

        if not self._enabled_connectors:
            issues.append("No connectors are enabled")

        # Check for quota budgets without corresponding enabled connectors
        for connector, budget in self._quota_budgets.items():
            if connector not in self._enabled_connectors:
                issues.append(f"Quota budget defined for disabled connector: {connector}")
            if budget <= 0:
                issues.append(f"Invalid quota budget for {connector}: {budget} (must be > 0)")

        # Check for policy modes without corresponding enabled connectors
        for connector in self._policy_modes:
            if connector not in self._enabled_connectors:
                issues.append(f"Policy mode defined for disabled connector: {connector}")

        # Check for missing policy modes
        for connector in self._enabled_connectors:
            if connector not in self._policy_modes:
                issues.append(f"Missing policy mode for enabled connector: {connector}")

        is_valid = len(issues) == 0
        return is_valid, issues

    @staticmethod
    def generate_html_diff_artifact(
        local_snapshot: dict[str, Any], remote_snapshot: dict[str, Any], out_path: Path
    ) -> Path:
        """Generate deterministic side-by-side HTML diff artifact."""
        json_options = orjson.OPT_INDENT_2 | orjson.OPT_SORT_KEYS
        local_lines = orjson.dumps(local_snapshot, option=json_options).decode().splitlines()
        remote_lines = orjson.dumps(remote_snapshot, option=json_options).decode().splitlines()
        html = HtmlDiff(tabsize=2, wrapcolumn=120).make_file(
            fromlines=local_lines,
            tolines=remote_lines,
            fromdesc="local",
            todesc="remote",
            context=False,
            numlines=0,
        )
        out_path.parent.mkdir(parents=True, exist_ok=True)
        out_path.write_text(html, encoding="utf-8")
        return out_path

    @staticmethod
    def detect_remote_orphans(remote_ids: list[str], local_ids: list[str]) -> RemoteOrphanReport:
        """Return remote IDs that are missing from local WORK_STREAM IDs."""
        local_set = set(local_ids)
        orphan_ids = sorted({item_id for item_id in remote_ids if item_id not in local_set})
        return RemoteOrphanReport(
            remote_ids=sorted(remote_ids),
            local_ids=sorted(local_ids),
            orphan_ids=orphan_ids,
        )

    @staticmethod
    def detect_local_orphans(local_ids: list[str], mapped_remote_ids: list[str]) -> LocalOrphanReport:
        """Return local IDs that are not present in remote tracker mappings."""
        local_set = set(local_ids)
        remote_set = set(mapped_remote_ids)
        orphan_ids = sorted(local_set - remote_set)
        return LocalOrphanReport(
            local_ids=sorted(local_set),
            mapped_remote_ids=sorted(remote_set),
            local_orphan_ids=orphan_ids,
            orphan_count=len(orphan_ids),
        )

    def append_artifact(
        self,
        *,
        sync_id: str,
        source: str,
        operator: str,
        cycle_number: int,
        secret: str,
    ) -> SyncProvenanceStamp:
        """Append a signed artifact to the in-memory audit chain."""
        from datetime import datetime, timezone

        base_stamp = SyncProvenanceStamp(
            sync_id=sync_id,
            timestamp=datetime.now(timezone.utc).isoformat(),
            source=source,
            operator=operator,
            cycle_number=cycle_number,
        )
        staged = self._artifact_chain + [base_stamp]
        self._artifact_chain = chain_provenance_stamps(staged, secret)
        return self._artifact_chain[-1]

    def verify_artifact_chain(self, secret: str) -> tuple[bool, str]:
        """Verify that the artifact chain is continuous and signatures are valid."""
        return verify_provenance_chain(self._artifact_chain, secret)

    def artifact_chain(self) -> list[dict[str, Any]]:
        """Return audit chain artifacts as dictionaries."""
        return [stamp.to_dict() for stamp in self._artifact_chain]
