"""File coordination and optimistic concurrency control for the agent mesh."""

import hashlib
import json
import time
from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional

import structlog

logger = structlog.get_logger(__name__)


class HLCTimestamp:
    """Hybrid Logical Clock (HLC) (SCLI-P6.2)."""

    def __init__(self, physical: int = 0, logical: int = 0) -> None:
        self.physical = physical or int(time.time() * 1000)
        self.logical = logical

    def update(self, other: Optional["HLCTimestamp"] = None) -> "HLCTimestamp":
        """Update clock with physical and/or other logical clock."""
        now = int(time.time() * 1000)
        if other:
            self.physical = max(self.physical, other.physical, now)
            if self.physical == other.physical:
                self.logical = max(self.logical, other.logical) + 1
            else:
                self.logical = 0
        else:
            self.physical = max(self.physical, now)
            if self.physical == now:
                self.logical = 0
            else:
                self.logical += 1
        return self

    def __str__(self) -> str:
        return f"{self.physical}:{self.logical:04x}"

    @classmethod
    def parse(cls, s: str) -> "HLCTimestamp":
        """Parse HLC timestamp from string."""
        parts = s.split(":")
        if len(parts) == 2:
            return cls(int(parts[0]), int(parts[1], 16))
        return cls()


class OptimisticConcurrencyControl:
    """OCC version tracking (SCLI-P6.1)."""

    def __init__(self, mesh_root: Path) -> None:
        self.version_dir = mesh_root / "versions"
        self.version_dir.mkdir(parents=True, exist_ok=True, mode=0o1777)

    def get_version(self, file_path: Path) -> str:
        """Get current version hash of a file."""
        if not file_path.exists():
            return "empty"

        with open(file_path, "rb") as f:
            return hashlib.sha256(f.read()).hexdigest()

    def claim_version(self, file_path: Path, agent_id: str) -> str:
        """Record the version of a file at claim time (SCLI-P6.1)."""
        version = self.get_version(file_path)
        claim_data = {"agent_id": agent_id, "version": version, "timestamp": str(HLCTimestamp().update())}

        file_id = hashlib.sha256(str(file_path).encode()).hexdigest()
        with open(self.version_dir / f"{file_id}-{agent_id}.json", "w") as f:
            json.dump(claim_data, f)

        return version

    def verify_version(self, file_path: Path, agent_id: str) -> bool:
        """Verify the file hasn't changed since it was claimed."""
        file_id = hashlib.sha256(str(file_path).encode()).hexdigest()
        claim_file = self.version_dir / f"{file_id}-{agent_id}.json"

        if not claim_file.exists():
            return True  # No claim found, proceed at own risk?

        with open(claim_file) as f:
            claim_data = json.load(f)

        current_version = self.get_version(file_path)
        return current_version == claim_data["version"]


class FileClaimsRegistry:
    """Lease-based file claims registry (SCLI-P6.3–P6.4)."""

    def __init__(self, mesh_root: Path) -> None:
        self.claims_dir = mesh_root / "claims"
        self.claims_dir.mkdir(parents=True, exist_ok=True, mode=0o1777)

    def acquire_lease(self, file_path: Path, agent_id: str, mode: str = "exclusive", ttl: int = 30) -> bool:
        """Acquire a lease on a file (SCLI-P6.3)."""
        file_id = hashlib.sha256(str(file_path).encode()).hexdigest()
        claim_file = self.claims_dir / f"{file_id}.lock"

        # Check if already held by someone else and not expired
        if claim_file.exists():
            with open(claim_file) as f:
                data = json.load(f)
            if time.time() < data["expires_at"]:
                if data["agent_id"] != agent_id:
                    return False

        # Write new/renew lease (SCLI-P6.4)
        lease_data = {
            "agent_id": agent_id,
            "mode": mode,
            "expires_at": time.time() + ttl,
            "timestamp": str(HLCTimestamp().update()),
        }

        with open(claim_file, "w") as f:
            json.dump(lease_data, f)
        return True

    def release_lease(self, file_path: Path, agent_id: str) -> bool:
        """Release a lease on a file."""
        file_id = hashlib.sha256(str(file_path).encode()).hexdigest()
        claim_file = self.claims_dir / f"{file_id}.lock"

        if claim_file.exists():
            with open(claim_file) as f:
                data = json.load(f)
            if data["agent_id"] == agent_id:
                claim_file.unlink()
                return True

        return False

    def _cleanup_expired_lock(self, lock_file: Path, now: float) -> int:
        """Clean up a single expired lock file. Returns 1 if deleted, 0 otherwise."""
        with open(lock_file) as f:
            data = json.load(f)
        if now > data["expires_at"]:
            lock_file.unlink()
            return 1
        return 0

    def cleanup_expired(self) -> int:
        """Background cleanup daemon (SCLI-P6.4)."""
        count = 0
        now = time.time()
        for lock_file in self.claims_dir.glob("*.lock"):
            count += self._cleanup_expired_lock(lock_file, now)
        return count


@dataclass
class EditIntent:
    """Describes an agent's planned edit operation (TGNT-P7.2).

    An intent captures *what* an agent plans to do before it commits,
    enabling trial-merge conflict prediction.
    """

    agent_id: str
    file_path: str
    operation: str  # "modify", "create", "delete"
    line_ranges: list[tuple[int, int]] = field(default_factory=list)
    new_content: str | None = None
    timestamp: str | None = None

    def __post_init__(self) -> None:
        if self.timestamp is None:
            self.timestamp = str(HLCTimestamp().update())


@dataclass
class ConflictPrediction:
    """Result of a trial-merge conflict prediction (TGNT-P7.2)."""

    has_conflict: bool
    conflicting_files: list[str] = field(default_factory=list)
    details: str = ""


class IntentRegistry:
    """Registry for agent edit intents (TGNT-P7.2).

    Agents register their planned edits before committing. The registry
    enables trial-merge conflict prediction by comparing intents.
    """

    def __init__(self, mesh_root: Path) -> None:
        self.intents_dir = mesh_root / "intents"
        self.intents_dir.mkdir(parents=True, exist_ok=True, mode=0o1777)

    def register_intent(self, intent: EditIntent) -> Path:
        """Register an agent's edit intent."""
        intent_id = hashlib.sha256(f"{intent.agent_id}:{intent.file_path}:{intent.timestamp}".encode()).hexdigest()
        intent_file = self.intents_dir / f"{intent_id}.json"
        data = {
            "agent_id": intent.agent_id,
            "file_path": intent.file_path,
            "operation": intent.operation,
            "line_ranges": intent.line_ranges,
            "new_content": intent.new_content,
            "timestamp": intent.timestamp,
        }
        with open(intent_file, "w") as f:
            json.dump(data, f)
        return intent_file

    def _load_intent(self, intent_file: Path, agent_id: str | None) -> EditIntent | None:
        """Load an intent from file, optionally filtering by agent_id."""
        with open(intent_file) as f:
            data = json.load(f)
        if agent_id is None or data["agent_id"] == agent_id:
            return EditIntent(
                agent_id=data["agent_id"],
                file_path=data["file_path"],
                operation=data["operation"],
                line_ranges=data.get("line_ranges", []),
                new_content=data.get("new_content"),
                timestamp=data.get("timestamp"),
            )
        return None

    def _delete_intent_if_owned(self, intent_file: Path, agent_id: str) -> bool:
        """Delete an intent file if owned by the specified agent. Returns True if deleted."""
        with open(intent_file) as f:
            data = json.load(f)
        if data["agent_id"] == agent_id:
            intent_file.unlink()
            return True
        return False

    def get_intents(self, agent_id: str | None = None) -> list[EditIntent]:
        """Retrieve registered intents, optionally filtered by agent."""
        intents: list[EditIntent] = []
        for intent_file in self.intents_dir.glob("*.json"):
            intent = self._load_intent(intent_file, agent_id)
            if intent is not None:
                intents.append(intent)
        return intents

    def clear_intents(self, agent_id: str) -> int:
        """Clear all intents for a given agent (post-commit cleanup)."""
        count = 0
        for intent_file in self.intents_dir.glob("*.json"):
            if self._delete_intent_if_owned(intent_file, agent_id):
                count += 1
        return count


def _line_ranges_overlap(
    ranges_a: list[tuple[int, int]],
    ranges_b: list[tuple[int, int]],
) -> bool:
    """Return True if any range in ranges_a overlaps with any in ranges_b."""
    for start_a, end_a in ranges_a:
        for start_b, end_b in ranges_b:
            if start_a <= end_b and start_b <= end_a:
                return True
    return False


def predict_merge_conflicts(
    intent_a: EditIntent,
    intent_b: EditIntent,
) -> ConflictPrediction:
    """Predict whether two intents will conflict (TGNT-P7.2).

    Performs a trial-merge analysis by comparing planned edit operations:
    - Different files: no conflict.
    - Both delete the same file: no conflict (idempotent).
    - One creates, other modifies/deletes: conflict.
    - Both modify with overlapping line ranges: conflict.
    - Both modify with no line ranges specified: conflict (conservative).
    """
    # Different files never conflict
    if intent_a.file_path != intent_b.file_path:
        return ConflictPrediction(has_conflict=False)

    fp = intent_a.file_path

    # Both delete: idempotent, no conflict
    if intent_a.operation == "delete" and intent_b.operation == "delete":
        return ConflictPrediction(has_conflict=False)

    # One creates, other does something else on same path: conflict
    if "create" in (intent_a.operation, intent_b.operation):
        if intent_a.operation != intent_b.operation:
            return ConflictPrediction(
                has_conflict=True,
                conflicting_files=[fp],
                details=f"Create vs {intent_a.operation}/{intent_b.operation} on {fp}",
            )
        # Both create same file: conflict
        return ConflictPrediction(
            has_conflict=True,
            conflicting_files=[fp],
            details=f"Dual create on {fp}",
        )

    # One deletes, other modifies: conflict
    if "delete" in (intent_a.operation, intent_b.operation):
        return ConflictPrediction(
            has_conflict=True,
            conflicting_files=[fp],
            details=f"Delete vs modify on {fp}",
        )

    # Both modify: check line ranges
    if intent_a.line_ranges and intent_b.line_ranges:
        if _line_ranges_overlap(intent_a.line_ranges, intent_b.line_ranges):
            return ConflictPrediction(
                has_conflict=True,
                conflicting_files=[fp],
                details=f"Overlapping line ranges on {fp}",
            )
        return ConflictPrediction(has_conflict=False)

    # Both modify but no line ranges: conservative conflict
    return ConflictPrediction(
        has_conflict=True,
        conflicting_files=[fp],
        details=f"Both modify {fp} without line ranges (conservative)",
    )
