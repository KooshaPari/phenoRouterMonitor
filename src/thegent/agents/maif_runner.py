"""MAIF-aware agent runner wrapper."""

import logging
from pathlib import Path
from typing import TYPE_CHECKING, Any, Optional

from thegent.agents.base import AgentRunner, RunResult
from thegent.execution import RunMeta

if TYPE_CHECKING:
    from thegent.orchestration.execution.engine import ExecutionEngine
    from thegent.utils.routing_impl.route_executor import RoutingDecision

_log = logging.getLogger(__name__)


class MAIFAgentRunner(AgentRunner):
    """Wraps an AgentRunner to automatically generate MAIF artifacts using ExecutionEngine."""

    def __init__(self, runner: AgentRunner, engine: Optional["ExecutionEngine"] = None) -> None:
        self.runner = runner
        if engine is None:
            from thegent.orchestration.execution.engine import ExecutionEngine

            engine = ExecutionEngine()
        self.engine = engine

    def run(
        self,
        prompt: str,
        cwd: Path | None = None,
        mode: str = "write",
        timeout: int = 90,
        *,
        run_id: str | None = None,
        owner: str = "unknown",
        agent_name: str = "unknown",
        **kwargs: Any,
    ) -> tuple[RunResult, "RoutingDecision"]:
        """Run the agent and generate MAIF artifacts.

        This method overloads the base run() to accept metadata required for MAIF.
        """
        import uuid
        from datetime import UTC, datetime

        effective_run_id = run_id or f"run_{uuid.uuid4().hex[:8]}"

        run_meta = RunMeta(
            run_id=effective_run_id,
            prompt=prompt,
            owner=owner,
            agent=agent_name,
            cwd=str((cwd or Path.cwd()).resolve()),
            started_at_utc=datetime.now(UTC).isoformat(),
        )

        return self.engine.execute(runner=self.runner, run_meta=run_meta, cwd=cwd, mode=mode, timeout=timeout, **kwargs)
