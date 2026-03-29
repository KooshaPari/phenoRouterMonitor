"""
Markdown File Scanner

Scans directories for markdown files, organizing them by modification date
and location. Supports configurable scan parameters and exclusion patterns.
"""

import json
import os
from collections import defaultdict
from dataclasses import dataclass, field
from datetime import UTC, datetime
from pathlib import Path
from typing import Any


@dataclass
class ScanConfig:
    """Configuration for markdown file scanning."""

    locations: dict[str, dict[str, Any]] = field(default_factory=dict)
    """Location configurations: {name: {path, recursive, max_depth, excludes}}"""

    exclude_patterns: set[str] = field(
        default_factory=lambda: {
            "node_modules",
            ".venv",
            ".git",
            "__pycache__",
            ".pytest_cache",
        }
    )
    """Directory patterns to exclude from scanning"""

    min_date: str | None = None
    """Minimum modification date (YYYY-MM format)"""

    output_dir: Path | None = None
    """Directory to save scan results"""

    def __post_init__(self) -> None:
        """Set default output directory if not provided."""
        if self.output_dir is None:
            self.output_dir = Path.home() / ".thegent" / "scans"


class MarkdownScanner:
    """Scans directories for markdown files and organizes by date/location."""

    def __init__(self, config: ScanConfig) -> None:
        self.config = config
        self.scan_results: dict[str, dict[str, list[str]]] = defaultdict(lambda: defaultdict(list))

    def get_file_date(self, filepath: Path) -> str | None:
        """Get file modification date as YYYY-MM."""
        try:
            stat = os.stat(filepath)
            mtime = datetime.fromtimestamp(stat.st_mtime, tz=UTC)
            date_str = mtime.strftime("%Y-%m")

            # Apply min_date filter if configured
            if self.config.min_date and date_str < self.config.min_date:
                return None

            return date_str
        except (OSError, ValueError):
            return None

    def should_exclude(self, filepath: Path) -> bool:
        """Check if filepath should be excluded."""
        path_str = str(filepath)
        return any(pattern in path_str for pattern in self.config.exclude_patterns)

    def scan_directory(self, base_path: str, recursive: bool = True, max_depth: int | None = None) -> list[Path]:
        """Scan directory for .md files."""
        md_files = []
        base = Path(base_path)

        if not base.exists():
            return []

        if recursive:
            for md_file in base.rglob("*.md"):
                if self.should_exclude(md_file):
                    continue

                if max_depth is not None:
                    try:
                        depth = len(md_file.relative_to(base).parts)
                        if depth > max_depth:
                            continue
                    except ValueError:
                        continue

                md_files.append(md_file)
        else:
            for md_file in base.glob("*.md"):
                if not self.should_exclude(md_file):
                    md_files.append(md_file)

        return md_files

    def scan(self) -> dict[str, dict[str, list[str]]]:
        """Perform scan of all configured locations."""
        self.scan_results.clear()

        for name, loc_config in self.config.locations.items():
            path = loc_config.get("path")
            recursive = loc_config.get("recursive", True)
            max_depth = loc_config.get("max_depth")

            if not path:
                continue

            files = self.scan_directory(path, recursive=recursive, max_depth=max_depth)

            for file in files:
                date = self.get_file_date(file)
                if date:
                    self.scan_results[date][name].append(str(file))

        return self.scan_results

    def save_results(self, output_path: Path | None = None) -> Path:
        """Save scan results to JSON file."""
        if output_path is None:
            timestamp = datetime.now(tz=UTC).strftime("%Y%m%d_%H%M%S")
            config_output_dir = self.config.output_dir
            if config_output_dir is None:
                msg = "output_dir is not configured"
                raise ValueError(msg)
            output_path = config_output_dir / f"scan_{timestamp}.json"

        output_path.parent.mkdir(parents=True, exist_ok=True)

        # Build queue structure
        queue_data = {
            "scan_date": datetime.now(tz=UTC).isoformat(),
            "scan_params": {
                name: {
                    "recursive": loc_config.get("recursive", True),
                    "max_depth": loc_config.get("max_depth"),
                }
                for name, loc_config in self.config.locations.items()
            },
            "summary": {},
            "queue": [],
        }

        # Sort months (newest first)
        months = sorted(self.scan_results.keys(), reverse=True)

        for month in months:
            total = sum(len(files) for files in self.scan_results[month].values())
            queue_data["summary"][month] = {
                "total": total,
                "by_location": {loc: len(files) for loc, files in self.scan_results[month].items()},
            }

            # Add to queue
            month_entry = {"month": month, "total_files": total, "locations": []}

            for location in sorted(self.scan_results[month].keys()):
                files = sorted(self.scan_results[month][location])
                month_entry["locations"].append({"location": location, "file_count": len(files), "files": files})

            queue_data["queue"].append(month_entry)

        with open(output_path, "w") as f:
            json.dump(queue_data, f, indent=2)

        return output_path

    def get_summary(self) -> dict[str, Any]:
        """Get summary statistics of scan results."""
        months = sorted(self.scan_results.keys(), reverse=True)
        total_files = sum(len(files) for month_data in self.scan_results.values() for files in month_data.values())

        return {
            "total_files": total_files,
            "months": len(months),
            "by_month": {
                month: {
                    "total": sum(len(files) for files in month_data.values()),
                    "by_location": {loc: len(files) for loc, files in month_data.items()},
                }
                for month, month_data in self.scan_results.items()
            },
        }
