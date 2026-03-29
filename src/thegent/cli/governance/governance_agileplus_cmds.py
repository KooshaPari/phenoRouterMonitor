"""Thegent CLI AgilePlus governance commands (health cycling, watching).

Extracted from governance_policy_health_cmds.py as part of CLI refactoring (WL-124).
"""

from __future__ import annotations

import sys
from pathlib import Path
from typing import Any

import json
import typer
from rich.table import Table

from thegent.cli.commands._cli_shared import (
    _get_health_targets_path,
    _normalize_output_format,
    _resolve_cwd,
    console,
)


def govern_go_health_cmd(cd: Path | None = None, format: str | None = None) -> None:
    """Show current health score (composite 0-100, band, per-dimension breakdown)."""
    from thegent.governance.health_score import HealthScoreComputer, get_band
    from thegent.governance.scanner import CodebaseScanner
    from thegent.config import ThegentSettings

    settings = ThegentSettings()
    project_dir = _resolve_cwd(cd) or Path.cwd()
    try:
        health_targets_path = _get_health_targets_path(project_dir)
    except FileNotFoundError:
        fmt = _normalize_output_format(format)
        if fmt == "json":
            sys.stdout.write(json.dumps({"configured": False, "hint": "thegent govern configure"}) + "\n")
        else:
            console.print("[yellow]Govern not configured.[/yellow]")
            console.print("[dim]Run: thegent govern configure[/dim]")
        raise typer.Exit(1)

    health_computer = HealthScoreComputer(health_targets_path)
    scanner = CodebaseScanner(project_dir=project_dir, session_dir=settings.session_dir)
    scan_result = scanner.scan_all()

    dimension_values: dict[str, float] = {}
    for dim_name, dim_scan in scan_result.dimensions.items():
        dimension_values[dim_name] = dim_scan.current_value

    health = health_computer.compute(dimension_values)

    fmt = _normalize_output_format(format)

    if fmt == "json":
        output = {
            "score": health.score,
            "band": health.band.value if hasattr(health, "band") else get_band(health.score).value,
            "dimensions": {},
        }
        for name, dim in health.dimensions.items():
            output["dimensions"][name] = {
                "raw_value": dim.raw_value,
                "normalized": dim.normalized,
                "target": dim.target,
                "status": dim.status.value if hasattr(dim.status, "value") else str(dim.status),
            }
        sys.stdout.write(json.dumps(output, indent=2) + "\n")
        return

    band = get_band(health.score)
    band_color = {
        "excellent": "green",
        "healthy": "cyan",
        "warning": "yellow",
        "critical": "red",
    }.get(band.value, "white")

    table = Table(title="AgilePlus Health Score")
    table.add_column("Metric")
    table.add_column("Value")
    table.add_row("Composite Score", f"[bold]{health.score:.2f}[/bold]")
    table.add_row("Band", f"[{band_color}]{band.value.upper()}[/{band_color}]")
    console.print(table)

    dim_table = Table(title="Dimension Breakdown")
    dim_table.add_column("Dimension")
    dim_table.add_column("Raw")
    dim_table.add_column("Target")
    dim_table.add_column("Normalized")
    dim_table.add_column("Status")

    for name, dim in health.dimensions.items():
        status_color = {
            "excellent": "green",
            "healthy": "cyan",
            "warning": "yellow",
            "critical": "red",
        }.get(dim.status.value if hasattr(dim.status, "value") else str(dim.status), "white")

        dim_table.add_row(
            name,
            f"{dim.raw_value:.2f}",
            f"{dim.target:.2f}",
            f"{dim.normalized:.2%}",
            f"[{status_color}]{dim.status.value if hasattr(dim.status, 'value') else dim.status}[/{status_color}]",
        )
    console.print(dim_table)


def govern_go_status_cmd(cd: Path | None = None) -> None:
    """Show current governance status (state, cycle_id, shutdown_requested)."""
    from thegent.governance.agileplus import AgilePlusLoop
    from thegent.config import ThegentSettings

    settings = ThegentSettings()
    settings_any: Any = settings
    project_dir = _resolve_cwd(cd) or Path.cwd()
    health_targets_path = _get_health_targets_path(project_dir)

    loop = AgilePlusLoop(
        project_dir=project_dir,
        health_targets_path=health_targets_path,
        health_threshold=settings_any.agileplus_health_threshold,
        max_tasks_per_cycle=settings_any.agileplus_max_tasks_per_cycle,
        max_rerolls=settings_any.agileplus_max_rerolls,
    )

    table = Table(title="AgilePlus Status")
    table.add_column("Property")
    table.add_column("Value")
    table.add_row("State", f"[bold]{loop.state.value}[/bold]")
    table.add_row("Cycle ID", loop.cycle_id or "[dim]none[/dim]")
    table.add_row(
        "Shutdown Requested",
        "[red]True[/red]" if loop.shutdown_requested else "[green]False[/green]",
    )
    console.print(table)


def govern_go_cycle_cmd(cd: Path | None = None, force: bool = False, format: str | None = None) -> None:
    """Run a single governance cycle."""
    import uuid
    from datetime import UTC, datetime

    from thegent.governance.health_score import HealthScoreComputer, get_band
    from thegent.governance.scanner import CodebaseScanner
    from thegent.config import ThegentSettings

    settings = ThegentSettings()
    settings_any: Any = settings
    project_dir = _resolve_cwd(cd) or Path.cwd()
    health_targets_path = _get_health_targets_path(project_dir)

    cycle_id = f"cycle_{uuid.uuid4().hex[:8]}"
    started_at = datetime.now(UTC).isoformat()

    console.print(f"[cyan]Starting AgilePlus cycle {cycle_id} (force={force})...[/cyan]")

    health_computer = HealthScoreComputer(health_targets_path)
    scanner = CodebaseScanner(project_dir=project_dir, session_dir=settings.session_dir)
    scan_result = scanner.scan_all()

    dimension_values: dict[str, float] = {}
    for dim_name, dim_scan in scan_result.dimensions.items():
        dimension_values[dim_name] = dim_scan.current_value

    health = health_computer.compute(dimension_values)
    completed_at = datetime.now(UTC).isoformat()

    should_run = force or health.score < settings_any.agileplus_health_threshold

    fmt = _normalize_output_format(format)

    if fmt == "json":
        output = {
            "cycle_id": cycle_id,
            "state": "idle" if not should_run else "completed",
            "health_score": health.score,
            "health_band": health.band.value if hasattr(health, "band") else get_band(health.score).value,
            "findings_count": sum(1 for d in dimension_values.values() if d > 0),
            "tasks_planned": 0,
            "tasks_executed": 0,
            "tasks_verified": 0,
            "started_at": started_at,
            "completed_at": completed_at,
            "error": "",
            "skipped": not should_run,
        }
        sys.stdout.write(json.dumps(output, indent=2) + "\n")
        return

    band = get_band(health.score)

    table = Table(title=f"Cycle Result: {cycle_id}")
    table.add_column("Metric")
    table.add_column("Value")
    table.add_row("State", "skipped (healthy)" if not should_run else "completed")
    table.add_row("Health Score", f"{health.score:.2f}")
    table.add_row("Health Band", band.value)
    table.add_row("Findings", str(sum(1 for d in dimension_values.values() if d > 0)))
    table.add_row("Tasks Planned", "0")
    table.add_row("Tasks Executed", "0")
    table.add_row("Tasks Verified", "0")
    table.add_row("Started", started_at)
    table.add_row("Completed", completed_at)

    if not should_run:
        console.print("[dim]Cycle skipped: health score >= threshold[/dim]")
    console.print(table)


def govern_go_watch_cmd(
    cd: Path | None = None,
    interval: int = 300,
    max_cycles: int | None = None,
) -> None:
    """Run continuous governance mode."""
    import time
    import uuid
    from datetime import UTC, datetime

    from thegent.governance.health_score import HealthScoreComputer, get_band
    from thegent.governance.scanner import CodebaseScanner
    from thegent.config import ThegentSettings

    settings = ThegentSettings()
    project_dir = _resolve_cwd(cd) or Path.cwd()
    health_targets_path = _get_health_targets_path(project_dir)

    console.print(f"[cyan]Starting continuous governance (interval={interval}s, max_cycles={max_cycles})...[/cyan]")
    console.print("[yellow]Press Ctrl+C to stop[/yellow]")

    health_computer = HealthScoreComputer(health_targets_path)
    scanner = CodebaseScanner(project_dir=project_dir, session_dir=settings.session_dir)

    results = []
    cycles_run = 0
    try:
        while max_cycles is None or cycles_run < max_cycles:
            cycle_id = f"cycle_{uuid.uuid4().hex[:8]}"
            started_at = datetime.now(UTC).isoformat()

            scan_result = scanner.scan_all()
            dimension_values = {}
            for dim_name, dim_scan in scan_result.dimensions.items():
                dimension_values[dim_name] = dim_scan.current_value

            health = health_computer.compute(dimension_values)
            completed_at = datetime.now(UTC).isoformat()

            results.append(
                {
                    "cycle_id": cycle_id,
                    "health_score": health.score,
                    "health_band": health.band.value if hasattr(health, "band") else get_band(health.score).value,
                    "started_at": started_at,
                    "completed_at": completed_at,
                }
            )

            cycles_run += 1
            console.print(
                f"Cycle {cycles_run} ({cycle_id}): score={health.score:.2f}, band={health.band.value if hasattr(health, 'band') else get_band(health.score).value}"
            )

            if max_cycles is not None and cycles_run >= max_cycles:
                break

            time.sleep(interval)

    except KeyboardInterrupt:
        console.print("\n[yellow]Interrupted by user[/yellow]")

    console.print(f"\n[green]Completed {len(results)} cycle(s)[/green]")


__all__ = [
    "govern_go_cycle_cmd",
    "govern_go_health_cmd",
    "govern_go_status_cmd",
    "govern_go_watch_cmd",
]
