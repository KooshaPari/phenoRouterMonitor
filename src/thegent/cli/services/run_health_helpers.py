"""Helpers for run/session health snapshot policy and JSONL log management."""

from __future__ import annotations

import hashlib
import orjson as json
from pathlib import Path
import os
from typing import Any, Callable

from thegent.config import ThegentSettings


def hash_health_payload(payload: dict[str, Any]) -> dict[str, str]:
    """Return a stable health payload signature excluding volatile metadata fields."""
    payload_for_hash = {
        key: value for key, value in payload.items() if key not in {"generated_at_utc", "payload_signature"}
    }
    body = json.dumps(payload_for_hash, option=json.OPT_SORT_KEYS).decode()
    return {"algorithm": "sha256", "value": hashlib.sha256(body.encode()).hexdigest()}


def resolve_health_policy(
    *,
    policy_profile: str | None,
    strict: bool,
    min_healthy_ratio: float,
    health_policy_profiles: dict[str, dict[str, Any]],
) -> dict[str, Any]:
    """Resolve health policy config from profile (when present) plus explicit flags."""
    profile = "custom"
    effective_strict = bool(strict)
    threshold = float(min_healthy_ratio)
    profile_exists = True
    if policy_profile:
        key = str(policy_profile).strip().lower()
        selected = health_policy_profiles.get(key)
        if selected is not None:
            profile = key
            effective_strict = bool(selected["strict"])
            threshold = float(selected["min_healthy_ratio"])
        else:
            profile_exists = False
    threshold = max(threshold, 0.0)
    threshold = min(threshold, 1.0)
    return {
        "profile": profile,
        "profile_exists": profile_exists,
        "strict": effective_strict,
        "min_healthy_ratio": threshold,
    }


def health_snapshot_log_path() -> Path:
    """Resolve canonical health snapshot jsonl path and ensure parent directory exists."""
    settings = ThegentSettings()
    raw = str(settings.health_snapshot_path) if settings.health_snapshot_path else ""
    path = Path(raw).expanduser() if raw else Path.home() / ".thegent" / "health-snapshots.jsonl"
    path.parent.mkdir(parents=True, exist_ok=True)
    return path


def health_snapshot_max_lines() -> int:
    """Resolve health snapshot retention line-count with sane fallback bounds."""
    default_lines = 5000
    env_value = os.environ.get("THGENT_HEALTH_SNAPSHOT_MAX_LINES")
    if env_value is not None:
        raw_env = str(env_value).strip()
        if not raw_env:
            return default_lines
        try:
            return max(100, int(raw_env))
        except ValueError:
            return default_lines
    return default_lines


def compact_health_snapshot_log(
    *,
    log_path_resolver: Callable[[], Path] = health_snapshot_log_path,
    max_lines_resolver: Callable[[], int] = health_snapshot_max_lines,
) -> None:
    """Trim health snapshot log to max configured lines while keeping newest entries."""
    path = log_path_resolver()
    if not path.exists():
        return
    limit = max_lines_resolver()
    try:
        lines = path.read_text(encoding="utf-8").splitlines()
    except OSError:
        return
    if len(lines) <= limit:
        return
    trimmed = lines[-limit:]
    try:
        path.write_text("\n".join(trimmed) + "\n", encoding="utf-8")
    except OSError:
        return


def health_scope_key(payload: dict[str, Any]) -> dict[str, Any]:
    """Build deterministic scope key for snapshot-series grouping."""
    query = payload.get("generated_query", {}) or {}
    scope: dict[str, Any] = {
        "payload_type": payload.get("payload_type", ""),
        "owner": query.get("owner"),
        "all": bool(query.get("all", False)),
        "strict": bool(query.get("strict", False)),
        "policy_profile": payload.get("policy_profile", "custom"),
    }
    if payload.get("payload_type") == "session_contract_health_gate":
        scope["min_healthy_ratio"] = float(query.get("min_healthy_ratio", 1.0))
    if payload.get("payload_type") == "session_contract_health_report":
        scope["top_blocked"] = int(query.get("top_blocked", 25))
    return scope


def coerce_issue_types(value: Any) -> list[str]:
    """Normalize issue collection values to deterministic list[str] representation."""
    if value is None:
        return []
    if isinstance(value, dict):
        return [str(v) for v in value]
    if isinstance(value, (list | tuple | set)):
        return [str(v) for v in value]
    return [str(value)]


def load_previous_health_snapshot(
    scope_key: dict[str, Any],
    *,
    log_path_resolver: Callable[[], Path] = health_snapshot_log_path,
) -> dict[str, Any] | None:
    """Return newest matching snapshot record for the provided scope."""
    path = log_path_resolver()
    if not path.exists():
        return None
    try:
        lines = path.read_text(encoding="utf-8").splitlines()
    except OSError:
        return None
    for line in reversed(lines):
        line = line.strip()
        if not line:
            continue
        try:
            rec = json.loads(line)
        except json.JSONDecodeError:
            continue
        if rec.get("record_type") != "health_snapshot":
            continue
        if rec.get("scope_key") == scope_key:
            return rec
    return None


def append_health_snapshot(
    payload: dict[str, Any],
    scope_key: dict[str, Any],
    *,
    log_path_resolver: Callable[[], Path] = health_snapshot_log_path,
    compact_log_fn: Callable[[], None] = compact_health_snapshot_log,
    coerce_issue_types_fn: Callable[[Any], list[str]] = coerce_issue_types,
) -> None:
    """Append one health snapshot record and trigger retention compaction."""
    path = log_path_resolver()
    issue_types: list[str] = []
    if payload.get("payload_type") == "session_contract_health_report":
        issue_types = sorted([str(k) for k in (payload.get("issue_counts") or {})])
    else:
        seen: set[str] = set()
        for row in payload.get("blocked_sessions", []) or []:
            for issue in coerce_issue_types_fn(row.get("issues", [])):
                seen.add(issue)
        issue_types = sorted(seen)
    rec = {
        "record_type": "health_snapshot",
        "captured_at_utc": payload.get("generated_at_utc", ""),
        "scope_key": scope_key,
        "schema_version": payload.get("schema_version", ""),
        "payload_type": payload.get("payload_type", ""),
        "status": payload.get("status", ""),
        "pass": payload.get("pass", False),
        "total": payload.get("total", 0),
        "healthy_count": payload.get("healthy_count", 0),
        "unhealthy_count": payload.get("unhealthy_count", 0),
        "blocked_count": payload.get("blocked_count", 0),
        "blocked_ratio": payload.get("blocked_ratio", 0.0),
        "issue_types": issue_types,
        "issue_counts": payload.get("issue_counts", {}),
        "payload_signature": payload.get("payload_signature", {}),
    }
    try:
        with path.open("a", encoding="utf-8") as fh:
            fh.write(json.dumps(rec, option=json.OPT_SORT_KEYS).decode())
            fh.write("\n")
    except OSError:
        return
    compact_log_fn()
