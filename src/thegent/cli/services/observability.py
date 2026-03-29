"""Observability service helpers split out of CLI impl."""

import hashlib
import orjson as json
from collections.abc import Callable
from datetime import UTC, datetime
from pathlib import Path
from typing import Any

from thegent.config import ThegentSettings


def get_server_meta_impl(
    *,
    health_payload_schema_version: str,
    health_payload_types: tuple[str, ...],
    observe_summary_payload_schema_version: str,
    observe_summary_payload_types: tuple[str, ...],
    health_policy_profiles: list[str],
) -> dict[str, Any]:
    """Build server metadata payload for thegent://meta."""
    from thegent.contracts.registry import CONTRACT_SCHEMA_VERSION
    from thegent.models.catalog import ROUTE_SCHEMA_VERSION
    from thegent.operations import Operation
    from thegent.orchestration_modes import MultiAgentMode
    from thegent.output_parser import OUTPUT_PARSER_SCHEMA_VERSION

    return {
        "server": "thegent",
        "version": "1.0",
        "capabilities": ["tools", "resources", "prompts", "progress", "elicitation", "event_store"],
        "health_payload_schema_version": health_payload_schema_version,
        "health_payload_types": list(health_payload_types),
        "observe_summary_payload_schema_version": observe_summary_payload_schema_version,
        "observe_summary_payload_types": list(observe_summary_payload_types),
        "health_policy_profiles": health_policy_profiles,
        "output_parser_schema_version": OUTPUT_PARSER_SCHEMA_VERSION,
        "route_schema_version": ROUTE_SCHEMA_VERSION,
        "contract_schema_version": CONTRACT_SCHEMA_VERSION,
        "operations": [op.value for op in Operation],
        "orchestration_modes": [m.value for m in MultiAgentMode],
    }


def sweep_impl(
    *,
    drift_window: int,
    structural_budget: float,
    semantic_budget: float,
    include_audit: bool,
    update_calibration_fn: Callable[[], dict[str, Any]],
) -> dict[str, Any]:
    """Policy drift sweep service."""
    from thegent.contracts.telemetry import ContractTelemetry
    from thegent.execution import Auditor, EscalationQueue, RunRegistry

    settings = ThegentSettings()
    session_dir = Path(settings.session_dir).expanduser().resolve()

    ct = ContractTelemetry(session_dir)
    drift_issues = ct.detect_drift(window_size=drift_window)
    budget = ct.get_drift_budget_status(
        structural_budget_pct=structural_budget,
        semantic_budget_pct=semantic_budget,
    )
    if not budget["within_budget"]:
        drift_issues.append(
            f"Drift budget exceeded: structural {budget['structural_rate_pct']}% "
            f"(budget {budget['structural_budget_pct']}%), semantic {budget['semantic_rate_pct']}% "
            f"(budget {budget['semantic_budget_pct']}%)"
        )

    queue = EscalationQueue(session_dir)
    past_sla_items = queue.list_pending(past_sla_only=True, limit=100)

    if past_sla_items and bool(getattr(settings, "escalation_sla_breach_alert", False)):
        import logging

        _sweep_log = logging.getLogger(__name__)
        _sweep_log.warning(
            "Escalation SLA breach: %d item(s) past SLA. Run: thegent govern escalate list --past-sla",
            len(past_sla_items),
        )

    audit_result: dict[str, Any] | None = None
    if include_audit:
        registry = RunRegistry(session_dir)
        auditor = Auditor(registry.registry_path)
        audit_result = auditor.verify_registry()

    has_issues = bool(drift_issues) or bool(past_sla_items)
    if include_audit and audit_result and audit_result.get("status") not in ("passed", "empty"):
        has_issues = True

    cal_results = update_calibration_fn()
    return {
        "drift_issues": drift_issues,
        "drift_budget": budget,
        "past_sla_count": len(past_sla_items),
        "past_sla_items": past_sla_items,
        "calibration": cal_results,
        "audit": audit_result,
        "pass": not has_issues,
    }


def build_observe_summary_escalation(
    *,
    pending: list[dict[str, Any]],
    past_sla: list[dict[str, Any]],
    now: datetime,
    top_escalations: int,
) -> dict[str, Any]:
    """Build sorted escalation rows with SLA deltas for observe-summary payloads."""

    def _parse_utc(value: str | None) -> datetime | None:
        if not value:
            return None
        try:
            parsed = datetime.fromisoformat(value)
        except ValueError:
            if value.endswith("Z"):
                try:
                    parsed = datetime.fromisoformat(value)
                except ValueError:
                    return None
            else:
                return None
        if parsed.tzinfo is None:
            return parsed.replace(tzinfo=UTC)
        return parsed.astimezone(UTC)

    def _to_sla_delta(item: dict[str, Any]) -> dict[str, Any]:
        escalate_by = _parse_utc(item.get("escalate_by_utc"))
        blocked_at = _parse_utc(item.get("blocked_at_utc"))
        if escalate_by is None:
            return {
                "run_id": item.get("run_id"),
                "owner": item.get("owner"),
                "agent": item.get("agent"),
                "lane": item.get("lane"),
                "reason": item.get("reason"),
                "priority": item.get("priority", 0),
                "past_sla": bool(item.get("past_sla", False)),
                "sla_minutes": item.get("sla_minutes", 0),
                "blocked_at_utc": item.get("blocked_at_utc"),
                "escalate_by_utc": item.get("escalate_by_utc"),
                "minutes_overdue": None,
                "minutes_remaining": None,
                "blocked_to_now_seconds": None,
            }

        overdue = now - escalate_by
        overdue_seconds = overdue.total_seconds()
        blocked_to_now = now - blocked_at if blocked_at is not None else None
        return {
            "run_id": item.get("run_id"),
            "owner": item.get("owner"),
            "agent": item.get("agent"),
            "lane": item.get("lane"),
            "reason": item.get("reason"),
            "priority": item.get("priority", 0),
            "past_sla": bool(item.get("past_sla", False)),
            "sla_minutes": item.get("sla_minutes", 0),
            "blocked_at_utc": item.get("blocked_at_utc"),
            "escalate_by_utc": item.get("escalate_by_utc"),
            "minutes_overdue": round(overdue_seconds / 60.0, 2) if overdue_seconds > 0 else 0.0,
            "minutes_remaining": round(-overdue_seconds / 60.0, 2) if overdue_seconds <= 0 else 0.0,
            "blocked_to_now_seconds": round(blocked_to_now.total_seconds(), 2) if blocked_to_now is not None else None,
        }

    escalation_rows = sorted(
        (_to_sla_delta(item) for item in pending),
        key=lambda row: (
            0 if row["past_sla"] else 1,
            -int(row.get("priority", 0)),
            row.get("blocked_at_utc") or "",
        ),
    )
    return {
        "escalation_rows": escalation_rows,
        "top_rows": escalation_rows[: max(0, top_escalations)],
        "past_sla_count": len(past_sla),
    }


def build_observe_summary_trend(
    *,
    trend_samples: int | Any,
    provider: str | None,
    drift_window: int,
    structural_budget_pct: float,
    semantic_budget_pct: float,
    limit: int,
    top_escalations: int,
    now: datetime,
    kpis: dict[str, Any],
    budget: dict[str, Any],
    backlog_count: int,
    past_sla_count: int,
    build_scope_fn: Callable[..., dict[str, Any]],
    hash_scope_fn: Callable[[dict[str, Any]], str],
    load_snapshots_fn: Callable[[str, str, int], list[dict[str, Any]]],
    parse_timestamp_fn: Callable[[str | None], datetime | None],
    freshness_bucket_fn: Callable[..., str],
    classify_health_fn: Callable[..., dict[str, Any]],
) -> dict[str, Any]:
    """Build trend summary and snapshot scope metadata for observe-summary payloads."""
    try:
        trend_samples_requested = int(trend_samples)
    except (TypeError, ValueError):
        trend_samples_requested = 0
    trend_samples_requested = max(trend_samples_requested, 0)

    trend_effective_samples = trend_samples_requested if trend_samples_requested > 1 else 0
    trend_sampling_mode = "enabled" if trend_effective_samples > 0 else "disabled"
    trend_previous_samples_requested = max(0, trend_effective_samples - 1)
    trend_scope_key = build_scope_fn(
        provider=provider,
        drift_window=drift_window,
        structural_budget_pct=structural_budget_pct,
        semantic_budget_pct=semantic_budget_pct,
        limit=limit,
        top_escalations=top_escalations,
    )
    trend_scope_signature = hash_scope_fn(trend_scope_key)
    trend_scope_key_json = json.dumps(trend_scope_key, option=json.OPT_SORT_KEYS).decode()
    trend_records: list[dict[str, Any]] = []
    if trend_previous_samples_requested:
        trend_records = load_snapshots_fn(
            trend_scope_signature,
            trend_scope_key_json,
            trend_previous_samples_requested,
        )

    trend_snapshot_ids = [
        str((record or {}).get("captured_at_utc", ""))
        for record in trend_records
        if str((record or {}).get("captured_at_utc", ""))
    ]
    trend_snapshot_ids_csv = ", ".join(trend_snapshot_ids)
    trend_snapshot_ids_hash = hashlib.sha256(trend_snapshot_ids_csv.encode("utf-8")).hexdigest()

    baseline_snapshot = trend_records[-1] if trend_records else None
    baseline_available = bool(trend_previous_samples_requested > 0 and trend_records)
    baseline_captured_at_utc = baseline_snapshot.get("captured_at_utc") if baseline_snapshot else None
    trend_snapshot_expected_count = trend_previous_samples_requested
    trend_snapshot_deficit = max(0, trend_snapshot_expected_count - len(trend_records))
    trend_snapshot_invalid_timestamps = 0
    parsed_snapshot_timestamps: list[datetime] = []
    for record in trend_records:
        parsed = parse_timestamp_fn(record.get("captured_at_utc"))
        if parsed is None:
            trend_snapshot_invalid_timestamps += 1
            continue
        parsed_snapshot_timestamps.append(parsed)

    trend_snapshot_interval_seconds_avg = None
    trend_snapshot_interval_seconds_min = None
    trend_snapshot_interval_seconds_max = None
    trend_snapshot_gap_count = 0
    trend_snapshot_window_seconds = None
    trend_snapshot_coverage_pct = None

    if trend_snapshot_expected_count > 0:
        trend_snapshot_coverage_pct = round((len(trend_records) / trend_snapshot_expected_count) * 100.0, 6)

    if len(parsed_snapshot_timestamps) >= 2:
        ordered = sorted(parsed_snapshot_timestamps)
        diffs: list[int] = []
        for idx in range(1, len(ordered)):
            diffs.append(int((ordered[idx] - ordered[idx - 1]).total_seconds()))
        if diffs:
            trend_snapshot_interval_seconds_avg = int(sum(diffs) / len(diffs))
            trend_snapshot_interval_seconds_min = min(diffs)
            trend_snapshot_interval_seconds_max = max(diffs)
            trend_snapshot_gap_count = len(diffs)
            trend_snapshot_window_seconds = int((ordered[-1] - ordered[0]).total_seconds())

    latest_snapshot = trend_records[0] if trend_records else None
    trend_snapshot_freshness_seconds = None
    if latest_snapshot:
        latest_ts = parse_timestamp_fn(latest_snapshot.get("captured_at_utc"))
        if latest_ts is not None:
            trend_snapshot_freshness_seconds = int((now - latest_ts).total_seconds())
    trend_snapshot_freshness_bucket = freshness_bucket_fn(
        trend_snapshot_freshness_seconds,
        fresh_seconds=3600,
        warm_seconds=21600,
        stale_seconds=86400,
    )

    def _delta(current: Any, baseline_value: Any) -> Any:
        if not baseline_available:
            return None
        if current is None or baseline_value is None:
            return None
        try:
            return float(current) - float(baseline_value)
        except (TypeError, ValueError):
            return None

    baseline_kpis: dict[str, Any] = {}
    if baseline_snapshot:
        baseline_kpis = {
            "total_events": baseline_snapshot.get("total_events"),
            "fallback_rate": baseline_snapshot.get("fallback_rate"),
            "success_rate": baseline_snapshot.get("success_rate"),
            "avg_confidence": baseline_snapshot.get("avg_confidence"),
            "structural_drift_pct": baseline_snapshot.get("structural_drift_pct"),
            "semantic_drift_pct": baseline_snapshot.get("semantic_drift_pct"),
        }
        baseline_drifts = {
            "structural_rate_pct": baseline_snapshot.get("drift_structural_rate_pct"),
            "semantic_rate_pct": baseline_snapshot.get("drift_semantic_rate_pct"),
        }
        baseline_escalation = {
            "backlog_count": baseline_snapshot.get("backlog_count"),
            "past_sla_count": baseline_snapshot.get("past_sla_count"),
        }
    else:
        baseline_drifts = {}
        baseline_escalation = {}

    trend_health = classify_health_fn(
        enabled=trend_sampling_mode == "enabled",
        baseline_available=baseline_available,
        trend_snapshot_coverage_pct=trend_snapshot_coverage_pct,
        trend_snapshot_deficit=trend_snapshot_deficit,
        trend_snapshot_invalid_timestamps=trend_snapshot_invalid_timestamps,
        trend_snapshot_freshness_bucket=trend_snapshot_freshness_bucket,
        trend_snapshot_gap_count=trend_snapshot_gap_count,
        trend_sampling_mode=trend_sampling_mode,
    )

    total_events = float(kpis.get("total", 0.0))
    fallback_rate = float(kpis.get("fallback_rate", 0.0))
    success_rate = float(kpis.get("success_rate", 0.0))
    avg_confidence = float(kpis.get("avg_confidence", 0.0))
    structural_drift_pct = float(kpis.get("structural_drift_pct", 0.0))
    semantic_drift_pct = float(kpis.get("semantic_drift_pct", 0.0))
    drift_structural_rate_pct = float(budget.get("structural_rate_pct", 0.0))
    drift_semantic_rate_pct = float(budget.get("semantic_rate_pct", 0.0))

    trend_snapshot_recommendations = trend_health.get("trend_snapshot_recommendations", [])
    trend_summary = {
        "enabled": trend_sampling_mode == "enabled",
        "trend_sampling_mode": trend_sampling_mode,
        "trend_samples_requested": trend_samples_requested,
        "trend_effective_samples": trend_effective_samples,
        "history_sample_count": len(trend_records),
        "trend_previous_samples_requested": trend_previous_samples_requested,
        "trend_snapshot_expected_count": trend_snapshot_expected_count,
        "trend_snapshot_deficit": trend_snapshot_deficit,
        "trend_snapshot_interval_seconds_avg": trend_snapshot_interval_seconds_avg,
        "trend_snapshot_interval_seconds_min": trend_snapshot_interval_seconds_min,
        "trend_snapshot_interval_seconds_max": trend_snapshot_interval_seconds_max,
        "trend_snapshot_gap_count": trend_snapshot_gap_count,
        "trend_snapshot_invalid_timestamps": trend_snapshot_invalid_timestamps,
        "trend_snapshot_coverage_pct": trend_snapshot_coverage_pct,
        "trend_snapshot_freshness_seconds": trend_snapshot_freshness_seconds,
        "trend_snapshot_freshness_bucket": trend_snapshot_freshness_bucket,
        "trend_snapshot_ids": trend_snapshot_ids,
        "trend_snapshot_ids_csv": trend_snapshot_ids_csv,
        "trend_snapshot_ids_hash": trend_snapshot_ids_hash,
        "trend_snapshot_window_seconds": trend_snapshot_window_seconds,
        "baseline_available": baseline_available,
        "baseline_captured_at_utc": baseline_captured_at_utc,
        "trend_snapshot_health": trend_health.get("trend_snapshot_health"),
        "trend_snapshot_health_score": trend_health.get("trend_snapshot_health_score"),
        "trend_snapshot_health_breakdown": trend_health.get("trend_snapshot_health_breakdown", {}),
        "trend_snapshot_recommendations": trend_snapshot_recommendations,
        "trend_snapshot_recommendation_count": len(trend_snapshot_recommendations),
        "trend_snapshot_recommendations_csv": ", ".join(trend_snapshot_recommendations),
        "total_events_delta": _delta(total_events, baseline_kpis.get("total_events")),
        "fallback_rate_delta": _delta(fallback_rate, baseline_kpis.get("fallback_rate")),
        "success_rate_delta": _delta(success_rate, baseline_kpis.get("success_rate")),
        "avg_confidence_delta": _delta(avg_confidence, baseline_kpis.get("avg_confidence")),
        "structural_drift_pct_delta": _delta(structural_drift_pct, baseline_kpis.get("structural_drift_pct")),
        "semantic_drift_pct_delta": _delta(semantic_drift_pct, baseline_kpis.get("semantic_drift_pct")),
        "drift_structural_rate_pct_delta": _delta(
            drift_structural_rate_pct, baseline_drifts.get("structural_rate_pct")
        ),
        "drift_semantic_rate_pct_delta": _delta(drift_semantic_rate_pct, baseline_drifts.get("semantic_rate_pct")),
        "backlog_count_delta": _delta(backlog_count, baseline_escalation.get("backlog_count")),
        "past_sla_count_delta": _delta(past_sla_count, baseline_escalation.get("past_sla_count")),
        "scope_signature": trend_scope_signature,
        "scope_key_json": trend_scope_key_json,
    }

    return {
        "trend_summary": trend_summary,
        "trend_scope_key": trend_scope_key,
        "trend_scope_signature": trend_scope_signature,
        "trend_scope_key_json": trend_scope_key_json,
        "trend_snapshot_ids": trend_snapshot_ids,
        "trend_samples_requested": trend_samples_requested,
    }
