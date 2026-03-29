"""PostAgentRun hook dispatcher wiring for agent and orchestration surfaces."""

from __future__ import annotations

import orjson as json
import os
from thegent.infra.shim_subprocess import run as shim_run
from pathlib import Path
from typing import Any

from thegent.agents.base import RunResult


def _serialize_result(result: Any) -> dict[str, Any]:
    """Convert common runner result objects into JSON-serializable dictionaries."""
    if isinstance(result, RunResult):
        return {
            "exit_code": result.exit_code,
            "stdout": result.stdout,
            "stderr": result.stderr,
            "timed_out": result.timed_out,
            "context_tokens_used": result.context_tokens_used,
            "context_window_max": result.context_window_max,
            "audio_transcript": result.audio_transcript,
            "grounding_sources": result.grounding_sources,
        }
    if isinstance(result, dict):
        return result
    if hasattr(result, "model_dump"):
        dumped = result.model_dump()  # type: ignore[attr-defined]
        if isinstance(dumped, dict):
            return dumped
    if hasattr(result, "__dict__"):
        return dict(vars(result))
    return {"value": str(result)}


def dispatch_post_agent_run_hook(
    result: Any,
    run_id: str | None,
    session_id: str | None,
    cwd: Path | None,
    extra_context: dict[str, Any] | None,
) -> None:
    """Dispatch ``hook-dispatcher postagentrun`` and fail fast on execution errors."""
    context = extra_context or {}
    payload = {
        "result": _serialize_result(result),
        "run_id": run_id,
        "session_id": session_id,
        "cwd": str(cwd) if cwd is not None else None,
        "context": context,
    }

    env = os.environ.copy()
    env["THGENT_RUN_ID"] = run_id or ""
    env["THGENT_SESSION_ID"] = session_id or ""
    policy = str(context.get("vetter_policy") or os.environ.get("THGENT_VETTER_POLICY", ""))
    env["THGENT_VETTER_POLICY"] = policy

    proc = shim_run(
        ["hook-dispatcher", "postagentrun"],
        input=json.dumps(payload).decode(),
        capture_output=True,
        text=True,
        check=False,
        cwd=str(cwd) if cwd is not None else None,
        env=env,
    )
    if proc.returncode != 0:
        detail = (proc.stderr or "").strip() or (proc.stdout or "").strip() or f"exit_code={proc.returncode}"
        raise RuntimeError(f"hook-dispatcher postagentrun failed: {detail}")


# Backward-compatible alias for historical import path used by tests/callers.
_dispatch_post_agent_run_hook = dispatch_post_agent_run_hook
