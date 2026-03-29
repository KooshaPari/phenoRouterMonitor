"""SLO trend serialization for WL-135 B90-W2-F4.

Provides SloTrend, load_trend, and serialize_trend for reading and
serializing windowed SLO metric history from the .quality/slo-metrics.jsonl file.

Fail-fast: all functions raise loudly if the file is missing or malformed.
No fallbacks, no silent errors, no legacy compatibility shims.

# @trace WL-135 B90-W2-F4
"""

from __future__ import annotations

import json
from dataclasses import asdict, dataclass
from datetime import UTC, datetime, timedelta
from pathlib import Path
from typing import Any

from thegent.governance.slo_metrics import SloMetric

_DEFAULT_JSONL_PATH = Path(".quality") / "slo-metrics.jsonl"


@dataclass
class SloTrend:
    """Windowed collection of SLO metric snapshots.

    Attributes:
        metrics: Ordered list of SloMetric records within the window.
        window_days: Number of days the window covers.
        generated_at: ISO-8601 UTC timestamp of when this trend was built.
    """

    metrics: list[SloMetric]
    window_days: int
    generated_at: str


def _parse_jsonl_line(line: str, line_number: int) -> SloMetric:
    """Parse one JSONL line into an SloMetric.

    Raises:
        ValueError: if the line is not valid JSON or missing required fields.
    """
    stripped = line.strip()
    if not stripped:
        raise ValueError(f"Line {line_number}: empty line in JSONL file")
    try:
        record: dict[str, Any] = json.loads(stripped)
    except json.JSONDecodeError as exc:
        raise ValueError(f"Line {line_number}: invalid JSON — {exc}") from exc

    required = {
        "file_loc",
        "function_loc_p95",
        "impl_importers",
        "cross_boundary_import_edges",
        "cli_help_p95_ms",
        "run_command_p95_ms",
        "decomposition_checkpoint_pass_rate",
        "timestamp",
        "source",
    }
    missing = required - record.keys()
    if missing:
        raise ValueError(f"Line {line_number}: missing fields {sorted(missing)}")

    return SloMetric(
        file_loc=float(record["file_loc"]),
        function_loc_p95=float(record["function_loc_p95"]),
        impl_importers=float(record["impl_importers"]),
        cross_boundary_import_edges=float(record["cross_boundary_import_edges"]),
        cli_help_p95_ms=float(record["cli_help_p95_ms"]),
        run_command_p95_ms=float(record["run_command_p95_ms"]),
        decomposition_checkpoint_pass_rate=float(record["decomposition_checkpoint_pass_rate"]),
        timestamp=str(record["timestamp"]),
        source=str(record["source"]),
    )


def load_trend(path: str | Path = _DEFAULT_JSONL_PATH, window_days: int = 7) -> SloTrend:
    """Read JSONL from path, filter to last window_days days, return SloTrend.

    Raises:
        FileNotFoundError: if the JSONL file does not exist.
        ValueError: if any line is malformed or missing required fields.
    """
    jsonl_path = Path(path)
    if not jsonl_path.exists():
        raise FileNotFoundError(
            f"SLO metrics JSONL file not found: {jsonl_path}. Run the SLO emitter first to populate it."
        )

    cutoff = datetime.now(UTC) - timedelta(days=window_days)
    metrics: list[SloMetric] = []

    text = jsonl_path.read_text(encoding="utf-8")
    for line_number, line in enumerate(text.splitlines(), start=1):
        if not line.strip():
            continue
        metric = _parse_jsonl_line(line, line_number)
        # Filter by window
        try:
            ts = datetime.fromisoformat(metric.timestamp)
        except ValueError as exc:
            raise ValueError(f"Line {line_number}: cannot parse timestamp '{metric.timestamp}' — {exc}") from exc
        if ts.tzinfo is None:
            ts = ts.replace(tzinfo=UTC)
        if ts >= cutoff:
            metrics.append(metric)

    return SloTrend(
        metrics=metrics,
        window_days=window_days,
        generated_at=datetime.now(UTC).isoformat(),
    )


def serialize_trend(trend: SloTrend) -> str:
    """Return a JSON string representation of the SloTrend.

    The output is a single JSON object with:
      - window_days: int
      - generated_at: str
      - metrics: list of metric dicts

    Raises:
        TypeError: if trend contains un-serializable fields (fail-fast).
    """
    payload = {
        "window_days": trend.window_days,
        "generated_at": trend.generated_at,
        "metrics": [asdict(m) for m in trend.metrics],
    }
    return json.dumps(payload, sort_keys=True, indent=2)
