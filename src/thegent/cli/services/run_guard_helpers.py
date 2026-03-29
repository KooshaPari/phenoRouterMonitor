"""Helpers for run_impl pre-flight guard checks and concurrency gating."""

from __future__ import annotations

import logging
import uuid
from pathlib import Path
from typing import Any

from rich.console import Console

from thegent.config import ThegentSettings

_LAST_GUARDRAIL_DIAGNOSTIC: dict[str, Any] = {"status": "not_checked"}


def suggest_terminal_reuse(*, settings: ThegentSettings, cwd: Path, console: Console, log: logging.Logger) -> None:
    """Emit lightweight terminal reuse hints for the current cwd."""
    if not settings.terminal_management_enabled:
        return

    try:
        import importlib

        routing_mod = importlib.import_module("thegent.utils.routing_impl")
        task_router = getattr(routing_mod, "TaskRouter", None)
        if task_router:
            router = task_router(settings)
            existing_pane = router.find_active_terminal_for_path(str(cwd))
            if existing_pane:
                console.print(
                    f"[bold yellow]Found existing terminal session for this path: {existing_pane}[/bold yellow]"
                )
                console.print(f"[dim]You can attach with: thegent terminal attach {existing_pane}[/dim]")
    except Exception as exc:
        log.debug("Terminal discovery failed: %s", exc)


def enforce_input_guardrails(
    *,
    settings: ThegentSettings,
    prompt: str,
    agent: str | None,
    model: str | None,
    cwd: Path,
    run_id: str | None,
) -> dict[str, Any] | None:
    """Validate prompt guardrails and return an error payload when blocked."""
    global _LAST_GUARDRAIL_DIAGNOSTIC  # noqa: PLW0603
    if not settings.input_guardrails_enabled:
        _LAST_GUARDRAIL_DIAGNOSTIC = {"status": "disabled"}
        return None

    try:
        from thegent.governance.input_guardrails import guardrails_from_env
    except (ModuleNotFoundError, ImportError) as exc:
        _LAST_GUARDRAIL_DIAGNOSTIC = {
            "status": "error",
            "error_type": "import_error",
            "detail": str(exc)[:200],
        }
        return None

    try:
        guardrails = guardrails_from_env()
    except (AttributeError, RuntimeError, ValueError) as exc:
        _LAST_GUARDRAIL_DIAGNOSTIC = {
            "status": "error",
            "error_type": "initialization_error",
            "detail": str(exc)[:200],
        }
        return None

    try:
        result = guardrails.check(prompt=prompt, agent=agent or "", model=model, cwd=cwd)
        if not result.passed:
            _LAST_GUARDRAIL_DIAGNOSTIC = {
                "status": "blocked",
                "rail_id": result.rail_id,
                "reason": result.reason,
            }
            return {
                "error": f"Input guardrail failed ({result.rail_id}): {result.reason}",
                "remediation": result.remediation,
                "exit_code": 1,
                "run_id": run_id or f"run_err_{uuid.uuid4().hex[:8]}",
            }
        _LAST_GUARDRAIL_DIAGNOSTIC = {"status": "passed"}
    except (AttributeError, RuntimeError, ValueError) as exc:
        _LAST_GUARDRAIL_DIAGNOSTIC = {
            "status": "error",
            "error_type": "evaluation_error",
            "detail": str(exc)[:200],
        }
        return None
    except TypeError as exc:
        _LAST_GUARDRAIL_DIAGNOSTIC = {
            "status": "error",
            "error_type": "evaluation_contract_error",
            "detail": str(exc)[:200],
        }
        return None

    return None


def get_last_guardrail_diagnostic() -> dict[str, Any]:
    """Return machine-readable diagnostics for the last guardrail evaluation."""
    return dict(_LAST_GUARDRAIL_DIAGNOSTIC)


def enforce_concurrency_limit(
    *,
    settings: ThegentSettings,
    lane: str,
    agent: str | None,
    owner: str | None,
    rid: str,
    run_id: str | None,
    speculative: bool,
    task_id: str | None,
    log: logging.Logger,
) -> dict[str, Any] | None:
    """Acquire a concurrency slot and return a blocking payload when denied."""
    from thegent.execution import ConcurrencyController

    max_concurrency = getattr(settings, "max_concurrency", 1000)
    if not isinstance(max_concurrency, int):
        max_concurrency = 1000
    use_load_based = getattr(settings, "concurrency_load_based", False)
    if not isinstance(use_load_based, bool):
        use_load_based = False

    harness_type = None
    if agent:
        lowered = agent.lower()
        if "codex" in lowered or "dex" in lowered:
            harness_type = "codex"
        elif "claude" in lowered or "clode" in lowered:
            harness_type = "claude"
        elif "droid" in lowered or "roid" in lowered:
            harness_type = "droid"

    controller = ConcurrencyController(
        settings.session_dir,
        max_concurrency=max_concurrency,
        use_load_based=use_load_based,
    )
    acquired = controller.acquire(
        lane=lane,
        harness_type=harness_type,
        priority=lane,
        owner=owner or "unknown",
        run_id=rid,
        speculative=speculative,
    )
    if acquired:
        return None

    if task_id:
        try:
            from thegent.governance.teammates import TeammateManager

            manager = TeammateManager(settings.cache_dir / "teammates.json")
            manager.update_status(
                task_id,
                "failed",
                summary="Run blocked: Concurrency limit reached (resource contention).",
            )
        except Exception as exc:
            log.debug("Failed to update teammate delegation status: %s", exc)

    if use_load_based:
        from thegent.orchestration.resource.load_based_limits import (
            LimitGateConfig,
            compute_dynamic_limit,
            sample_resources,
        )

        snapshot = sample_resources()
        model_dump = getattr(settings, "model_dump", None)
        settings_payload = model_dump() if callable(model_dump) else {}
        if not isinstance(settings_payload, dict):
            settings_payload = {}
        config = LimitGateConfig.from_dict(settings_payload)
        effective_limit, _ = compute_dynamic_limit(snapshot, config)

        bottlenecks = controller.get_bottlenecks() if hasattr(controller, "get_bottlenecks") else {}
        bottleneck_msg = ""
        if bottlenecks.get("resource_contention"):
            bottleneck_msg = f" Resource contention detected: {len(bottlenecks['resource_contention'])} issue(s)."

        return {
            "error": (
                f"Resource-based concurrency limit reached (current: {effective_limit} slots)."
                f"{bottleneck_msg} Task queued or blocked."
            ),
            "exit_code": 1,
            "run_id": run_id or f"run_err_{uuid.uuid4().hex[:8]}",
            "bottlenecks": bottlenecks,
        }

    return {
        "error": f"Concurrency limit reached ({max_concurrency}). Task queued or blocked.",
        "exit_code": 1,
        "run_id": run_id or f"run_err_{uuid.uuid4().hex[:8]}",
    }
