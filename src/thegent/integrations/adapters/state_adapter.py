"""State management adapter for workstream autosync.

Handles local state persistence, checkpoints, and trends.
"""

import orjson as json
from datetime import datetime, UTC
from pathlib import Path
from typing import Any


class StateAdapter:
    """Adapter for state management operations."""

    def __init__(self, config: Any):
        self.config = config
        self._status_path = config.status_file_path or Path("docs/reference/autosync_status.json")
        self._trend_path = config.trend_path or Path("docs/reference/workstream_autosync_trend.jsonl")
        self._cycle_metrics_path = config.cycle_metrics_path or Path(
            "docs/reference/workstream_autosync_cycle_metrics.jsonl"
        )
        self._change_digest_path = config.change_digest_path or Path(
            "artifacts/workstream_autosync_change_digest.jsonl"
        )

    # Path methods
    def get_status_path(self) -> Path:
        return self._status_path

    def get_trend_path(self) -> Path:
        return self._trend_path

    def get_cycle_metrics_path(self) -> Path:
        return self._cycle_metrics_path

    def get_change_digest_path(self) -> Path:
        return self._change_digest_path

    def get_autosync_metrics_path(self) -> Path:
        return self.config.autosync_prometheus_export_path or Path("docs/reference/workstream_autosync_metrics.prom")

    def get_cycle_manifest_path(self) -> Path:
        return self._status_path.parent / "autosync_cycle_manifest.jsonl"

    def get_failure_queue_path(self) -> Path:
        return self._status_path.parent / "autosync_failure_queue.json"

    def get_checkpoint_path(self, checkpoint_id: str) -> Path:
        """Get path for checkpoint file."""
        return self._status_path.parent / f"autosync_checkpoint_{checkpoint_id}.json"

    def get_snapshot_path(self) -> Path:
        """Get path for snapshot file."""
        timestamp = datetime.now(UTC).strftime("%Y%m%d_%H%M%S")
        return self._status_path.parent / f"autosync_snapshot_{timestamp}.json"

    def get_latest_snapshot_age_seconds(self) -> int | None:
        """Get age of latest snapshot in seconds."""
        snapshot_candidates = sorted(self._status_path.parent.glob("autosync_snapshot_*.json"))
        if not snapshot_candidates:
            return None
        try:
            latest = max(snapshot_candidates, key=lambda p: p.stat().st_mtime)
            age = datetime.now(UTC) - datetime.fromtimestamp(latest.stat().st_mtime, tz=UTC)
            return max(0, int(age.total_seconds()))
        except OSError:
            return None

    def append_trend_sample(self, sample: dict[str, Any]) -> None:
        """Append trend sample to file."""
        try:
            self._trend_path.parent.mkdir(parents=True, exist_ok=True)
            with open(self._trend_path, "a") as f:
                f.write(json.dumps(sample).decode() + "\n")
        except Exception:
            pass

    def read_last_manifest_hash(self) -> str:
        """Read last manifest hash from status."""
        try:
            if self._status_path.exists():
                data = json.loads(self._status_path.read_text())
                return data.get("last_manifest_hash", "")
        except Exception:
            pass
        return ""

    def write_status(self, status: dict[str, Any]) -> None:
        """Write status to file."""
        try:
            self._status_path.parent.mkdir(parents=True, exist_ok=True)
            self._status_path.write_text(json.dumps(status, option=json.OPT_INDENT_2).decode())
        except Exception:
            pass

    def compact_snapshots(self, keep_count: int = 10) -> None:
        """Compact old snapshots, keeping only the most recent."""
        try:
            snapshots = sorted(
                self._status_path.parent.glob("autosync_snapshot_*.json"), key=lambda p: p.stat().st_mtime, reverse=True
            )
            for old in snapshots[keep_count:]:
                old.unlink()
        except Exception:
            pass


__all__ = ["StateAdapter"]


# Register with unified adapter registry
from thegent.adapters.ports import AdapterRegistry


class StateAdapterWrapper:
    """State adapter wrapper for registry."""

    def __init__(self, config: Any | None = None):
        self._adapter = StateAdapter(config) if config is not None else None

    def call(self, **kwargs) -> dict[str, str]:
        return {"status": "state_adapter_ready"}


AdapterRegistry.register("state", StateAdapterWrapper())
