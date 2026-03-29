"""WP-13001: Multi-org policy federation.

Implements FR-FED-001 through FR-FED-006 (WL-020):
  FR-FED-001: Three-level namespace org.project.environment with hierarchy
  FR-FED-002: Federated policy resolution (env -> project -> org -> global)
  FR-FED-003: Jurisdiction profiles (EU-AI-ACT, US-SEC) as additive overlays
  FR-FED-004: Cross-namespace consent relay with SHA-256 provenance signatures
  FR-FED-005: Most-restrictive-wins conflict arbitration + policy_arbitration.jsonl
  FR-FED-006: Federation health + drift observability endpoint
"""

from __future__ import annotations

import hashlib
import orjson as json
import logging
import time
from datetime import UTC, datetime
from pathlib import Path
from typing import Any

_log = logging.getLogger(__name__)

# ---------------------------------------------------------------------------
# FR-FED-001: Three-level namespace
# ---------------------------------------------------------------------------

_RESTRICTIVE_MIN_KEYS: frozenset[str] = frozenset(
    {"risk_threshold", "cost_cap", "sla_minutes", "escalation_sla_minutes"}
)
_RESTRICTIVE_MAX_KEYS: frozenset[str] = frozenset({"audit_retention_days", "max_tokens", "max_retries"})
_RESTRICTIVE_OR_KEYS: frozenset[str] = frozenset(
    {"human_in_loop_required", "require_human_approval", "deny", "require_audit"}
)


class PolicyNamespace:
    """Namespace identifier for org/project/env (FR-FED-001)."""

    def __init__(
        self,
        org: str,
        project: str = "default",
        environment: str = "production",
    ) -> None:
        self.org = org
        self.project = project
        self.env = environment

    def get_hierarchy(self) -> list[str]:
        """Return resolution order (FR-FED-002): env -> project.default -> org.default -> global."""
        hierarchy = [
            f"{self.org}.{self.project}.{self.env}",
            f"{self.org}.{self.project}.default",
            f"{self.org}.default.default",
        ]
        if "global" not in hierarchy:
            hierarchy.append("global")
        return hierarchy

    def __repr__(self) -> str:
        return f"{self.org}.{self.project}.{self.env}"

    def __str__(self) -> str:
        return f"{self.org}.{self.project}.{self.env}"


# ---------------------------------------------------------------------------
# FR-FED-003: Jurisdiction profiles
# ---------------------------------------------------------------------------

JURISDICTION_PROFILES: dict[str, dict[str, Any]] = {
    "EU-AI-ACT": {
        "jurisdiction_profile": "EU-AI-ACT",
        "human_in_loop_required": True,
        "risk_threshold": 0.7,
        "require_audit": True,
        "audit_retention_days": 3650,  # 10 years
        "require_human_approval": True,
        "data_minimization": True,
    },
    "US-SEC": {
        "jurisdiction_profile": "US-SEC",
        "audit_retention_days": 2555,  # 7 years
        "require_audit": True,
        "data_minimization": False,
    },
}

_REGION_TO_PROFILE: dict[str, str] = {
    "EU": "EU-AI-ACT",
    "EEA": "EU-AI-ACT",
    "US": "US-SEC",
    "US-SEC": "US-SEC",
    "EU-AI-ACT": "EU-AI-ACT",
}


def _apply_jurisdiction_overlay(base: dict[str, Any], profile_name: str) -> dict[str, Any]:
    """Apply a jurisdiction profile as an additive overlay (FR-FED-003).

    Constraints are additive (union), not overriding.  The most-restrictive
    value wins when both the base policy and the profile define the same key.
    """
    profile = JURISDICTION_PROFILES.get(profile_name, {})
    if not profile:
        return dict(base)

    result = dict(base)
    for k, v in profile.items():
        if k not in result:
            result[k] = v
        elif k in _RESTRICTIVE_MIN_KEYS:
            result[k] = min(result[k], v) if isinstance(result[k], (int | float)) else v
        elif k in _RESTRICTIVE_MAX_KEYS:
            result[k] = max(result[k], v) if isinstance(result[k], (int | float)) else v
        elif k in _RESTRICTIVE_OR_KEYS:
            result[k] = result[k] or v
        else:
            # Non-conflicting key from profile: apply additively
            result[k] = v
    return result


# ---------------------------------------------------------------------------
# FR-FED-005: Conflict arbitration log
# ---------------------------------------------------------------------------


class ArbitrationLog:
    """Appends conflict arbitration decisions to policy_arbitration.jsonl (FR-FED-005)."""

    def __init__(self, session_dir: Path) -> None:
        self.session_dir = Path(session_dir)
        self.log_path = self.session_dir / "policy_arbitration.jsonl"

    def record(
        self,
        namespace: str,
        policy_id: str,
        key: str,
        competing_values: list[Any],
        chosen_value: Any,
        rule: str,
    ) -> None:
        self.session_dir.mkdir(parents=True, exist_ok=True)
        entry: dict[str, Any] = {
            "timestamp_utc": datetime.now(UTC).isoformat(),
            "namespace": namespace,
            "policy_id": policy_id,
            "key": key,
            "competing_values": competing_values,
            "chosen_value": chosen_value,
            "arbitration_rule": rule,
        }
        with self.log_path.open("a", encoding="utf-8") as f:
            f.write(json.dumps(entry).decode() + "\n")


# ---------------------------------------------------------------------------
# FR-FED-004: Consent relay
# ---------------------------------------------------------------------------


class ConsentRelayStore:
    """Persists cross-namespace consent relay artifacts (FR-FED-004)."""

    def __init__(self, session_dir: Path) -> None:
        self.session_dir = Path(session_dir)
        self.store_path = self.session_dir / "consent_relay.jsonl"

    def store(self, artifact: dict[str, Any]) -> None:
        self.session_dir.mkdir(parents=True, exist_ok=True)
        with self.store_path.open("a", encoding="utf-8") as f:
            f.write(json.dumps(artifact).decode() + "\n")

    def list_active(self, run_id: str | None = None) -> list[dict[str, Any]]:
        if not self.store_path.exists():
            return []
        items: list[dict[str, Any]] = []
        with self.store_path.open("r", encoding="utf-8") as f:
            for line in f:
                line = line.strip()
                if not line:
                    continue
                try:
                    item = json.loads(line)
                    if item.get("status") != "active":
                        continue
                    if run_id and item.get("run_id") != run_id:
                        continue
                    items.append(item)
                except json.JSONDecodeError:
                    continue
        return items


# ---------------------------------------------------------------------------
# Main FederatedPolicyManager
# ---------------------------------------------------------------------------


class FederatedPolicyManager:
    """Manages federated policy resolution and health (WP-13001, FR-FED-001..006).

    Directory layout under base_dir:
        <org>/<project>/<env>/<policy_id>.json
    """

    def __init__(self, base_dir: Path, session_dir: Path | None = None) -> None:
        self.base_dir = Path(base_dir)
        self.session_dir = Path(session_dir) if session_dir else self.base_dir
        self._arbitration_log = ArbitrationLog(self.session_dir)
        self._consent_store = ConsentRelayStore(self.session_dir)

    # -- FR-FED-001/002: Namespace + Resolution --------------------------------

    def get_federation_health(self) -> dict[str, Any]:
        """Return federation health status (FR-FED-006)."""
        namespaces: list[str] = []
        if self.base_dir.exists():
            for org_p in self.base_dir.iterdir():
                if org_p.is_dir():
                    for proj_p in org_p.iterdir():
                        if proj_p.is_dir():
                            for env_p in proj_p.iterdir():
                                if env_p.is_dir():
                                    namespaces.append(f"{org_p.name}.{proj_p.name}.{env_p.name}")
        status = "healthy" if namespaces else "empty"
        drift_report = self._detect_drift(namespaces)
        return {
            "status": status,
            "namespace_count": len(namespaces),
            "namespaces": namespaces,
            "drift": drift_report,
            "checked_at_utc": datetime.now(UTC).isoformat(),
        }

    def join_namespace(self, ns_str: str) -> bool:
        """Register current node with a federated namespace (WP-13006)."""
        parts = ns_str.split(".")
        if len(parts) < 3:
            _log.error("Invalid namespace format: %s. Expected org.project.env", ns_str)
            return False

        target_dir = self.base_dir / parts[0] / parts[1] / parts[2]
        target_dir.mkdir(parents=True, exist_ok=True)
        _log.info("Joined namespace: %s", ns_str)
        return True

    def leave_namespace(self, ns_str: str) -> bool:
        """Remove registration for a federated namespace."""
        parts = ns_str.split(".")
        if len(parts) < 3:
            return False

        target_dir = self.base_dir / parts[0] / parts[1] / parts[2]
        if target_dir.exists():
            import shutil

            shutil.rmtree(target_dir)
            _log.info("Left namespace: %s", ns_str)
            return True
        return False

    def resolve_policy(self, ns: PolicyNamespace, policy_id: str) -> dict[str, Any]:
        """Resolve policy by traversing namespace hierarchy (FR-FED-002).

        Resolution order: environment -> project.default -> org.default -> global
        Returns the first matching policy file found.
        """
        for ns_str in ns.get_hierarchy():
            parts = ns_str.split(".")
            if len(parts) >= 3:
                path = self.base_dir / parts[0] / parts[1] / parts[2] / f"{policy_id}.json"
                if path.exists():
                    return json.loads(path.read_text(encoding="utf-8"))
        return {}

    # -- FR-FED-003: Jurisdiction overlays ------------------------------------

    def apply_jurisdiction_constraints(self, policy: dict[str, Any], region: str) -> dict[str, Any]:
        """Apply jurisdiction overlay as additive constraints (FR-FED-003).

        Profiles: EU-AI-ACT, US-SEC.  Constraints are additive (union) — the
        most-restrictive value wins when both policy and profile define a key.
        """
        profile_name = _REGION_TO_PROFILE.get(region.upper())
        if not profile_name:
            return dict(policy)
        return _apply_jurisdiction_overlay(policy, profile_name)

    def apply_jurisdiction_profile(self, policy: dict[str, Any], profile_name: str) -> dict[str, Any]:
        """Apply a named jurisdiction profile directly (FR-FED-003)."""
        return _apply_jurisdiction_overlay(policy, profile_name)

    # -- FR-FED-004: Cross-namespace consent relay ----------------------------

    def relay_consent(
        self,
        ns1: PolicyNamespace,
        ns2: PolicyNamespace,
        run_id: str,
        approver: str,
    ) -> dict[str, Any]:
        """Relay approval consent between namespaces with provenance signature (FR-FED-004).

        Generates a traceable relay artifact with SHA-256 signature over the
        payload: ns1:ns2:run_id:approver:timestamp.
        """
        ts = time.time()
        payload = f"{ns1}:{ns2}:{run_id}:{approver}:{ts}"
        signature = hashlib.sha256(payload.encode()).hexdigest()

        relay_artifact: dict[str, Any] = {
            "type": "consent_relay",
            "version": "1.0",
            "run_id": run_id,
            "source_namespace": str(ns1),
            "target_namespace": str(ns2),
            "approver": approver,
            "timestamp": ts,
            "provenance_signature": signature,
            "status": "active",
        }

        self._consent_store.store(relay_artifact)
        _log.info("Relayed consent from %s to %s for run %s", ns1, ns2, run_id)
        return relay_artifact

    def get_consent_relays(self, run_id: str | None = None) -> list[dict[str, Any]]:
        """Return active consent relay artifacts (FR-FED-004)."""
        return self._consent_store.list_active(run_id=run_id)

    # -- FR-FED-005: Most-restrictive-wins conflict arbitration ---------------

    def arbitrate_conflict(
        self,
        policies: list[dict[str, Any]],
        namespace: str = "unknown",
        policy_id: str = "unknown",
    ) -> dict[str, Any]:
        """Arbitrate conflicts using 'most restrictive wins' (FR-FED-005).

        For each conflicting key:
        - Numeric thresholds where lower = stricter: take min
        - Numeric limits where higher = stricter: take max
        - Boolean flags where True = stricter: take OR
        - Other keys: last value wins (last policy has highest precedence)

        Logs each arbitration decision to policy_arbitration.jsonl.
        """
        if not policies:
            return {}

        out: dict[str, Any] = {}
        competing: dict[str, list[Any]] = {}

        for p in policies:
            for k, v in p.items():
                if k not in out:
                    out[k] = v
                    competing[k] = [v]
                else:
                    competing[k].append(v)
                    prev = out[k]
                    chosen, rule = self._most_restrictive(k, prev, v)
                    if chosen != prev:
                        out[k] = chosen
                        self._arbitration_log.record(
                            namespace=namespace,
                            policy_id=policy_id,
                            key=k,
                            competing_values=competing[k],
                            chosen_value=chosen,
                            rule=rule,
                        )

        out["arbitration_applied"] = True
        return out

    def _most_restrictive(self, key: str, a: Any, b: Any) -> tuple[Any, str]:
        """Return (chosen_value, rule_description) for the more restrictive of a and b."""
        if key in _RESTRICTIVE_MIN_KEYS:
            if isinstance(a, (int | float)) and isinstance(b, (int | float)):
                chosen = min(a, b)
                return chosen, "min_value (lower is more restrictive)"
        if key in _RESTRICTIVE_MAX_KEYS:
            if isinstance(a, (int | float)) and isinstance(b, (int | float)):
                chosen = max(a, b)
                return chosen, "max_value (higher is more restrictive)"
        if key in _RESTRICTIVE_OR_KEYS:
            chosen = bool(a) or bool(b)
            return chosen, "boolean_or (True is more restrictive)"
        # Default: last-writer wins (b takes precedence)
        return b, "last_writer_wins"

    # -- FR-FED-006: Health + drift observability ----------------------------

    def _detect_drift(self, namespaces: list[str]) -> dict[str, Any]:
        """Detect policy drift: namespaces missing an org-level parent (FR-FED-006)."""
        drifted: list[dict[str, Any]] = []
        for ns_str in namespaces:
            parts = ns_str.split(".")
            if len(parts) < 3:
                continue
            org, project, env = parts[0], parts[1], parts[2]
            if project == "default" and env == "default":
                continue
            # Check if org-level policy dir exists
            org_default_dir = self.base_dir / org / "default" / "default"
            if not org_default_dir.exists():
                drifted.append(
                    {
                        "namespace": ns_str,
                        "issue": "no_org_level_policy",
                        "missing": f"{org}.default.default",
                    }
                )
        return {
            "drifted_count": len(drifted),
            "drifted_namespaces": drifted,
            "status": "drift_detected" if drifted else "in_sync",
        }

    def get_federation_health_endpoint(self) -> dict[str, Any]:
        """Return the GET /governance/federation/health response payload (FR-FED-006).

        Includes policy sync status across namespaces and drift detection.
        """
        health = self.get_federation_health()
        return {
            "endpoint": "GET /governance/federation/health",
            "version": "1.0",
            "health": health,
        }


# ---------------------------------------------------------------------------
# FederationManager (lightweight multi-org coordinator, unchanged API)
# ---------------------------------------------------------------------------


class FederationManager:
    """Manages policy federation across multiple organizations."""

    def __init__(self, session_dir: Path) -> None:
        self.session_dir = session_dir
        self.peers_path = session_dir / "federation_peers.json"

    def sync_policies(self, peer_id: str) -> None:
        """Sync governance policies from a peer organization."""
        _log.info("Syncing policies from federation peer %s", peer_id)

    def get_effective_policy(self, policy_id: str) -> dict[str, Any]:
        """Resolve a policy, considering federated overrides."""
        local_path = self.session_dir / "contracts" / f"{policy_id}.json"
        if local_path.exists():
            return json.loads(local_path.read_text(encoding="utf-8"))
        return {}
