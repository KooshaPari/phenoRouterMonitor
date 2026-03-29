"""Phench: deterministic project-state runtime control plane CLI."""

from __future__ import annotations

import sys
from importlib import import_module
from pathlib import Path
from typing import Any

# Ensure src/ is on sys.path when this module is loaded directly (e.g., in tests)
_src_dir = str(Path(__file__).resolve().parents[3])
if _src_dir not in sys.path:
    sys.path.insert(0, _src_dir)

import orjson as json
import typer
from rich.console import Console

from thegent.cli.apps.phench_env import register_env_commands
from thegent.cli.apps.phench_modules import register_modules_commands
from thegent.cli.apps.phench_observability import register_observability_commands
from thegent.cli.apps.phench_projects import register_projects_run
from thegent.cli.apps.phench_repos import register_repos_commands
from thegent.cli.apps.phench_run import register_run_commands
from thegent.cli.apps.phench_snapshot import register_snapshot_commands
from thegent.cli.apps.phench_sync import register_sync_commands
from thegent.cli.apps.phench_target import register_target_commands
from thegent.cli.apps.phench_timeline import register_timeline_commands
from thegent.phench import (
    add_module_to_target,
    add_repo,
    audit_shared_modules,
    audit_shared_modules_across_repos,
    bootstrap_target,
    build_scan_candidates,
    build_project_execution_matrix,
    create_target_snapshot,
    discover_repos,
    get_env_profile,
    import_repos,
    init_target,
    list_modules,
    list_target_snapshots,
    list_targets,
    load_target_lock,
    lock_target,
    materialize_target,
    run_env_doctor_for_target,
    run_target,
    set_env_profile,
    set_repo_ref,
    show_target_snapshot,
    sync_project_modules_from_repos,
    sync_target,
    target_status,
    target_timeline,
)

console = Console()

# Sub-apps
target_app = typer.Typer(help="Target lifecycle: init, add-repo, lock, materialize.")
repos_app = typer.Typer(help="Discover and audit repos in a workspace.")
env_app = typer.Typer(help="Environment preflight commands for targets.")
snapshot_app = typer.Typer(help="Capture and inspect target snapshots.")
modules_app = typer.Typer(help="Manage cross-repo module manifests and shared-module discovery.")
projects_app = typer.Typer(help="Guided target selection and execution workflows.")

app = typer.Typer(
    help="Phench: deterministic project-state runtime control plane.",
    add_completion=False,
)

app.add_typer(target_app, name="target")
app.add_typer(repos_app, name="repos")
app.add_typer(env_app, name="env")
app.add_typer(snapshot_app, name="snapshot")
app.add_typer(modules_app, name="modules")
app.add_typer(projects_app, name="projects")


def _service_attr(name: str) -> Any:
    """Load a function from the phench service module."""
    service_module = import_module("thegent.phench.service")
    try:
        return getattr(service_module, name)
    except AttributeError as exc:
        raise RuntimeError(f"Phench service helper is unavailable: {name}") from exc


# Dynamic wrappers so tests can monkeypatch module-level symbols.
def _init_target_dispatch(*args: Any, **kwargs: Any) -> Any:
    return init_target(*args, **kwargs)


def _bootstrap_target_dispatch(*args: Any, **kwargs: Any) -> Any:
    return bootstrap_target(*args, **kwargs)


def _import_repos_dispatch(*args: Any, **kwargs: Any) -> Any:
    return import_repos(*args, **kwargs)


def _add_repo_dispatch(*args: Any, **kwargs: Any) -> Any:
    return add_repo(*args, **kwargs)


def _set_repo_ref_dispatch(*args: Any, **kwargs: Any) -> Any:
    return set_repo_ref(*args, **kwargs)


def _lock_target_dispatch(*args: Any, **kwargs: Any) -> Any:
    return lock_target(*args, **kwargs)


def _materialize_target_dispatch(*args: Any, **kwargs: Any) -> Any:
    return materialize_target(*args, **kwargs)


def _discover_repos_dispatch(*args: Any, **kwargs: Any) -> Any:
    return discover_repos(*args, **kwargs)


def _timeline_dispatch(*args: Any, **kwargs: Any) -> Any:
    return target_timeline(*args, **kwargs)


def _run_dispatch(*args: Any, **kwargs: Any) -> Any:
    return run_target(*args, **kwargs)


def _run_env_doctor_dispatch(*args: Any, **kwargs: Any) -> Any:
    return run_env_doctor_for_target(*args, **kwargs)


def _set_env_profile_dispatch(*args: Any, **kwargs: Any) -> Any:
    return set_env_profile(*args, **kwargs)


def _get_env_profile_dispatch(*args: Any, **kwargs: Any) -> Any:
    return get_env_profile(*args, **kwargs)


def _sync_target_dispatch(*args: Any, **kwargs: Any) -> Any:
    return sync_target(*args, **kwargs)


def _snapshot_create_dispatch(*args: Any, **kwargs: Any) -> Any:
    return create_target_snapshot(*args, **kwargs)


def _snapshot_list_dispatch(*args: Any, **kwargs: Any) -> Any:
    return list_target_snapshots(*args, **kwargs)


def _snapshot_show_dispatch(*args: Any, **kwargs: Any) -> Any:
    return show_target_snapshot(*args, **kwargs)


def _sync_project_modules_from_repos_dispatch(*args: Any, **kwargs: Any) -> Any:
    return sync_project_modules_from_repos(*args, **kwargs)


def _audit_shared_modules_across_repos_dispatch(*args: Any, **kwargs: Any) -> Any:
    return audit_shared_modules_across_repos(*args, **kwargs)


def _target_status_dispatch(*args: Any, **kwargs: Any) -> Any:
    return target_status(*args, **kwargs)


def _audit_shared_modules_dispatch(*args: Any, **kwargs: Any) -> Any:
    return audit_shared_modules(*args, **kwargs)


def _build_project_execution_matrix_dispatch(*args: Any, **kwargs: Any) -> Any:
    return build_project_execution_matrix(*args, **kwargs)


def scan_shared_modules_across_repos(*args: Any, **kwargs: Any) -> Any:
    return _service_attr("scan_shared_modules_across_repos")(*args, **kwargs)


def materialize_module_candidate_manifest(*args: Any, **kwargs: Any) -> Any:
    return _service_attr("materialize_module_candidate_manifest")(*args, **kwargs)


# Register sub-command groups.
register_target_commands(
    target_app,
    init_target_fn=_init_target_dispatch,
    bootstrap_target_fn=_bootstrap_target_dispatch,
    import_repos_fn=_import_repos_dispatch,
    add_repo_fn=_add_repo_dispatch,
    set_repo_ref_fn=_set_repo_ref_dispatch,
    lock_target_fn=_lock_target_dispatch,
    materialize_target_fn=_materialize_target_dispatch,
)
register_repos_commands(repos_app, discover_repos_fn=_discover_repos_dispatch)
register_env_commands(
    env_app,
    run_env_doctor_for_target_fn=_run_env_doctor_dispatch,
    set_env_profile_fn=_set_env_profile_dispatch,
    get_env_profile_fn=_get_env_profile_dispatch,
)
register_snapshot_commands(
    snapshot_app,
    create_target_snapshot_fn=_snapshot_create_dispatch,
    list_target_snapshots_fn=_snapshot_list_dispatch,
    show_target_snapshot_fn=_snapshot_show_dispatch,
)
register_modules_commands(
    modules_app,
    sync_project_modules_from_repos_fn=_sync_project_modules_from_repos_dispatch,
    audit_shared_modules_across_repos_fn=_audit_shared_modules_across_repos_dispatch,
)
register_timeline_commands(app, target_timeline_fn=_timeline_dispatch)
register_run_commands(app, run_target_fn=_run_dispatch)
register_sync_commands(app, sync_target_fn=_sync_target_dispatch)
register_observability_commands(
    app,
    list_targets_fn=list_targets,
    target_timeline_fn=_timeline_dispatch,
    target_status_fn=_target_status_dispatch,
    run_target_fn=_run_dispatch,
    audit_shared_modules_fn=_audit_shared_modules_dispatch,
)
register_projects_run(
    projects_app,
    list_targets_fn=list_targets,
    list_modules_fn=list_modules,
    load_target_lock_fn=load_target_lock,
    target_timeline_fn=_timeline_dispatch,
    target_status_fn=_target_status_dispatch,
    lock_target_fn=_lock_target_dispatch,
    materialize_target_fn=_materialize_target_dispatch,
    run_target_fn=_run_dispatch,
    build_matrix_fn=_build_project_execution_matrix_dispatch,
)


@target_app.command("add-module", help="Add a module's repos to a target.")
def target_add_module_cmd(
    name: str = typer.Argument(..., help="Target name."),
    module: str = typer.Option(..., "--module", help="Module name."),
    ref: str | None = typer.Option(None, "--ref", help="Override selected ref for all repos."),
    exclude: list[str] = typer.Option([], "--exclude", help="Repo IDs to exclude; may be repeated."),
) -> None:
    try:
        lock = add_module_to_target(
            name,
            module_name=module,
            selected_ref=ref,
            exclude_repos=set(exclude) if exclude else None,
        )
    except ValueError as exc:
        raise typer.BadParameter(str(exc)) from exc
    from dataclasses import asdict
    lock_dict = asdict(lock)
    payload = {
        "target": lock.target_name,
        "module": module,
        "repos": [repo["repo_id"] for repo in lock_dict["repos"]],
    }
    console.print_json(json.dumps(payload).decode())


@app.command("scan-shared-repos", help="Audit shared modules across repos in a phenotype workspace.")
def scan_shared_repos_cmd(
    repos_root: str | None = typer.Option(None, "--repos-root", help="Root directory to scan."),
    exclude: list[str] = typer.Option([], "--exclude", help="Repo IDs to exclude; may be repeated."),
    min_repo_count: int = typer.Option(2, "--min-repo-count", "--min-repos", help="Minimum repo count for a shared module."),
    repos_root_mode: str = typer.Option("auto", "--repos-root-mode", help="Root discovery: auto|projects|phenotype."),
    candidate_name_regex: str | None = typer.Option(None, "--candidate-regex", "--candidate-name-regex", help="Regex to filter candidate names."),
    candidates: bool = typer.Option(
        False,
        "--candidates",
        help="Include module_candidates in output.",
    ),
    omit_candidates: bool = typer.Option(
        False,
        "--omit-candidates",
        help="Omit module_candidates from output.",
    ),
) -> None:
    # omit_candidates takes precedence over candidates when both are passed
    if omit_candidates:
        candidates = False

    try:
        state = scan_shared_modules_across_repos(
            repos_root=None if repos_root is None else Path(repos_root),
            exclude_repos={value.strip() for value in exclude if value.strip()},
            min_repo_count=min_repo_count,
            repos_root_mode=repos_root_mode,
            candidate_name_regex=candidate_name_regex,
            candidates=candidates,
            omit_candidates=omit_candidates,
        )
    except ValueError as exc:
        raise typer.BadParameter(str(exc)) from exc

    # Build module_candidates from shared_modules if --candidates was requested
    # and the service didn't already include them
    if candidates and "module_candidates" not in state:
        shared = state.get("shared_modules", {})
        state["module_candidates"] = build_scan_candidates(shared)

    console.print_json(json.dumps(state).decode())


@app.command(
    "materialize-module-manifest",
    help="Materialize module manifest into Phenotype/projects/modules/<module>.",
)
def materialize_module_manifest_cmd(
    module: str = typer.Option(..., "--module", help="Module name."),
    repos_root: str | None = typer.Option(None, "--repos-root", help="Root directory to scan."),
    repos_root_mode: str | None = typer.Option(None, "--repos-root-mode", help="Root discovery mode."),
    repo: list[str] = typer.Option([], "--repo", help="Explicit repo IDs to pin; may be repeated."),
    min_repo_count: int = typer.Option(2, "--min-repo-count", "--min-repos", help="Minimum repo count."),
    output_dir: str | None = typer.Option(None, "--output-dir", help="Output directory for manifests."),
    dry_run: bool = typer.Option(False, "--dry-run", help="Print changes without writing."),
    print_snippets: bool = typer.Option(False, "--print-snippets", "--print-target-snippets", help="Print shell snippets."),
) -> None:
    try:
        payload = materialize_module_candidate_manifest(
            module=module,
            repos_root=None if repos_root is None else Path(repos_root),
            repos_root_mode=repos_root_mode,
            repos=repo if repo else None,
            min_repo_count=min_repo_count,
            output_dir=None if output_dir is None else Path(output_dir),
            dry_run=dry_run,
        )
    except ValueError as exc:
        raise typer.BadParameter(str(exc)) from exc

    if print_snippets:
        module_name = payload.get("module_name", module)
        payload["shell_snippets"] = [
            f"thegent phench target init {module_name} --mode stack",
            f"thegent phench target add-module {module_name} --module {module_name}",
        ]
    console.print_json(json.dumps(payload).decode())


# Alias for CLI entry-point compatibility.
phench_cli = app

__all__ = ["app", "phench_cli"]
