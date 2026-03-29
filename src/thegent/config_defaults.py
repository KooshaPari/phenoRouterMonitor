"""Shared defaults and field factories for thegent settings."""

from collections.abc import Callable
from pathlib import Path

DEFAULT_COST_BUDGET_BY_CATEGORY: dict[str, float] = {
    "fast": 50.0,
    "normal": 200.0,
    "complex": 150.0,
    "high_complex": 50.0,
}

DEFAULT_HITL_CHECKPOINTS: list[str] = ["pre_execution"]

DEFAULT_SANDBOX_ENV_ALLOWLIST: list[str] = [
    "PATH",
    "HOME",
    "LANG",
    "USER",
    "TERM",
    "PYTHONUNBUFFERED",
]

DEFAULT_MAC_KEEP_AWAKE_AGENTS: list[str] = [
    "claude",
    "codex",
    "cursor-agent",
    "opencode",
    "cursor-api",
    "droid",
    "gemini",
    "copilot",
]

DEFAULT_WORKSTREAM_AUTOSYNC_MIGRATION_PHASES: list[dict[str, str]] = [
    {
        "phase": "phase1-detect",
        "description": "Respect existing opt-in repos; do not force-enable autosync.",
    },
    {
        "phase": "phase2-default-on",
        "description": "Enable autosync by default for newly initialized repos.",
    },
    {
        "phase": "phase3-enforced",
        "description": "Treat autosync as required unless explicitly emergency-stopped.",
    },
]


def expanded_path_factory(path: str) -> Callable[[], Path]:
    """Return a zero-arg factory that expands user-relative paths."""

    def _factory() -> Path:
        return Path(path).expanduser()

    return _factory


def default_cost_budget_by_category() -> dict[str, float]:
    """Return default per-category monthly cost budget limits."""
    return dict(DEFAULT_COST_BUDGET_BY_CATEGORY)


def default_hitl_checkpoints() -> list[str]:
    """Return default HITL checkpoints."""
    return list(DEFAULT_HITL_CHECKPOINTS)


def default_sandbox_env_allowlist() -> list[str]:
    """Return default environment allowlist for sandboxed runs."""
    return list(DEFAULT_SANDBOX_ENV_ALLOWLIST)


def default_mac_keep_awake_agents() -> list[str]:
    """Return default agents that trigger macOS caffeinate."""
    return list(DEFAULT_MAC_KEEP_AWAKE_AGENTS)


def default_workstream_autosync_migration_phases() -> list[dict[str, str]]:
    """Return the default phased migration plan for autosync enablement."""
    return [dict(phase) for phase in DEFAULT_WORKSTREAM_AUTOSYNC_MIGRATION_PHASES]


def autosync_phase1_enabled(*, explicit_env: str | None, repo_previously_opted_in: bool) -> bool:
    """Resolve phase-1 autosync enablement behavior.

    Phase 1 keeps existing behavior for repos that already opted in while
    preserving a hard explicit env override when provided.
    """
    if explicit_env is not None:
        return explicit_env.strip().lower() in {"1", "true", "yes", "on"}
    return bool(repo_previously_opted_in)
