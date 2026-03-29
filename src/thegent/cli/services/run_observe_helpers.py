"""Observe-summary trend/snapshot helper services extracted from CLI impl surface."""

from __future__ import annotations

import hashlib
import orjson as json
import os
from datetime import UTC, datetime
from pathlib import Path
from typing import Any

from thegent.config import ThegentSettings

OBSERVE_SUMMARY_SCHEMA_VERSION = "observe-summary-schema-v1"


def hash_observe_summary_payload(payload: dict[str, Any]) -> dict[str, str]:
    """Return a stable hash for an observe-summary payload."""
    payload_for_hash = {
        key: value for key, value in payload.items() if key not in {"generated_at_utc", "payload_signature"}
    }
    body = json.dumps(payload_for_hash, option=json.OPT_SORT_KEYS).decode()
    return {"algorithm": "sha256", "value": hashlib.sha256(body.encode("utf-8")).hexdigest()}


def build_observe_summary_trend_scope(
    *,
    provider: str | None,
    drift_window: int,
    structural_budget_pct: float,
    semantic_budget_pct: float,
    limit: int,
    top_escalations: int,
) -> dict[str, Any]:
    return {
        "payload_type": "observe_summary",
        "provider": provider,
        "drift_window": int(drift_window),
        "structural_budget_pct": float(structural_budget_pct),
        "semantic_budget_pct": float(semantic_budget_pct),
        "limit": int(limit),
        "top_escalations": int(top_escalations),
    }


def hash_observe_summary_trend_scope(scope_key: dict[str, Any]) -> str:
    scope_key_json = json.dumps(scope_key, option=json.OPT_SORT_KEYS).decode()
    return hashlib.sha256(scope_key_json.encode("utf-8")).hexdigest()


def parse_observe_summary_timestamp(value: str | None) -> datetime | None:
    if not value:
        return None
    try:
        parsed = datetime.fromisoformat(str(value))
    except ValueError:
        value = str(value).replace("Z", "+00:00")
        try:
            parsed = datetime.fromisoformat(value)
        except ValueError:
            return None
    if parsed.tzinfo is None:
        return parsed.replace(tzinfo=UTC)
    return parsed.astimezone(UTC)


def parse_observe_summary_env_float(name: str, default: float) -> float:
    raw = os.environ.get(name, "").strip()
    if not raw:
        return float(default)
    try:
        value = float(raw)
    except (TypeError, ValueError):
        return float(default)
    return value


def parse_observe_summary_env_int(name: str, default: int) -> int:
    raw = os.environ.get(name, "").strip()
    if not raw:
        return int(default)
    try:
        value = int(raw)
    except (TypeError, ValueError):
        return int(default)
    return value


def observe_summary_freshness_bucket(
    freshness_seconds: int | None,
    *,
    fresh_seconds: int,
    warm_seconds: int,
    stale_seconds: int,
) -> str:
    if freshness_seconds is None:
        return "unknown"
    if freshness_seconds < 0:
        return "future"
    if freshness_seconds <= fresh_seconds:
        return "fresh"
    if freshness_seconds <= warm_seconds:
        return "warm"
    if freshness_seconds <= stale_seconds:
        return "stale"
    return "critical"


def load_observe_summary_snapshots(
    scope_signature: str,
    scope_key_json: str,
    limit: int,
) -> list[dict[str, Any]]:
    path = health_snapshot_log_path()
    if not path.exists():
        return []
    snapshots: list[dict[str, Any]] = []
    try:
        lines = path.read_text(encoding="utf-8").splitlines()
    except OSError:
        return []
    for line in reversed(lines):
        line = line.strip()
        if not line:
            continue
        try:
            rec = json.loads(line)
        except json.JSONDecodeError:
            continue
        if rec.get("record_type") != "observe_summary_snapshot":
            continue
        if (
            rec.get("trend_scope_signature") != scope_signature
            and rec.get("scope_signature") != scope_signature
            and rec.get("scope_key_json") != scope_key_json
        ):
            continue
        snapshots.append(rec)
        if len(snapshots) >= limit:
            break
    return snapshots


def classify_observe_summary_trend_health(
    *,
    enabled: bool,
    baseline_available: bool,
    trend_snapshot_coverage_pct: float | None,
    trend_snapshot_deficit: int,
    trend_snapshot_invalid_timestamps: int,
    trend_snapshot_freshness_bucket: str,
    trend_snapshot_gap_count: int,
    trend_sampling_mode: str,
) -> dict[str, Any]:
    policy: dict[str, Any] = {
        "healthy_threshold": parse_observe_summary_env_int("THGENT_OBSERVE_SUMMARY_TREND_HEALTH_GOOD_THRESHOLD", 95),
        "warning_threshold": parse_observe_summary_env_int("THGENT_OBSERVE_SUMMARY_TREND_HEALTH_WARNING_THRESHOLD", 80),
        "degraded_threshold": parse_observe_summary_env_int(
            "THGENT_OBSERVE_SUMMARY_TREND_HEALTH_DEGRADED_THRESHOLD", 50
        ),
        "min_coverage_pct": parse_observe_summary_env_float(
            "THGENT_OBSERVE_SUMMARY_TREND_HEALTH_MIN_COVERAGE_PCT", 80.0
        ),
        "max_invalid_timestamps": parse_observe_summary_env_int(
            "THGENT_OBSERVE_SUMMARY_TREND_HEALTH_MAX_INVALID_TIMESTAMPS", 0
        ),
        "coverage_penalty_per_pct": parse_observe_summary_env_float(
            "THGENT_OBSERVE_SUMMARY_TREND_HEALTH_COVERAGE_PENALTY_PER_PCT", 1.25
        ),
        "deficit_penalty_per_missing_sample": parse_observe_summary_env_float(
            "THGENT_OBSERVE_SUMMARY_TREND_HEALTH_DEFICIT_PENALTY_PER_MISSING_SAMPLE", 15
        ),
        "invalid_timestamp_penalty_per_event": parse_observe_summary_env_float(
            "THGENT_OBSERVE_SUMMARY_TREND_HEALTH_INVALID_TIMESTAMP_PENALTY_PER_EVENT", 12
        ),
        "stale_penalty": parse_observe_summary_env_float("THGENT_OBSERVE_SUMMARY_TREND_HEALTH_STALE_PENALTY", 8),
        "critical_penalty": parse_observe_summary_env_float("THGENT_OBSERVE_SUMMARY_TREND_HEALTH_CRITICAL_PENALTY", 20),
        "unknown_or_future_penalty": parse_observe_summary_env_float(
            "THGENT_OBSERVE_SUMMARY_TREND_HEALTH_UNKNOWN_OR_FUTURE_PENALTY", 30
        ),
        "gap_penalty": parse_observe_summary_env_float("THGENT_OBSERVE_SUMMARY_TREND_HEALTH_GAP_PENALTY", 10),
        "missing_baseline_penalty": parse_observe_summary_env_float(
            "THGENT_OBSERVE_SUMMARY_TREND_HEALTH_MISSING_BASELINE_PENALTY", 45
        ),
    }
    policy_signature = hashlib.sha256(
        json.dumps(policy, option=json.OPT_SORT_KEYS).decode().encode("utf-8")
    ).hexdigest()

    if not enabled:
        recommendations = [
            "Enable trend sampling with --trend-samples >= 2 to produce trend quality signals.",
            f"Use trend-sampling mode: {trend_sampling_mode}.",
        ]
        return {
            "trend_snapshot_health": "disabled",
            "trend_snapshot_health_score": None,
            "trend_snapshot_health_breakdown": {
                "policy_signature": policy_signature,
                "policy": policy,
                "reason": "trend_disabled",
                "trend_sampling_mode": trend_sampling_mode,
                "enabled": False,
                "recommendations": recommendations,
                "penalties": {
                    "enabled": 0,
                    "baseline": 0,
                    "coverage": 0,
                    "deficit": 0,
                    "invalid_timestamps": 0,
                    "freshness": 0,
                    "gap": 0,
                },
            },
            "trend_snapshot_recommendations": recommendations,
        }

    penalties: dict[str, float] = {
        "coverage": 0.0,
        "deficit": 0.0,
        "invalid_timestamps": 0.0,
        "freshness": 0.0,
        "gap": 0.0,
        "baseline": 0.0,
    }
    coverage_shortfall = 0.0
    if trend_snapshot_coverage_pct is None:
        coverage_shortfall = 0.0
        penalties["coverage"] = 0.0
    elif trend_snapshot_coverage_pct < policy["min_coverage_pct"]:
        coverage_shortfall = policy["min_coverage_pct"] - trend_snapshot_coverage_pct
        penalties["coverage"] = round(coverage_shortfall * policy["coverage_penalty_per_pct"], 6)

    if trend_snapshot_deficit > 0:
        penalties["deficit"] = trend_snapshot_deficit * policy["deficit_penalty_per_missing_sample"]

    if trend_snapshot_invalid_timestamps > policy["max_invalid_timestamps"]:
        penalties["invalid_timestamps"] = (
            trend_snapshot_invalid_timestamps * policy["invalid_timestamp_penalty_per_event"]
        )

    if trend_snapshot_freshness_bucket == "stale":
        penalties["freshness"] = policy["stale_penalty"]
    elif trend_snapshot_freshness_bucket == "critical":
        penalties["freshness"] = policy["critical_penalty"]
    elif trend_snapshot_freshness_bucket in {"future", "unknown"}:
        penalties["freshness"] = policy["unknown_or_future_penalty"]

    penalties["gap"] = trend_snapshot_gap_count * policy["gap_penalty"]
    if not baseline_available:
        penalties["baseline"] = policy["missing_baseline_penalty"]

    score = 100.0 - sum(penalties.values())
    score = max(0.0, min(100.0, score))
    health = "critical"
    if score >= policy["healthy_threshold"]:
        health = "good"
    elif score >= policy["warning_threshold"]:
        health = "warning"
    elif score >= policy["degraded_threshold"]:
        health = "degraded"

    recommendations: list[str] = []
    if coverage_shortfall > 0:
        recommendations.append(
            "Increase capture coverage by reducing trend sample window or lowering requested samples."
        )
    if trend_snapshot_deficit > 0:
        recommendations.append("Trend history is incomplete; expected samples were not all available.")
    if trend_snapshot_invalid_timestamps > policy["max_invalid_timestamps"]:
        recommendations.append("Snapshot contains invalid/missing timestamps; normalize capture time format.")
    if trend_snapshot_freshness_bucket in {"stale", "critical"}:
        recommendations.append("Trend freshness is degraded; capture cadence may be too low.")
    if trend_snapshot_gap_count > 0:
        recommendations.append("Snapshot gaps detected; verify persistence and scheduler cadence.")
    if not baseline_available:
        recommendations.append("No baseline snapshot available; next run may enable full delta reporting.")
    if not recommendations:
        recommendations.append("Trend quality is healthy.")

    return {
        "trend_snapshot_health": health,
        "trend_snapshot_health_score": round(score),
        "trend_snapshot_health_breakdown": {
            "policy_signature": policy_signature,
            "policy": policy,
            "healthy_threshold": policy["healthy_threshold"],
            "warning_threshold": policy["warning_threshold"],
            "degraded_threshold": policy["degraded_threshold"],
            "coverage": {
                "coverage_pct": trend_snapshot_coverage_pct,
                "coverage_shortfall_pct": coverage_shortfall,
                "coverage_penalty": penalties["coverage"],
                "min_coverage_pct": policy["min_coverage_pct"],
            },
            "deficit": {
                "trend_snapshot_deficit": trend_snapshot_deficit,
                "penalty_per_missing": policy["deficit_penalty_per_missing_sample"],
                "deficit_penalty": penalties["deficit"],
            },
            "invalid_timestamps": {
                "count": trend_snapshot_invalid_timestamps,
                "max_allowed": policy["max_invalid_timestamps"],
                "penalty_per_event": policy["invalid_timestamp_penalty_per_event"],
                "invalid_timestamp_penalty": penalties["invalid_timestamps"],
            },
            "freshness": {
                "bucket": trend_snapshot_freshness_bucket,
                "penalty": penalties["freshness"],
            },
            "gap": {
                "gap_count": trend_snapshot_gap_count,
                "penalty_per_gap": policy["gap_penalty"],
                "gap_penalty": penalties["gap"],
            },
            "baseline": {
                "baseline_available": baseline_available,
                "baseline_penalty": penalties["baseline"],
                "missing_baseline_penalty": policy["missing_baseline_penalty"],
            },
            "trend_sampling_mode": trend_sampling_mode,
            "enabled": enabled,
            "score": round(score),
            "recommendations": recommendations,
            "penalties": {
                "coverage_penalty": penalties["coverage"],
                "deficit_penalty": penalties["deficit"],
                "invalid_timestamps_penalty": penalties["invalid_timestamps"],
                "freshness_penalty": penalties["freshness"],
                "gap_penalty": penalties["gap"],
                "baseline_penalty": penalties["baseline"],
            },
        },
        "trend_snapshot_recommendations": recommendations,
    }


def append_observe_summary_snapshot(
    payload: dict[str, Any],
    trend_scope_key: dict[str, Any],
    trend_scope_signature: str,
    scope_key_json: str,
    trend_snapshot_ids: list[str],
    trend_summary: dict[str, Any],
) -> None:
    record = {
        "record_type": "observe_summary_snapshot",
        "captured_at_utc": payload.get("generated_at_utc", ""),
        "scope_key": trend_scope_key,
        "scope_key_json": scope_key_json,
        "scope_signature": trend_scope_signature,
        "trend_scope_signature": trend_scope_signature,
        "trend_previous_samples_requested": trend_summary.get("trend_previous_samples_requested", 0),
        "trend_snapshot_expected_count": trend_summary.get("trend_snapshot_expected_count", 0),
        "trend_snapshot_deficit": trend_summary.get("trend_snapshot_deficit", 0),
        "trend_snapshot_interval_seconds_avg": trend_summary.get("trend_snapshot_interval_seconds_avg"),
        "trend_snapshot_interval_seconds_min": trend_summary.get("trend_snapshot_interval_seconds_min"),
        "trend_snapshot_interval_seconds_max": trend_summary.get("trend_snapshot_interval_seconds_max"),
        "trend_snapshot_gap_count": trend_summary.get("trend_snapshot_gap_count", 0),
        "trend_snapshot_invalid_timestamps": trend_summary.get("trend_snapshot_invalid_timestamps", 0),
        "trend_snapshot_coverage_pct": trend_summary.get("trend_snapshot_coverage_pct"),
        "trend_snapshot_freshness_bucket": trend_summary.get("trend_snapshot_freshness_bucket", "unknown"),
        "trend_snapshot_freshness_seconds": trend_summary.get("trend_snapshot_freshness_seconds"),
        "trend_snapshot_health": trend_summary.get("trend_snapshot_health", "disabled"),
        "trend_snapshot_health_score": trend_summary.get("trend_snapshot_health_score"),
        "trend_snapshot_recommendations": trend_summary.get("trend_snapshot_recommendations", []),
        "trend_snapshot_health_breakdown": trend_summary.get("trend_snapshot_health_breakdown", {}),
        "trend_snapshot_ids": trend_snapshot_ids,
        "trend_snapshot_ids_csv": trend_summary.get("trend_snapshot_ids_csv", ""),
        "trend_snapshot_ids_hash": trend_summary.get("trend_snapshot_ids_hash", ""),
        "trend_snapshot_window_seconds": trend_summary.get("trend_snapshot_window_seconds"),
        "trend_sampling_mode": trend_summary.get("trend_sampling_mode", "disabled"),
        "trend_enabled": trend_summary.get("enabled", False),
        "schema_version": payload.get("payload_schema_version", OBSERVE_SUMMARY_SCHEMA_VERSION),
        "payload_type": "observe_summary",
        "status": payload.get("status", ""),
        "total_events": payload.get("kpis", {}).get("total_events", 0),
        "fallback_rate": payload.get("kpis", {}).get("fallback_rate", 0.0),
        "success_rate": payload.get("kpis", {}).get("success_rate", 0.0),
        "avg_confidence": payload.get("kpis", {}).get("avg_confidence", 0.0),
        "structural_drift_pct": payload.get("kpis", {}).get("structural_drift_pct", 0.0),
        "semantic_drift_pct": payload.get("kpis", {}).get("semantic_drift_pct", 0.0),
        "drift_structural_rate_pct": payload.get("drift", {}).get("structural_rate_pct", 0.0),
        "drift_semantic_rate_pct": payload.get("drift", {}).get("semantic_rate_pct", 0.0),
        "backlog_count": payload.get("escalation", {}).get("backlog_count", 0),
        "past_sla_count": payload.get("escalation", {}).get("past_sla_count", 0),
        "provider": payload.get("generated_query", {}).get("provider", None),
        "drift_window": payload.get("generated_query", {}).get("drift_window", 0),
        "structural_budget_pct": payload.get("generated_query", {}).get("structural_budget_pct", 0.0),
        "semantic_budget_pct": payload.get("generated_query", {}).get("semantic_budget_pct", 0.0),
        "top_escalations": payload.get("generated_query", {}).get("top_escalations", 0),
        "limit": payload.get("generated_query", {}).get("limit", 0),
        "trend_samples_requested": payload.get("generated_query", {}).get("trend_samples", 0),
        "trend_effective_samples": trend_summary.get("trend_effective_samples", 0),
        "trend_scope_payload_type": trend_scope_key.get("payload_type", "observe_summary"),
    }

    path = health_snapshot_log_path()
    try:
        with path.open("a", encoding="utf-8") as fh:
            fh.write(json.dumps(record, option=json.OPT_SORT_KEYS).decode())
            fh.write("\n")
    except OSError:
        return
    compact_health_snapshot_log()


def hash_health_payload(payload: dict[str, Any]) -> dict[str, str]:
    """Return a stable hash for a health payload while ignoring timestamp/signature fields."""
    payload_for_hash = {
        key: value for key, value in payload.items() if key not in {"generated_at_utc", "payload_signature"}
    }
    body = json.dumps(payload_for_hash, option=json.OPT_SORT_KEYS).decode()
    return {"algorithm": "sha256", "value": hashlib.sha256(body.encode()).hexdigest()}


def health_snapshot_log_path() -> Path:
    settings = ThegentSettings()
    raw = str(settings.health_snapshot_path) if settings.health_snapshot_path else ""
    path = Path(raw).expanduser() if raw else Path.home() / ".thegent" / "health-snapshots.jsonl"
    path.parent.mkdir(parents=True, exist_ok=True)
    return path


def health_snapshot_max_lines() -> int:
    settings = ThegentSettings()
    raw = str(settings.health_snapshot_max_lines)
    if not raw:
        return 5000
    try:
        value = int(raw)
    except ValueError:
        return 5000
    return max(100, value)


def compact_health_snapshot_log() -> None:
    path = health_snapshot_log_path()
    if not path.exists():
        return
    limit = health_snapshot_max_lines()
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


def _coerce_issue_types(value: Any) -> list[str]:
    if value is None:
        return []
    if isinstance(value, dict):
        return [str(v) for v in value]
    if isinstance(value, (list | tuple | set)):
        return [str(v) for v in value]
    return [str(value)]


def load_previous_health_snapshot(scope_key: dict[str, Any]) -> dict[str, Any] | None:
    path = health_snapshot_log_path()
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


def append_health_snapshot(payload: dict[str, Any], scope_key: dict[str, Any]) -> None:
    path = health_snapshot_log_path()
    issue_types: list[str] = []
    if payload.get("payload_type") == "session_contract_health_report":
        issue_types = sorted([str(k) for k in (payload.get("issue_counts") or {})])
    else:
        seen: set[str] = set()
        for row in payload.get("blocked_sessions", []) or []:
            for issue in _coerce_issue_types(row.get("issues", [])):
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
    compact_health_snapshot_log()
