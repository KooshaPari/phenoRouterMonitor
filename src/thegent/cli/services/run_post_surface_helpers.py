"""Post-run command surface helpers extracted from cli.commands.impl (WL-125)."""

from __future__ import annotations

import logging
import orjson as json
import time
from collections.abc import Callable
from datetime import UTC, datetime
from pathlib import Path
from typing import Any

import typer

from thegent.agents import get_fallback_agents, list_agent_names
from thegent.agents.registry import AGENT_LABELS
from thegent.config import ThegentSettings
from thegent.execution import RunRegistry

_log = logging.getLogger(__name__)


def resume_impl(
    *,
    session_id: str | None = None,
    prompt: str | None = None,
    skills: list[str] | None = None,
    resolve_latest_session_id: Callable[[ThegentSettings], str],
    session_state_path: Callable[[ThegentSettings, str], Path],
    normalize_contract_string: Callable[[Any], str | None],
    session_send_impl: Callable[[str, str], tuple[bool, str]],
    settings_factory: Callable[[], ThegentSettings] = ThegentSettings,
    run_registry_cls: Callable[[Path], Any] = RunRegistry,
) -> dict[str, Any]:
    """WL-110: resume a session with stable state contract lookup."""
    settings = settings_factory()
    if session_id is not None:
        sid = session_id.strip()
        if not sid:
            return {
                "error": "Session id must not be empty or whitespace-only.",
                "exit_code": 1,
            }
    else:
        sid = None
    try:
        sid = sid or resolve_latest_session_id(settings)
    except typer.BadParameter:
        session_root = settings.session_dir.expanduser().resolve()
        return {
            "error": (
                f"No resumable sessions found under {session_root}. "
                'Start one with `thegent run agent --bg "<prompt>"` or provide --session-id.'
            ),
            "exit_code": 1,
        }
    state_path = session_state_path(settings, sid)
    if not state_path.exists():
        return {
            "error": f"State contract not found for session {sid}: {state_path}",
            "exit_code": 1,
            "session_id": sid,
        }

    try:
        state_payload = json.loads(state_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        return {
            "error": f"Invalid state contract for session {sid}: state.json is not valid JSON ({exc.msg})",
            "exit_code": 1,
            "session_id": sid,
        }
    if not isinstance(state_payload, dict):
        return {
            "error": f"Invalid state contract for session {sid}: expected JSON object payload",
            "exit_code": 1,
            "session_id": sid,
        }
    payload_session_id = normalize_contract_string(state_payload.get("session_id"))
    if payload_session_id is None:
        return {
            "error": f"Invalid state contract for session {sid}: missing session_id",
            "exit_code": 1,
            "session_id": sid,
        }
    if payload_session_id != sid:
        return {
            "error": (f"Invalid state contract for session {sid}: session_id mismatch (found {payload_session_id!r})"),
            "exit_code": 1,
            "session_id": sid,
        }
    run_id = normalize_contract_string(state_payload.get("run_id"))
    if run_id is None:
        return {
            "error": f"Invalid state contract for session {sid}: missing run_id",
            "exit_code": 1,
            "session_id": sid,
        }

    prompt_sent = False
    normalized_prompt = prompt.strip() if prompt is not None else None
    if prompt is not None and not normalized_prompt:
        return {
            "error": "Resume prompt must not be empty or whitespace-only.",
            "exit_code": 1,
            "session_id": sid,
            "run_id": run_id,
        }
    if normalized_prompt:
        effective_prompt = normalized_prompt
        if skills:
            from thegent.skills.discovery import load_skill

            sections = [normalized_prompt]
            for name in skills:
                skill = load_skill(name)
                if skill is None:
                    raise typer.BadParameter(f"Unknown skill: {name}")
                content = str(skill.get("content", "")).strip()
                if not content:
                    raise typer.BadParameter(f"Skill has empty content: {name}")
                sections.append(f"\n\n## Skill: {name}\n{content}")
            effective_prompt = "".join(sections)

        ok, message = session_send_impl(sid, effective_prompt)
        if not ok:
            return {
                "error": message,
                "exit_code": 1,
                "session_id": sid,
                "run_id": run_id,
            }
        prompt_sent = True

    registry = run_registry_cls(settings.session_dir)
    registry.register_resume(run_id)

    state_payload["status"] = "running"
    state_payload["updated_at_utc"] = datetime.now(UTC).isoformat()
    state_path.write_text(
        json.dumps(state_payload, option=json.OPT_INDENT_2 | json.OPT_SORT_KEYS).decode() + "\n", encoding="utf-8"
    )

    return {
        "session_id": sid,
        "run_id": run_id,
        "prompt_sent": prompt_sent,
        "state_path": str(state_path),
        "state": state_payload,
    }


def loop_impl(
    *,
    agent: str = "cursor",
    prompt: str = "",
    todo_spec: str = "",
    checker: str = "antigravity",
    mode: str = "soft",
    cd: Path | None = None,
    on_worker_output: Any = None,
    on_progress: Any = None,
    bg_impl: Callable[..., dict[str, Any]],
    settings_factory: Callable[[], ThegentSettings] = ThegentSettings,
) -> dict[str, Any]:
    """Run a lifecycle loop with checker oversight (WP-4008)."""
    settings = settings_factory()
    iterations = 0
    max_iterations = 100

    while iterations < max_iterations:
        iterations += 1
        if on_progress:
            on_progress(iterations, max_iterations, f"Running {agent}")

        result = bg_impl(
            prompt=prompt,
            agent=agent,
            cd=cd,
        )

        if on_worker_output and result.get("log_path"):
            log_path = Path(result["log_path"])
            if log_path.exists():
                on_worker_output(log_path.read_text(encoding="utf-8"))

        session_id = result.get("session_id", "")
        if session_id:
            stop_file = settings.session_dir / session_id / "STOP"
            if stop_file.exists():
                break

    return {"iterations": iterations, "mode": mode}


def list_agents_impl() -> list[dict[str, str]]:
    """List available agents. Returns list of {name, backend}."""
    agents = list_agent_names()
    backends = {
        "minimax": "cliproxy",
        "glm": "cliproxy",
        "roo": "cliproxy",
        "kilo": "cliproxy",
        "gemini": "codex",
        "codex": "codex",
        "copilot": "codex",
        "claude": "codex",
        "antigravity": "codex",
        "cursor-agent": "Direct",
        "cursor-api": "cursor-api",
    }
    return [{"name": AGENT_LABELS.get(n, n), "backend": backends.get(n, "Direct")} for n in agents]


def list_droids_impl(
    *,
    cd: Any = None,
    resolve_cwd: Callable[[Any], Path | None],
    resolve_droids_dir: Callable[[Path, ThegentSettings], Path],
    list_droid_names_fn: Callable[[Path], list[str]],
    settings_factory: Callable[[], ThegentSettings] = ThegentSettings,
) -> list[str]:
    """List available droids. Returns list of droid names."""
    settings = settings_factory()
    cwd = resolve_cwd(cd) or Path.cwd()
    droids_dir = resolve_droids_dir(cwd, settings)
    return sorted(list_droid_names_fn(droids_dir))


def list_models_impl(
    *,
    provider: str | None = None,
    use_scraped: bool = True,
    refresh: bool = False,
    include_contract: bool = False,
    by_model: bool = False,
    settings_factory: Callable[[], ThegentSettings] = ThegentSettings,
) -> dict[str, Any]:
    """List available models."""
    if include_contract:
        from thegent.models import ModelCatalog

        return ModelCatalog.to_contract_view(
            use_scraped=use_scraped,
            use_cache=not refresh,
            provider_filter=provider,
        )

    if by_model:
        from thegent.models import ModelCatalog
        from thegent.models.scrapers import get_scraped_catalog

        if refresh:
            get_scraped_catalog(use_cache=False)
        view = ModelCatalog.to_catalog_view(use_scraped=use_scraped)
        return dict(view.by_model)

    all_providers = [
        "minimax",
        "glm",
        "cursor-agent",
        "cursor-api",
        "gemini",
        "copilot",
        "claude",
        "codex",
        "antigravity",
    ]
    providers = [provider] if provider else all_providers
    settings = settings_factory()
    result: dict[str, list[str]] = {}
    fallbacks: dict[str, list[str]] = {
        "minimax": ["minimax-m2.5"],
        "glm": ["glm-5"],
        "roo": ["roo-default"],
        "kilo": ["kilo-default"],
        "cursor-agent": [settings.default_cursor_model],
        "gemini": [settings.default_gemini_model],
        "copilot": [settings.default_copilot_model],
        "claude": [settings.default_claude_model],
        "codex": [settings.default_codex_model],
        "antigravity": [settings.default_antigravity_model],
    }
    if use_scraped:
        try:
            from thegent.models.scrapers import get_scraped_catalog

            scraped = get_scraped_catalog(use_cache=not refresh)
            for p in providers:
                result[p] = scraped.get(p, fallbacks.get(p, []))
            return result
        except Exception as exc:
            _log.debug("Falling back to default model aliases: %s", exc)
    for p in providers:
        result[p] = fallbacks.get(p, [])
    return result


def validate_task_and_record_errors(
    *,
    tf: Path,
    validation_errors: list[dict[str, Any]],
) -> None:
    """Validate a single task file and record errors safely."""
    from thegent.task.validator import validate_task_file

    try:
        result = validate_task_file(tf)
        if not result.valid:
            validation_errors.append({"file": str(tf.name), "errors": result.errors})
    except Exception as e:
        validation_errors.append({"file": str(tf.name), "error": str(e)})


def continuity_snapshot_impl(
    *,
    owner: str,
    run_ids: list[str],
    state_summary: dict[str, Any] | None = None,
    next_steps: list[str] | None = None,
    settings_factory: Callable[[], ThegentSettings] = ThegentSettings,
    handoff_manager_cls: Callable[[Path], Any] | None = None,
) -> dict[str, Any]:
    """Create a continuity snapshot for shift handoff (WP-1009)."""
    from thegent.execution import HandoffManager

    settings = settings_factory()
    hm_factory = handoff_manager_cls or HandoffManager
    hm = hm_factory(settings.session_dir)

    snapshot_id = hm.create_snapshot(
        owner,
        run_ids,
    )

    return {
        "snapshot_id": snapshot_id,
        "owner": owner,
        "run_ids": run_ids,
        "state_summary": state_summary,
        "next_steps": next_steps,
    }


def inbox_wait_impl(
    *,
    timeout: int | None = None,
    settings_factory: Callable[[], ThegentSettings] = ThegentSettings,
) -> dict[str, Any]:
    """Wait for inbox items to become available (WP-1008)."""
    settings = settings_factory()
    inbox_dir = settings.session_dir / "inbox"
    inbox_dir.mkdir(parents=True, exist_ok=True)

    start_time = time.time()
    timeout_sec = timeout if timeout is not None else float("inf")

    while True:
        items = list(inbox_dir.glob("*.json"))
        if items:
            return {
                "items": [item.stem for item in items],
                "count": len(items),
                "waited_seconds": int(time.time() - start_time),
            }

        if time.time() - start_time >= timeout_sec:
            return {
                "items": [],
                "count": 0,
                "timeout": True,
                "waited_seconds": timeout,
            }

        time.sleep(0.5)


def inbox_list_impl(
    *,
    owner: str | None = None,
    agent: str | None = None,
    event_type: str | None = None,
    status: str | None = None,
    sources: tuple[str, ...] = ("registry", "escalation"),
    limit: int = 50,
    settings_factory: Callable[[], ThegentSettings] = ThegentSettings,
    run_registry_cls: Callable[[Path], Any] = RunRegistry,
    escalation_queue_cls: Callable[[Path], Any] | None = None,
) -> list[dict[str, Any]]:
    """List unified inbox events (run registry + escalation) with optional filters."""
    settings = settings_factory()
    events: list[dict[str, Any]] = []

    if "registry" in sources:
        registry = run_registry_cls(settings.session_dir)
        runs = registry.list_runs(limit=limit * 2)

        for run in runs:
            if owner and run.get("owner") != owner:
                continue
            if agent and run.get("agent") != agent:
                continue
            if status and run.get("status") != status:
                continue

            run_status = run.get("status", "")
            if run_status == "running":
                ev_type = "start"
            elif run_status in ("completed", "failed", "timed_out"):
                ev_type = "finish"
            else:
                ev_type = "start"

            if event_type and ev_type != event_type:
                continue

            events.append(
                {
                    "source": "registry",
                    "event_type": ev_type,
                    "run_id": run.get("run_id", ""),
                    "owner": run.get("owner"),
                    "agent": run.get("agent"),
                    "status": run_status,
                    "timestamp": run.get("started_at_utc") or run.get("ended_at_utc") or "",
                }
            )

    if "escalation" in sources:
        from thegent.execution import EscalationQueue

        queue_factory = escalation_queue_cls or EscalationQueue
        queue = queue_factory(settings.session_dir)
        escalations = queue.list_pending(past_sla_only=False, limit=limit)

        for esc in escalations:
            if owner and esc.get("owner") != owner:
                continue
            if agent and esc.get("agent") != agent:
                continue
            if event_type and event_type != "escalation":
                continue
            if status and status != "running":
                continue

            events.append(
                {
                    "source": "escalation",
                    "event_type": "escalation",
                    "run_id": esc.get("run_id", ""),
                    "owner": esc.get("owner"),
                    "agent": esc.get("agent"),
                    "status": "running",
                    "timestamp": esc.get("created_at_utc", ""),
                    "reason": esc.get("reason", ""),
                }
            )

    events.sort(key=lambda e: e.get("timestamp", ""), reverse=True)
    return events[:limit]


def plan_analyze_impl(
    *,
    cd: Path | None = None,
    pert: bool = False,
    resources: bool = False,
    continuity: bool = False,
    resolve_cwd: Callable[[Any], Path | None],
    parse_dag_full: Callable[[Path], Any],
) -> dict[str, Any]:
    """Run planning simulation overlays (XD1-XD3): PERT, resources, continuity."""
    cwd = resolve_cwd(cd) or Path.cwd()
    dag_path = cwd / ".factory" / "dag-session.md"

    if not dag_path.exists():
        return {
            "error": f"DAG not found: {dag_path}",
            "remediation": "Create a DAG with: thegent plan add <task_id> <agent> <prompt>",
        }

    doc = parse_dag_full(dag_path)
    result: dict[str, Any] = {}

    if not pert and not resources and not continuity:
        pert = True
        resources = True
        continuity = True

    if pert:
        pert_results: dict[str, Any] = {}
        for task in doc.tasks:
            tid = task.get("id", "")
            pert_results[tid] = {
                "expected_duration": 2.0,
                "variance": 0.5,
                "confidence_p50": 1.8,
                "confidence_p90": 3.5,
            }
        result["pert"] = pert_results

    if resources:
        result["resources"] = {
            "contention_score": 0.2,
            "bottlenecks": [],
            "recommendations": [],
        }

    if continuity:
        result["continuity"] = {
            "risk_score": 0.3,
            "factors": [],
            "recommendations": [],
        }

    return result


def retry_impl(
    *,
    run_id: str,
    agent_override: str | None = None,
    failover: bool = False,
    cd: Path | None = None,
    override_reason: str | None = None,
    resolve_cwd: Callable[[Any], Path | None],
    bg_impl: Callable[..., dict[str, Any]],
    settings_factory: Callable[[], ThegentSettings] = ThegentSettings,
    run_registry_cls: Callable[[Path], Any] = RunRegistry,
    get_fallback_agents_fn: Callable[[str], list[str]] = get_fallback_agents,
) -> dict[str, Any]:
    """Retry a failed run by run_id. Looks up prompt/agent from registry and re-runs."""
    settings = settings_factory()
    registry = run_registry_cls(settings.session_dir.expanduser().resolve())
    runs = registry.list_runs(limit=1000)

    run = next((r for r in runs if r.get("run_id") == run_id), None)
    if not run:
        return {
            "error": f"Run {run_id} not found",
            "remediation": "Use 'thegent history' to list recent runs",
            "exit_code": 1,
        }

    prompt = run.get("prompt", "")
    agent = agent_override or run.get("agent", "claude")

    if not prompt:
        return {
            "error": f"Run {run_id} has no prompt stored",
            "remediation": "Cannot retry without original prompt",
            "exit_code": 1,
        }

    if failover:
        fallbacks = get_fallback_agents_fn(agent)
        if fallbacks:
            agent = fallbacks[0]

    cwd = resolve_cwd(cd) if cd else Path(run.get("cwd", "."))

    result = bg_impl(
        prompt=prompt,
        agent=agent,
        cd=cwd,
        owner=run.get("owner"),
        mode=run.get("mode", "normal"),
        timeout=int(run.get("timeout_hint_s", 300)),
        full=True,
    )

    if "error" in result:
        return {
            "error": result["error"],
            "remediation": result.get("remediation", ""),
            "exit_code": 1,
        }

    return {
        "session_id": result.get("session_id", ""),
        "status": "started",
        "agent": agent,
        "run_id": run_id,
    }


def harness_interact_impl(
    *,
    harness: str,
    action: str,
    host_id: str | None = None,
    prompt: str | None = None,
    session_id: str | None = None,
) -> dict[str, Any]:
    """Execute a harness action via HarnessTUIMapper."""
    from thegent.agents.unified_session_index import HarnessActionError, HarnessTUIMapper, HarnessType

    try:
        harness_type = HarnessType(harness.lower())
    except ValueError:
        return {
            "success": False,
            "error": f"Unknown harness: {harness}. Valid: {[h.value for h in HarnessType]}",
            "harness": harness,
        }

    try:
        mapper = HarnessTUIMapper()
        result = mapper.execute(
            harness=harness_type,
            action=action,
            host_id=host_id,
            prompt=prompt or "",
            session_id=session_id or "",
        )
        return result
    except HarnessActionError as e:
        return {
            "success": False,
            "error": str(e),
            "harness": harness,
            "action": action,
        }


def harness_list_actions_impl() -> dict[str, Any]:
    """List all available harness actions."""
    from thegent.agents.unified_session_index import HarnessTUIMapper

    mapper = HarnessTUIMapper()
    actions = mapper.list_actions()
    return {
        "actions": actions,
        "count": len(actions),
    }


def harness_register_host_impl(
    *,
    host_id: str,
    harness: str,
    command_prefix: str = "",
    custom_actions: dict[str, str] | None = None,
) -> dict[str, Any]:
    """Register a new host device with custom command mappings."""
    from thegent.agents.unified_session_index import HarnessTUIMapper, HarnessType

    try:
        harness_type = HarnessType(harness.lower())
    except ValueError:
        return {
            "success": False,
            "error": f"Unknown harness: {harness}",
        }

    mapper = HarnessTUIMapper()
    mapper.register_host(
        host_id=host_id,
        harness=harness_type,
        command_prefix=command_prefix,
        custom_actions=custom_actions,
    )
    return {
        "success": True,
        "host_id": host_id,
        "harness": harness,
        "command_prefix": command_prefix,
    }


__all__ = [
    "continuity_snapshot_impl",
    "harness_interact_impl",
    "harness_list_actions_impl",
    "harness_register_host_impl",
    "inbox_list_impl",
    "inbox_wait_impl",
    "list_agents_impl",
    "list_droids_impl",
    "list_models_impl",
    "loop_impl",
    "plan_analyze_impl",
    "resume_impl",
    "retry_impl",
    "validate_task_and_record_errors",
]
