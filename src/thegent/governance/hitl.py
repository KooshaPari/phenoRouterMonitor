"""Human-in-the-loop (HITL) coordination and approval workflows (WP-3001, WP-3008).

Traces to: G-GP-05, FR-GOV-HITL (WL-019)
"""

from __future__ import annotations

import hashlib
import orjson as json
import logging
import uuid
from datetime import UTC, datetime
from pathlib import Path
from typing import Any

from thegent.integrations.base import SerializableMixin

logger = logging.getLogger(__name__)


class HITLDecision(SerializableMixin):
    """Result of a HITL gate evaluation (WL-019-A)."""

    def __init__(
        self,
        required: bool,
        run_id: str,
        policy: str,
        reason: str = "",
        checkpoint: str = "pre_execution",
    ) -> None:
        self.required = required
        self.run_id = run_id
        self.policy = policy
        self.reason = reason
        self.checkpoint = checkpoint

    def __repr__(self) -> str:
        return f"HITLDecision(required={self.required}, run_id={self.run_id!r}, policy={self.policy!r})"


class RunContext:
    """Lightweight context object used by evaluate_hitl (WL-019-A)."""

    def __init__(
        self,
        run_id: str,
        agent: str,
        lane: str = "standard",
        confidence: float | None = None,
        owner: str = "unknown",
        prompt: str = "",
        environment: str = "development",
    ) -> None:
        self.run_id = run_id
        self.agent = agent
        self.lane = lane
        self.confidence = confidence
        self.owner = owner
        self.prompt = prompt
        self.environment = environment


class GovernanceEventLog:
    """Writes and reads governance events from governance_events.jsonl (WL-019-A)."""

    def __init__(self, session_dir: Path) -> None:
        self.session_dir = Path(session_dir)
        self.events_path = self.session_dir / "governance_events.jsonl"

    def emit(self, event: dict[str, Any]) -> None:
        """Append a governance event to the log."""
        self.session_dir.mkdir(parents=True, exist_ok=True)
        with self.events_path.open("a", encoding="utf-8") as f:
            f.write(json.dumps(event).decode() + "\n")

    def list_pending_approvals(self, run_id: str | None = None) -> list[dict[str, Any]]:
        """Return all await_approval events that are not yet resolved."""
        if not self.events_path.exists():
            return []
        items: list[dict[str, Any]] = []
        with self.events_path.open("r", encoding="utf-8") as f:
            for line in f:
                line = line.strip()
                if not line:
                    continue
                try:
                    ev = json.loads(line)
                    if ev.get("event_type") != "await_approval":
                        continue
                    if ev.get("status") != "pending":
                        continue
                    if run_id is not None and ev.get("run_id") != run_id:
                        continue
                    items.append(ev)
                except json.JSONDecodeError:
                    continue
        return items

    def update_status(self, run_id: str, new_status: str, reason: str | None = None) -> bool:
        """Update the status of a pending await_approval event. Returns True on success."""
        if not self.events_path.exists():
            return False
        lines = self.events_path.read_text(encoding="utf-8").splitlines()
        updated = False
        new_lines: list[str] = []
        for line in lines:
            if not line.strip():
                continue
            try:
                ev = json.loads(line)
                if (
                    ev.get("run_id") == run_id
                    and ev.get("event_type") == "await_approval"
                    and ev.get("status") == "pending"
                ):
                    ev["status"] = new_status
                    ev["resolved_at_utc"] = datetime.now(UTC).isoformat()
                    if reason is not None:
                        ev["resolution_reason"] = reason
                    updated = True
                new_lines.append(json.dumps(ev).decode())
            except json.JSONDecodeError:
                new_lines.append(line)
        if updated:
            self.events_path.write_text("\n".join(new_lines) + "\n", encoding="utf-8")
        return updated


class PolicyEngine:
    """Evaluates HITL gate decisions for a run context (WL-019-A).

    This is a standalone HITL-focused policy evaluator. The main execution
    PolicyEngine lives in thegent.execution and handles broader policy
    (cost, trust score, circuit breaker, OPA).  This class handles the
    require_human_approval checkpoint logic and emits await_approval events.
    """

    def __init__(self, settings: Any, session_dir: Path | None = None) -> None:
        self.settings = settings
        _raw_dir = session_dir or getattr(settings, "session_dir", None)
        self.session_dir = Path(_raw_dir).expanduser().resolve() if _raw_dir else Path.cwd()
        self._event_log = GovernanceEventLog(self.session_dir)

    def evaluate_hitl(self, run_context: RunContext) -> HITLDecision:
        """Evaluate whether the run requires human approval (G-GP-05, WL-019-A).

        When the require_human_approval policy fires:
        - Block run execution
        - Emit await_approval event to governance_events.jsonl
        - Return HITLDecision(required=True, run_id=..., policy=...)
        """
        hitl_enabled: bool = bool(getattr(self.settings, "hitl_enabled", False))
        checkpoints: list[str] = list(getattr(self.settings, "hitl_checkpoints", ["pre_execution"]))

        if not hitl_enabled or "pre_execution" not in checkpoints:
            return HITLDecision(
                required=False,
                run_id=run_context.run_id,
                policy="hitl_disabled",
                reason="HITL gate not enabled or pre_execution checkpoint not configured",
            )

        triggered, policy_name, reason = self._check_require_human_approval(run_context)

        if not triggered:
            return HITLDecision(
                required=False,
                run_id=run_context.run_id,
                policy="no_policy_match",
                reason="No require_human_approval policy fired",
            )

        event: dict[str, Any] = {
            "event_type": "await_approval",
            "event_id": f"hitl_{uuid.uuid4().hex[:8]}",
            "run_id": run_context.run_id,
            "policy": policy_name,
            "reason": reason,
            "checkpoint": "pre_execution",
            "agent": run_context.agent,
            "lane": run_context.lane,
            "owner": run_context.owner,
            "environment": run_context.environment,
            "status": "pending",
            "emitted_at_utc": datetime.now(UTC).isoformat(),
        }
        self._event_log.emit(event)
        logger.info(
            "HITL gate fired: run_id=%s policy=%s reason=%s",
            run_context.run_id,
            policy_name,
            reason,
        )

        return HITLDecision(
            required=True,
            run_id=run_context.run_id,
            policy=policy_name,
            reason=reason,
            checkpoint="pre_execution",
        )

    def _check_require_human_approval(self, ctx: RunContext) -> tuple[bool, str, str]:
        """Return (triggered, policy_name, reason).

        Policies that fire require_human_approval:
        1. Lane is 'critical' and confidence is below 0.9 (or absent)
        2. Environment is 'production' and confidence is absent
        3. Lane is 'recovery' and environment is 'production'
        """
        lane = ctx.lane or "standard"
        env = (ctx.environment or "development").lower()
        conf = ctx.confidence

        if lane == "critical" and (conf is None or conf < 0.9):
            missing = "absent" if conf is None else f"{conf:.2f}"
            return (
                True,
                "require_human_approval.critical_lane_low_confidence",
                f"Critical lane requires confidence >= 0.9 (current: {missing})",
            )

        if env == "production" and conf is None:
            return (
                True,
                "require_human_approval.production_no_confidence",
                "Production runs require a confidence score for HITL gate",
            )

        if lane == "recovery" and env == "production":
            return (
                True,
                "require_human_approval.production_recovery",
                "Recovery actions in production require human approval",
            )

        return False, "", ""

    def await_approval(
        self,
        run_id: str,
        policy: str,
        reason: str,
        agent: str = "unknown",
        lane: str = "standard",
        checkpoint: str = "pre_execution",
    ) -> dict[str, Any]:
        """Emit an await_approval event for a run requiring human approval.

        Args:
            run_id: Unique identifier for the run
            policy: Policy name that triggered the approval requirement
            reason: Human-readable reason for the approval requirement
            agent: Agent name executing the run
            lane: Execution lane (standard, critical, recovery)
            checkpoint: Checkpoint name (pre_execution, post_execution)

        Returns:
            Event dict that was emitted
        """
        from datetime import UTC, datetime
        import uuid

        event: dict[str, Any] = {
            "event_type": "await_approval",
            "event_id": f"hitl_{uuid.uuid4().hex[:8]}",
            "run_id": run_id,
            "policy": policy,
            "reason": reason,
            "checkpoint": checkpoint,
            "agent": agent,
            "lane": lane,
            "owner": "unknown",
            "environment": "development",
            "status": "pending",
            "emitted_at_utc": datetime.now(UTC).isoformat(),
        }
        self._event_log.emit(event)
        logger.info(
            "HITL gate fired: run_id=%s policy=%s reason=%s",
            run_id,
            policy,
            reason,
        )

        return event


class HITLApprovalWorkflow:
    """Implements the approve/reject workflow for HITL-blocked runs (WL-019-B).

    Reads pending approvals from governance_events.jsonl and updates their
    status to 'approved' or 'rejected', then signals continuation or
    cancellation of the blocked run.
    """

    def __init__(self, session_dir: Path) -> None:
        self.session_dir = Path(session_dir)
        self._event_log = GovernanceEventLog(self.session_dir)

    def approve(self, run_id: str, reason: str | None = None) -> dict[str, Any]:
        """Approve a HITL-blocked run.

        Updates governance_events.jsonl status to 'approved'.
        Returns a result dict with success status and run_id.
        Raises ValueError if no pending approval found for run_id.
        """
        pending = self._event_log.list_pending_approvals(run_id=run_id)
        if not pending:
            raise ValueError(f"No pending HITL approval found for run_id={run_id!r}")

        updated = self._event_log.update_status(run_id=run_id, new_status="approved", reason=reason)
        if not updated:
            raise RuntimeError(f"Failed to update governance_events.jsonl for run_id={run_id!r}")

        self._emit_resolution_event(run_id=run_id, resolution="approved", reason=reason)
        logger.info("HITL run_id=%s APPROVED (reason=%s)", run_id, reason)

        return {
            "success": True,
            "run_id": run_id,
            "resolution": "approved",
            "reason": reason,
            "resolved_at_utc": datetime.now(UTC).isoformat(),
        }

    def reject(self, run_id: str, reason: str | None = None) -> dict[str, Any]:
        """Reject a HITL-blocked run.

        Updates governance_events.jsonl status to 'rejected'.
        Returns a result dict with success status and run_id.
        Raises ValueError if no pending approval found for run_id.
        """
        pending = self._event_log.list_pending_approvals(run_id=run_id)
        if not pending:
            raise ValueError(f"No pending HITL approval found for run_id={run_id!r}")

        updated = self._event_log.update_status(run_id=run_id, new_status="rejected", reason=reason)
        if not updated:
            raise RuntimeError(f"Failed to update governance_events.jsonl for run_id={run_id!r}")

        self._emit_resolution_event(run_id=run_id, resolution="rejected", reason=reason)
        logger.info("HITL run_id=%s REJECTED (reason=%s)", run_id, reason)

        return {
            "success": True,
            "run_id": run_id,
            "resolution": "rejected",
            "reason": reason,
            "resolved_at_utc": datetime.now(UTC).isoformat(),
        }

    def list_pending(self) -> list[dict[str, Any]]:
        """Return all pending HITL approval events."""
        return self._event_log.list_pending_approvals()

    def await_approval(
        self,
        run_id: str,
        policy: str,
        reason: str,
        agent: str = "unknown",
        lane: str = "standard",
        owner: str = "unknown",
        environment: str = "development",
        checkpoint: str = "pre_execution",
        unified_diff: str | None = None,
    ) -> dict[str, Any]:
        """Emit an await_approval event for a run requiring human approval.

        Args:
            run_id: Unique identifier for the run
            policy: Policy name that triggered the approval requirement
            reason: Human-readable reason for the approval requirement
            agent: Agent name executing the run
            lane: Execution lane (standard, critical, recovery)
            owner: Owner of the run
            environment: Execution environment (development, production)
            checkpoint: Checkpoint name (pre_execution, post_execution)
            unified_diff: Optional unified diff string for code review

        Returns:
            Dict with event_id and run_id
        """
        event: dict[str, Any] = {
            "event_type": "await_approval",
            "event_id": f"hitl_{uuid.uuid4().hex[:8]}",
            "run_id": run_id,
            "policy": policy,
            "reason": reason,
            "checkpoint": checkpoint,
            "agent": agent,
            "lane": lane,
            "owner": owner,
            "environment": environment,
            "status": "pending",
            "emitted_at_utc": datetime.now(UTC).isoformat(),
            # WL-100: Keep schema stable for UI/CLI renderers.
            "unified_diff": unified_diff or "",
        }
        has_diff = bool(unified_diff)

        self._event_log.emit(event)
        logger.info(
            "HITL await_approval: run_id=%s policy=%s reason=%s has_diff=%s",
            run_id,
            policy,
            reason,
            has_diff,
        )

        return {
            "success": True,
            "event_id": event["event_id"],
            "run_id": run_id,
            "has_diff": has_diff,
            "emitted_at_utc": event["emitted_at_utc"],
        }

    def _emit_resolution_event(self, run_id: str, resolution: str, reason: str | None) -> None:
        payload = f"{run_id}:{resolution}:{reason or ''}:{datetime.now(UTC).isoformat()}"
        signature = hashlib.sha256(payload.encode()).hexdigest()
        event: dict[str, Any] = {
            "event_type": "hitl_resolution",
            "run_id": run_id,
            "resolution": resolution,
            "reason": reason,
            "provenance_signature": signature,
            "emitted_at_utc": datetime.now(UTC).isoformat(),
        }
        self._event_log.emit(event)


class HITLManager:
    """Manages human-in-the-loop signals and approvals (legacy compat, WP-3001)."""

    def __init__(self) -> None:
        self._approvals: dict[str, bool] = {}

    def request_approval(self, request_id: str, action: str, context: dict[str, Any]) -> str:
        """Issue an approval request and return its ID."""
        logger.info("HITL approval requested for action: %s", action)
        self._approvals[request_id] = False
        return request_id

    def approve(self, request_id: str) -> None:
        """Record an approval for a request."""
        if request_id in self._approvals:
            self._approvals[request_id] = True
            logger.info("HITL request %s approved", request_id)

    def is_approved(self, request_id: str) -> bool:
        """Check if a request has been approved."""
        return self._approvals.get(request_id, False)
