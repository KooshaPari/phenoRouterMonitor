"""In-process agent runner (MTSP-02).

Provides thread-safe cwd isolation for running agents within the same process.
"""

import logging
import os
import threading
from contextlib import contextmanager
from pathlib import Path
from typing import Any

from thegent.governance.post_agent_run_hook import dispatch_post_agent_run_hook

_log = logging.getLogger(__name__)

# Global lock for thread-safe chdir (MTSP-02)
_CHDIR_LOCK = threading.Lock()


@contextmanager
def isolated_cwd(new_cwd: Path):
    """Context manager for thread-safe cwd isolation."""
    with _CHDIR_LOCK:
        old_cwd = Path.cwd()
        try:
            os.chdir(new_cwd)
            yield
        finally:
            os.chdir(old_cwd)


class InProcessAgentRunner:
    """Agent runner that executes in-process with cwd isolation (MTSP-02)."""

    def __init__(self, agent_name: str, base_runner: Any) -> None:
        self.agent_name = agent_name
        self.base_runner = base_runner

    def run(
        self,
        prompt: str,
        cd: Path,
        mode: str = "write",
        timeout: int = 600,
        run_id: str | None = None,
        session_id: str | None = None,
        **kwargs: Any,
    ) -> dict[str, Any]:
        """Run the agent with isolated cwd."""
        _log.info("MTSP-02: Running agent %s in-process with isolated cwd: %s", self.agent_name, cd)

        # Ensure directory exists
        cd.mkdir(parents=True, exist_ok=True)

        with isolated_cwd(cd):
            forward_kwargs = dict(kwargs)
            if run_id is not None:
                forward_kwargs["run_id"] = run_id
            if session_id is not None:
                forward_kwargs["session_id"] = session_id
            result = self.base_runner.run(prompt=prompt, cd=cd, mode=mode, timeout=timeout, **forward_kwargs)
            if run_id is not None or session_id is not None:
                dispatch_post_agent_run_hook(
                    result=result,
                    run_id=run_id,
                    session_id=session_id,
                    cwd=cd,
                    extra_context={
                        "agent": self.agent_name,
                        "mode": mode,
                        "timeout_seconds": timeout,
                    },
                )
            return result

    async def run_async(
        self,
        prompt: str,
        cd: Path,
        mode: str = "write",
        timeout: int = 600,
        run_id: str | None = None,
        session_id: str | None = None,
        **kwargs: Any,
    ) -> dict[str, Any]:
        """Run the agent asynchronously (not truly async if base runner is sync)."""
        # For now, we just wrap the sync run in a thread if needed,
        # but most base runners are sync.
        import asyncio

        return await asyncio.to_thread(
            self.run,
            prompt,
            cd,
            mode,
            timeout,
            run_id=run_id,
            session_id=session_id,
            **kwargs,
        )
