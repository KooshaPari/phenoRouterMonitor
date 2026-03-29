"""Audit CLI commands for thegent.

WBS: wp-71004-audit-cli
FR Traceability: FR-VER-005 (audit log and diff CLI)

Commands:
    thegent audit log  [--project NAME] [--limit N]
    thegent audit diff <sha1> <sha2> [--project NAME]
    thegent audit list [--agent ID] [--session ID]
"""

from __future__ import annotations

from pathlib import Path
from typing import Annotated

from rich.console import Console
from rich.table import Table
from typer import Argument, Typer

from thegent.audit.shadow_audit_git import ShadowAuditGit
from thegent.registry.project_registry import ProjectRegistry

console = Console()

# Default paths for databases
DEFAULT_AUDIT_DB = Path.home() / ".thegent" / "audit.db"
DEFAULT_REGISTRY_DB = Path.home() / ".thegent" / "registry.db"


def _get_registry() -> ProjectRegistry:
    """Get or create the project registry."""
    DEFAULT_REGISTRY_DB.parent.mkdir(parents=True, exist_ok=True)
    return ProjectRegistry(DEFAULT_REGISTRY_DB)


def _get_shadow() -> ShadowAuditGit:
    """Get or create the shadow audit git."""
    DEFAULT_AUDIT_DB.parent.mkdir(parents=True, exist_ok=True)
    return ShadowAuditGit(DEFAULT_AUDIT_DB)


app = Typer(help="Audit commands for tracking agent git operations.")


@app.command()
def log(
    project: Annotated[str, Argument(help="Project name to show audit log for.")],
    limit: Annotated[int | None, Argument(help="Maximum number of entries to show.")] = None,
) -> None:
    """Show audit log entries for a project."""
    registry = _get_registry()
    shadow = _get_shadow()

    proj = registry.get_project(project)
    if proj is None:
        console.print(f"[red]Project '{project}' not found.[/red]")
        raise SystemExit(1)

    entries = shadow.get_entries(proj.id, limit=limit)

    if not entries:
        console.print(f"[yellow]No audit entries for project '{project}'.[/yellow]")
        return

    table = Table(title=f"Audit Log: {project}")
    table.add_column("SHA", style="cyan")
    table.add_column("Message", style="white")
    table.add_column("Timestamp", style="dim")

    for entry in reversed(entries):  # Show oldest first
        table.add_row(
            entry.sha[:8],
            entry.message[:60],
            entry.timestamp.strftime("%Y-%m-%d %H:%M:%S"),
        )

    console.print(table)


@app.command()
def diff(
    sha1: Annotated[str, Argument(help="First commit SHA.")],
    sha2: Annotated[str, Argument(help="Second commit SHA.")],
    project: Annotated[str, Argument(help="Project name.")],
) -> None:
    """Show diff between two audit entries."""
    registry = _get_registry()
    shadow = _get_shadow()

    proj = registry.get_project(project)
    if proj is None:
        console.print(f"[red]Project '{project}' not found.[/red]")
        raise SystemExit(1)

    entry1 = shadow.get_entry_by_sha(proj.id, sha1)
    entry2 = shadow.get_entry_by_sha(proj.id, sha2)

    if entry1 is None:
        console.print(f"[red]SHA '{sha1}' not found.[/red]")
        raise SystemExit(1)

    if entry2 is None:
        console.print(f"[red]SHA '{sha2}' not found.[/red]")
        raise SystemExit(1)

    console.print(f"[cyan]Entry 1:[/cyan] {entry1.sha[:8]} - {entry1.message}")
    if entry1.diff:
        console.print(entry1.diff)

    console.print()
    console.print(f"[cyan]Entry 2:[/cyan] {entry2.sha[:8]} - {entry2.message}")
    if entry2.diff:
        console.print(entry2.diff)


@app.command("list")
def audit_list(
    agent: Annotated[str | None, Argument(help="Filter by agent ID.")] = None,
    session: Annotated[str | None, Argument(help="Filter by session ID.")] = None,
) -> None:
    """List audit entries with optional filters.

    This command wraps the MAIF audit log with filtering capabilities.
    """
    # This would integrate with MAIF for full audit listing
    # For now, show the shadow audit git entries
    shadow = _get_shadow()
    registry = _get_registry()

    projects = registry.list_projects()

    if not projects:
        console.print("[yellow]No projects registered.[/yellow]")
        return

    table = Table(title="Audit Entries")
    table.add_column("Project", style="cyan")
    table.add_column("SHA", style="cyan")
    table.add_column("Message", style="white")

    for proj in projects:
        entries = shadow.get_entries(proj.id, limit=10)
        for entry in entries:
            if agent and agent not in entry.message and agent not in proj.name:
                continue
            table.add_row(
                proj.name,
                entry.sha[:8],
                entry.message[:60],
            )

    console.print(table)


if __name__ == "__main__":
    app()
