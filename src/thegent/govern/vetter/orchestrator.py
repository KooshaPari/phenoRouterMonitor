"""VetterOrchestrator: runs checks in order, aggregates verdict, emits governance events.

WL-092 baseline behavior is preserved by default:
- failed checks -> rejected

Opt-in extensions:
- WL-093 escalation path (policy.escalate_on)
- WL-094 evidence append (always when evidence_store is configured)
- WL-096 revision queue path (run_context enable_revision_queue=true)

# @trace WL-092
# @trace WL-093
# @trace WL-094
# @trace WL-096
"""

from __future__ import annotations

import orjson as json
import time
from datetime import datetime, UTC
from pathlib import Path
from typing import Any

from thegent.govern.vetter.models import (
    VetterCheckResult,
    VetterPolicy,
    VetterResult,
    VetterVerdict,
)

_EU_AI_ACT = "EU-AI-ACT"
_EU_CRITICAL_CHECKS: frozenset[str] = frozenset({"safety", "quality_score"})


class VetterOrchestrator:
    """Orchestrates VetterCheck instances against agent output per a VetterPolicy.

    Constructor parameters:
      session_dir:      Path to the session directory where governance_events.jsonl is written.
      check_registry:   Mapping of check name -> VetterCheck instance (structural protocol).
      evidence_store:   Optional; not wired in WL-092 (reserved for WL-094).
      hitl_workflow:    Optional; not wired in WL-092 (reserved for WL-093).
      event_log:        Optional; not wired in WL-092 (reserved for WL-094).
      prompt_queue:     Optional; not wired in WL-092 (reserved for WL-096).
      federated_policy: Optional; not wired in WL-092 (reserved for WL-099).

    Fail fast, fail loudly. No silent error handling. No fallbacks.
    # @trace WL-092
    """

    def __init__(
        self,
        session_dir: Path,
        check_registry: dict[str, Any] | None = None,
        evidence_store: Any | None = None,
        hitl_workflow: Any | None = None,
        event_log: Any | None = None,
        prompt_queue: Any | None = None,
        federated_policy: Any | None = None,
    ) -> None:
        self.session_dir = session_dir
        self.check_registry: dict[str, Any] = check_registry if check_registry is not None else {}
        self.evidence_store = evidence_store
        self.hitl_workflow = hitl_workflow
        self.event_log = event_log
        self.prompt_queue = prompt_queue
        self.federated_policy = federated_policy
        self._revision_round_tracker: dict[str, int] = {}

    async def evaluate(
        self,
        result: Any,
        policy: VetterPolicy,
        run_context: dict[str, Any],
    ) -> VetterResult:
        """Run checks in order, aggregate verdict, emit vetter_decision event.

        Steps:
          1. Run each check named in policy.checks (in order) via the check_registry.
          2. On first failure, if policy.fail_fast is True: stop running further checks.
          3. Aggregate verdict:
               all passed  -> VetterVerdict.APPROVED
               any failed  -> VetterVerdict.REJECTED
          4. Emit a vetter_decision event to governance_events.jsonl in session_dir.
          5. Return VetterResult with verdict, check_results, duration_ms.

        Does NOT escalate (WL-093) or re-inject revision prompts (WL-096).
        Fail fast, fail loudly: check errors propagate; no silent catches.
        # @trace WL-092
        """
        run_id: str = str(run_context.get("run_id", "")).strip()
        start_ns = time.monotonic_ns()
        effective_policy = self._resolve_effective_policy(policy=policy, run_context=run_context)

        check_results: list[VetterCheckResult] = []

        output: str = getattr(result, "output", "") or ""

        for check_name in effective_policy.checks:
            check = self.check_registry[check_name]
            check_result: VetterCheckResult = await check.check(
                run_id=run_id,
                output=output,
                context=run_context,
            )
            check_results.append(check_result)
            if not check_result.passed and effective_policy.fail_fast:
                break

        failed_checks = [cr for cr in check_results if not cr.passed]
        failed_check_names = {cr.check_name for cr in failed_checks}
        any_failed = len(failed_checks) > 0

        should_escalate = any(name in failed_check_names for name in effective_policy.escalate_on)
        revision_enabled = bool(run_context.get("enable_revision_queue", False))
        current_round = self._resolve_current_revision_round(run_id=run_id, run_context=run_context)
        can_request_revision = revision_enabled and current_round < effective_policy.max_revision_rounds

        revision_prompt = self._build_revision_prompt(current_round, failed_checks) if can_request_revision else None

        if should_escalate:
            verdict = VetterVerdict.ESCALATED
        elif any_failed and can_request_revision:
            verdict = VetterVerdict.REVISION_REQUESTED
        elif any_failed and revision_enabled and current_round >= effective_policy.max_revision_rounds:
            verdict = VetterVerdict.ESCALATED if effective_policy.on_fail == "escalate" else VetterVerdict.REJECTED
        elif any_failed:
            verdict = VetterVerdict.REJECTED
        else:
            verdict = VetterVerdict.APPROVED

        duration_ms = int((time.monotonic_ns() - start_ns) // 1_000_000)

        self._emit_vetter_decision(
            run_id=run_id,
            verdict=verdict,
            check_results=check_results,
            duration_ms=duration_ms,
            run_context=run_context,
        )

        self._append_evidence(
            run_id=run_id,
            verdict=verdict,
            check_results=check_results,
            duration_ms=duration_ms,
            run_context=run_context,
        )

        escalation_reason: str | None = None
        if verdict == VetterVerdict.ESCALATED:
            escalation_reason = self._handle_escalation(
                run_id=run_id,
                output=output,
                policy=effective_policy,
                failed_check_names=sorted(failed_check_names),
                run_context=run_context,
            )

        if verdict == VetterVerdict.REVISION_REQUESTED:
            next_round = current_round + 1
            if run_id:
                self._revision_round_tracker[run_id] = next_round
            self._enqueue_revision_prompt(
                run_id=run_id,
                run_context=run_context,
                revision_prompt=revision_prompt or "",
                next_round=next_round,
            )
        elif run_id in self._revision_round_tracker:
            self._revision_round_tracker[run_id] = current_round

        return VetterResult(
            run_id=run_id,
            verdict=verdict,
            check_results=check_results,
            duration_ms=duration_ms,
            revision_prompt=revision_prompt,
            escalation_reason=escalation_reason,
        )

    def _resolve_effective_policy(self, policy: VetterPolicy, run_context: dict[str, Any]) -> VetterPolicy:
        effective_policy = policy
        federated_policy = self._resolve_federated_policy(run_context=run_context)
        jurisdiction_profile = str(run_context.get("jurisdiction_profile", "")).strip().upper()

        if federated_policy:
            merged = policy.model_dump(mode="python")
            merged.update(federated_policy)
            effective_policy = VetterPolicy.model_validate(merged)
            resolved_profile = str(federated_policy.get("jurisdiction_profile", "")).strip().upper()
            if resolved_profile:
                jurisdiction_profile = resolved_profile

        if jurisdiction_profile == _EU_AI_ACT:
            effective_policy = self._apply_eu_ai_act_overlay(effective_policy)

        return effective_policy

    def _resolve_federated_policy(self, run_context: dict[str, Any]) -> dict[str, Any]:
        if self.federated_policy is None:
            return {}

        required_keys = ("org", "project", "environment", "policy_id")
        if any(not run_context.get(key) for key in required_keys):
            return {}

        from thegent.governance.federation import PolicyNamespace

        namespace = PolicyNamespace(
            org=str(run_context["org"]),
            project=str(run_context["project"]),
            environment=str(run_context["environment"]),
        )
        resolved = self.federated_policy.resolve_policy(namespace, str(run_context["policy_id"]))
        if not isinstance(resolved, dict):
            raise TypeError("Federated policy manager resolve_policy() must return dict[str, Any]")
        return resolved

    def _apply_eu_ai_act_overlay(self, policy: VetterPolicy) -> VetterPolicy:
        critical_checks = _EU_CRITICAL_CHECKS.intersection(policy.checks)
        if not critical_checks:
            return policy

        merged_escalate_on = sorted(set(policy.escalate_on).union(critical_checks))
        if policy.on_fail == "escalate" and merged_escalate_on == policy.escalate_on:
            return policy

        policy_data = policy.model_dump(mode="python")
        policy_data["on_fail"] = "escalate"
        policy_data["escalate_on"] = merged_escalate_on
        return VetterPolicy.model_validate(policy_data)

    def _emit_vetter_decision(
        self,
        run_id: str,
        verdict: VetterVerdict,
        check_results: list[VetterCheckResult],
        duration_ms: int,
        run_context: dict[str, Any],
    ) -> None:
        """Append a vetter_decision event line to governance_events.jsonl.

        Appends (never overwrites) so multiple evaluate() calls accumulate events.
        File is created if it does not yet exist.
        # @trace WL-092
        """
        passed_checks = [cr.check_name for cr in check_results if cr.passed]
        failed_checks = [cr.check_name for cr in check_results if not cr.passed]
        session_id = str(run_context.get("session_id", "")).strip()

        self._write_event(
            event={
                "event_type": "vetter_decision",
                "timestamp": datetime.now(tz=UTC).isoformat(),
                "session_id": session_id,
                "run_id": run_id,
                "verdict": verdict.value,
                "passed_checks": passed_checks,
                "failed_checks": failed_checks,
                "duration_ms": duration_ms,
            },
        )

    def _write_event(self, event: dict[str, Any]) -> None:
        events_file: Path = self.session_dir / "governance_events.jsonl"
        self.session_dir.mkdir(parents=True, exist_ok=True)
        with events_file.open("a", encoding="utf-8") as fh:
            fh.write(json.dumps(event).decode() + "\n")
        if self.event_log is not None:
            self.event_log.emit(event)

    def _emit_vetter_escalation(
        self,
        run_id: str,
        escalation_lane: str,
        reason: str,
        run_context: dict[str, Any],
    ) -> None:
        session_id = str(run_context.get("session_id", "")).strip()
        self._write_event(
            event={
                "event_type": "vetter_escalation",
                "timestamp": datetime.now(tz=UTC).isoformat(),
                "session_id": session_id,
                "run_id": run_id,
                "status": "pending",
                "escalation_lane": escalation_lane,
                "reason": reason,
            },
        )

    def _append_evidence(
        self,
        run_id: str,
        verdict: VetterVerdict,
        check_results: list[VetterCheckResult],
        duration_ms: int,
        run_context: dict[str, Any],
    ) -> None:
        if self.evidence_store is None:
            return
        normalized_run_id = str(run_id).strip()
        if not normalized_run_id:
            raise RuntimeError("Evidence append requires non-empty run_id")
        session_id = str(run_context.get("session_id", "")).strip()
        if not session_id:
            raise RuntimeError("Evidence append requires non-empty session_id")
        passed_checks = [cr.check_name for cr in check_results if cr.passed]
        failed_checks = [cr.check_name for cr in check_results if not cr.passed]
        resource = f"session:{session_id}/run:{normalized_run_id}"
        self.evidence_store.append(
            kind="agent_decision",
            actor="vetter_orchestrator",
            resource=resource,
            payload={
                "verdict": verdict.value,
                "failed_checks": failed_checks,
                "passed_checks": passed_checks,
                "duration_ms": duration_ms,
            },
        )
        if hasattr(self.evidence_store, "verify_integrity") and not self.evidence_store.verify_integrity():
            raise RuntimeError("EvidenceStore hash-chain integrity failed after vetter append")

    def _resolve_current_revision_round(self, run_id: str, run_context: dict[str, Any]) -> int:
        raw_round = run_context.get("vetter_revision_round", 0)
        if isinstance(raw_round, bool) or not isinstance(raw_round, int):
            raise RuntimeError("Vetter revision round must be an integer >= 0")
        if raw_round < 0:
            raise RuntimeError("Vetter revision round must be an integer >= 0")
        tracked_round = self._revision_round_tracker.get(run_id, 0) if run_id else 0
        return max(raw_round, tracked_round)

    def _handle_escalation(
        self,
        run_id: str,
        output: str,
        policy: VetterPolicy,
        failed_check_names: list[str],
        run_context: dict[str, Any],
    ) -> str:
        if self.hitl_workflow is None:
            raise RuntimeError("Vetter escalation requires hitl_workflow when verdict is escalated")

        escalation_lane = policy.escalation_lane or "standard"
        canonical_escalate_on = sorted(set(policy.escalate_on))
        policy_escalate_on = ",".join(canonical_escalate_on) if canonical_escalate_on else "<none>"
        reason = (
            f"Vetter escalation requested: failed_checks={','.join(failed_check_names)} "
            f"policy_escalate_on={policy_escalate_on} policy_lane={escalation_lane}"
        )

        self._emit_vetter_escalation(
            run_id=run_id,
            escalation_lane=escalation_lane,
            reason=reason,
            run_context=run_context,
        )
        self.hitl_workflow.await_approval(
            run_id=run_id,
            policy="vetter_escalation",
            reason=reason,
            agent=run_context.get("agent", "unknown"),
            lane=run_context.get("lane", "standard"),
            owner=run_context.get("owner", "unknown"),
            environment=run_context.get("environment", "development"),
            checkpoint="post_execution",
            unified_diff=output or None,
        )
        return reason

    def _build_revision_prompt(
        self,
        current_round: int,
        failed_checks: list[VetterCheckResult],
    ) -> str:
        next_round = current_round + 1
        failed_ids = ", ".join(cr.check_name for cr in failed_checks) or "none"
        hints = [cr.message.strip() for cr in failed_checks if cr.message.strip()]
        hint_text = "\n".join(hints) if hints else "No revision hints provided."
        return (
            f"[VETTER REVISION REQUEST] Round: {next_round}\nFailed checks: {failed_ids}\nRevision hints:\n{hint_text}"
        )

    def _enqueue_revision_prompt(
        self,
        run_id: str,
        run_context: dict[str, Any],
        revision_prompt: str,
        next_round: int,
    ) -> None:
        if self.prompt_queue is None:
            raise RuntimeError("Vetter revision queue requires prompt_queue when revision is requested")
        normalized_run_id = str(run_id).strip()
        if not normalized_run_id:
            raise RuntimeError("Vetter revision queue requires non-empty run_id when revision is requested")
        self.prompt_queue.enqueue(
            revision_prompt,
            project_path=run_context.get("project_path"),
            metadata={
                "vetter_revision": True,
                "original_run_id": normalized_run_id,
                "round": next_round,
            },
        )
