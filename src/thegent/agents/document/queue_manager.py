"""
Queue Manager for Document Processing

Manages the processing queue, tracks progress, and provides queue operations
for iterating through documents by month and location.
"""

import json
from dataclasses import dataclass, field
from pathlib import Path


@dataclass
class QueueState:
    """State tracking for queue processing."""

    last_processed_month: str | None = None
    last_processed_location: str | None = None
    processed_files: set[str] = field(default_factory=set)
    skipped_files: set[str] = field(default_factory=set)
    failed_files: set[str] = field(default_factory=set)

    def mark_processed(self, filepath: str):
        """Mark a file as processed."""
        self.processed_files.add(filepath)
        if filepath in self.skipped_files:
            self.skipped_files.remove(filepath)
        if filepath in self.failed_files:
            self.failed_files.remove(filepath)

    def mark_skipped(self, filepath: str):
        """Mark a file as skipped."""
        self.skipped_files.add(filepath)
        if filepath in self.processed_files:
            self.processed_files.remove(filepath)

    def mark_failed(self, filepath: str):
        """Mark a file as failed."""
        self.failed_files.add(filepath)
        if filepath in self.processed_files:
            self.processed_files.remove(filepath)

    def to_dict(self) -> dict:
        """Convert to dictionary for serialization."""
        return {
            "last_processed_month": self.last_processed_month,
            "last_processed_location": self.last_processed_location,
            "processed_files": list(self.processed_files),
            "skipped_files": list(self.skipped_files),
            "failed_files": list(self.failed_files),
        }

    @classmethod
    def from_dict(cls, data: dict) -> "QueueState":
        """Create from dictionary."""
        return cls(
            last_processed_month=data.get("last_processed_month"),
            last_processed_location=data.get("last_processed_location"),
            processed_files=set(data.get("processed_files", [])),
            skipped_files=set(data.get("skipped_files", [])),
            failed_files=set(data.get("failed_files", [])),
        )


class QueueManager:
    """Manages document processing queue."""

    def __init__(self, queue_file: Path, state_file: Path | None = None) -> None:
        self.queue_file = Path(queue_file)
        if state_file is None:
            state_file = self.queue_file.parent / ".queue_state.json"
        self.state_file = Path(state_file)
        self.queue_data: dict | None = None
        self.state = QueueState()
        self._load_state()

    def _load_state(self):
        """Load processing state from file."""
        if self.state_file.exists():
            try:
                with open(self.state_file) as f:
                    data = json.load(f)
                    self.state = QueueState.from_dict(data)
            except (json.JSONDecodeError, KeyError):
                self.state = QueueState()
        else:
            self.state = QueueState()

    def _save_state(self):
        """Save processing state to file."""
        self.state_file.parent.mkdir(parents=True, exist_ok=True)
        with open(self.state_file, "w") as f:
            json.dump(self.state.to_dict(), f, indent=2)

    def load_queue(self) -> dict | None:
        """Load queue data from file."""
        if self.queue_data is None:
            with open(self.queue_file) as f:
                self.queue_data = json.load(f)
        return self.queue_data

    def get_next_month(self) -> dict | None:
        """Get the next month to process."""
        queue_data = self.load_queue()
        queue = queue_data.get("queue", [])

        if not queue:
            return None

        # If no last processed month, start with first (newest)
        if self.state.last_processed_month is None:
            return queue[0]

        # Find the month after last processed
        found = False
        for month_entry in queue:
            if found:
                return month_entry
            if month_entry["month"] == self.state.last_processed_month:
                found = True

        return None

    def get_month_files(self, month: str, location: str | None = None) -> list[str]:
        """Get all files for a specific month, optionally filtered by location."""
        queue_data = self.load_queue()

        for month_entry in queue_data.get("queue", []):
            if month_entry["month"] == month:
                files = []
                for loc_entry in month_entry["locations"]:
                    if location is None or loc_entry["location"] == location:
                        files.extend(loc_entry["files"])
                return files

        return []

    def list_months(self) -> list[dict]:
        """List all months in the queue."""
        queue_data = self.load_queue()
        return queue_data.get("queue", [])

    def get_summary(self) -> dict:
        """Get queue summary statistics."""
        queue_data = self.load_queue()
        queue = queue_data.get("queue", [])

        total_files = sum(entry["total_files"] for entry in queue)
        processed_count = len(self.state.processed_files)
        skipped_count = len(self.state.skipped_files)
        failed_count = len(self.state.failed_files)

        return {
            "total_files": total_files,
            "total_months": len(queue),
            "processed": processed_count,
            "skipped": skipped_count,
            "failed": failed_count,
            "remaining": total_files - processed_count - skipped_count - failed_count,
            "last_processed_month": self.state.last_processed_month,
            "last_processed_location": self.state.last_processed_location,
        }

    def mark_month_complete(self, month: str, location: str | None = None):
        """Mark a month/location as complete."""
        if location:
            self.state.last_processed_location = location
        else:
            self.state.last_processed_month = month
            self.state.last_processed_location = None
        self._save_state()

    def mark_file_processed(self, filepath: str):
        """Mark a file as processed."""
        self.state.mark_processed(filepath)
        self._save_state()

    def mark_file_skipped(self, filepath: str):
        """Mark a file as skipped."""
        self.state.mark_skipped(filepath)
        self._save_state()

    def mark_file_failed(self, filepath: str):
        """Mark a file as failed."""
        self.state.mark_failed(filepath)
        self._save_state()

    def get_unprocessed_files(self, month: str | None = None, location: str | None = None) -> list[str]:
        """Get list of unprocessed files."""
        if month:
            all_files = self.get_month_files(month, location)
        else:
            queue_data = self.load_queue()
            all_files = []
            for month_entry in queue_data.get("queue", []):
                for loc_entry in month_entry["locations"]:
                    if location is None or loc_entry["location"] == location:
                        all_files.extend(loc_entry["files"])

        return [f for f in all_files if f not in self.state.processed_files and f not in self.state.skipped_files]
