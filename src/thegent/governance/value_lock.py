"""WP-29001: Value-Lock (Immutable Ethical Constraints).
Provides a mechanism to lock core agent values and ethical constraints.
Ensures that even if self-evolution occurs, fundamental alignment principles cannot be removed.
"""

import hashlib
import json
import logging
from datetime import UTC, datetime
from pathlib import Path

from pydantic import BaseModel

_log = logging.getLogger(__name__)


class LockedPrinciple(BaseModel):
    """An ethically-locked principle that cannot be modified by autonomous loops."""

    principle_id: str
    description: str
    commitment_hash: str  # Hash of the initial definition
    locked_at: str = datetime.now(UTC).isoformat()


class ValueLock:
    """Manages immutable ethical constraints for thegent."""

    def __init__(self, lock_path: Path) -> None:
        self.lock_path = lock_path
        self.locked_principles: dict[str, LockedPrinciple] = {}
        self._load_locks()

    def _load_locks(self):
        """Load locked principles from disk."""
        if self.lock_path.exists():
            data = json.loads(self.lock_path.read_text(encoding="utf-8"))
            for k, v in data.items():
                self.locked_principles[k] = LockedPrinciple(**v)

    def lock_principle(self, principle_id: str, description: str):
        """Ethically lock a principle, preventing future modification."""
        if principle_id in self.locked_principles:
            _log.warning("Principle %s is already locked.", principle_id)
            return

        _log.info("Applying VALUE-LOCK to principle: %s", principle_id)

        # Commitment hash ensures the original intent is preserved
        c_hash = hashlib.sha256(description.encode()).hexdigest()

        lock = LockedPrinciple(principle_id=principle_id, description=description, commitment_hash=c_hash)

        self.locked_principles[principle_id] = lock
        self._save_locks()

    def validate_change(self, principle_id: str, new_description: str) -> bool:
        """Validate if a proposed change violates a Value-Lock."""
        lock = self.locked_principles.get(principle_id)
        if not lock:
            return True  # Not locked, allowed

        _log.warning("Attempted modification of VALUE-LOCKED principle: %s", principle_id)

        # In a strict value-lock, no change is allowed regardless of content
        # unless it matches the original commitment hash.
        new_hash = hashlib.sha256(new_description.encode()).hexdigest()
        if new_hash != lock.commitment_hash:
            _log.error("VALUE-LOCK VIOLATION: Proposal diverges from locked intent for %s", principle_id)
            return False

        return True

    def _save_locks(self):
        """Persist locked principles to disk."""
        data = {k: v.model_dump() for k, v in self.locked_principles.items()}
        self.lock_path.write_text(json.dumps(data, indent=2), encoding="utf-8")
