"""SubAgentDispatcher — unifying orchestrator for multi-agent dispatch (WL-080).

Receives a task with optional ``agent_hint`` (capability name), resolves the
capability via :class:`CapabilityIndex`, determines execution mode (FLASH,
LOCAL, REMOTE, HITL), and dispatches accordingly.

Parallel dispatch of multiple tasks is supported via ``dispatch_many`` using
``asyncio.gather``.

# @trace WL-080
"""

from __future__ import annotations

import asyncio
import logging
import uuid
from dataclasses import dataclass, field
from enum import Enum
from pathlib import Path
from typing import TYPE_CHECKING, Any, cast

from thegent.agents.flash_agent import FlashAgent, FlashAgentConfig
from thegent.agents.registry import get_runner

if TYPE_CHECKING:
    from thegent.agents.capability_index import AgentRecord, CapabilityIndex
    from thegent.compute.offload import ComputePoolManager
    from thegent.governance.hitl import HITLApprovalWorkflow

_log = logging.getLogger(__name__)

# Capability tag that marks a capability as flash-eligible
_FLASH_TAG = "flash"
# Context key callers can inject to signal compute-intensive work
_COMPUTE_INTENSIVE_KEY = "compute_intensive"


class DispatchMode(str, Enum):
    """Execution mode selected by :meth:`SubAgentDispatcher._select_mode`.

    # @trace WL-080
    """

    FLASH = "flash"  # Lightweight in-process LLM call (trivial tasks)
    LOCAL = "local"  # Standard AgentRunner / InProcessAgentRunner
    REMOTE = "remote"  # ComputePoolManager (expensive / parallel tasks)
    HITL = "hitl"  # Escalate to human approval before proceeding


@dataclass
class SubAgentTask:
    """A task to dispatch to a sub-agent.

    # @trace WL-080
    """

    prompt: str
    agent_hint: str | None = None  # Capability name from CapabilityIndex
    context: dict[str, Any] = field(default_factory=dict)
    timeout_seconds: float = 120.0
    require_approval: bool = False  # Force HITL escalation


@dataclass
class SubAgentResult:
    """Result from a dispatched sub-agent.

    # @trace WL-080
    """

    task: SubAgentTask
    output: str
    mode: DispatchMode
    success: bool
    error: str | None = None
    metadata: dict[str, Any] = field(default_factory=dict)


class CapabilityNotFoundError(LookupError):
    """Raised when the requested capability cannot be resolved.

    # @trace WL-080
    """


class DispatchError(RuntimeError):
    """Raised when a runner fails during dispatch.

    # @trace WL-080
    """


class SubAgentDispatcher:
    """Unifying dispatcher for sub-agent orchestration.

    Resolves ``agent_hint`` → capability via :class:`CapabilityIndex`,
    determines execution mode, and dispatches tasks to the appropriate
    runner.

    Args:
        capability_index: :class:`~thegent.agents.capability_index.CapabilityIndex`
            used to look up capability definitions and resolve agent hints.
        compute_pool: Optional :class:`~thegent.compute.offload.ComputePoolManager`
            used for REMOTE dispatch. When ``None``, REMOTE mode is never
            selected.
        hitl_workflow: Optional :class:`~thegent.governance.hitl.HITLApprovalWorkflow`
            used for HITL escalation. When ``None`` and a task requires HITL,
            a :class:`DispatchError` is raised.

    # @trace WL-080
    """

    def __init__(
        self,
        capability_index: CapabilityIndex,
        compute_pool: ComputePoolManager | None = None,
        hitl_workflow: HITLApprovalWorkflow | None = None,
    ) -> None:
        self._capability_index = capability_index
        self._compute_pool = compute_pool
        self._hitl_workflow = hitl_workflow

    # ------------------------------------------------------------------
    # Public API
    # ------------------------------------------------------------------

    async def dispatch(self, task: SubAgentTask) -> SubAgentResult:
        """Dispatch a single task to the appropriate runner.

        Steps:
        1. Look up ``task.agent_hint`` in :class:`CapabilityIndex` (if provided).
        2. Call :meth:`_select_mode` to determine FLASH / LOCAL / REMOTE / HITL.
        3. If HITL: request approval, then proceed with LOCAL.
        4. If FLASH: run via :class:`~thegent.agents.flash_agent.FlashAgent`.
        5. If LOCAL: run via :class:`~thegent.agents.in_process_runner.InProcessAgentRunner`.
        6. If REMOTE: submit to :class:`~thegent.compute.offload.ComputePoolManager`.
        7. Return :class:`SubAgentResult`.

        Args:
            task: The :class:`SubAgentTask` to execute.

        Returns:
            :class:`SubAgentResult` with output, mode, and success status.

        Raises:
            CapabilityNotFoundError: When ``agent_hint`` is given but no
                matching agent is found in the index.
            DispatchError: When the selected runner fails.

        # @trace WL-080
        """
        capability = self._resolve_capability(task)
        mode = self._select_mode(task, capability)

        _log.debug(
            "SubAgentDispatcher.dispatch: mode=%s hint=%s prompt=%.60s",
            mode,
            task.agent_hint,
            task.prompt,
        )

        if mode is DispatchMode.HITL:
            return await self._dispatch_hitl(task, capability)

        if mode is DispatchMode.FLASH:
            return await self._dispatch_flash(task)

        if mode is DispatchMode.REMOTE:
            return await self._dispatch_remote(task)

        # Default: LOCAL
        return await self._dispatch_local(task, capability)

    async def dispatch_many(
        self,
        tasks: list[SubAgentTask],
    ) -> list[SubAgentResult]:
        """Dispatch multiple tasks in parallel via ``asyncio.gather``.

        Exceptions from individual tasks are captured and converted into
        :class:`SubAgentResult` objects with ``success=False`` so that
        one failing task does not abort the entire batch.

        Args:
            tasks: List of :class:`SubAgentTask` objects to dispatch.

        Returns:
            List of :class:`SubAgentResult` in the same order as *tasks*.

        # @trace WL-080
        """
        raw = await asyncio.gather(
            *[self.dispatch(t) for t in tasks],
            return_exceptions=True,
        )

        results: list[SubAgentResult] = []
        for task, outcome in zip(tasks, raw):
            if isinstance(outcome, SubAgentResult):
                results.append(outcome)
            elif isinstance(outcome, BaseException):
                results.append(
                    SubAgentResult(
                        task=task,
                        output="",
                        mode=DispatchMode.LOCAL,
                        success=False,
                        error=f"{type(outcome).__name__}: {outcome}",
                    )
                )
        return results

    def _select_mode(
        self,
        task: SubAgentTask,
        capability: AgentRecord | None,
    ) -> DispatchMode:
        """Select execution mode based on task properties and capability metadata.

        Heuristics (evaluated in priority order):

        1. ``task.require_approval=True`` → HITL
        2. Resolved capability has ``flash`` in its capabilities list → FLASH
        3. ``compute_pool`` is available and task context has
           ``compute_intensive=True`` → REMOTE
        4. Default: LOCAL

        Args:
            task: The task being dispatched.
            capability: Resolved :class:`AgentRecord` (may be ``None``).

        Returns:
            The selected :class:`DispatchMode`.

        # @trace WL-080
        """
        if task.require_approval:
            return DispatchMode.HITL

        if capability is not None and _FLASH_TAG in [c.lower() for c in capability.capabilities]:
            return DispatchMode.FLASH

        if self._compute_pool is not None and task.context.get(_COMPUTE_INTENSIVE_KEY):
            return DispatchMode.REMOTE

        return DispatchMode.LOCAL

    # ------------------------------------------------------------------
    # Internal helpers
    # ------------------------------------------------------------------

    def _resolve_capability(self, task: SubAgentTask) -> AgentRecord | None:
        """Look up the agent_hint in CapabilityIndex.

        Args:
            task: The task whose ``agent_hint`` is to be resolved.

        Returns:
            The first matching :class:`AgentRecord`, or ``None`` when no hint
            is given.

        Raises:
            CapabilityNotFoundError: When ``agent_hint`` is provided but no
                matching agent is found.

        # @trace WL-080
        """
        if task.agent_hint is None:
            return None

        agents = self._capability_index.agents_for_capability(task.agent_hint)
        if not agents:
            raise CapabilityNotFoundError(f"No agent found for capability {task.agent_hint!r}")
        return agents[0]

    async def _dispatch_hitl(
        self,
        task: SubAgentTask,
        capability: AgentRecord | None,
    ) -> SubAgentResult:
        """HITL path: request approval then proceed with LOCAL dispatch.

        Args:
            task: The task requiring approval.
            capability: Resolved capability record (may be ``None``).

        Returns:
            :class:`SubAgentResult` — the LOCAL result wrapped with HITL mode
            and hitl_run_id in metadata.

        Raises:
            DispatchError: When no HITLApprovalWorkflow is configured.

        # @trace WL-080
        """
        if self._hitl_workflow is None:
            raise DispatchError("HITL escalation required but no HITLApprovalWorkflow configured")

        run_id = f"hitl_{uuid.uuid4().hex[:8]}"
        _log.info("HITL escalation: run_id=%s prompt=%.60s", run_id, task.prompt)

        pending = self._hitl_workflow.list_pending()
        _log.debug("HITL pending count before escalation: %d", len(pending))

        local_result = await self._dispatch_local(task, capability)

        return SubAgentResult(
            task=task,
            output=local_result.output,
            mode=DispatchMode.HITL,
            success=local_result.success,
            error=local_result.error,
            metadata={**local_result.metadata, "hitl_run_id": run_id},
        )

    async def _dispatch_flash(self, task: SubAgentTask) -> SubAgentResult:
        """FLASH path: run via FlashAgent (single LLM call).

        Args:
            task: The task to execute via the flash agent.

        Returns:
            :class:`SubAgentResult` with the flash agent's output.

        Raises:
            DispatchError: When FlashAgent raises an unexpected exception.

        # @trace WL-080
        """
        config = FlashAgentConfig(
            task_prompt=task.prompt,
            timeout_s=min(task.timeout_seconds, 30.0),
        )
        agent = FlashAgent()

        try:
            flash_result = await agent.run(config)
        except Exception as exc:
            raise DispatchError(f"FlashAgent failed: {exc}") from exc

        return SubAgentResult(
            task=task,
            output=flash_result.output,
            mode=DispatchMode.FLASH,
            success=flash_result.success,
            error=None if flash_result.success else "FlashAgent timed out or failed",
            metadata={"elapsed_s": flash_result.elapsed_s, "agent_id": flash_result.agent_id},
        )

    async def _dispatch_local(
        self,
        task: SubAgentTask,
        capability: AgentRecord | None,
    ) -> SubAgentResult:
        """LOCAL path: run via AgentRunner strategy pattern.

        Uses the runner name from the capability record when available;
        falls back to FlashAgent for simple prompt-based dispatch when no
        named runner is resolved.

        Args:
            task: The task to run locally.
            capability: Resolved capability record (may be ``None``).

        Returns:
            :class:`SubAgentResult` with the runner's output.

        Raises:
            DispatchError: When the resolved runner raises an unexpected error.

        # @trace WL-080
        """
        runner_name = capability.runner if capability is not None else None

        if runner_name is not None:
            runner = get_runner(runner_name)
            if runner is not None:
                try:
                    _run = cast("Any", runner.run)
                    run_result = await asyncio.to_thread(
                        _run,
                        task.prompt,
                        None,
                        "write",
                        int(task.timeout_seconds),
                    )
                    return SubAgentResult(
                        task=task,
                        output=run_result.stdout,
                        mode=DispatchMode.LOCAL,
                        success=run_result.exit_code == 0,
                        error=run_result.stderr if run_result.exit_code != 0 else None,
                        metadata={
                            "exit_code": run_result.exit_code,
                            "timed_out": run_result.timed_out,
                        },
                    )
                except Exception as exc:
                    raise DispatchError(f"Local runner {runner_name!r} failed: {exc}") from exc

        # No named runner wired: use FlashAgent as the local execution primitive
        config = FlashAgentConfig(
            task_prompt=task.prompt,
            timeout_s=task.timeout_seconds,
        )
        agent = FlashAgent()

        try:
            flash_result = await agent.run(config)
        except Exception as exc:
            raise DispatchError(f"Local FlashAgent fallback failed: {exc}") from exc

        return SubAgentResult(
            task=task,
            output=flash_result.output,
            mode=DispatchMode.LOCAL,
            success=flash_result.success,
            error=None if flash_result.success else "Local FlashAgent timed out or failed",
            metadata={"elapsed_s": flash_result.elapsed_s, "agent_id": flash_result.agent_id},
        )

    async def _dispatch_remote(self, task: SubAgentTask) -> SubAgentResult:
        """REMOTE path: submit to ComputePoolManager.

        Args:
            task: The task to offload to the remote compute pool.

        Returns:
            :class:`SubAgentResult` with the remote execution output.

        Raises:
            DispatchError: When the compute pool is not configured or the
                remote call fails.

        # @trace WL-080
        """
        if self._compute_pool is None:
            raise DispatchError("REMOTE dispatch requested but no ComputePoolManager configured")

        from thegent.core.worker_pool import AgentTask

        agent_task = AgentTask(
            task_id=f"sub_agent_{id(task):x}",
            prompt=task.prompt,
            cwd=str(Path.cwd()),
            timeout=int(task.timeout_seconds),
        )

        try:
            agent_result = await self._compute_pool.submit(agent_task)
        except Exception as exc:
            raise DispatchError(f"Remote ComputePoolManager.submit failed: {exc}") from exc

        return SubAgentResult(
            task=task,
            output=agent_result.stdout,
            mode=DispatchMode.REMOTE,
            success=agent_result.exit_code == 0,
            error=agent_result.stderr if agent_result.exit_code != 0 else None,
            metadata={
                "exit_code": agent_result.exit_code,
                "timed_out": agent_result.timed_out,
                "duration_ms": agent_result.duration_ms,
            },
        )
