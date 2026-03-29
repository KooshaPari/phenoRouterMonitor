"""AgilePlus core loop orchestrator.

State machine: IDLE -> SCANNING -> ANALYZING -> PLANNING -> DEPLOYING -> VERIFYING -> COMMITTING -> IDLE

The loop composes scanner, analyzer, planner, deployer, verifier, and evidence
ledger to form a complete autonomous governance cycle.
"""

from __future__ import annotations

import logging
import signal
from datetime import UTC, datetime
from enum import StrEnum
from typing import TYPE_CHECKING, Any

from pydantic import BaseModel

if TYPE_CHECKING:
    from pathlib import Path

    from thegent.cost.aggregator_controller import CostController
    from thegent.governance.analyzer import HealthAnalyzer
    from thegent.governance.backlog import BacklogManager
    from thegent.governance.evidence_ledger import EvidenceLedger
    from thegent.governance.scanner import CodebaseScanner
    from thegent.planning.remediation_planner import RemediationPlanner

_log = logging.getLogger(__name__)


class CycleState(StrEnum):
    """AgilePlus cycle states."""

    IDLE = "idle"
    SCANNING = "scanning"
    ANALYZING = "analyzing"
    PLANNING = "planning"
    DEPLOYING = "deploying"
    VERIFYING = "verifying"
    COMMITTING = "committing"
    ERROR = "error"


class CycleResult(BaseModel):
    """Result of a single AgilePlus cycle."""

    cycle_id: str
    state: CycleState
    health_score: float = 0.0
    health_band: str = ""
    findings_count: int = 0
    tasks_planned: int = 0
    tasks_executed: int = 0
    tasks_verified: int = 0
    budget_used: int = 0
    budget_remaining: int = 0
    started_at: str = ""
    completed_at: str = ""
    error: str = ""


class AgilePlusLoop:
    """Orchestrates the complete 4X governance cycle.

    States:
    - IDLE: Health >= threshold, no action needed
    - SCANNING: Running dimension scans
    - ANALYZING: Prioritizing findings
    - PLANNING: Generating remediation DAG
    - DEPLOYING: Spawning agents
    - VERIFYING: Post-task verification
    - COMMITTING: Recording to evidence ledger
    """

    def __init__(
        self,
        project_dir: Path,
        health_targets_path: Path,
        health_threshold: float = 90.0,
        max_tasks_per_cycle: int = 10,
        max_rerolls: int = 2,
        lifecycle_mode: str = "soft",
    ) -> None:
        self.project_dir = project_dir
        self.health_targets_path = health_targets_path
        self.health_threshold = health_threshold
        self.max_tasks_per_cycle = max_tasks_per_cycle
        self.max_rerolls = max_rerolls
        self.lifecycle_mode = lifecycle_mode

        self._state = CycleState.IDLE
        self._cycle_id = ""
        self._shutdown_requested = False

        # Components (initialized lazily)
        self._scanner: CodebaseScanner | None = None
        self._analyzer: HealthAnalyzer | None = None
        self._planner: RemediationPlanner | None = None
        self._backlog: BacklogManager | None = None
        self._cost_controller: CostController | None = None
        self._evidence_ledger: EvidenceLedger | None = None

        # Register signal handlers for graceful shutdown
        signal.signal(signal.SIGINT, self._signal_handler)
        signal.signal(signal.SIGTERM, self._signal_handler)

    @property
    def state(self) -> CycleState:
        """Current cycle state."""
        return self._state

    @property
    def cycle_id(self) -> str:
        """Current cycle ID."""
        return self._cycle_id

    @property
    def shutdown_requested(self) -> bool:
        """True if a shutdown has been requested."""
        return self._shutdown_requested

    # ----------------------------------------------------------------------
    # public API
    # ----------------------------------------------------------------------

    def run_once(
        self,
        force: bool = False,
    ) -> CycleResult:
        """Run a single governance cycle.

        Returns a CycleResult with the outcome of the complete
        scan-analyze-plan-deploy-verify-commit loop.
        """
        import uuid

        self._cycle_id = f"cycle_{uuid.uuid4().hex[:8]}"
        started_at = datetime.now(UTC).isoformat()

        _log.info("Starting AgilePlus cycle %s (force=%s)", self._cycle_id, force)

        result = CycleResult(
            cycle_id=self._cycle_id,
            state=CycleState.IDLE,
            started_at=started_at,
        )

        try:
            # Initialize components
            self._init_components()

            # Record cycle start
            self._evidence_ledger.record(
                event_type="cycle_started",
                cycle_id=self._cycle_id,
                state="idle",
            )

            # Step 1: SCANNING
            self._state = CycleState.SCANNING
            result.state = CycleState.SCANNING
            scan_result = self._run_scan()

            # Check if already healthy (unless forced)
            health = self._compute_health(scan_result)
            result.health_score = health.score
            result.health_band = health.band.value if hasattr(health, "band") else ""

            if health.score >= self.health_threshold and not force:
                _log.info(
                    "Health score %s >= threshold %s, idling",
                    health.score,
                    self.health_threshold,
                )
                self._evidence_ledger.record(
                    event_type="cycle_completed",
                    cycle_id=self._cycle_id,
                    state="skipped",
                    health_score=health.score,
                )
                result.state = CycleState.IDLE
                result.completed_at = datetime.now(UTC).isoformat()
                return result

            # Step 2: ANALYZING
            self._state = CycleState.ANALYZING
            result.state = CycleState.ANALYZING
            findings = self._run_analysis(scan_result)
            result.findings_count = len(findings)

            # Step 3: PLANNING
            self._state = CycleState.PLANNING
            result.state = CycleState.PLANNING
            _usage = self._cost_controller.get_today_usage()
            budget_remaining = _usage.calls_limit - _usage.calls_used
            plan = self._run_planning(findings, budget_remaining)
            result.tasks_planned = len(plan.tasks) if plan else 0

            # Step 4: DEPLOYING
            self._state = CycleState.DEPLOYING
            result.state = CycleState.DEPLOYING
            deployment_result = self._run_deployment(plan, scan_result)
            result.tasks_executed = deployment_result.tasks_completed if deployment_result else 0

            # Step 5: VERIFYING
            self._state = CycleState.VERIFYING
            result.state = CycleState.VERIFYING
            verified = self._run_verification(deployment_result, scan_result)
            result.tasks_verified = verified

            # Step 6: COMMITTING
            self._state = CycleState.COMMITTING
            result.state = CycleState.COMMITTING
            self._run_commitment(deployment_result, verified)

            # Final state
            self._state = CycleState.IDLE
            result.state = CycleState.IDLE
            _final_usage = self._cost_controller.get_today_usage()
            result.budget_used = _final_usage.calls_used
            result.budget_remaining = _final_usage.calls_limit - _final_usage.calls_used

            _log.info(
                "Cycle %s completed: score=%.2f, tasks=%d/%d",
                self._cycle_id,
                result.health_score,
                result.tasks_verified,
                result.tasks_planned,
            )

        except Exception as e:
            _log.exception("Cycle %s failed", self._cycle_id)
            self._state = CycleState.ERROR
            result.state = CycleState.ERROR
            result.error = str(e)

        result.completed_at = datetime.now(UTC).isoformat()
        return result

    def run_continuous(
        self,
        interval_seconds: int = 300,
        max_cycles: int | None = None,
    ) -> list[CycleResult]:
        """Run continuous governance cycles.

        Args:
            interval_seconds: Seconds between cycles
            max_cycles: Maximum cycles to run (None = infinite)

        Returns:
            List of CycleResult for each completed cycle
        """
        import time

        results: list[CycleResult] = []
        cycles_run = 0

        _log.info(
            "Starting continuous mode: interval=%ds, max_cycles=%s",
            interval_seconds,
            max_cycles,
        )

        while not self._shutdown_requested:
            result = self.run_once()
            results.append(result)
            cycles_run += 1

            if max_cycles and cycles_run >= max_cycles:
                _log.info("Reached max_cycles=%d, stopping", max_cycles)
                break

            if self._shutdown_requested:
                _log.info("Shutdown requested, stopping")
                break

            # Check if healthy enough to sleep longer
            if result.health_score >= self.health_threshold:
                _log.info(
                    "Health score %s >= threshold, sleeping for %ds",
                    result.health_score,
                    interval_seconds * 2,
                )
                time.sleep(interval_seconds * 2)
            else:
                time.sleep(interval_seconds)

        return results

    def get_status(self) -> dict[str, Any]:
        """Get current status without running a cycle."""
        return {
            "state": self._state.value if self._state else "unknown",
            "cycle_id": self._cycle_id,
            "shutdown_requested": self._shutdown_requested,
        }

    def request_shutdown(self) -> None:
        """Request graceful shutdown."""
        _log.info("Shutdown requested")
        self._shutdown_requested = True

    # ----------------------------------------------------------------------
    # component initialization
    # ----------------------------------------------------------------------

    def _init_components(self) -> None:
        """Initialize all governance components."""
        from thegent.config import ThegentSettings
        from thegent.cost.aggregator_controller import CostController
        from thegent.governance.analyzer import HealthAnalyzer
        from thegent.governance.backlog import BacklogManager
        from thegent.governance.evidence_ledger import EvidenceLedger
        from thegent.governance.health_score import HealthScoreComputer
        from thegent.governance.scanner import CodebaseScanner
        from thegent.planning.remediation_planner import RemediationPlanner

        settings = ThegentSettings()

        # Scanner
        self._scanner = CodebaseScanner(
            project_dir=self.project_dir,
            session_dir=settings.session_dir,
        )

        # Health score computer (for computing composite score)
        self._health_computer = HealthScoreComputer(self.health_targets_path)

        # Analyzer
        self._analyzer = HealthAnalyzer(self.health_targets_path)

        # Planner
        self._planner = RemediationPlanner(self.health_targets_path)

        # Backlog
        self._backlog = BacklogManager(self.project_dir)

        # Cost controller
        self._cost_controller = CostController(
            session_dir=settings.session_dir,
            health_targets_path=self.health_targets_path,
        )

        # Evidence ledger
        self._evidence_ledger = EvidenceLedger(self.project_dir)

        _log.debug("Components initialized for cycle %s", self._cycle_id)

    # ----------------------------------------------------------------------
    # cycle steps
    # ----------------------------------------------------------------------

    def _run_scan(self) -> Any:
        """Execute codebase scanning."""
        _log.info("Running dimension scans")
        self._evidence_ledger.record(
            event_type="scan_started",
            cycle_id=self._cycle_id,
        )
        result = self._scanner.scan()
        self._evidence_ledger.record(
            event_type="scan_completed",
            cycle_id=self._cycle_id,
            dimensions=len(result.dimensions),
        )
        return result

    def _compute_health(self, scan_result: Any) -> Any:
        """Compute composite health score from scan result."""
        return self._health_computer.compute(scan_result)

    def _run_analysis(self, scan_result: Any) -> list[Any]:
        """Analyze scan results and prioritize findings."""
        _log.info("Analyzing scan results")
        self._evidence_ledger.record(
            event_type="analysis_started",
            cycle_id=self._cycle_id,
        )

        # Get pending backlog items first
        backlog_findings = self._backlog.get_pending()

        # Analyze new findings
        new_findings = self._analyzer.analyze(scan_result)

        # Combine with backlog (backlog items have priority)
        all_findings = list(backlog_findings) + new_findings

        # Limit to max_tasks_per_cycle
        limited_findings = all_findings[: self.max_tasks_per_cycle]

        self._evidence_ledger.record(
            event_type="analysis_completed",
            cycle_id=self._cycle_id,
            findings_count=len(limited_findings),
        )
        return limited_findings

    def _run_planning(self, findings: list[Any], budget_remaining: int) -> Any:
        """Generate remediation plan from findings."""
        _log.info("Planning remediation for %d findings", len(findings))
        self._evidence_ledger.record(
            event_type="plan_created",
            cycle_id=self._cycle_id,
            findings_count=len(findings),
        )

        plan = self._planner.plan(
            findings=findings,
            budget_remaining_calls=budget_remaining,
        )

        self._evidence_ledger.record(
            event_type="plan_completed",
            cycle_id=self._cycle_id,
            tasks_count=len(plan.tasks),
            estimated_calls=plan.total_estimated_calls,
        )
        return plan

    def _run_deployment(self, plan: Any, pre_scan: Any) -> Any:
        """Execute remediation plan."""
        from thegent.governance.agent_deployer import AgentDeployer

        _log.info("Deploying %d tasks", len(plan.tasks) if plan else 0)
        self._evidence_ledger.record(
            event_type="deployment_started",
            cycle_id=self._cycle_id,
            tasks_count=len(plan.tasks) if plan else 0,
        )

        assert self._cost_controller is not None, "_cost_controller not initialized"
        deployer = AgentDeployer(
            cost_controller=self._cost_controller,
            verification_gate=None,  # Will be used in verification step
            max_concurrent=3,
            lifecycle_mode=self.lifecycle_mode,
        )

        result = deployer.deploy(
            plan=plan,
            pre_scan=pre_scan,
            cycle_id=self._cycle_id,
        )

        self._evidence_ledger.record(
            event_type="deployment_completed",
            cycle_id=self._cycle_id,
            tasks_completed=result.tasks_completed,
            tasks_failed=result.tasks_failed,
        )
        return result

    def _run_verification(self, deployment_result: Any, pre_scan: Any) -> int:
        """Verify task executions."""
        from thegent.governance.verification_gate import VerificationGate

        if not deployment_result or not deployment_result.executions:
            return 0

        _log.info("Verifying %d task executions", len(deployment_result.executions))

        assert self._scanner is not None, "_scanner not initialized"
        gate = VerificationGate(
            scanner=self._scanner,
            health_computer=self._health_computer,
            max_rerolls=self.max_rerolls,
        )

        verified_count = 0
        for execution in deployment_result.executions:
            # Find the corresponding task
            task = None
            for t in getattr(deployment_result, "tasks", []):
                if t.task_id == execution.task_id:
                    task = t
                    break

            if task is None:
                continue

            verification = gate.verify_task(
                task=task,
                execution=execution,
                pre_scan=pre_scan,
            )

            if verification.verdict.value == "pass":
                verified_count += 1
                self._backlog.resolve(task.finding_id)
            elif verification.verdict.value == "fail":
                if gate.should_reroll(task.get("attempts", 0)):
                    self._backlog.increment_attempt(task.finding_id)
                else:
                    self._backlog.defer(task.finding_id, reason="max_rerolls_exceeded")
            elif verification.verdict.value == "regression":
                self._backlog.defer(task.finding_id, reason="regression_detected")

        self._evidence_ledger.record(
            event_type="verification_completed",
            cycle_id=self._cycle_id,
            verified_count=verified_count,
        )
        return verified_count

    def _run_commitment(self, deployment_result: Any, verified_count: int) -> None:
        """Record cycle completion to evidence ledger."""
        self._evidence_ledger.record(
            event_type="cycle_completed",
            cycle_id=self._cycle_id,
            tasks_executed=deployment_result.tasks_completed if deployment_result else 0,
            tasks_verified=verified_count,
        )

    # ----------------------------------------------------------------------
    # signal handling
    # ----------------------------------------------------------------------

    def _signal_handler(self, signum: int, frame: Any) -> None:
        """Handle shutdown signals gracefully."""
        _log.info("Received signal %d, requesting shutdown", signum)
        self._shutdown_requested = True
