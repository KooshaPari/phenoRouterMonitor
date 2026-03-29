"""Single-Writer Lock Discipline (WL-175): Enforce serial access to shared resources.

Provides a file-based lock mechanism to ensure that only one agent or process
can write to critical resources at a time (e.g., WORK_STREAM.md, config files,
autosync state).

The lock is implemented using a lockfile at docs/reference/autosync.lock,
which contains the owner ID and timestamp. The lock can be acquired, released,
checked, and forcefully released in emergencies.

This is a lightweight alternative to distributed locks (etcd, Zookeeper) and
is suitable for single-machine, single-repository environments.
"""

import logging
import os
from datetime import datetime, UTC
from pathlib import Path

import orjson

logger = logging.getLogger(__name__)


# ---------------------------------------------------------------------------
# Exceptions
# ---------------------------------------------------------------------------


class WriterLockError(Exception):
    """Base exception for writer lock operations."""


class WriterLockAcquisitionError(WriterLockError):
    """Raised when lock acquisition fails."""


# ---------------------------------------------------------------------------
# Lock Implementation
# ---------------------------------------------------------------------------


class SingleWriterLock:
    """File-based single-writer lock for coordinating access to shared resources.

    The lock is stored as a JSON file at docs/reference/autosync.lock.
    When acquired, it contains the owner ID and acquisition timestamp.

    This lock is intended for single-machine scenarios and does not handle
    distributed or networked scenarios. For those, consider etcd or Zookeeper.

    Example:
        >>> lock = SingleWriterLock()
        >>> if lock.acquire("my-agent"):
        ...     try:
        ...         # Critical section
        ...         update_workstream()
        ...     finally:
        ...         lock.release("my-agent")
        ... else:
        ...     print("Lock held by", lock.get_owner())
    """

    DEFAULT_LOCK_PATH = Path("docs/reference/autosync.lock")

    def __init__(self, lock_path: Path | None = None):
        """Initialize the lock.

        Args:
            lock_path: Path to the lockfile. Defaults to docs/reference/autosync.lock.
        """
        self.lock_path = lock_path or self.DEFAULT_LOCK_PATH

    def acquire(self, owner_id: str) -> bool:
        """Acquire the lock.

        Args:
            owner_id: Identifier of the lock owner (e.g., agent name, process ID).

        Returns:
            True if the lock was acquired, False if already held by another owner.

        Raises:
            WriterLockAcquisitionError: If the lock file cannot be created.
        """
        if self.is_locked():
            return self.get_owner() == owner_id

        self.lock_path.parent.mkdir(parents=True, exist_ok=True)

        try:
            lock_data = {
                "owner": owner_id,
                "acquired_at": datetime.now(UTC).isoformat(),
            }
            flags = os.O_CREAT | os.O_EXCL | os.O_WRONLY
            fd = os.open(self.lock_path, flags, 0o644)
            with os.fdopen(fd, "w", encoding="utf-8") as handle:
                handle.write(orjson.dumps(lock_data, option=orjson.OPT_INDENT_2).decode("utf-8"))
            logger.debug("Lock acquired by %s at %s", owner_id, self.lock_path)
            return True
        except FileExistsError:
            return self.get_owner() == owner_id
        except Exception as e:
            logger.error("Failed to acquire lock: %s", e)
            raise WriterLockAcquisitionError(f"Failed to acquire lock: {e}") from e

    def release(self, owner_id: str) -> None:
        """Release the lock.

        Only the owner can release the lock. Attempting to release a lock
        owned by someone else is a no-op (does not raise).

        Args:
            owner_id: Identifier of the lock owner.
        """
        current_owner = self.get_owner()

        # Only the owner can release
        if current_owner and current_owner != owner_id:
            logger.warning(
                "Cannot release lock: owned by %s, release requested by %s",
                current_owner,
                owner_id,
            )
            return

        if self.lock_path.exists():
            try:
                self.lock_path.unlink()
                logger.debug("Lock released by %s", owner_id)
            except Exception as e:
                logger.error("Failed to release lock: %s", e)

    def is_locked(self) -> bool:
        """Check if the lock is currently held.

        Returns:
            True if the lockfile exists and is valid, False otherwise.
        """
        return self.lock_path.exists()

    def get_owner(self) -> str | None:
        """Get the current lock owner.

        Returns:
            The owner ID if the lock is held, None otherwise.
        """
        if not self.lock_path.exists():
            return None

        try:
            data = orjson.loads(self.lock_path.read_text(encoding="utf-8"))
            return data.get("owner")
        except Exception as e:
            logger.error("Failed to read lock file: %s", e)
            return None

    def force_release(self) -> None:
        """Force release the lock (emergency override).

        Use only in emergencies when the owner has crashed or is unresponsive.
        Does not check ownership.
        """
        if self.lock_path.exists():
            try:
                self.lock_path.unlink()
                logger.warning("Lock force-released")
            except Exception as e:
                logger.error("Failed to force-release lock: %s", e)
