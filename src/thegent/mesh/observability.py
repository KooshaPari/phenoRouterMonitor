"""Observability and metrics for the agent mesh."""

import json
import math
import time
from pathlib import Path
from typing import TypedDict
from rich.console import Console


class MetricBucket(TypedDict):
    count: int
    sum_value: float
    avg_value: float


class AgentSummary(TypedDict):
    count: int
    sum_value: float
    avg_value: float
    by_metric: dict[str, MetricBucket]


class TotalsSummary(TypedDict):
    count: int
    sum_value: float
    avg_value: float
    by_metric: dict[str, MetricBucket]


class MetricsSummary(TypedDict):
    agents: dict[str, AgentSummary]
    totals: TotalsSummary


class MeshLogger:
    """JSONL structured logging (SCLI-P13.1)."""

    def __init__(self, mesh_root: Path) -> None:
        self.log_file = mesh_root / "mesh.jsonl"

    def log(self, agent_id: str, event: str, data: dict | None = None):
        """Append a structured log entry."""
        entry = {"timestamp": time.time(), "agent_id": agent_id, "event": event, "data": data or {}}
        # PIPE_BUF aware atomic append (SCLI-P13.1)
        line = json.dumps(entry) + "\n"
        if len(line) <= 4096:  # PIPE_BUF limit for atomic write
            with open(self.log_file, "a") as f:
                f.write(line)
        else:
            # Fallback for large lines
            with open(self.log_file, "a") as f:
                f.write(line)


class MetricsAggregator:
    """Mesh metrics aggregation (SCLI-P13.2)."""

    def __init__(self, mesh_root: Path) -> None:
        self.metrics_dir = mesh_root / "metrics"
        self.metrics_dir.mkdir(parents=True, exist_ok=True, mode=0o1777)

    def record_metric(self, agent_id: str, name: str, value: float):
        """Record a single metric point."""
        if not agent_id:
            raise ValueError("agent_id must be non-empty")
        if not name:
            raise ValueError("name must be non-empty")
        if not isinstance(value, int | float):
            raise TypeError("value must be numeric")
        value_f = float(value)
        if not math.isfinite(value_f):
            raise ValueError("value must be finite")
        metric_file = self.metrics_dir / f"{agent_id}.jsonl"
        entry = {"timestamp": time.time(), "name": name, "value": value_f}
        with open(metric_file, "a") as f:
            f.write(json.dumps(entry) + "\n")

    def get_summary(self) -> MetricsSummary:
        """Aggregate metrics for all agents (SCLI-P13.2)."""
        summary: MetricsSummary = {
            "agents": {},
            "totals": {
                "count": 0,
                "sum_value": 0.0,
                "avg_value": 0.0,
                "by_metric": {},
            },
        }

        for metric_file in self.metrics_dir.glob("*.jsonl"):
            agent_id = metric_file.stem
            with open(metric_file) as f:
                points = [json.loads(line) for line in f if line.strip()]

            agent_count = len(points)
            agent_sum = sum(float(p["value"]) for p in points)
            by_metric: dict[str, MetricBucket] = {}
            for point in points:
                metric_name = str(point["name"])
                metric_bucket = by_metric.setdefault(metric_name, {"count": 0, "sum_value": 0.0, "avg_value": 0.0})
                metric_bucket["count"] += 1
                metric_bucket["sum_value"] += float(point["value"])

            for metric_bucket in by_metric.values():
                count = int(metric_bucket["count"])
                metric_bucket["avg_value"] = float(metric_bucket["sum_value"]) / count if count else 0.0

            summary["agents"][agent_id] = {
                "count": agent_count,
                "sum_value": agent_sum,
                "avg_value": agent_sum / agent_count if agent_count else 0.0,
                "by_metric": by_metric,
            }

            totals = summary["totals"]
            totals["count"] += agent_count
            totals["sum_value"] += agent_sum
            total_by_metric = totals["by_metric"]
            for metric_name, metric_bucket in by_metric.items():
                total_metric = total_by_metric.setdefault(metric_name, {"count": 0, "sum_value": 0.0, "avg_value": 0.0})
                total_metric["count"] += metric_bucket["count"]
                total_metric["sum_value"] += metric_bucket["sum_value"]

        totals = summary["totals"]
        totals["avg_value"] = totals["sum_value"] / totals["count"] if totals["count"] else 0.0
        for metric_bucket in totals["by_metric"].values():
            count = int(metric_bucket["count"])
            metric_bucket["avg_value"] = float(metric_bucket["sum_value"]) / count if count else 0.0
        return summary


def mesh_status_cmd(mesh_root: Path) -> None:
    """CLI 'mesh status' (SCLI-P13.3)."""
    import yaml

    console = Console(width=1000, soft_wrap=False)
    agents = list((mesh_root / "agents").glob("*.yaml"))
    if not agents:
        console.print("No registered agents.")
        return

    console.print(f"Mesh root: {mesh_root}")
    console.print(f"Registered agents: {len(agents)}")
    for manifest in sorted(agents):
        with open(manifest) as f:
            data = yaml.safe_load(f) or {}
        pid = data.get("pid", "unknown")
        agent_type = data.get("type") or data.get("name", "unknown")
        source = data.get("source", "unknown")
        console.print(f"- {manifest.stem}: pid={pid} type={agent_type} source={source}")
