"""Structured worktree governance CLI passthrough."""

from __future__ import annotations

import subprocess
from pathlib import Path
from typing import Literal

import typer
from rich.console import Console

console = Console()

worktree_governance_app = typer.Typer(
    help="Structured worktree governance commands backed by scripts/worktree_governance.sh."
)


def register_worktree_governance_commands(parent_app: typer.Typer) -> None:
    """Register the structured worktree governance namespace."""
    parent_app.add_typer(
        worktree_governance_app,
        name="governance",
        help="Structured worktree lifecycle management.",
    )


def _resolve_repo_root(root: Path | None) -> Path:
    if root is not None:
        return root
    proc = subprocess.run(
        ["git", "rev-parse", "--show-toplevel"],
        capture_output=True,
        text=True,
        check=False,
    )
    if proc.returncode != 0:
        raise typer.Exit(proc.returncode)
    return Path(proc.stdout.strip())


def _script_path(project_root: Path) -> Path:
    script = project_root / "scripts" / "worktree_governance.sh"
    if not script.exists():
        raise FileNotFoundError(f"missing worktree governance script: {script}")
    return script


def run_worktree_governance_script(project_root: Path, *args: str) -> subprocess.CompletedProcess[str]:
    """Run the canonical worktree governance script and return the completed process."""
    return subprocess.run(
        [str(_script_path(project_root)), *args],
        cwd=project_root,
        capture_output=True,
        text=True,
        check=False,
    )


def _run_script(project_root: Path, *args: str) -> None:
    proc = run_worktree_governance_script(project_root, *args)
    if proc.stdout:
        console.print(proc.stdout, end="")
    if proc.stderr:
        console.print(proc.stderr, end="", style="red")
    if proc.returncode != 0:
        raise typer.Exit(proc.returncode)


@worktree_governance_app.command("path")
def worktree_governance_path(
    domain: str = typer.Argument(..., help="Task classifier domain"),
    scale: str = typer.Argument(..., help="Task classifier scale"),
    change_anchor: str = typer.Argument(..., help="AgilePlus change anchor"),
    state: str = typer.Argument(..., help="Lifecycle state"),
    root: Path | None = typer.Option(None, "--root", "-r", help="Repository root"),
) -> None:
    """Print the canonical structured path for a worktree."""
    _run_script(_resolve_repo_root(root), "path", domain, scale, change_anchor, state)


@worktree_governance_app.command("new")
def worktree_governance_new(
    domain: str = typer.Argument(..., help="Task classifier domain"),
    scale: str = typer.Argument(..., help="Task classifier scale"),
    change_anchor: str = typer.Argument(..., help="AgilePlus change anchor"),
    start_point: str = typer.Argument("main", help="Start point for the new worktree"),
    root: Path | None = typer.Option(None, "--root", "-r", help="Repository root"),
) -> None:
    """Create a structured worktree."""
    _run_script(_resolve_repo_root(root), "new", domain, scale, change_anchor, start_point)


@worktree_governance_app.command("state")
def worktree_governance_state(
    change_anchor: str = typer.Argument(..., help="AgilePlus change anchor"),
    new_state: str = typer.Argument(..., help="New lifecycle state"),
    root: Path | None = typer.Option(None, "--root", "-r", help="Repository root"),
) -> None:
    """Move a worktree between lifecycle states."""
    _run_script(_resolve_repo_root(root), "state", change_anchor, new_state)


@worktree_governance_app.command("list")
def worktree_governance_list(
    root: Path | None = typer.Option(None, "--root", "-r", help="Repository root"),
) -> None:
    """List structured worktrees."""
    _run_script(_resolve_repo_root(root), "list")


@worktree_governance_app.command("prune")
def worktree_governance_prune(
    dry_run: bool = typer.Option(False, "--dry-run", "-n", help="Show what would be pruned"),
    root: Path | None = typer.Option(None, "--root", "-r", help="Repository root"),
) -> None:
    """Prune done or broken worktrees."""
    if dry_run:
        _run_script(_resolve_repo_root(root), "prune", "--dry-run")
    else:
        _run_script(_resolve_repo_root(root), "prune")


@worktree_governance_app.command("refresh")
def worktree_governance_refresh(
    change_anchor: str = typer.Argument(..., help="AgilePlus change anchor"),
    remote: str = typer.Option("origin", "--remote", help="Remote to fetch before refresh"),
    upstream_ref: str | None = typer.Option(
        None,
        "--ref",
        "--upstream",
        help="Upstream ref to rebase/merge onto (default: remote/change-anchor branch)",
    ),
    strategy: Literal["rebase", "merge"] = typer.Option(
        "rebase",
        "--strategy",
        help="Refresh strategy (rebase recommended for canary and PR worktrees)",
    ),
    root: Path | None = typer.Option(None, "--root", "-r", help="Repository root"),
) -> None:
    """Fetch and refresh a structured worktree against an upstream branch."""
    args = ["refresh", change_anchor, "--remote", remote, "--strategy", strategy]
    if upstream_ref:
        args.extend(["--ref", upstream_ref])
    _run_script(_resolve_repo_root(root), *args)


@worktree_governance_app.command("migrate-legacy")
def worktree_governance_migrate_legacy(
    legacy_path: Path = typer.Argument(..., help="Legacy worktree path outside the canonical root"),
    domain: str = typer.Argument(..., help="Canonical task classifier domain"),
    scale: str = typer.Argument(..., help="Canonical task classifier scale"),
    change_anchor: str = typer.Argument(..., help="AgilePlus change anchor"),
    state: str = typer.Argument("active", help="Canonical lifecycle state"),
    root: Path | None = typer.Option(None, "--root", "-r", help="Repository root"),
) -> None:
    """Migrate a legacy worktree into the canonical structured root."""
    _run_script(
        _resolve_repo_root(root),
        "migrate-legacy",
        str(legacy_path),
        domain,
        scale,
        change_anchor,
        state,
    )


@worktree_governance_app.command("check")
def worktree_governance_check(
    root: Path | None = typer.Option(None, "--root", "-r", help="Repository root"),
) -> None:
    """Validate the structured worktree inventory."""
    _run_script(_resolve_repo_root(root), "check")
