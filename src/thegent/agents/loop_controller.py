"""Lifecycle Loop Controller with Checker Agent oversight."""

import logging
import time
from collections.abc import Callable
from enum import StrEnum
from typing import Any

from pydantic import BaseModel
from rich.progress import BarColumn, Progress, SpinnerColumn, TaskProgressColumn, TextColumn, TimeRemainingColumn

# Type for verification callback: (task_id, result) -> Any
VerificationCallback = Callable[[str, Any], Any]

from thegent.agents.base import RunResult
from thegent.agents.checker import CheckerAgent, CheckerDecision, CheckerResult
from thegent.agents.presets import get_preset, match_preset
from thegent.agents.resilience import TransientAgentError, with_retry
from thegent.config import ThegentSettings

_log = logging.getLogger(__name__)


class LoopMode(StrEnum):
    """Lifecycle loop modes."""

    SOFT = "soft"
    HARD = "hard"


class LoopState(BaseModel):
    """Current state of a Lifecycle loop."""

    session_id: str
    iteration: int = 0
    last_response: str = ""
    mode: LoopMode = LoopMode.SOFT
    stopped: bool = False
    stop_reason: str | None = None
    last_cost_usd: float | None = None
    last_model: str | None = None


class LifecycleController:
    """Handles agent execution loops (Ralph Wiggum loops) with Checker Agent oversight."""

    def __init__(
        self,
        settings: ThegentSettings,
        worker_agent_name: str,
        checker_agent_name: str = "antigravity",
        mode: LoopMode = LoopMode.SOFT,
        max_iterations: int = 10,
        worker_model: str | None = None,
        task_id: str | None = None,
        verification_callback: VerificationCallback | None = None,
    ) -> None:
        self.settings = settings
        self.worker_agent_name = worker_agent_name
        self.worker_model = worker_model
        self.checker = CheckerAgent(settings, agent_name=checker_agent_name)
        self.task_id = task_id
        self.verification_callback = verification_callback
        self.mode = mode
        self.max_iterations = max_iterations
        self.state = LoopState(
            session_id=f"rw-{int(time.time())}",
            mode=mode,
        )

    @with_retry(max_attempts=3, min_wait=2.0, max_wait=60.0)
    def _run_worker_with_retry(self, current_prompt: str) -> dict[str, Any]:
        """Run worker agent; raises TransientAgentError on retryable failure."""
        # tach-ignore(agents should not architecturally depend on cli)
        from thegent.cli.commands.impl import run_impl

        result = run_impl(
            agent=None if self.worker_model else self.worker_agent_name,
            prompt=current_prompt,
            cd=self.settings.cwd,
            mode="write",
            timeout=self.settings.default_timeout,
            model=self.worker_model,
            provider=self.worker_agent_name if self.worker_model else None,
        )
        if result.get("exit_code") == 0:
            return result
        stdout = result.get("stdout", "")
        stderr = result.get("stderr", "")
        combined = f"{stdout}\n{stderr}"
        retryable = any(kw in combined.lower() for kw in ["rate limit", "timeout", "502", "503", "504", "transient"])
        if retryable:
            rr = RunResult(
                exit_code=result.get("exit_code", 1),
                stdout=stdout,
                stderr=stderr,
                timed_out=result.get("timed_out", False),
            )
            raise TransientAgentError(rr)
        return result

    def run_loop(
        self,
        initial_prompt: str,
        todo_spec: str,
        on_worker_output: Callable[[str], None] | None = None,
        on_progress: Callable[[int, int, str], None] | None = None,
    ) -> LoopState:
        """Execute the Lifecycle loop."""
        import json

        current_prompt = initial_prompt
        session_dir = self.settings.session_dir / self.state.session_id
        session_dir.mkdir(parents=True, exist_ok=True)

        _log.info("Starting Lifecycle loop session=%s mode=%s", self.state.session_id, self.mode)

        with Progress(
            SpinnerColumn(),
            TextColumn("[progress.description]{task.description}"),
            BarColumn(),
            TaskProgressColumn(),
            TextColumn("[dim]{task.fields[status]}[/dim]"),
            TimeRemainingColumn(),
            transient=False,
        ) as progress:
            task = progress.add_task(
                f"[cyan]Lifecycle loop ({self.mode.value} mode)",
                total=self.max_iterations,
                status="starting...",
            )

            while self.state.iteration < self.max_iterations and not self.state.stopped:
                self.state.iteration += 1
                _log.info("Iteration %d/%d", self.state.iteration, self.max_iterations)
                progress.update(
                    task,
                    completed=self.state.iteration,
                    status=f"iteration {self.state.iteration}/{self.max_iterations}",
                )

                if on_progress:
                    on_progress(self.state.iteration, self.max_iterations, f"Starting iteration {self.state.iteration}")

                # 1. Takeover/Stop Check (Phase 4: Human Takeover)
                # Check for external stop signal file
                if (session_dir / "STOP").exists():
                    self.state.stopped = True
                    self.state.stop_reason = "External stop signal received"
                    (session_dir / "STOP").unlink()
                    break

                # Check for external takeover prompt
                takeover_file = session_dir / "takeover.json"
                if takeover_file.exists():
                    try:
                        data = json.loads(takeover_file.read_text(encoding="utf-8"))
                        current_prompt = data["prompt"]
                        _log.info("Human takeover detected. Injecting prompt.")
                        if on_progress:
                            on_progress(
                                self.state.iteration, self.max_iterations, "Human takeover detected. Injecting prompt."
                            )
                        takeover_file.unlink()
                    except Exception as e:
                        _log.error("Failed to read takeover file: %s", e)

                # 2. Governance Pre-check (WP-3001)
                try:
                    from thegent.execution import PolicyEngine, RunMeta

                    pe = PolicyEngine(self.settings)
                    temp_run = RunMeta(
                        run_id=self.state.session_id,
                        agent=self.worker_agent_name,
                        prompt=current_prompt,
                        cwd=str(self.settings.cwd),
                        owner="loop_controller",
                        mode="write",
                        lane="standard",
                    )
                    effect, reason = pe.evaluate(temp_run)
                    gov_report = {
                        "status": effect,
                        "denials": [reason] if effect == "deny" else [],
                        "warnings": [reason] if effect == "warn" else [],
                    }
                except Exception as e:
                    _log.error("Governance pre-check failed: %s. Using default allow.", e)
                    effect, reason = "allow", str(e)
                    gov_report = {"status": "ok", "denials": [], "warnings": []}

                if on_progress:
                    on_progress(self.state.iteration, self.max_iterations, f"Policy check: {effect}")

                if effect == "deny":
                    self.state.stopped = True
                    self.state.stop_reason = f"Policy denied: {reason}"
                    from thegent.governance.escalation import EscalationQueue

                    eq = EscalationQueue(self.settings)
                    eq.add(run_id=self.state.session_id, reason=f"Policy denial: {reason}", priority=3)
                    break

                # 3. Run Worker Agent with Retry (WP-2002, tenacity)
                try:
                    if on_progress:
                        on_progress(
                            self.state.iteration, self.max_iterations, f"Running worker agent: {self.worker_agent_name}"
                        )
                    result = self._run_worker_with_retry(current_prompt)
                    if result.get("exit_code") != 0:
                        self.state.stopped = True
                        self.state.stop_reason = f"Worker failed (code {result.get('exit_code')})"
                        break
                except TransientAgentError as e:
                    _log.warning(
                        "Worker failed after retries: %s", e.result.stderr[:200] if e.result.stderr else str(e)
                    )
                    self.state.stopped = True
                    self.state.stop_reason = f"Worker failed after retries (code {e.result.exit_code})"
                    break
                except Exception as e:
                    _log.error("Worker execution failed: %s", e)
                    self.state.stopped = True
                    self.state.stop_reason = f"Worker exception: {e}"
                    break

                if self.state.stopped:
                    break

                stdout = result.get("stdout", "")
                stderr = result.get("stderr", "")
                combined = f"{stdout}\n{stderr}"
                self.state.last_response = combined
                self.state.last_cost_usd = result.get("cost_usd")
                self.state.last_model = result.get("model")

                if on_worker_output:
                    on_worker_output(combined)

                # 4. Soft Loop Check: allow human override/stop (SOFT mode)
                if self.mode == LoopMode.SOFT:
                    if "STOP" in combined:
                        self.state.stopped = True
                        self.state.stop_reason = "Human stop signal detected (SOFT mode)"
                        break

                # 5. Preset Prompt Routing (WP-1201 Phase 1)
                matched_preset = match_preset(combined)
                if matched_preset:
                    current_prompt = matched_preset.prompt
                    _log.info("Matched output to preset: %s", matched_preset.id)
                    if on_progress:
                        on_progress(self.state.iteration, self.max_iterations, f"Matched preset: {matched_preset.id}")
                    continue

                # 6. Invoke Checker Agent (WP-1201 Phase 2/3 - LLM Fallback)
                try:
                    # tach-ignore(agents should not architecturally depend on cli)
                    from thegent.cli.commands.impl import dag_status_impl

                    wbs_status = dag_status_impl(self.settings.cwd)

                    if on_progress:
                        on_progress(
                            self.state.iteration,
                            self.max_iterations,
                            f"Invoking checker agent: {self.checker.agent_name}",
                        )

                    decision_result = self.checker.decide(
                        governance_report=gov_report,
                        todo_spec=todo_spec,
                        wbs_status=wbs_status,
                        agent_response=combined,
                    )
                except Exception as e:
                    _log.error("Checker failed: %s. Using default CONTINUE.", e)
                    decision_result = CheckerResult(decision=CheckerDecision.CONTINUE, reason=str(e))

                _log.info("Checker decision: %s (reason: %s)", decision_result.decision, decision_result.reason)
                if on_progress:
                    on_progress(
                        self.state.iteration, self.max_iterations, f"Checker decision: {decision_result.decision}"
                    )

                if decision_result.decision == CheckerDecision.KILL:
                    self.state.stopped = True
                    self.state.stop_reason = f"Checker terminated: {decision_result.reason}"

                    if any(
                        kw in (decision_result.reason or "").lower() for kw in ["security", "cost", "risk", "policy"]
                    ):
                        from thegent.governance.escalation import EscalationPriority, EscalationQueue

                        eq = EscalationQueue(self.settings)
                        eq.escalate(
                            run_id=self.state.session_id,
                            prompt=current_prompt,
                            reason=f"Checker termination: {decision_result.reason}",
                            agent=self.worker_agent_name,
                            priority=EscalationPriority.NORMAL,
                        )
                    break

                if decision_result.decision == CheckerDecision.CONTINUE:
                    preset = get_preset("continue")
                    current_prompt = preset.prompt if preset else "Continue"
                elif decision_result.decision == CheckerDecision.RE_PROMPT:
                    current_prompt = decision_result.prompt or "Please continue."

            # Update progress to completed
            progress.update(task, status=f"done: {self.state.stop_reason or 'completed'}")

        if self.state.iteration >= self.max_iterations and not self.state.stopped:
            self.state.stop_reason = "Max iterations reached"

        if self.verification_callback and self.task_id:
            try:
                self.verification_callback(self.task_id, self.state)
            except Exception as e:
                _log.warning("Verification callback failed: %s", e)

        return self.state


# Aliases for backward compatibility and internal branding
LifecycleLoopController = LifecycleController
RalphWiggumController = LifecycleController
