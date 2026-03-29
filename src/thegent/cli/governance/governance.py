"""Governance and escalation service helpers for CLI commands."""

from __future__ import annotations

import asyncio
import orjson as json
from pathlib import Path
from types import SimpleNamespace
from typing import TYPE_CHECKING, Any

from pydantic import BaseModel, ConfigDict
from thegent.config import ThegentSettings

if TYPE_CHECKING:
    from thegent.govern.vetter.models import VetterCheckResult, VetterPolicy


def _session_dir() -> Any:
    settings = ThegentSettings()
    return settings.session_dir.expanduser().resolve()


def _resolve_session_dir(session: str | None) -> Path:
    if session is None:
        return _session_dir()
    return Path(session).expanduser().resolve()


def _resolve_run_record(session_dir: Path, run_id: str) -> dict[str, Any]:
    registry_path = session_dir / "run_registry.jsonl"
    if not registry_path.exists():
        raise FileNotFoundError(f"Run registry not found: {registry_path}")

    merged: dict[str, Any] = {}
    found = False
    with registry_path.open("r", encoding="utf-8") as registry_file:
        for line in registry_file:
            stripped = line.strip()
            if not stripped:
                continue
            payload = json.loads(stripped)
            if payload.get("run_id") != run_id:
                continue
            merged.update(payload)
            found = True

    if not found:
        raise ValueError(f"Run not found in registry: {run_id}")
    return merged


def _resolve_log_path(session_dir: Path, raw_path: str | None, stream_name: str) -> Path:
    if not raw_path:
        raise ValueError(f"Run metadata missing {stream_name}_path")

    log_path = Path(raw_path).expanduser()
    if not log_path.is_absolute():
        log_path = (session_dir / log_path).resolve()
    if not log_path.exists():
        raise FileNotFoundError(f"{stream_name} log not found: {log_path}")
    return log_path


def _vetter_contract_path(policy: str) -> Path:
    return Path(__file__).resolve().parents[4] / "contracts" / "vetter" / f"{policy}.json"


class _SchemaOutputModel(BaseModel):
    model_config = ConfigDict(extra="allow")


def _load_vetter_components() -> tuple[Any, Any, Any, Any, Any, Any, Any, Any]:
    from thegent.govern.vetter.checks import (
        DiffSizeVetterCheck,
        QualityScoreVetterCheck,
        RuffVetterCheck,
        SafetyVetterCheck,
        SchemaVetterCheck,
        TestPassVetterCheck,
    )
    from thegent.govern.vetter.models import VetterPolicy
    from thegent.govern.vetter.orchestrator import VetterOrchestrator

    return (
        DiffSizeVetterCheck,
        QualityScoreVetterCheck,
        RuffVetterCheck,
        SafetyVetterCheck,
        SchemaVetterCheck,
        TestPassVetterCheck,
        VetterPolicy,
        VetterOrchestrator,
    )


def _build_check_registry(policy: VetterPolicy) -> dict[str, Any]:
    (
        DiffSizeVetterCheck,
        QualityScoreVetterCheck,
        RuffVetterCheck,
        SafetyVetterCheck,
        SchemaVetterCheck,
        TestPassVetterCheck,
        _,
        _,
    ) = _load_vetter_components()
    judge_model = "gpt-4o-mini"
    required_names = set(policy.checks).union(policy.escalate_on)
    checks: dict[str, Any] = {}

    if "diff_size" in required_names:
        checks["diff_size"] = DiffSizeVetterCheck(
            max_lines_changed=int(policy.thresholds.get("diff_size", 500)),
            name="diff_size",
        )
    if "safety" in required_names:
        checks["safety"] = SafetyVetterCheck(name="safety")
    if "test_pass" in required_names:
        checks["test_pass"] = TestPassVetterCheck(name="test_pass")
    if "ruff" in required_names:
        checks["ruff"] = RuffVetterCheck(name="ruff")
    if "schema" in required_names:
        checks["schema"] = SchemaVetterCheck(schema_model=_SchemaOutputModel, name="schema")
    if "quality_score" in required_names:
        checks["quality_score"] = QualityScoreVetterCheck(
            name="quality_score",
            judge_model=judge_model,
            pass_threshold=float(policy.thresholds.get("quality_score", 0.75)),
            min_criterion_score=int(policy.thresholds.get("min_criterion_score", 1)),
        )

    return checks


def _resolve_policy_bundle(policy: str) -> tuple[VetterPolicy, dict[str, Any]]:
    from thegent.govern.vetter.models import VetterPolicy

    contract_path = _vetter_contract_path(policy)
    if not contract_path.exists():
        raise FileNotFoundError(f"Vetter policy contract not found: {contract_path}")

    raw_policy = json.loads(contract_path.read_text(encoding="utf-8"))
    policy_obj = VetterPolicy.model_validate(raw_policy)
    check_registry = _build_check_registry(policy_obj)
    missing = [check_name for check_name in policy_obj.checks if check_name not in check_registry]
    if missing:
        raise ValueError(f"Vetter policy references unsupported check(s): {', '.join(sorted(missing))}")
    return policy_obj, check_registry


def _serialize_checks(checks: list[VetterCheckResult]) -> list[dict[str, Any]]:
    return [check.model_dump(mode="json") for check in checks]


def escalate_add_impl(
    run_id: str,
    reason: str,
    sla_minutes: int = 30,
    owner: str | None = None,
    agent: str | None = None,
    lane: str = "standard",
    priority: int = 0,
) -> None:
    """WP-3008: Add a blocked run to the escalation queue."""
    from thegent.execution import EscalationQueue

    queue = EscalationQueue(_session_dir())
    queue.add(
        run_id=run_id,
        reason=reason,
        sla_minutes=sla_minutes,
        owner=owner,
        agent=agent,
        lane=lane,
        priority=priority,
    )


def escalate_approve_impl(run_id: str) -> bool:
    """WP-3008: Approve an escalation, marking it as approved in the queue (G-GP-05)."""
    from thegent.execution import EscalationQueue

    queue = EscalationQueue(_session_dir())
    return queue.resolve(run_id=run_id, resolution="approved")


def escalate_list_impl(past_sla_only: bool = False, limit: int = 50) -> list[dict[str, Any]]:
    """WP-3008: List escalation queue items (blocked runs with SLA)."""
    from thegent.execution import EscalationQueue

    queue = EscalationQueue(_session_dir())
    return queue.list_pending(past_sla_only=past_sla_only, limit=limit)


def escalate_resolve_impl(run_id: str, resolution: str = "resolved") -> bool:
    """WP-3008: Mark an escalation item as resolved."""
    from thegent.execution import EscalationQueue

    queue = EscalationQueue(_session_dir())
    return queue.resolve(run_id=run_id, resolution=resolution)


def govern_approve_impl(run_id: str, reason: str | None = None) -> dict[str, Any]:
    """WL-019-B: Approve a HITL-blocked run, updating governance_events.jsonl to 'approved'."""
    from thegent.governance.hitl import HITLApprovalWorkflow

    workflow = HITLApprovalWorkflow(_session_dir())
    return workflow.approve(run_id=run_id, reason=reason)


def govern_reject_impl(run_id: str, reason: str | None = None) -> dict[str, Any]:
    """WL-019-B: Reject a HITL-blocked run, updating governance_events.jsonl to 'rejected'."""
    from thegent.governance.hitl import HITLApprovalWorkflow

    workflow = HITLApprovalWorkflow(_session_dir())
    return workflow.reject(run_id=run_id, reason=reason)


def govern_list_pending_impl() -> list[dict[str, Any]]:
    """WL-019-B: List all pending HITL approval events from governance_events.jsonl."""
    from thegent.governance.hitl import HITLApprovalWorkflow

    workflow = HITLApprovalWorkflow(_session_dir())
    return workflow.list_pending()


def govern_get_pending_approval_impl(
    *,
    run_id: str,
    session: str | None = None,
) -> dict[str, Any]:
    """WL-100: Return a single pending approval event for a run."""
    from thegent.governance.hitl import HITLApprovalWorkflow

    workflow = HITLApprovalWorkflow(_resolve_session_dir(session))
    pending = workflow.list_pending()
    for event in pending:
        if event.get("run_id") == run_id:
            return event
    raise ValueError(f"No pending HITL approval found for run_id={run_id!r}")


def govern_vet_impl(
    *,
    run_id: str,
    policy: str = "default",
    session: str | None = None,
    dry_run: bool = False,
    org: str | None = None,
    project: str | None = None,
    environment: str | None = None,
    policy_id: str | None = None,
) -> dict[str, Any]:
    """WL-098: Vet a recorded run by run_id using VetterOrchestrator."""
    session_dir = _resolve_session_dir(session)
    run_record = _resolve_run_record(session_dir, run_id)

    stdout_path = _resolve_log_path(session_dir, run_record.get("stdout_path"), "stdout")
    stderr_path = _resolve_log_path(session_dir, run_record.get("stderr_path"), "stderr")
    stdout_text = stdout_path.read_text(encoding="utf-8")
    stderr_text = stderr_path.read_text(encoding="utf-8")

    policy_config, check_registry = _resolve_policy_bundle(policy)
    check_names = list(policy_config.checks)

    payload: dict[str, Any] = {
        "run_id": run_id,
        "policy": policy,
        "session_dir": str(session_dir),
        "stdout_path": str(stdout_path),
        "stderr_path": str(stderr_path),
        "dry_run": dry_run,
        "check_names": check_names,
    }
    if dry_run:
        payload["verdict"] = "dry_run"
        payload["checks"] = [
            {
                "check_name": check_name,
                "passed": None,
                "message": "not executed (--dry-run)",
                "metadata": {},
            }
            for check_name in check_names
        ]
        return payload

    (
        _,
        _,
        _,
        _,
        _,
        _,
        _,
        VetterOrchestrator,
    ) = _load_vetter_components()
    orchestrator = VetterOrchestrator(session_dir=session_dir, check_registry=check_registry)
    run_context = {
        "run_id": run_id,
        "session_id": run_record.get("correlation_id", ""),
        "stdout": stdout_text,
        "stderr": stderr_text,
        "cwd": run_record.get("cwd"),
        "agent": run_record.get("agent"),
        "model": run_record.get("model"),
    }
    if org:
        run_context["org"] = org
    if project:
        run_context["project"] = project
    if environment:
        run_context["environment"] = environment
    if policy_id:
        run_context["policy_id"] = policy_id
    output_text = f"{stdout_text}\n{stderr_text}" if stderr_text else stdout_text
    result_obj = SimpleNamespace(output=output_text)
    vetter_result = asyncio.run(orchestrator.evaluate(result=result_obj, policy=policy_config, run_context=run_context))

    payload.update(
        {
            "verdict": vetter_result.verdict.value,
            "checks": _serialize_checks(vetter_result.check_results),
            "duration_ms": vetter_result.duration_ms,
            "revision_prompt": vetter_result.revision_prompt,
            "escalation_reason": vetter_result.escalation_reason,
        }
    )
    return payload
