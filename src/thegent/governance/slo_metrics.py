"""SLO metric emitter for CLI decomposition governance (WL-135 B90-W2-A5).

Provides SloMetric, SloThresholds, and SloEmitter for recording and
evaluating decomposition-focused SLO metrics defined in Wave-1 A4.

Metric schema (v1.0):
    file_loc                       — total LOC per file
    function_loc_p95               — p95 function LOC across a module
    impl_importers                 — count of modules importing impl.py
    cross_boundary_import_edges    — count of core→tooling import edges
    cli_help_p95_ms                — p95 latency (ms) for `thegent --help`
    run_command_p95_ms             — p95 latency (ms) for `thegent run`
    decomposition_checkpoint_pass_rate — fraction of checkpoint gates passing

Thresholds (green/yellow/red):
    green  = metric within green bounds
    yellow = metric between green and red (warn zone)
    red    = metric beyond red trigger threshold

Fail-fast: SloEmitter raises IOError loudly if the JSONL write fails.
No fallbacks, no silent errors.
"""
# @trace WL-135 B90-W2-A5

from __future__ import annotations

import json
from dataclasses import asdict, dataclass, field
from datetime import UTC, datetime
from pathlib import Path
from typing import Any

_DEFAULT_OUTPUT_PATH = Path(".quality") / "slo-metrics.jsonl"


@dataclass
class SloMetric:
    """Single SLO measurement snapshot.

    All numeric fields use float for uniformity; integer metrics like
    file_loc should be passed as int (Python will coerce to float in
    the JSON output via json.dumps with no loss of precision for ints).
    """

    # LOC metrics
    file_loc: float
    function_loc_p95: float
    # Coupling metrics
    impl_importers: float
    cross_boundary_import_edges: float
    # CLI SLO metrics
    cli_help_p95_ms: float
    run_command_p95_ms: float
    # Process metric
    decomposition_checkpoint_pass_rate: float
    # Provenance
    timestamp: str = field(default_factory=lambda: datetime.now(UTC).isoformat())
    source: str = "unknown"


@dataclass(frozen=True)
class SloThresholds:
    """Green and red thresholds for each SLO metric field.

    green_max / green_min define the "all clear" zone.
    red_min  / red_max   define the "alert" zone.
    Fields that have no threshold in a direction use float('inf') / 0.

    Thresholds derived from Wave-1 A4 artifact:
        file_loc              green_max=1200 red_min=1800
        function_loc_p95      green_max=80   red_min=120
        impl_importers        green_max=20   red_min=35
        cross_boundary…       green_max=25   red_min=40
        cli_help_p95_ms       green_max=250  red_min=400
        run_command_p95_ms    green_max=500  red_min=800
        checkpoint_pass_rate  green_min=1.0  red_max=0.95
    """

    # file_loc: lower is better
    file_loc_green_max: float = 1200.0
    file_loc_red_min: float = 1800.0

    # function_loc_p95: lower is better
    function_loc_p95_green_max: float = 80.0
    function_loc_p95_red_min: float = 120.0

    # impl_importers: lower is better
    impl_importers_green_max: float = 20.0
    impl_importers_red_min: float = 35.0

    # cross_boundary_import_edges: lower is better
    cross_boundary_import_edges_green_max: float = 25.0
    cross_boundary_import_edges_red_min: float = 40.0

    # cli_help_p95_ms: lower is better
    cli_help_p95_ms_green_max: float = 250.0
    cli_help_p95_ms_red_min: float = 400.0

    # run_command_p95_ms: lower is better
    run_command_p95_ms_green_max: float = 500.0
    run_command_p95_ms_red_min: float = 800.0

    # decomposition_checkpoint_pass_rate: higher is better
    decomposition_checkpoint_pass_rate_green_min: float = 1.0
    decomposition_checkpoint_pass_rate_red_max: float = 0.95


def _evaluate_field_lower_is_better(
    value: float,
    green_max: float,
    red_min: float,
) -> str:
    """Return 'green', 'yellow', or 'red' for a metric where lower is better."""
    if value <= green_max:
        return "green"
    if value >= red_min:
        return "red"
    return "yellow"


def _evaluate_field_higher_is_better(
    value: float,
    green_min: float,
    red_max: float,
) -> str:
    """Return 'green', 'yellow', or 'red' for a metric where higher is better."""
    if value >= green_min:
        return "green"
    if value <= red_max:
        return "red"
    return "yellow"


def evaluate(metric: SloMetric, thresholds: SloThresholds) -> dict[str, str]:
    """Evaluate all metric fields and return per-field status strings.

    Returns:
        dict mapping metric field name to one of: 'green', 'yellow', 'red'.
    """
    return {
        "file_loc": _evaluate_field_lower_is_better(
            metric.file_loc,
            thresholds.file_loc_green_max,
            thresholds.file_loc_red_min,
        ),
        "function_loc_p95": _evaluate_field_lower_is_better(
            metric.function_loc_p95,
            thresholds.function_loc_p95_green_max,
            thresholds.function_loc_p95_red_min,
        ),
        "impl_importers": _evaluate_field_lower_is_better(
            metric.impl_importers,
            thresholds.impl_importers_green_max,
            thresholds.impl_importers_red_min,
        ),
        "cross_boundary_import_edges": _evaluate_field_lower_is_better(
            metric.cross_boundary_import_edges,
            thresholds.cross_boundary_import_edges_green_max,
            thresholds.cross_boundary_import_edges_red_min,
        ),
        "cli_help_p95_ms": _evaluate_field_lower_is_better(
            metric.cli_help_p95_ms,
            thresholds.cli_help_p95_ms_green_max,
            thresholds.cli_help_p95_ms_red_min,
        ),
        "run_command_p95_ms": _evaluate_field_lower_is_better(
            metric.run_command_p95_ms,
            thresholds.run_command_p95_ms_green_max,
            thresholds.run_command_p95_ms_red_min,
        ),
        "decomposition_checkpoint_pass_rate": _evaluate_field_higher_is_better(
            metric.decomposition_checkpoint_pass_rate,
            thresholds.decomposition_checkpoint_pass_rate_green_min,
            thresholds.decomposition_checkpoint_pass_rate_red_max,
        ),
    }


class SloEmitter:
    """Emit SloMetric records to a JSONL file and evaluate them against thresholds.

    Fail-fast: raises IOError if the write fails. No silent errors.
    """

    def __init__(self, output_path: Path = _DEFAULT_OUTPUT_PATH) -> None:
        self._output_path = output_path

    def emit(self, metric: SloMetric) -> None:
        """Append metric as a JSONL line to the output file.

        Raises:
            IOError: if the output file cannot be written.
        """
        self._output_path.parent.mkdir(parents=True, exist_ok=True)
        record: dict[str, Any] = asdict(metric)
        line = json.dumps(record, sort_keys=True) + "\n"
        with self._output_path.open("a", encoding="utf-8") as fh:
            fh.write(line)

    def evaluate(self, metric: SloMetric, thresholds: SloThresholds) -> dict[str, str]:
        """Evaluate metric against thresholds and return per-field status."""
        return evaluate(metric, thresholds)

    @property
    def output_path(self) -> Path:
        """Return the configured JSONL output path."""
        return self._output_path
