"""Governance health and status commands (WL-124).

Core health score computation, status monitoring, and governance cycles.
"""

from __future__ import annotations

import json
import sys
import uuid
from pathlib import Path

import typer

from rich.table import Table

from thegent.cli.commands._cli_shared import (
    ThegentSettings,
    _bootstrap_metric_contracts,
    _get_health_targets_path,
    _HEALTH_TARGETS_TEMPLATE,
    _normalize_output_format,
    _resolve_cwd,
    console,
)
from thegent.cli.governance.governance_health_helpers import (
    build_cycle_json_output,
    build_cycle_result_table,
    build_health_dimensions_table,
    build_health_json_output,
    build_health_summary_table,
    count_findings,
    extract_dimension_values,
    resolve_band_value,
)


def govern_configure_cmd(cd: Path | None = None, force: bool = False) -> None:
    """Bootstrap governance: create contracts/health-targets.json if missing."""
    project_dir = _resolve_cwd(cd) or Path.cwd()
    contracts_dir = project_dir / "contracts"
    health_targets = contracts_dir / "health-targets.json"
    if health_targets.exists() and not force:
        console.print("[green]Govern already configured.[/green]")
        return
    contracts_dir.mkdir(parents=True, exist_ok=True)
    health_targets.write_text(_HEALTH_TARGETS_TEMPLATE, encoding="utf-8")
    _bootstrap_metric_contracts(project_dir, force=force)
    console.print("[green]Govern configured.[/green] Run: thegent govern go health")


def govern_go_health_cmd(cd: Path | None = None, format: str | None = None) -> None:
    """Show current health score (composite 0-100, band, per-dimension breakdown)."""
    from thegent.governance.health_score import HealthScoreComputer, get_band
    from thegent.governance.scanner import CodebaseScanner

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

    # Create health score computer
    health_computer = HealthScoreComputer(health_targets_path)

    # Create scanner and run scan
    scanner = CodebaseScanner(project_dir=project_dir, session_dir=settings.session_dir)
    scan_result = scanner.scan_all()

    # Extract dimension values from scan result
    dimension_values = extract_dimension_values(scan_result)

    # Compute health score
    health = health_computer.compute(dimension_values)

    fmt = _normalize_output_format(format)

    if fmt == "json":
        output = build_health_json_output(health, get_band)
        sys.stdout.write(json.dumps(output, indent=2) + "\n")
        return

    # Rich output
    band = get_band(health.score)
    table = build_health_summary_table(health.score, band.value)
    console.print(table)

    # Dimensions table
    dim_table = build_health_dimensions_table(health)
    console.print(dim_table)


def govern_go_status_cmd(cd: Path | None = None) -> None:
    """Show current governance status (state, cycle_id, shutdown_requested)."""
    from thegent.config import ThegentSettings
    from thegent.governance.agileplus import AgilePlusLoop

    settings = ThegentSettings()
    project_dir = _resolve_cwd(cd) or Path.cwd()
    health_targets_path = _get_health_targets_path(project_dir)

    # Create loop instance to check status (doesn't run cycle)
    loop = AgilePlusLoop(
        project_dir=project_dir,
        health_targets_path=health_targets_path,
        health_threshold=settings.agileplus_health_threshold,
        max_tasks_per_cycle=settings.agileplus_max_tasks_per_cycle,
        max_rerolls=settings.agileplus_max_rerolls,
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
    from datetime import UTC, datetime

    from thegent.config import ThegentSettings
    from thegent.governance.health_score import HealthScoreComputer, get_band
    from thegent.governance.scanner import CodebaseScanner

    settings = ThegentSettings()
    project_dir = _resolve_cwd(cd) or Path.cwd()
    health_targets_path = _get_health_targets_path(project_dir)

    cycle_id = f"cycle_{uuid.uuid4().hex[:8]}"
    started_at = datetime.now(UTC).isoformat()

    console.print(f"[cyan]Starting AgilePlus cycle {cycle_id} (force={force})...[/cyan]")

    # Compute health score
    health_computer = HealthScoreComputer(health_targets_path)
    scanner = CodebaseScanner(project_dir=project_dir, session_dir=settings.session_dir)
    scan_result = scanner.scan_all()

    dimension_values = extract_dimension_values(scan_result)

    health = health_computer.compute(dimension_values)
    completed_at = datetime.now(UTC).isoformat()
    health_band_value = resolve_band_value(health, get_band)
    findings_count = count_findings(dimension_values)

    # Determine if we should run full cycle based on health threshold
    should_run = force or health.score < settings.agileplus_health_threshold

    fmt = _normalize_output_format(format)

    if fmt == "json":
        output = build_cycle_json_output(
            cycle_id=cycle_id,
            should_run=should_run,
            health_score=health.score,
            health_band=health_band_value,
            findings_count=findings_count,
            started_at=started_at,
            completed_at=completed_at,
        )
        sys.stdout.write(json.dumps(output, indent=2) + "\n")
        return

    table = build_cycle_result_table(
        cycle_id=cycle_id,
        should_run=should_run,
        health_score=health.score,
        health_band=health_band_value,
        findings_count=findings_count,
        started_at=started_at,
        completed_at=completed_at,
    )

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
    from datetime import UTC, datetime

    from thegent.config import ThegentSettings
    from thegent.governance.health_score import HealthScoreComputer, get_band
    from thegent.governance.scanner import CodebaseScanner

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

            # Compute health score
            scan_result = scanner.scan_all()
            dimension_values = extract_dimension_values(scan_result)

            health = health_computer.compute(dimension_values)
            health_band_value = resolve_band_value(health, get_band)
            completed_at = datetime.now(UTC).isoformat()

            results.append(
                {
                    "cycle_id": cycle_id,
                    "health_score": health.score,
                    "health_band": health_band_value,
                    "started_at": started_at,
                    "completed_at": completed_at,
                }
            )

            cycles_run += 1
            console.print(f"Cycle {cycles_run} ({cycle_id}): score={health.score:.2f}, band={health_band_value}")

            if max_cycles is not None and cycles_run >= max_cycles:
                break

            # Wait for next cycle
            time.sleep(interval)

    except KeyboardInterrupt:
        console.print("\n[yellow]Interrupted by user[/yellow]")

    console.print(f"\n[green]Completed {len(results)} cycle(s)[/green]")


__all__ = [
    "govern_configure_cmd",
    "govern_go_health_cmd",
    "govern_go_status_cmd",
    "govern_go_cycle_cmd",
    "govern_go_watch_cmd",
]
