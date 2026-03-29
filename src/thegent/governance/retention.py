"""WP-3006: Compliance evidence retention."""

import logging
import time

from thegent.config import ThegentSettings

_log = logging.getLogger(__name__)


class EvidenceRetentionManager:
    """Manages retention and archival of compliance evidence."""

    def __init__(self, settings: ThegentSettings) -> None:
        self.settings = settings
        self.evidence_dir = settings.session_dir / "evidence"
        self.archive_dir = settings.session_dir / "archive"
        self.retention_days = 30  # Default retention

    def enforce_retention(self) -> dict[str, int]:
        """
        Scan evidence and archive or delete based on policy.
        Returns counts of processed items.
        """
        results = {"archived": 0, "deleted": 0}

        if not self.evidence_dir.exists():
            return results

        now = time.time()
        retention_secs = self.retention_days * 86400

        # Scan for old evidence files
        for f in self.evidence_dir.glob("*"):
            if not f.is_file():
                continue

            mtime = f.stat().st_mtime
            age = now - mtime

            if age > retention_secs:
                # Archive instead of delete for compliance
                self.archive_dir.mkdir(parents=True, exist_ok=True)
                dest = self.archive_dir / f.name

                try:
                    f.replace(dest)
                    results["archived"] += 1
                    _log.info("Archived expired evidence: %s", f.name)
                except Exception as e:
                    _log.error("Failed to archive %s: %s", f.name, e)

        return results

    def list_archived(self) -> list[str]:
        """Return list of archived evidence files."""
        if not self.archive_dir.exists():
            return []
        return [f.name for f in self.archive_dir.glob("*")]
