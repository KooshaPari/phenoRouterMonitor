"""Compliance evidence snapshot scheduler.

Takes periodic snapshots of compliance artifacts and stores them with timestamps
for audit trail and historical analysis.

FR traceability: WL-302 (Compliance Evidence Snapshot Scheduler)
"""

from __future__ import annotations

import logging
from datetime import datetime, UTC
from pathlib import Path

import orjson

logger = logging.getLogger(__name__)


class ComplianceSnapshotScheduler:
    """Schedules periodic snapshots of compliance evidence.

    Attributes:
        schedule_interval_hours: Hours between scheduled snapshots (default: 24).
        snapshot_dir: Directory to store snapshot files.
    """

    def __init__(
        self,
        snapshot_dir: Path | str,
        schedule_interval_hours: int = 24,
    ) -> None:
        """Initialize the scheduler.

        Args:
            snapshot_dir: Directory for storing snapshots.
            schedule_interval_hours: Hours between snapshots (default: 24).

        Raises:
            ValueError: If schedule_interval_hours is not positive.
        """
        if schedule_interval_hours <= 0:
            raise ValueError("schedule_interval_hours must be positive")

        self.snapshot_dir = Path(snapshot_dir)
        self.schedule_interval_hours = schedule_interval_hours
        self.snapshot_dir.mkdir(parents=True, exist_ok=True)

    def take_snapshot(self, artifacts: list[dict]) -> Path:
        """Take a snapshot of compliance artifacts.

        Writes a JSON snapshot file with timestamp to snapshot_dir.

        Args:
            artifacts: List of artifact dicts to snapshot.

        Returns:
            Path to the written snapshot file.

        Raises:
            ValueError: If artifacts is not a list or contains non-dict items.
        """
        if not isinstance(artifacts, list):
            raise ValueError("artifacts must be a list")

        for artifact in artifacts:
            if not isinstance(artifact, dict):
                raise ValueError("All artifacts must be dicts")

        now = datetime.now(UTC)
        timestamp = now.isoformat().replace("+00:00", "Z").replace(":", "-").replace(".", "-")
        filename = f"snapshot_{timestamp}.json"
        filepath = self.snapshot_dir / filename

        snapshot_data = {
            "timestamp": now.isoformat(),
            "artifacts": artifacts,
            "artifact_count": len(artifacts),
        }

        filepath.write_bytes(orjson.dumps(snapshot_data, option=orjson.OPT_INDENT_2))

        logger.info(f"Took snapshot with {len(artifacts)} artifacts: {filepath}")
        return filepath

    def should_run(self, last_run: datetime | None) -> bool:
        """Check if a snapshot should be taken.

        Returns True if last_run is None or more than schedule_interval_hours
        have elapsed since last_run.

        Args:
            last_run: Timestamp of last snapshot, or None if never run.

        Returns:
            True if a snapshot should be taken.
        """
        if last_run is None:
            return True

        now = datetime.now(UTC)
        elapsed_hours = (now - last_run).total_seconds() / 3600
        return elapsed_hours >= self.schedule_interval_hours

    def list_snapshots(self) -> list[Path]:
        """List all snapshot files in order.

        Returns:
            Sorted list of snapshot file paths (oldest first).
        """
        if not self.snapshot_dir.exists():
            return []

        # Find files that match the snapshot timestamp pattern
        # Pattern: snapshot_YYYY-MM-DDTHH-MM-SS-xxxxxx+HH-MM.json (with Z replacing +00:00)
        snapshots = []
        for file in self.snapshot_dir.glob("snapshot_*.json"):
            # Validate that this is a real snapshot by checking timestamp format
            # Real snapshots have ISO-like timestamps with dashes replacing colons
            filename = file.name
            # snapshot_YYYY-MM-DD format with hyphens is our pattern
            if filename.startswith("snapshot_") and len(filename) > len("snapshot_.json"):
                # Extract the timestamp part (between snapshot_ and .json)
                ts_part = filename[len("snapshot_") : -len(".json")]
                # Valid timestamps should have at least 10 chars for YYYY-MM-DD part
                if len(ts_part) >= 10 and ts_part[:4].isdigit() and ts_part[5:7].isdigit():
                    snapshots.append(file)

        return sorted(snapshots)
