"""Agent deployment from remediation DAG.

Walks the DAG topologically, groups independent tasks into ready batches,
checks budget, and spawns agents via the canonical entry point.
"""

from __future__ import annotations

import logging
from concurrent.futures import ThreadPoolExecutor, as_completed
from datetime import UTC, datetime
from graphlib import TopologicalSorter
from typing import Any, Protocol

from pydantic import BaseModel, Field

from thegent.agents.loop_controller import LifecycleController, LoopMode
from thegent.config import ThegentSettings

_log = logging.getLogger(__name__)


def _resolve_agent_for_task(agent_role: str, dimension: str) -> tuple[str, str]:
    """Resolve (agent, model) via task routing. Returns (agent_name, model_alias)."""
    from thegent.models.catalog import resolve_route

    _ = agent_role or "workhorse"
    fallback = resolve_route("minimax-m2.5", policy="cheapest") or resolve_route("claude-haiku-4-5", policy="cheapest")
    if fallback:
        return fallback[0], fallback[1]
    return "interactive_agent", "claude-sonnet-4-5"


class TaskExecutionResult(BaseModel):
    """Result of executing a single remediation task."""

    task_id: str
    run_id: str
    exit_code: int
    started_at: str = Field(default_factory=lambda: datetime.now(UTC).isoformat())
    completed_at: str = ""
    error: str = ""
    cost_usd: float | None = None
    agent_used: str = ""


class DeploymentResult(BaseModel):
    """Result of deploying a full remediation plan."""

    plan_id: str
    cycle_id: str
    tasks_attempted: int = 0
    tasks_completed: int = 0
    tasks_failed: int = 0
    total_calls_used: int = 0
    total_cost_usd: float = 0.0
    executions: list[TaskExecutionResult] = Field(default_factory=list)
    started_at: str = Field(default_factory=lambda: datetime.now(UTC).isoformat())
    completed_at: str = ""


class CostControllerProtocol(Protocol):
    """Protocol for cost controller."""

    def record_call(self, dimension: str, agent: str, *, cost_usd: float | None = None) -> None: ...
    def can_spawn(self, estimated_calls: int = 1) -> bool: ...
    def get_tier(self) -> Any: ...
    def calls_remaining(self) -> int: ...
    def get_today_usage(self) -> Any: ...


class VerificationGateProtocol(Protocol):
    """Protocol for verification gate."""

    def verify_task(self, task: Any, execution: Any, pre_scan: Any) -> Any: ...
    def should_reroll(self, attempts: int) -> bool: ...


class AgentDeployer:
    """Deploys remediation tasks from a DAG, respecting dependencies and budget."""

    def __init__(
        self,
        cost_controller: CostControllerProtocol,
        verification_gate: VerificationGateProtocol | None = None,
        max_concurrent: int = 3,
        lifecycle_mode: str = "soft",
        checker_agent_name: str = "antigravity",
    ) -> None:
        self.cost_controller = cost_controller
        self.verification_gate = verification_gate
        self.max_concurrent = max_concurrent
        self.lifecycle_mode = lifecycle_mode
        self.checker_agent_name = checker_agent_name
        self._settings = ThegentSettings()

    def deploy(
        self,
        plan: Any,
        pre_scan: Any,
        cycle_id: str,
    ) -> DeploymentResult:
        """Execute a full remediation plan.

        Walks DAG topologically, groups ready tasks into batches,
        spawns agents for each batch respecting max_concurrent.
        """
        if not plan.tasks:
            _log.info("No tasks in plan, skipping deployment")
            return DeploymentResult(
                plan_id=plan.plan_id,
                cycle_id=cycle_id,
            )

        dag = plan.dag_edges or {}
        ts = TopologicalSorter(dag)
        ts.prepare()

        result = DeploymentResult(
            plan_id=plan.plan_id,
            cycle_id=cycle_id,
        )

        # Track completed tasks and their results
        completed: dict[str, TaskExecutionResult] = {}
        pending = {t.task_id: t for t in plan.tasks}

        while pending:
            # Find ready tasks (all dependencies satisfied)
            ready: list[Any] = []
            for _, task in list(pending.items()):
                deps = task.dependencies or []
                if all(d in completed for d in deps):
                    ready.append(task)

            # Submit ready tasks in parallel (up to max_concurrent)
            batch_size = min(self.max_concurrent, len(ready))
            tasks_to_run = ready[:batch_size]
            with ThreadPoolExecutor(max_workers=batch_size) as executor:
                futures = {}
                for task in tasks_to_run:
                    if not self.cost_controller.can_spawn(task.estimated_cost_calls):
                        _log.warning(
                            "Budget exhausted, deferring task %s",
                            task.task_id,
                        )
                        continue
                    fut = executor.submit(self._execute_task, task, cycle_id)
                    futures[fut] = task
                    result.tasks_attempted += 1
                    del pending[task.task_id]

                for fut in as_completed(futures):
                    task = futures[fut]
                    try:
                        execution = fut.result()
                    except Exception as e:
                        _log.exception("Task %s raised exception", task.task_id)
                        execution = TaskExecutionResult(
                            task_id=task.task_id,
                            run_id="failed",
                            exit_code=1,
                            error=str(e),
                        )
                    execution.completed_at = datetime.now(UTC).isoformat()

                    if execution.exit_code == 0:
                        result.tasks_completed += 1
                        result.total_calls_used += 1
                        if execution.cost_usd is not None:
                            result.total_cost_usd += execution.cost_usd
                        self.cost_controller.record_call(
                            task.dimension or task.task_id,
                            execution.agent_used or "unknown",
                            cost_usd=execution.cost_usd,
                        )
                    else:
                        result.tasks_failed += 1
                        _log.error(
                            "Task %s failed with exit code %d",
                            execution.task_id,
                            execution.exit_code,
                        )

                    completed[execution.task_id] = execution
                    result.executions.append(execution)

            if not ready:
                break

        result.completed_at = datetime.now(UTC).isoformat()
        return result

    def _execute_task(
        self,
        task: Any,
        cycle_id: str,
    ) -> TaskExecutionResult:
        """Execute a single remediation task via LifecycleController."""
        import uuid

        task_id = task.task_id
        prompt = task.prompt_template
        agent_role = getattr(task, "agent_role", None) or "workhorse"
        dimension = getattr(task, "dimension", "") or ""

        agent_name, model_alias = _resolve_agent_for_task(agent_role, dimension)

        _log.info(
            "Executing task %s: agent=%s model=%s, dimension=%s, mode=%s",
            task_id,
            agent_name,
            model_alias,
            dimension,
            self.lifecycle_mode,
        )

        try:
            run_id = f"run_{uuid.uuid4().hex[:8]}"

            # Fresh LifecycleController per task (enables parallel execution)
            controller = LifecycleController(
                settings=self._settings,
                worker_agent_name=agent_name,
                worker_model=model_alias,
                checker_agent_name=self.checker_agent_name,
                mode=LoopMode(self.lifecycle_mode),
                max_iterations=1,
            )
            loop_state = controller.run_loop(
                initial_prompt=prompt,
                todo_spec=task.todo_spec if hasattr(task, "todo_spec") else "",
            )

            exit_code = 0 if not loop_state.stopped else 1

            return TaskExecutionResult(
                task_id=task_id,
                run_id=run_id,
                exit_code=exit_code,
                error=loop_state.stop_reason or "",
                cost_usd=loop_state.last_cost_usd,
                agent_used=loop_state.last_model or model_alias,
            )

        except Exception as e:
            _log.exception("Task %s raised exception", task_id)
            return TaskExecutionResult(
                task_id=task_id,
                run_id="failed",
                exit_code=1,
                error=str(e),
            )

    def get_ready_batch(
        self,
        plan: Any,
        completed_task_ids: set[str],
    ) -> list[Any]:
        """Get tasks ready to execute (all dependencies completed)."""
        _dag = plan.dag_edges or {}
        ready = []

        for task in plan.tasks:
            if task.task_id in completed_task_ids:
                continue
            deps = task.dependencies or []
            if all(d in completed_task_ids for d in deps):
                ready.append(task)

        return ready
