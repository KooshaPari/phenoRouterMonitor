"""Shared helpers for governance health-related CLI commands."""

from __future__ import annotations

from typing import Any, Callable

from rich.table import Table


def extract_dimension_values(scan_result: Any) -> dict[str, float]:
    """Extract per-dimension current values from scanner output."""
    return {dim_name: dim_scan.current_value for dim_name, dim_scan in scan_result.dimensions.items()}


def resolve_band_value(health: Any, get_band: Callable[[float], Any]) -> str:
    """Resolve health band string with fallback support."""
    band = health.band if hasattr(health, "band") else get_band(health.score)
    return band.value if hasattr(band, "value") else str(band)


def resolve_status_value(status: Any) -> str:
    """Resolve status enum to printable value."""
    return status.value if hasattr(status, "value") else str(status)


def build_health_json_output(health: Any, get_band: Callable[[float], Any]) -> dict[str, Any]:
    """Build the JSON payload for `govern go health`."""
    output = {
        "score": health.score,
        "band": resolve_band_value(health, get_band),
        "dimensions": {},
    }
    for name, dim in health.dimensions.items():
        output["dimensions"][name] = {
            "raw_value": dim.raw_value,
            "normalized": dim.normalized,
            "target": dim.target,
            "status": resolve_status_value(dim.status),
        }
    return output


def build_health_summary_table(score: float, band_value: str) -> Table:
    """Build rich table for top-level health summary."""
    band_color = {
        "excellent": "green",
        "healthy": "cyan",
        "warning": "yellow",
        "critical": "red",
    }.get(band_value, "white")

    table = Table(title="AgilePlus Health Score")
    table.add_column("Metric")
    table.add_column("Value")
    table.add_row("Composite Score", f"[bold]{score:.2f}[/bold]")
    table.add_row("Band", f"[{band_color}]{band_value.upper()}[/{band_color}]")
    return table


def build_health_dimensions_table(health: Any) -> Table:
    """Build rich table for per-dimension health values."""
    dim_table = Table(title="Dimension Breakdown")
    dim_table.add_column("Dimension")
    dim_table.add_column("Raw")
    dim_table.add_column("Target")
    dim_table.add_column("Normalized")
    dim_table.add_column("Status")

    for name, dim in health.dimensions.items():
        status_value = resolve_status_value(dim.status)
        status_color = {
            "excellent": "green",
            "healthy": "cyan",
            "warning": "yellow",
            "critical": "red",
        }.get(status_value, "white")

        dim_table.add_row(
            name,
            f"{dim.raw_value:.2f}",
            f"{dim.target:.2f}",
            f"{dim.normalized:.2%}",
            f"[{status_color}]{status_value}[/{status_color}]",
        )
    return dim_table


def count_findings(dimension_values: dict[str, float]) -> int:
    """Count dimensions with positive findings."""
    return sum(1 for value in dimension_values.values() if value > 0)


def build_cycle_json_output(
    *,
    cycle_id: str,
    should_run: bool,
    health_score: float,
    health_band: str,
    findings_count: int,
    started_at: str,
    completed_at: str,
) -> dict[str, Any]:
    """Build the JSON payload for `govern go cycle`."""
    return {
        "cycle_id": cycle_id,
        "state": "idle" if not should_run else "completed",
        "health_score": health_score,
        "health_band": health_band,
        "findings_count": findings_count,
        "tasks_planned": 0,
        "tasks_executed": 0,
        "tasks_verified": 0,
        "started_at": started_at,
        "completed_at": completed_at,
        "error": "",
        "skipped": not should_run,
    }


def build_cycle_result_table(
    *,
    cycle_id: str,
    should_run: bool,
    health_score: float,
    health_band: str,
    findings_count: int,
    started_at: str,
    completed_at: str,
) -> Table:
    """Build rich table for `govern go cycle` output."""
    table = Table(title=f"Cycle Result: {cycle_id}")
    table.add_column("Metric")
    table.add_column("Value")
    table.add_row("State", "skipped (healthy)" if not should_run else "completed")
    table.add_row("Health Score", f"{health_score:.2f}")
    table.add_row("Health Band", health_band)
    table.add_row("Findings", str(findings_count))
    table.add_row("Tasks Planned", "0")
    table.add_row("Tasks Executed", "0")
    table.add_row("Tasks Verified", "0")
    table.add_row("Started", started_at)
    table.add_row("Completed", completed_at)
    return table
