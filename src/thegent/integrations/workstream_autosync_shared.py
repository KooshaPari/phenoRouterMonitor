"""Shared models, parser, and config loading for workstream autosync."""

import logging
import os
import re
import hashlib
from dataclasses import dataclass, field
from datetime import datetime, timedelta, UTC
from enum import Enum
from pathlib import Path
from typing import Any

import orjson as json

from thegent.config_defaults import autosync_phase1_enabled
from thegent.integrations.capability_alerts import ConnectorSLAThresholds
from thegent.integrations.base import SerializableMixin

OPEN_STATUSES: set[str] = {"BACKLOG", "IN PROGRESS", "REVIEW", "TODO", "OPEN"}
WL_ID_PATTERN = re.compile(r"^WL-\d+$")

logger = logging.getLogger(__name__)

# ---------------------------------------------------------------------------
# Utility Models
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class MaintenanceWindow:
    """Parsed maintenance window configuration for a connector."""

    connector: str
    start_utc: datetime
    end_utc: datetime
    reason: str = ""
    project: str = "default"

    def is_active(self, now: datetime) -> bool:
        """Return whether the window is active at the given timestamp."""
        now_utc = now.astimezone(UTC)
        return self.start_utc <= now_utc <= self.end_utc


@dataclass(frozen=True)
class WorkstreamPartition:
    """Partition for large sync ranges."""

    start: int
    end: int

    @property
    def count(self) -> int:
        """Return number of items in the partition."""
        return max(0, self.end - self.start)


@dataclass(frozen=True)
class SyncCheckpoint(SerializableMixin):
    """Minimal checkpoint used for rolling resume."""

    connector: str
    direction: str
    start_index: int
    total_partitions: int
    partition_size: int
    created_at: datetime

    @classmethod
    def from_dict(cls, payload: dict[str, Any]) -> "SyncCheckpoint":
        """Construct a checkpoint from serialized state."""
        created_at = payload["created_at"]
        if isinstance(created_at, str):
            created_at_dt = datetime.fromisoformat(created_at.replace("Z", "+00:00"))
        else:
            created_at_dt = created_at
        return cls(
            connector=payload["connector"],
            direction=payload["direction"],
            start_index=int(payload["start_index"]),
            total_partitions=int(payload["total_partitions"]),
            partition_size=int(payload["partition_size"]),
            created_at=created_at_dt,
        )


@dataclass(frozen=True)
class FailureRecord:
    """Failure entry for queue pruning and retry heuristics."""

    operation_id: str
    connector: str
    item_id: str
    message: str
    occurred_at: datetime
    retry_class: str = "permanent"
    correlation_id: str | None = None

    def to_dict(self) -> dict[str, Any]:
        """Serialize failure record."""
        return {
            "operation_id": self.operation_id,
            "connector": self.connector,
            "item_id": self.item_id,
            "message": self.message,
            "occurred_at": self.occurred_at.isoformat(),
            "retry_class": self.retry_class,
            "correlation_id": self.correlation_id,
        }


@dataclass(frozen=True)
class ConnectorHealthProbeResult:
    """Connector health probe state for a cycle."""

    connector: str
    status: str
    reason: str | None = None

    def to_dict(self) -> dict[str, Any]:
        return {
            "connector": self.connector,
            "status": self.status,
            "reason": self.reason,
        }


def compute_adaptive_sync_interval(
    *,
    base_interval_seconds: int,
    min_interval_seconds: int,
    max_interval_seconds: int,
    drift_rate: float,
    error_rate: float,
    load_factor: float,
) -> int:
    """Compute next-cycle interval from drift/error/load metrics.

    Higher drift or error rates shrink the interval.
    Higher load factor expands the interval to reduce pressure.
    """
    floor = max(1, min_interval_seconds)
    ceiling = max(floor, max_interval_seconds)
    base = max(floor, min(base_interval_seconds, ceiling))
    pressure = (0.6 * drift_rate) + (0.4 * error_rate) - (0.35 * load_factor)
    multiplier = 1.0 - pressure
    candidate = round(base * multiplier)
    return max(floor, min(ceiling, candidate))


# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------


class SyncDirection(str, Enum):
    """Sync direction (read-only, write-only, bidirectional)."""

    READ_ONLY = "read_only"
    WRITE_ONLY = "write_only"
    BIDIRECTIONAL = "bidirectional"


class RemoteMissingItemPolicy(str, Enum):
    """Policy for local WL items missing from remote connector snapshots."""

    IGNORE = "ignore"
    ARCHIVE = "archive"
    DELETE = "delete"


@dataclass
class WorkstreamAutosyncConfig:
    """Configuration for workstream autosync."""

    enabled: bool = False
    cycle_interval_seconds: int = 300  # 5 minutes default
    github_enabled: bool = False
    github_owner: str = ""
    github_project_number: int = 0
    github_direction: SyncDirection = SyncDirection.BIDIRECTIONAL
    linear_enabled: bool = False
    linear_api_key: str = ""
    linear_team_key: str = ""
    linear_direction: SyncDirection = SyncDirection.BIDIRECTIONAL
    github_sandbox_mode: bool = False
    github_sandbox_project_number: int = 0
    work_stream_path: Path | None = None
    status_file_path: Path | None = None
    checkpoint_file_path: Path | None = None
    cycle_manifest_path: Path | None = None
    dry_run: bool = False
    shadow_mode: bool = False
    max_partition_size: int = 200
    maintenance_windows: list[MaintenanceWindow] = field(default_factory=list)
    checkpoint_ttl_seconds: int = 3600
    migration_phase: str = "phase1-detect"
    repo_previously_opted_in: bool = False
    conflict_ttl_seconds: int = 1800
    allowed_tags: list[str] = field(default_factory=list)
    failure_queue_path: Path | None = None
    failure_queue_retention_seconds: int = 60 * 60 * 24
    connector_mapping_cache_path: Path | None = None
    simulation_mode: bool = False
    snapshot_retention_count: int = 20
    trend_file_path: Path | None = None
    artifact_encryption_enabled: bool = False
    artifact_encryption_key: str = ""
    strict_tag_validation: bool = False
    strict_title_validation: bool = False
    adaptive_interval_enabled: bool = False
    adaptive_interval_min_seconds: int = 30
    adaptive_interval_max_seconds: int = 900
    connector_health_states: dict[str, str] = field(default_factory=dict)
    incident_bundle_path: Path | None = None
    metadata_ttl_seconds: int = 3600
    metadata_last_refreshed_at: datetime | None = None
    bootstrap_connector: str = "github"
    bootstrap_required_fields: list[str] = field(default_factory=list)
    bootstrap_mapping_cache_path: Path | None = None
    project_id: str = "default"
    require_actor_identity: bool = False
    actor_id: str = ""
    actor_signature: str = ""
    actor_signing_key: str = ""
    connector_capabilities: dict[str, list[str]] = field(default_factory=dict)
    required_connector_capabilities: dict[str, list[str]] = field(default_factory=dict)
    connector_sla_thresholds: dict[str, ConnectorSLAThresholds] = field(default_factory=dict)
    payload_checksum_enforced: bool = False
    expected_payload_checksum: str = ""
    emergency_stop_enabled: bool = True
    emergency_stop_file_path: Path | None = None
    emergency_stop_env_var: str = "THGENT_AUTOSYNC_EMERGENCY_STOP"
    wl_ignore_list: list[str] = field(default_factory=list)
    scope_areas: list[str] = field(default_factory=list)
    scope_statuses: list[str] = field(default_factory=list)
    scope_priorities: list[str] = field(default_factory=list)
    scope_wl_ranges: list[str] = field(default_factory=list)
    remote_missing_item_policy: RemoteMissingItemPolicy = RemoteMissingItemPolicy.IGNORE
    github_write_timeout_seconds: float = 30.0
    github_read_timeout_seconds: float = 30.0
    linear_write_timeout_seconds: float = 30.0
    linear_read_timeout_seconds: float = 30.0
    github_auto_close_issues: bool = False
    github_auto_close_comment: str | None = "Closed automatically from autosync."
    error_budget_max_consecutive_failures: int = 3
    error_budget_max_failure_rate: float = 0.5
    error_budget_escalation_after: int = 5
    autosync_stale_snapshot_seconds: int = 3600
    connector_circuit_breaker_failure_threshold: int = 3
    connector_circuit_breaker_success_threshold: int = 1
    connector_circuit_breaker_timeout_seconds: float = 60.0
    change_digest_path: Path | None = None
    reflection_event_log_path: Path | None = None
    autosync_prometheus_export_path: Path | None = None
    cycle_metrics_path: Path | None = None
    writer_lock_enabled: bool = True
    writer_lock_path: Path | None = None
    rate_limit_max_retries: int = 2
    rate_limit_initial_wait: float = 1.0
    rate_limit_max_wait: float = 16.0
    rate_limit_multiplier: float = 2.0
    standalone_mode: bool = True  # Always succeed, even if credentials missing

    @property
    def effective_partition_size(self) -> int:
        """Return a bounded partition size for planning."""
        return max(1, self.max_partition_size)

    @property
    def normalized_wl_ignore_list(self) -> list[str]:
        """Return canonical WL IDs to skip during sync cycles."""
        normalized: set[str] = set()
        for raw in self.wl_ignore_list:
            candidate = raw.strip().upper()
            if not candidate:
                continue
            if not WL_ID_PATTERN.fullmatch(candidate):
                raise ValueError(f"Invalid WL ignore ID: {raw}")
            normalized.add(candidate)
        return sorted(normalized)

    @property
    def normalized_scope_areas(self) -> set[str]:
        return {area.strip().lower() for area in self.scope_areas if area.strip()}

    @property
    def normalized_scope_statuses(self) -> set[str]:
        return {status.strip().upper() for status in self.scope_statuses if status.strip()}

    @property
    def normalized_scope_priorities(self) -> set[str]:
        return {priority.strip().upper() for priority in self.scope_priorities if priority.strip()}

    @property
    def normalized_scope_wl_ranges(self) -> list[tuple[int, int]]:
        ranges: list[tuple[int, int]] = []
        for raw in self.scope_wl_ranges:
            token = raw.strip().upper()
            if not token:
                continue
            if ".." not in token:
                raise ValueError(f"Invalid WL range filter: {raw}")
            start_raw, end_raw = token.split("..", 1)
            if not start_raw.startswith("WL-") or not end_raw.startswith("WL-"):
                raise ValueError(f"Invalid WL range filter: {raw}")
            try:
                start_num = int(start_raw[3:])
                end_num = int(end_raw[3:])
            except ValueError as exc:
                raise ValueError(f"Invalid WL range filter: {raw}") from exc
            if end_num < start_num:
                raise ValueError(f"WL range end must be >= start: {raw}")
            ranges.append((start_num, end_num))
        return ranges

    def matches_scope_filters(self, item: "WorkstreamItem") -> bool:
        """Return whether a work item is included by configured sync scope filters."""
        if self.normalized_scope_areas and item.area.strip().lower() not in self.normalized_scope_areas:
            return False
        if self.normalized_scope_statuses and item.status.strip().upper() not in self.normalized_scope_statuses:
            return False
        if self.normalized_scope_priorities and item.priority.strip().upper() not in self.normalized_scope_priorities:
            return False
        wl_ranges = self.normalized_scope_wl_ranges
        if wl_ranges:
            if not item.item_id.upper().startswith("WL-"):
                return False
            try:
                wl_number = int(item.item_id[3:])
            except ValueError:
                return False
            return any(start <= wl_number <= end for start, end in wl_ranges)
        return True

    def is_valid(self) -> bool:
        """Check if config has at least one platform enabled."""
        if not self.enabled:
            return False
        github_valid = bool(self.github_enabled and self.github_owner and self.github_project_number > 0)
        linear_valid = bool(self.linear_enabled and self.linear_api_key and self.linear_team_key)
        return github_valid or linear_valid

    def should_sync_github(self) -> bool:
        """Check if GitHub sync is enabled and configured."""
        return self.enabled and self.github_enabled and bool(self.github_owner) and self.github_project_number > 0

    def should_sync_linear(self) -> bool:
        """Check if Linear sync is enabled and configured."""
        return self.enabled and self.linear_enabled and bool(self.linear_api_key)

    def github_can_read(self) -> bool:
        """Check if GitHub direction allows reading."""
        return self.should_sync_github() and self.github_direction in (
            SyncDirection.READ_ONLY,
            SyncDirection.BIDIRECTIONAL,
        )

    def github_can_write(self) -> bool:
        """Check if GitHub direction allows writing."""
        return self.should_sync_github() and self.github_direction in (
            SyncDirection.WRITE_ONLY,
            SyncDirection.BIDIRECTIONAL,
        )

    def linear_can_read(self) -> bool:
        """Check if Linear direction allows reading."""
        return self.should_sync_linear() and self.linear_direction in (
            SyncDirection.READ_ONLY,
            SyncDirection.BIDIRECTIONAL,
        )

    def linear_can_write(self) -> bool:
        """Check if Linear direction allows writing."""
        return self.should_sync_linear() and self.linear_direction in (
            SyncDirection.WRITE_ONLY,
            SyncDirection.BIDIRECTIONAL,
        )

    def is_maintenance_active(self, connector: str, at: datetime | None = None, project: str | None = None) -> bool:
        """Return whether a connector is currently in planned maintenance."""
        now = at or datetime.now(UTC)
        target = connector.lower()
        project_target = (project or self.project_id).strip().lower()
        for window in self.maintenance_windows:
            if window.connector not in (target, "all", "*"):
                continue
            if window.project.strip().lower() not in {project_target, "all", "*"}:
                continue
            if window.is_active(now):
                return True
        return False

    def is_emergency_stop_active(self) -> bool:
        """Return whether emergency-stop controls are active."""
        if not self.emergency_stop_enabled:
            return False

        env_value = os.getenv(self.emergency_stop_env_var, "").strip().lower()
        if env_value in {"1", "true", "yes", "on"}:
            return True

        stop_path = self.emergency_stop_file_path
        if stop_path and stop_path.exists():
            return True

        return False

    def effective_github_project_number(self) -> int:
        """Return effective GitHub project target (sandbox-aware)."""
        if self.github_sandbox_mode and self.github_sandbox_project_number > 0:
            return self.github_sandbox_project_number
        return self.github_project_number


# ---------------------------------------------------------------------------
# Exceptions
# ---------------------------------------------------------------------------


class WorkstreamAutosyncError(Exception):
    """Base exception for workstream autosync errors."""


class WorkstreamAutosyncAuthError(WorkstreamAutosyncError):
    """Authentication/authorization error."""


class WorkstreamAutosyncConfigError(WorkstreamAutosyncError):
    """Configuration validation error."""


class WorkstreamAutosyncMaintenanceError(WorkstreamAutosyncError):
    """Raised when a maintenance window blocks sync operations."""


class WorkstreamDuplicateTitleError(WorkstreamAutosyncConfigError):
    """Raised for duplicate workstream titles."""


class RetryClass(str, Enum):
    """Error classes driving retry/backoff policy."""

    TRANSIENT = "transient"
    RATE_LIMIT = "rate_limit"
    PERMANENT = "permanent"


# ---------------------------------------------------------------------------
# Work Item Models
# ---------------------------------------------------------------------------


@dataclass
class WorkstreamItem:
    """Parsed work stream item."""

    item_id: str
    title: str
    status: str  # BACKLOG, IN PROGRESS, COMPLETED, CLAIMED
    priority: str  # P0, P1, P2
    area: str
    owner: str | None = None
    blocked_by: str | None = None
    source_line: int = 0
    board_id: str | None = None  # External board ID if known
    tags: list[str] = field(default_factory=list)
    sla_hours: float | None = None
    last_synced: datetime | None = None
    raw_section: str | None = None

    def to_dict(self) -> dict[str, Any]:
        """Convert to dictionary for JSON serialization."""
        return {
            "item_id": self.item_id,
            "title": self.title,
            "status": self.status,
            "priority": self.priority,
            "area": self.area,
            "owner": self.owner,
            "blocked_by": self.blocked_by,
            "source_line": self.source_line,
            "board_id": self.board_id,
            "tags": self.tags,
            "sla_hours": self.sla_hours,
            "last_synced": self.last_synced.isoformat() if self.last_synced else None,
            "raw_section": self.raw_section,
        }


@dataclass
class SyncOperation:
    """Record of a single sync operation."""

    operation_id: str
    platform: str  # github, linear
    direction: str  # read, write
    items_processed: int = 0
    items_successful: int = 0
    items_failed: int = 0
    errors: list[str] = field(default_factory=list)
    correlation_id: str | None = None
    started_at: datetime = field(default_factory=lambda: datetime.now(UTC))
    completed_at: datetime | None = None
    duration_seconds: float | None = None

    def to_dict(self) -> dict[str, Any]:
        """Convert to dictionary for JSON serialization."""
        return {
            "operation_id": self.operation_id,
            "platform": self.platform,
            "direction": self.direction,
            "items_processed": self.items_processed,
            "items_successful": self.items_successful,
            "items_failed": self.items_failed,
            "errors": self.errors,
            "correlation_id": self.correlation_id,
            "started_at": self.started_at.isoformat(),
            "completed_at": self.completed_at.isoformat() if self.completed_at else None,
            "duration_seconds": self.duration_seconds,
        }


@dataclass
class SyncCycleManifest:
    """Append-only cycle manifest for reproducible autosync audit."""

    cycle_number: int
    started_at: str
    status: str
    inputs: dict[str, Any]
    decisions: dict[str, Any]
    outputs: dict[str, Any]
    previous_manifest_hash: str = ""
    manifest_hash: str = ""

    def with_hash(self) -> "SyncCycleManifest":
        """Return a copy with deterministic manifest hash."""
        payload = {
            "cycle_number": self.cycle_number,
            "started_at": self.started_at,
            "status": self.status,
            "inputs": self.inputs,
            "decisions": self.decisions,
            "outputs": self.outputs,
            "previous_manifest_hash": self.previous_manifest_hash,
        }
        digest = hashlib.sha256(json.dumps(payload, option=json.OPT_SORT_KEYS)).hexdigest()
        return SyncCycleManifest(
            cycle_number=self.cycle_number,
            started_at=self.started_at,
            status=self.status,
            inputs=self.inputs,
            decisions=self.decisions,
            outputs=self.outputs,
            previous_manifest_hash=self.previous_manifest_hash,
            manifest_hash=digest,
        )

    def to_dict(self) -> dict[str, Any]:
        return {
            "cycle_number": self.cycle_number,
            "started_at": self.started_at,
            "status": self.status,
            "inputs": self.inputs,
            "decisions": self.decisions,
            "outputs": self.outputs,
            "previous_manifest_hash": self.previous_manifest_hash,
            "manifest_hash": self.manifest_hash,
        }


class SyncFailureQueue:
    """Store and prune recent sync failures."""

    def __init__(self, retention_seconds: int) -> None:
        self.retention_seconds = max(1, retention_seconds)
        self._entries: list[FailureRecord] = []

    def push(
        self,
        operation_id: str,
        connector: str,
        item_id: str,
        message: str,
        *,
        retry_class: RetryClass = RetryClass.PERMANENT,
        correlation_id: str | None = None,
    ) -> None:
        """Record a failed item operation."""
        self._entries.append(
            FailureRecord(
                operation_id=operation_id,
                connector=connector,
                item_id=item_id,
                message=message,
                occurred_at=datetime.now(UTC),
                retry_class=retry_class.value,
                correlation_id=correlation_id,
            )
        )
        self.prune_expired()

    def prune_expired(self, now: datetime | None = None) -> None:
        """Drop failures older than retention window."""
        now_utc = now or datetime.now(UTC)
        cutoff = now_utc - timedelta(seconds=self.retention_seconds)
        self._entries = [entry for entry in self._entries if entry.occurred_at >= cutoff]

    def snapshot(self) -> list[FailureRecord]:
        """Return all active failure records."""
        return list(self._entries)

    def to_dict_list(self) -> list[dict[str, Any]]:
        """Serialize active entries."""
        return [entry.to_dict() for entry in self._entries]

    def replace_records(self, entries: list[FailureRecord]) -> None:
        """Replace queue entries with persisted state."""
        self._entries = list(entries)


# ---------------------------------------------------------------------------
# Workstream Parser
# ---------------------------------------------------------------------------


class WorkstreamParser:
    """Parse WORK_STREAM.md and extract work items."""

    ITEM_PATTERN = re.compile(r"^###\s+\[(WL-\d+)\]\s+(.+?)$", re.MULTILINE)

    STATUS_PATTERN = re.compile(r"\*\*Status:\*\*\s+(.+?)(?:\n|$)")
    PRIORITY_PATTERN = re.compile(r"\*\*Priority:\*\*\s+(\w+)")
    AREA_PATTERN = re.compile(r"\*\*Area:\*\*\s+(.+?)(?:\n|$)")
    OWNER_PATTERN = re.compile(r"\*\*Owner:\*\*\s+(.+?)(?:\n|$)")
    BLOCKED_BY_PATTERN = re.compile(r"\*\*Blocked by:\*\*\s+(.+?)(?:\n|$)")
    TAGS_PATTERN = re.compile(r"\*\*Tags:\*\*\s+(.+?)(?:\n|$)")
    SLA_PATTERN = re.compile(r"\*\*SLA:\*\*\s+(.+?)(?:\n|$)")

    @classmethod
    def _parse_sla_hours(cls, raw: str | None) -> float | None:
        """Parse SLA value into fractional hours."""
        if raw is None:
            return None
        value = raw.strip().lower()
        if not value:
            return None
        if value.endswith("h"):
            return float(value[:-1])
        if value.endswith("m"):
            return float(value[:-1]) / 60
        if value.endswith("d"):
            return float(value[:-1]) * 24
        return float(value)

    @classmethod
    def _parse_tags(cls, raw: str | None) -> list[str]:
        """Parse comma-separated tags and normalize to lower-case."""
        if raw is None:
            return []
        return [tag.strip().lower() for tag in raw.split(",") if tag.strip()]

    @staticmethod
    def _parse_checkpoint_partitions(items_count: int, partition_size: int) -> list[WorkstreamPartition]:
        """Build deterministic range partitions for large item streams."""
        step = max(1, partition_size)
        partitions = []
        start = 0
        while start < items_count:
            end = min(items_count, start + step)
            partitions.append(WorkstreamPartition(start=start, end=end))
            start = end
        return partitions

    @classmethod
    def split_items(cls, items: list[WorkstreamItem], partition_size: int) -> list[list[WorkstreamItem]]:
        """Split items into partitions by configured size."""
        partitions: list[list[WorkstreamItem]] = []
        for chunk in cls._parse_checkpoint_partitions(len(items), partition_size):
            partitions.append(items[chunk.start : chunk.end])
        return partitions

    @classmethod
    def parse_items(cls, work_stream_path: Path) -> list[WorkstreamItem]:
        """Parse WORK_STREAM.md and extract work items.

        Args:
            work_stream_path: Path to WORK_STREAM.md

        Returns:
            List of parsed work items
        """
        items: list[WorkstreamItem] = []

        if not work_stream_path.exists():
            logger.warning("WORK_STREAM.md not found at %s", work_stream_path)
            return items

        try:
            content = work_stream_path.read_text(encoding="utf-8")
        except Exception as e:
            logger.error("Failed to read WORK_STREAM.md: %s", e)
            return items

        # Find all item headers
        for match in cls.ITEM_PATTERN.finditer(content):
            item_id = match.group(1)
            title = match.group(2).strip()
            start_pos = match.start()
            source_line = content[:start_pos].count("\n") + 1

            # Extract metadata after header
            section_start = match.end()
            section_end = content.find("###", section_start)
            if section_end == -1:
                section_end = len(content)

            section = content[section_start:section_end]

            # Extract fields
            status_match = cls.STATUS_PATTERN.search(section)
            status = status_match.group(1).strip() if status_match else "BACKLOG"

            priority_match = cls.PRIORITY_PATTERN.search(section)
            priority = priority_match.group(1) if priority_match else "P2"

            area_match = cls.AREA_PATTERN.search(section)
            area = area_match.group(1).strip() if area_match else "unknown"

            owner_match = cls.OWNER_PATTERN.search(section)
            owner = owner_match.group(1).strip() if owner_match else None

            blocked_by_match = cls.BLOCKED_BY_PATTERN.search(section)
            blocked_by = blocked_by_match.group(1).strip() if blocked_by_match else None

            tags_match = cls.TAGS_PATTERN.search(section)
            tags = cls._parse_tags(tags_match.group(1).strip() if tags_match else None)

            sla_match = cls.SLA_PATTERN.search(section)
            sla_hours = cls._parse_sla_hours(sla_match.group(1).strip() if sla_match else None)

            item = WorkstreamItem(
                item_id=item_id,
                title=title,
                status=status,
                priority=priority,
                area=area,
                owner=owner,
                blocked_by=blocked_by,
                tags=tags,
                sla_hours=sla_hours,
                source_line=source_line,
                raw_section=section.strip() if section else None,
            )

            items.append(item)

        logger.info("Parsed %d work stream items from %s", len(items), work_stream_path)
        return items

    @classmethod
    def duplicate_titles(
        cls,
        items: list[WorkstreamItem],
        *,
        allow_none: bool = True,
    ) -> list[tuple[str, list[WorkstreamItem]]]:
        """Return duplicate titles mapped to item clusters."""
        by_title: dict[str, list[WorkstreamItem]] = {}
        for item in items:
            by_title.setdefault(item.title.strip(), []).append(item)
        duplicates = [(title, group) for title, group in by_title.items() if len(group) > 1]
        if allow_none:
            return duplicates
        return [(title, group) for title, group in duplicates if title]

    @classmethod
    def validate_tags(
        cls,
        items: list[WorkstreamItem],
        *,
        allowed_tags: list[str],
        strict: bool = False,
    ) -> tuple[bool, list[str]]:
        """Validate local tag taxonomy against an allowed set."""
        allowed = {tag.lower().strip() for tag in allowed_tags}
        invalid: set[str] = set()
        for item in items:
            for tag in item.tags:
                if tag.lower() not in allowed:
                    invalid.add(tag)
        is_valid = len(invalid) == 0 or not strict
        return is_valid, sorted(invalid)

    @classmethod
    def open_blocker_digest(cls, items: list[WorkstreamItem]) -> list[str]:
        """Build a digest list for items still blocked by dependencies."""
        digest: list[str] = []
        for item in items:
            if item.status not in OPEN_STATUSES or not item.blocked_by:
                continue
            if item.blocked_by.strip().lower() in {"none", "n/a"}:
                continue
            digest.append(f"{item.item_id}:{item.title} -> {item.blocked_by}")
        return digest

    @classmethod
    def sync_sla_annotations(cls, text: str, *, items: list[WorkstreamItem]) -> str:
        """Ensure each SLA field is reflected in markdown content."""
        if not items:
            return text

        updates: dict[str, str] = {}
        for item in items:
            if item.sla_hours is None:
                continue
            updates[item.item_id] = f"{item.sla_hours}h"

        if not updates:
            return text

        def _rewrite_section(match: re.Match[str]) -> str:
            item_id = match.group(1)
            title = match.group(2)
            body = match.group(3)
            if item_id not in updates:
                return match.group(0)

            replacement = body
            sla_line = f"**SLA:** {updates[item_id]}"
            if cls.SLA_PATTERN.search(body):
                replacement = cls.SLA_PATTERN.sub(sla_line, body)
            elif body.strip():
                replacement = f"{body.rstrip()}\n{sla_line}"
            else:
                replacement = f"{sla_line}\n{body}"
            return f"### [{item_id}] {title}\n{replacement}"

        pattern = re.compile(
            r"###\s+\[(WL-\d+)\]\s+(.+?)\n((?:.|\n)*?)(?=\n###\s+\[WL-|\Z)",
            re.MULTILINE,
        )
        return pattern.sub(_rewrite_section, text)

    @classmethod
    def sync_status_annotations(cls, text: str, *, statuses: dict[str, str]) -> str:
        """Ensure status lines are synchronized from remote updates."""
        if not statuses:
            return text

        def _rewrite_section(match: re.Match[str]) -> str:
            item_id = match.group(1)
            title = match.group(2)
            body = match.group(3)
            remote_status = statuses.get(item_id)
            if not remote_status:
                return match.group(0)

            status_line = f"**Status:** {remote_status}"
            if cls.STATUS_PATTERN.search(body):
                updated_body = cls.STATUS_PATTERN.sub(status_line, body)
            elif body.strip():
                updated_body = f"{status_line}\n{body}"
            else:
                updated_body = f"{status_line}\n"

            return f"### [{item_id}] {title}\n{updated_body}"

        pattern = re.compile(
            r"###\s+\[(WL-\d+)\]\s+(.+?)\n((?:.|\n)*?)(?=\n###\s+\[WL-|\Z)",
            re.MULTILINE,
        )
        return pattern.sub(_rewrite_section, text)


def build_owner_metadata(owner: str) -> dict[str, str]:
    """Build canonical owner fields for local/GitHub/Linear propagation."""
    normalized = owner.strip()
    if not normalized:
        raise ValueError("owner must not be empty")
    return {
        "owner": normalized,
        "github_owner": normalized,
        "linear_assignee": normalized,
    }


def validate_env_profile_drift(
    profiles: dict[str, dict[str, str]],
    required_keys: set[str],
    allowed_drift_keys: set[str] | None = None,
) -> tuple[bool, dict[str, list[str]]]:
    """Validate required autosync keys are consistent across env profiles."""
    allowed = allowed_drift_keys or set()
    drift: dict[str, list[str]] = {}
    env_names = sorted(profiles.keys())
    for env_name in env_names:
        profile = profiles[env_name]
        missing = sorted(key for key in required_keys if key not in profile)
        if missing:
            drift[f"{env_name}:missing"] = missing

    for key in sorted(required_keys):
        if key in allowed:
            continue
        values = {profiles[name].get(key, "") for name in env_names}
        if len(values) > 1:
            drift[f"drift:{key}"] = sorted(values)

    return len(drift) == 0, drift


# ---------------------------------------------------------------------------
# Configuration Loader
# ---------------------------------------------------------------------------


def load_autosync_config_from_env() -> WorkstreamAutosyncConfig:
    """Load autosync configuration from environment variables.

    Environment variables:
        THGENT_WORKSTREAM_AUTOSYNC_ENABLED: Enable autosync (true/false)
        THGENT_WORKSTREAM_AUTOSYNC_INTERVAL: Cycle interval in seconds (default: 300)
        THGENT_GITHUB_ENABLED: Enable GitHub sync (true/false)
        THGENT_GITHUB_OWNER: GitHub repository owner
        THGENT_GITHUB_PROJECT_NUMBER: GitHub project v2 number
    THGENT_GITHUB_DIRECTION: Sync direction (read_only|write_only|bidirectional)
        THGENT_LINEAR_ENABLED: Enable Linear sync (true/false)
        THGENT_LINEAR_API_KEY: Linear API key
        THGENT_LINEAR_TEAM_KEY: Linear team key
    THGENT_LINEAR_DIRECTION: Sync direction (read_only|write_only|bidirectional)
    THGENT_WORKSTREAM_PATH: Path to WORK_STREAM.md
    THGENT_AUTOSYNC_STATUS_PATH: Path to autosync status JSON
    THGENT_WORKSTREAM_AUTOSYNC_CHECKPOINT_PATH: Path to checkpoint JSON
    THGENT_WORKSTREAM_AUTOSYNC_MAX_PARTITION_SIZE: Dynamic partition size
    THGENT_WORKSTREAM_AUTOSYNC_TTL_SECONDS: Checkpoint TTL seconds
    THGENT_AUTOSYNC_MAINTENANCE_WINDOWS: Maintenance windows (`connector:start:end:project:reason` or `connector:start:end:reason` legacy; JSON array also supported)
    THGENT_WORKSTREAM_TAG_TAXONOMY: Comma-separated approved tags
    THGENT_WORKSTREAM_AUTOSYNC_FAILURE_QUEUE_PATH: Path to failure queue JSON
    THGENT_WORKSTREAM_AUTOSYNC_FAILURE_QUEUE_TTL_SECONDS: Failure queue retention seconds
    THGENT_WORKSTREAM_AUTOSYNC_CHANGE_DIGEST_PATH: Path for hourly change digest JSONL output
    THGENT_WORKSTREAM_AUTOSYNC_REFLECTION_EVENT_LOG_PATH: Path for reflection event log JSONL output
    THGENT_AUTOSYNC_CONNECTOR_SLA_THRESHOLDS: JSON connector SLA thresholds (`{"github":{"p95_latency_ms":250,"max_failure_rate":0.2}}`)
    THGENT_WORKSTREAM_STRICT_TAG_VALIDATION: Require tags to match taxonomy
    THGENT_WORKSTREAM_STRICT_TITLE_VALIDATION: Require duplicate titles to fail
    THGENT_AUTOSYNC_EMERGENCY_STOP_ENABLED: Enable emergency stop checks
    THGENT_AUTOSYNC_EMERGENCY_STOP_FILE_PATH: Sentinel file path for emergency stop
    THGENT_AUTOSYNC_EMERGENCY_STOP_ENV_VAR: Env var name checked for emergency stop activation

    Returns:
        WorkstreamAutosyncConfig instance
    """
    import os

    def parse_bool(value: str | None, default: bool = False) -> bool:
        if value is None:
            return default
        return value.lower() in ("true", "1", "yes", "on")

    def parse_int(value: str | None, default: int) -> int:
        if value is None:
            return default
        try:
            return int(value)
        except ValueError:
            return default

    def parse_float(value: str | None, default: float) -> float:
        if value is None:
            return default
        try:
            return float(value)
        except ValueError:
            return default

    def parse_wl_ignore_list(raw: str | None) -> list[str]:
        if raw is None:
            return []
        value = raw.strip()
        if not value:
            return []
        if value.startswith("["):
            payload = json.loads(value)
            if not isinstance(payload, list):
                raise ValueError("THGENT_WORKSTREAM_WL_IGNORE_LIST JSON must be a list")
            candidates = [str(item).strip().upper() for item in payload if str(item).strip()]
        else:
            candidates = [token.strip().upper() for token in value.split(",") if token.strip()]
        for candidate in candidates:
            if not WL_ID_PATTERN.fullmatch(candidate):
                raise ValueError(f"Invalid WL ignore ID: {candidate}")
        return sorted(set(candidates))

    def parse_json_windows(raw: str | None) -> list[MaintenanceWindow]:
        if not raw:
            return []
        maintenance_windows: list[MaintenanceWindow] = []
        candidate_windows = raw.strip()
        if candidate_windows.startswith("["):
            try:
                payload = json.loads(candidate_windows)
            except json.JSONDecodeError as exc:
                logger.warning("Invalid THGENT_AUTOSYNC_MAINTENANCE_WINDOWS JSON: %s", exc)
                return []
            if not isinstance(payload, list):
                return []
            for item in payload:
                if not isinstance(item, dict):
                    continue
                connector = str(item.get("connector", "all")).strip().lower() or "all"
                project = str(item.get("project", "default")).strip().lower() or "default"
                start_raw = item.get("start_utc") or item.get("start")
                end_raw = item.get("end_utc") or item.get("end")
                reason = str(item.get("reason", "")).strip()
                if start_raw is None or end_raw is None:
                    continue
                try:
                    maintenance_windows.append(
                        MaintenanceWindow(
                            connector=connector,
                            start_utc=datetime.fromisoformat(str(start_raw).replace("Z", "+00:00")),
                            end_utc=datetime.fromisoformat(str(end_raw).replace("Z", "+00:00")),
                            reason=reason,
                            project=project,
                        )
                    )
                except (TypeError, ValueError):
                    logger.debug("Skipping malformed maintenance window item: %s", item)
            return maintenance_windows

        for raw_entry in candidate_windows.split(";"):
            raw_entry = raw_entry.strip()
            if not raw_entry:
                continue

            connector, separator, remainder = raw_entry.partition(":")
            if not separator:
                logger.debug("Skipping malformed maintenance window token: %s", raw_entry)
                continue

            def _parse_iso_pair(raw: str) -> tuple[datetime, datetime, str] | None:
                if not raw:
                    return None
                iso_pattern = re.compile(r"^(\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z|[+\-]\d{2}:\d{2}))")
                start_match = iso_pattern.match(raw)
                if not start_match:
                    return None
                start_raw = start_match.group(0)
                cursor = len(start_raw)
                if cursor >= len(raw) or raw[cursor] != ":":
                    return None
                cursor += 1

                end_raw_remainder = raw[cursor:]
                end_match = iso_pattern.match(end_raw_remainder)
                if not end_match:
                    return None
                end_raw = end_match.group(0)
                cursor += len(end_raw)
                metadata = end_raw_remainder[len(end_raw) :]
                metadata = metadata.removeprefix(":")

                try:
                    start_utc = datetime.fromisoformat(start_raw.replace("Z", "+00:00"))
                    end_utc = datetime.fromisoformat(end_raw.replace("Z", "+00:00"))
                except (TypeError, ValueError):
                    return None
                return start_utc, end_utc, metadata

            parsed_window = _parse_iso_pair(remainder)
            if parsed_window is None:
                logger.debug("Skipping malformed maintenance window token: %s", raw_entry)
                continue
            start_utc, end_utc, remainder = parsed_window
            if ":" in remainder:
                project, reason = remainder.split(":", 1)
                project = project.strip() or "default"
                reason = reason.strip()
            else:
                project = "default"
                reason = remainder.strip()
            try:
                maintenance_windows.append(
                    MaintenanceWindow(
                        connector=connector.strip().lower() or "all",
                        start_utc=start_utc,
                        end_utc=end_utc,
                        reason=reason,
                        project=project,
                    )
                )
            except (TypeError, ValueError):
                logger.debug("Skipping malformed maintenance window token: %s", raw_entry)
        return maintenance_windows

    def parse_tag_taxonomy(raw: str | None) -> list[str]:
        if not raw:
            return []
        raw = raw.strip()
        if not raw:
            return []
        if raw.startswith("["):
            try:
                payload = json.loads(raw)
            except json.JSONDecodeError:
                return []
            if not isinstance(payload, list):
                return []
            return [str(tag).strip().lower() for tag in payload if str(tag).strip()]
        return [tag.strip().lower() for tag in raw.split(",") if tag.strip()]

    def parse_connector_health_states(raw: str | None) -> dict[str, str]:
        if not raw:
            return {}
        try:
            payload = json.loads(raw)
        except json.JSONDecodeError:
            return {}
        if not isinstance(payload, dict):
            return {}
        states: dict[str, str] = {}
        for key, value in payload.items():
            states[str(key).strip().lower()] = str(value).strip().lower()
        return states

    def parse_bootstrap_mappings(raw: str | None) -> list[str]:
        if not raw:
            return []
        return [field.strip() for field in raw.split(",") if field.strip()]

    def parse_scope_tokens(raw: str | None, *, upper: bool = False) -> list[str]:
        if not raw:
            return []
        values = [token.strip() for token in raw.split(",") if token.strip()]
        if upper:
            return [value.upper() for value in values]
        return values

    def parse_capability_map(raw: str | None) -> dict[str, list[str]]:
        if not raw:
            return {}
        try:
            payload = json.loads(raw)
        except json.JSONDecodeError:
            return {}
        if not isinstance(payload, dict):
            return {}
        normalized: dict[str, list[str]] = {}
        for connector, values in payload.items():
            if isinstance(values, list):
                normalized[str(connector).strip().lower()] = [str(value).strip().lower() for value in values]
        return normalized

    def parse_connector_sla_thresholds(raw: str | None) -> dict[str, ConnectorSLAThresholds]:
        if not raw:
            return {}
        try:
            payload = json.loads(raw)
        except json.JSONDecodeError:
            return {}
        if not isinstance(payload, dict):
            return {}

        normalized: dict[str, ConnectorSLAThresholds] = {}
        for connector, value in payload.items():
            if not isinstance(value, dict):
                continue
            p95_latency_ms = value.get("p95_latency_ms")
            max_failure_rate = value.get("max_failure_rate")
            if p95_latency_ms is None or max_failure_rate is None:
                logger.debug(
                    "Skipping malformed connector SLA threshold for %s: missing p95_latency_ms or max_failure_rate: %s",
                    connector,
                    value,
                )
                continue
            try:
                p95_latency_value = float(p95_latency_ms)
                max_failure_value = float(max_failure_rate)
                normalized[connector.strip().lower()] = ConnectorSLAThresholds(
                    p95_latency_ms=p95_latency_value,
                    max_failure_rate=max_failure_value,
                )
            except (TypeError, ValueError):
                logger.debug("Skipping malformed connector SLA threshold for %s: %s", connector, value)
                continue
        return normalized

    github_direction = os.getenv("THGENT_GITHUB_DIRECTION", "bidirectional")
    try:
        github_dir = SyncDirection(github_direction)
    except ValueError:
        logger.warning("Invalid THGENT_GITHUB_DIRECTION: %s, using bidirectional", github_direction)
        github_dir = SyncDirection.BIDIRECTIONAL

    linear_direction = os.getenv("THGENT_LINEAR_DIRECTION", "bidirectional")
    try:
        linear_dir = SyncDirection(linear_direction)
    except ValueError:
        logger.warning("Invalid THGENT_LINEAR_DIRECTION: %s, using bidirectional", linear_direction)
        linear_dir = SyncDirection.BIDIRECTIONAL

    work_stream_path = os.getenv("THGENT_WORKSTREAM_PATH")
    status_file_path = os.getenv("THGENT_AUTOSYNC_STATUS_PATH")
    checkpoint_file_path = os.getenv("THGENT_WORKSTREAM_AUTOSYNC_CHECKPOINT_PATH")
    cycle_manifest_path = os.getenv("THGENT_WORKSTREAM_AUTOSYNC_CYCLE_MANIFEST_PATH")
    cycle_metrics_path = os.getenv("THGENT_WORKSTREAM_AUTOSYNC_CYCLE_METRICS_PATH")
    change_digest_path = os.getenv("THGENT_WORKSTREAM_AUTOSYNC_CHANGE_DIGEST_PATH")
    reflection_event_log_path = os.getenv("THGENT_WORKSTREAM_AUTOSYNC_REFLECTION_EVENT_LOG_PATH")
    failure_queue_path = os.getenv("THGENT_WORKSTREAM_AUTOSYNC_FAILURE_QUEUE_PATH")
    autosync_prometheus_export_path = os.getenv("THGENT_AUTOSYNC_PROMETHEUS_EXPORT_PATH")
    connector_mapping_cache_path = os.getenv("THGENT_CONNECTOR_MAPPING_CACHE_PATH")
    trend_file_path = os.getenv("THGENT_WORKSTREAM_AUTOSYNC_TREND_PATH")
    writer_lock_path = os.getenv("THGENT_WORKSTREAM_AUTOSYNC_LOCK_PATH")
    emergency_stop_file_path = os.getenv("THGENT_AUTOSYNC_EMERGENCY_STOP_FILE_PATH")
    incident_bundle_path = os.getenv("THGENT_WORKSTREAM_INCIDENT_BUNDLE_PATH")
    bootstrap_mapping_cache_path = os.getenv("THGENT_CONNECTOR_MAPPING_CACHE_PATH")
    emergency_stop_env_var = os.getenv("THGENT_AUTOSYNC_EMERGENCY_STOP_ENV_VAR", "THGENT_AUTOSYNC_EMERGENCY_STOP")
    maintenance_windows = parse_json_windows(os.getenv("THGENT_AUTOSYNC_MAINTENANCE_WINDOWS"))
    allowed_tags = parse_tag_taxonomy(os.getenv("THGENT_WORKSTREAM_TAG_TAXONOMY"))
    wl_ignore_list = parse_wl_ignore_list(os.getenv("THGENT_WORKSTREAM_WL_IGNORE_LIST"))
    connector_health_states = parse_connector_health_states(os.getenv("THGENT_CONNECTOR_HEALTH_STATES"))
    connector_capabilities = parse_capability_map(os.getenv("THGENT_CONNECTOR_CAPABILITIES"))
    required_connector_capabilities = parse_capability_map(os.getenv("THGENT_REQUIRED_CONNECTOR_CAPABILITIES"))
    connector_sla_thresholds = parse_connector_sla_thresholds(os.getenv("THGENT_AUTOSYNC_CONNECTOR_SLA_THRESHOLDS"))
    bootstrap_required_fields = parse_bootstrap_mappings(os.getenv("THGENT_BOOTSTRAP_REQUIRED_FIELDS"))
    scope_areas = parse_scope_tokens(os.getenv("THGENT_WORKSTREAM_SYNC_SCOPE_AREAS"))
    scope_statuses = parse_scope_tokens(os.getenv("THGENT_WORKSTREAM_SYNC_SCOPE_STATUSES"), upper=True)
    scope_priorities = parse_scope_tokens(os.getenv("THGENT_WORKSTREAM_SYNC_SCOPE_PRIORITIES"), upper=True)
    scope_wl_ranges = parse_scope_tokens(os.getenv("THGENT_WORKSTREAM_SYNC_SCOPE_WL_RANGES"), upper=True)
    remote_missing_policy_raw = os.getenv("THGENT_WORKSTREAM_REMOTE_MISSING_ITEM_POLICY", "ignore").strip().lower()
    try:
        remote_missing_policy = RemoteMissingItemPolicy(remote_missing_policy_raw)
    except ValueError as exc:
        raise ValueError(f"Invalid THGENT_WORKSTREAM_REMOTE_MISSING_ITEM_POLICY: {remote_missing_policy_raw}") from exc
    metadata_last_refreshed_at_raw = os.getenv("THGENT_METADATA_LAST_REFRESHED_AT")
    metadata_last_refreshed_at = None
    if metadata_last_refreshed_at_raw:
        metadata_last_refreshed_at = datetime.fromisoformat(metadata_last_refreshed_at_raw.replace("Z", "+00:00"))
    explicit_enabled = os.getenv("THGENT_WORKSTREAM_AUTOSYNC_ENABLED")
    repo_previously_opted_in = parse_bool(os.getenv("THGENT_WORKSTREAM_AUTOSYNC_PREVIOUS_OPT_IN"), default=False)
    migration_phase = os.getenv("THGENT_WORKSTREAM_AUTOSYNC_MIGRATION_PHASE", "phase1-detect").strip().lower()
    error_budget_max_consecutive_failures = parse_int(
        os.getenv("THGENT_AUTOSYNC_ERROR_BUDGET_MAX_CONSECUTIVE_FAILURES", "3"),
        default=3,
    )
    error_budget_max_failure_rate = parse_float(
        os.getenv("THGENT_AUTOSYNC_ERROR_BUDGET_MAX_FAILURE_RATE", "0.5"),
        default=0.5,
    )
    error_budget_escalation_after = parse_int(
        os.getenv("THGENT_AUTOSYNC_ERROR_BUDGET_ESCALATION_AFTER", "5"), default=5
    )
    autosync_stale_snapshot_seconds = parse_int(
        os.getenv("THGENT_AUTOSYNC_STALE_SNAPSHOT_SECONDS", "3600"),
        default=3600,
    )
    enabled = (
        parse_bool(explicit_enabled)
        if explicit_enabled is not None
        else autosync_phase1_enabled(
            explicit_env=explicit_enabled,
            repo_previously_opted_in=repo_previously_opted_in,
        )
    )

    return WorkstreamAutosyncConfig(
        enabled=enabled,
        migration_phase=migration_phase,
        repo_previously_opted_in=repo_previously_opted_in,
        cycle_interval_seconds=parse_int(os.getenv("THGENT_WORKSTREAM_AUTOSYNC_INTERVAL", "300"), default=300),
        checkpoint_ttl_seconds=parse_int(
            os.getenv("THGENT_WORKSTREAM_AUTOSYNC_TTL_SECONDS", str(3600)),
            default=3600,
        ),
        conflict_ttl_seconds=parse_int(
            os.getenv("THGENT_WORKSTREAM_AUTOSYNC_CONFLICT_TTL_SECONDS", "1800"),
            default=1800,
        ),
        github_enabled=parse_bool(os.getenv("THGENT_GITHUB_ENABLED")),
        github_owner=os.getenv("THGENT_GITHUB_OWNER", ""),
        github_project_number=parse_int(os.getenv("THGENT_GITHUB_PROJECT_NUMBER", "0"), default=0),
        github_direction=github_dir,
        github_sandbox_mode=parse_bool(os.getenv("THGENT_GITHUB_SANDBOX_MODE")),
        github_sandbox_project_number=parse_int(os.getenv("THGENT_GITHUB_SANDBOX_PROJECT_NUMBER", "0"), default=0),
        linear_enabled=parse_bool(os.getenv("THGENT_LINEAR_ENABLED")),
        linear_api_key=os.getenv("THGENT_LINEAR_API_KEY", ""),
        linear_team_key=os.getenv("THGENT_LINEAR_TEAM_KEY", ""),
        linear_direction=linear_dir,
        work_stream_path=Path(work_stream_path) if work_stream_path else None,
        status_file_path=Path(status_file_path) if status_file_path else None,
        checkpoint_file_path=Path(checkpoint_file_path) if checkpoint_file_path else None,
        cycle_manifest_path=Path(cycle_manifest_path) if cycle_manifest_path else None,
        cycle_metrics_path=Path(cycle_metrics_path) if cycle_metrics_path else None,
        change_digest_path=Path(change_digest_path) if change_digest_path else None,
        reflection_event_log_path=(Path(reflection_event_log_path) if reflection_event_log_path else None),
        maintenance_windows=maintenance_windows,
        max_partition_size=parse_int(os.getenv("THGENT_WORKSTREAM_AUTOSYNC_MAX_PARTITION_SIZE", "200"), default=200),
        allowed_tags=allowed_tags,
        adaptive_interval_enabled=parse_bool(os.getenv("THGENT_WORKSTREAM_ADAPTIVE_INTERVAL_ENABLED")),
        adaptive_interval_min_seconds=parse_int(
            os.getenv("THGENT_WORKSTREAM_ADAPTIVE_INTERVAL_MIN_SECONDS", "30"),
            default=30,
        ),
        adaptive_interval_max_seconds=parse_int(
            os.getenv("THGENT_WORKSTREAM_ADAPTIVE_INTERVAL_MAX_SECONDS", "900"),
            default=900,
        ),
        connector_health_states=connector_health_states,
        incident_bundle_path=Path(incident_bundle_path) if incident_bundle_path else None,
        metadata_ttl_seconds=parse_int(os.getenv("THGENT_METADATA_TTL_SECONDS", "3600"), default=3600),
        metadata_last_refreshed_at=metadata_last_refreshed_at,
        bootstrap_connector=os.getenv("THGENT_BOOTSTRAP_CONNECTOR", "github"),
        bootstrap_required_fields=bootstrap_required_fields,
        bootstrap_mapping_cache_path=Path(bootstrap_mapping_cache_path) if bootstrap_mapping_cache_path else None,
        project_id=os.getenv("THGENT_WORKSTREAM_PROJECT_ID", "default"),
        require_actor_identity=parse_bool(os.getenv("THGENT_AUTOSYNC_REQUIRE_ACTOR_IDENTITY")),
        actor_id=os.getenv("THGENT_AUTOSYNC_ACTOR_ID", ""),
        actor_signature=os.getenv("THGENT_AUTOSYNC_ACTOR_SIGNATURE", ""),
        actor_signing_key=os.getenv("THGENT_AUTOSYNC_ACTOR_SIGNING_KEY", ""),
        connector_capabilities=connector_capabilities,
        required_connector_capabilities=required_connector_capabilities,
        connector_sla_thresholds=connector_sla_thresholds,
        payload_checksum_enforced=parse_bool(os.getenv("THGENT_AUTOSYNC_PAYLOAD_CHECKSUM_ENFORCED")),
        expected_payload_checksum=os.getenv("THGENT_AUTOSYNC_EXPECTED_PAYLOAD_CHECKSUM", ""),
        failure_queue_path=Path(failure_queue_path) if failure_queue_path else None,
        connector_mapping_cache_path=Path(connector_mapping_cache_path) if connector_mapping_cache_path else None,
        scope_areas=scope_areas,
        scope_statuses=scope_statuses,
        scope_priorities=scope_priorities,
        scope_wl_ranges=scope_wl_ranges,
        remote_missing_item_policy=remote_missing_policy,
        autosync_prometheus_export_path=(
            Path(autosync_prometheus_export_path) if autosync_prometheus_export_path else None
        ),
        wl_ignore_list=wl_ignore_list,
        github_write_timeout_seconds=parse_float(os.getenv("THGENT_GITHUB_WRITE_TIMEOUT_SECONDS"), default=30.0),
        github_read_timeout_seconds=parse_float(os.getenv("THGENT_GITHUB_READ_TIMEOUT_SECONDS"), default=30.0),
        linear_write_timeout_seconds=parse_float(os.getenv("THGENT_LINEAR_WRITE_TIMEOUT_SECONDS"), default=30.0),
        linear_read_timeout_seconds=parse_float(os.getenv("THGENT_LINEAR_READ_TIMEOUT_SECONDS"), default=30.0),
        connector_circuit_breaker_failure_threshold=parse_int(
            os.getenv("THGENT_AUTOSYNC_CONNECTOR_BREAKER_FAILURE_THRESHOLD"),
            default=3,
        ),
        connector_circuit_breaker_success_threshold=parse_int(
            os.getenv("THGENT_AUTOSYNC_CONNECTOR_BREAKER_SUCCESS_THRESHOLD"),
            default=1,
        ),
        connector_circuit_breaker_timeout_seconds=parse_float(
            os.getenv("THGENT_AUTOSYNC_CONNECTOR_BREAKER_TIMEOUT_SECONDS"),
            default=60.0,
        ),
        failure_queue_retention_seconds=parse_int(
            os.getenv("THGENT_WORKSTREAM_AUTOSYNC_FAILURE_QUEUE_TTL_SECONDS", str(60 * 60 * 24)),
            default=60 * 60 * 24,
        ),
        simulation_mode=parse_bool(os.getenv("THGENT_WORKSTREAM_AUTOSYNC_SIMULATION_MODE")),
        snapshot_retention_count=parse_int(os.getenv("THGENT_WORKSTREAM_AUTOSYNC_SNAPSHOT_RETENTION", "20"), 20),
        trend_file_path=Path(trend_file_path) if trend_file_path else None,
        artifact_encryption_enabled=parse_bool(os.getenv("THGENT_AUTOSYNC_ARTIFACT_ENCRYPTION")),
        artifact_encryption_key=os.getenv("THGENT_AUTOSYNC_ARTIFACT_KEY", ""),
        strict_tag_validation=parse_bool(os.getenv("THGENT_WORKSTREAM_STRICT_TAG_VALIDATION")),
        strict_title_validation=parse_bool(os.getenv("THGENT_WORKSTREAM_STRICT_TITLE_VALIDATION")),
        emergency_stop_enabled=parse_bool(os.getenv("THGENT_AUTOSYNC_EMERGENCY_STOP_ENABLED"), default=True),
        emergency_stop_file_path=Path(emergency_stop_file_path) if emergency_stop_file_path else None,
        emergency_stop_env_var=emergency_stop_env_var,
        writer_lock_enabled=parse_bool(os.getenv("THGENT_WORKSTREAM_AUTOSYNC_LOCK_ENABLED"), default=True),
        writer_lock_path=Path(writer_lock_path) if writer_lock_path else None,
        rate_limit_max_retries=parse_int(os.getenv("THGENT_AUTOSYNC_RATE_LIMIT_MAX_RETRIES"), default=2),
        rate_limit_initial_wait=parse_float(os.getenv("THGENT_AUTOSYNC_RATE_LIMIT_INITIAL_WAIT"), default=1.0),
        rate_limit_max_wait=parse_float(os.getenv("THGENT_AUTOSYNC_RATE_LIMIT_MAX_WAIT"), default=16.0),
        rate_limit_multiplier=parse_float(os.getenv("THGENT_AUTOSYNC_RATE_LIMIT_MULTIPLIER"), default=2.0),
        standalone_mode=parse_bool(os.getenv("THGENT_AUTOSYNC_STANDALONE_MODE"), default=True),
        shadow_mode=parse_bool(os.getenv("THGENT_WORKSTREAM_AUTOSYNC_SHADOW_MODE")),
        error_budget_max_consecutive_failures=error_budget_max_consecutive_failures,
        error_budget_max_failure_rate=error_budget_max_failure_rate,
        error_budget_escalation_after=error_budget_escalation_after,
        autosync_stale_snapshot_seconds=autosync_stale_snapshot_seconds,
    )
