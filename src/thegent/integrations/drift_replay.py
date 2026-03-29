"""Drift Replay Tool for deterministic debugging.

WL-317: Drift Replay Tool
Replays drift scenarios from archived manifests for deterministic debugging.
"""

from __future__ import annotations

import orjson
from dataclasses import dataclass
from datetime import datetime
from pathlib import Path
from typing import cast

from thegent.integrations.base import SerializableMixin


@dataclass
class DriftManifest(SerializableMixin):
    """A manifest of drift events for replay."""

    manifest_id: str
    cycle_id: str
    drifts: list[dict]
    captured_at: datetime


class DriftReplayEngine:
    """Engine for archiving and replaying drift manifests."""

    @staticmethod
    def archive_manifest(manifest: DriftManifest, archive_dir: Path) -> Path:
        """Archive a drift manifest to disk.

        Args:
            manifest: DriftManifest to archive.
            archive_dir: Directory to store archived manifest.

        Returns:
            Path where manifest was written.

        Raises:
            ValueError: If archive_dir does not exist.
        """
        if not archive_dir.exists():
            raise ValueError(f"Archive directory does not exist: {archive_dir}")

        output_path = archive_dir / f"{manifest.manifest_id}.json"
        output_path.write_bytes(orjson.dumps(manifest.to_dict(), option=orjson.OPT_INDENT_2))

        return output_path

    @staticmethod
    def load_manifest(manifest_id: str, archive_dir: Path) -> DriftManifest:
        """Load a drift manifest from archive.

        Args:
            manifest_id: ID of manifest to load.
            archive_dir: Directory containing archived manifests.

        Returns:
            Loaded DriftManifest instance.

        Raises:
            FileNotFoundError: If manifest file not found.
        """
        manifest_path = archive_dir / f"{manifest_id}.json"
        if not manifest_path.exists():
            raise FileNotFoundError(f"Manifest not found: {manifest_path}")

        data = orjson.loads(manifest_path.read_bytes())

        return cast("DriftManifest", DriftManifest.from_dict(data))

    @staticmethod
    def list_manifests(archive_dir: Path) -> list[str]:
        """List all manifest IDs in archive directory.

        Args:
            archive_dir: Directory containing archived manifests.

        Returns:
            Sorted list of manifest IDs (without .json extension).
        """
        if not archive_dir.exists():
            return []

        manifest_files = archive_dir.glob("*.json")
        manifest_ids = [f.stem for f in manifest_files]
        return sorted(manifest_ids)

    @staticmethod
    def replay(manifest: DriftManifest) -> list[dict]:
        """Replay drifts from a manifest.

        Deterministically replays drift events from the manifest.

        Args:
            manifest: DriftManifest to replay.

        Returns:
            List of drift events from the manifest.
        """
        return manifest.drifts
