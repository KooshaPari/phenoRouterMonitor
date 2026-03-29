"""Maildir-style teammate task queue for agent mesh coordination (heliosShield Phase 11).

All file operations are atomic using os.rename (POSIX rename guarantee).
The queue is fully crash-recoverable: no in-memory state is required.
Tasks stranded in `cur/` after a crash are visible via `list_pending()`.
"""

import json
import logging
import time
import uuid
from pathlib import Path
from typing import Any

_log = logging.getLogger(__name__)

# Sentinel returned when a task file disappears between listing and reading
# (another worker raced us). Callers get None from dequeue().
_TASK_NOT_FOUND = object()


class MaildirQueue:
    """Filesystem-backed task queue using Maildir conventions.

    Directory layout::

        <path>/
          tmp/   # staging area; task is written here first
          new/   # ready to be claimed by a worker
          cur/   # claimed by a worker; in-flight

    Atomic delivery: write to ``tmp/`` then ``os.rename`` to ``new/``.
    Atomic claim: ``os.rename`` from ``new/`` to ``cur/``.

    Task file format (JSON)::

        {
            "id":         "<uuid>",
            "payload":    <any JSON-serialisable value>,
            "priority":   <int, 0-9; lower = higher priority>,
            "created_at": <unix timestamp float>,
            "attempts":   <int>,
            "owner":      <agent id, optional>
        }
    """

    def __init__(self, path: Path) -> None:
        self.path = Path(path)
        self._tmp = self.path / "tmp"
        self._new = self.path / "new"
        self._cur = self.path / "cur"
        self._init_dirs()

    # ------------------------------------------------------------------
    # Initialisation
    # ------------------------------------------------------------------

    def _init_dirs(self) -> None:
        """Create the three Maildir sub-directories if they do not exist."""
        for d in (self._tmp, self._new, self._cur):
            d.mkdir(parents=True, exist_ok=True, mode=0o1777)

    # ------------------------------------------------------------------
    # Public API
    # ------------------------------------------------------------------

    def enqueue(self, task: dict[str, Any], priority: int = 5) -> str:
        """Write *task* to the queue atomically.

        1. Serialise the envelope to ``tmp/<id>``.
        2. ``os.rename`` to ``new/<id>`` (atomic on POSIX).

        Args:
            task:     Arbitrary JSON-serialisable payload.
            priority: Integer 0-9; lower numbers are consumed first by
                      :py:meth:`dequeue`.  Defaults to 5 (middle).

        Returns:
            The unique task ID string.
        """
        task_id = str(uuid.uuid4())
        envelope: dict[str, Any] = {
            "id": task_id,
            "payload": task,
            "priority": int(priority),
            "created_at": time.time(),
            "attempts": 0,
        }
        tmp_path = self._tmp / task_id
        new_path = self._new / task_id

        # Write to tmp first, then atomically rename into new/
        tmp_path.write_text(json.dumps(envelope), encoding="utf-8")
        tmp_path.rename(new_path)

        _log.debug("enqueue task=%s priority=%d", task_id, priority)
        return task_id

    def dequeue(self, owner: str | None = None) -> dict[str, Any] | None:
        """Claim the highest-priority task from ``new/``.

        Moves the chosen file from ``new/`` to ``cur/`` atomically.
        Returns ``None`` when the queue is empty.

        Priority ordering: tasks with a lower ``priority`` value (e.g. 0)
        are returned before higher values.  Ties are broken by
        ``created_at`` (oldest first, FIFO within the same priority).

        Args:
            owner: Optional agent identifier to associate with the claimed task.

        Returns:
            The task envelope dict, or ``None`` if the queue is empty.
        """
        candidates = self._list_envelopes(self._new)
        if not candidates:
            return None

        # Sort: ascending priority, then ascending created_at (FIFO)
        candidates.sort(key=lambda e: (e["priority"], e["created_at"]))

        for envelope in candidates:
            task_id = envelope["id"]
            new_path = self._new / task_id
            cur_path = self._cur / task_id
            try:
                new_path.rename(cur_path)
            except FileNotFoundError:
                # Another worker claimed it between our listing and our rename;
                # try the next candidate.
                _log.debug("task %s already claimed, skipping", task_id)
                continue

            # Increment attempts counter in-place (not a delivery guarantee,
            # just informational; a crash between rename and this write is
            # acceptable — the field may stay at its previous value).
            envelope["attempts"] += 1
            if owner is not None:
                envelope["owner"] = owner
            cur_path.write_text(json.dumps(envelope), encoding="utf-8")

            _log.debug("dequeue task=%s (attempt %d)", task_id, envelope["attempts"])
            return envelope

        return None

    def ack(self, task_id: str) -> None:
        """Acknowledge successful completion of *task_id*.

        Removes the task file from ``cur/``.  Silently ignores a missing file
        (idempotent — safe to call more than once).

        Args:
            task_id: The task ID returned by :py:meth:`enqueue`.
        """
        cur_path = self._cur / task_id
        try:
            cur_path.unlink()
            _log.debug("ack task=%s", task_id)
        except FileNotFoundError:
            _log.debug("ack task=%s not found (already removed?)", task_id)

    def nack(self, task_id: str) -> None:
        """Negative-acknowledge *task_id*: return it to ``new/`` for retry.

        Moves the file from ``cur/`` back to ``new/`` atomically.
        Silently ignores a missing file (idempotent).

        Args:
            task_id: The task ID returned by :py:meth:`enqueue`.
        """
        cur_path = self._cur / task_id
        new_path = self._new / task_id
        try:
            cur_path.rename(new_path)
            _log.debug("nack task=%s (returned to new/)", task_id)
        except FileNotFoundError:
            _log.debug("nack task=%s not found in cur/", task_id)

    def list_pending(self) -> list[dict[str, Any]]:
        """Return all pending tasks from both ``new/`` and ``cur/``.

        Tasks in ``cur/`` are in-flight (being processed or stranded after a
        crash).  Tasks in ``new/`` are waiting to be claimed.

        Returns:
            List of task envelope dicts, unsorted.
        """
        return self._list_envelopes(self._new) + self._list_envelopes(self._cur)

    def reclaim_owner(self, owner: str) -> int:
        """Move all tasks in ``cur/`` owned by *owner* back to ``new/``."""
        reclaimed = 0
        for entry in self._cur.iterdir():
            if not entry.is_file():
                continue

            try:
                data = json.loads(entry.read_text(encoding="utf-8"))
            except (FileNotFoundError, json.JSONDecodeError):
                continue

            if data.get("owner") != owner:
                continue

            new_path = self._new / entry.name
            try:
                entry.rename(new_path)
            except FileNotFoundError:
                continue
            reclaimed += 1

        return reclaimed

    # ------------------------------------------------------------------
    # Internal helpers
    # ------------------------------------------------------------------

    def _list_envelopes(self, directory: Path) -> list[dict[str, Any]]:
        """Read and parse every JSON file in *directory*.

        Silently skips files that disappear mid-read or that are not valid
        JSON (e.g. partial writes left in ``tmp/``).
        """
        results: list[dict[str, Any]] = []
        for entry in directory.iterdir():
            if not entry.is_file():
                continue
            try:
                data = json.loads(entry.read_text(encoding="utf-8"))
                results.append(data)
            except (FileNotFoundError, json.JSONDecodeError) as exc:
                _log.debug("skipping %s: %s", entry, exc)
        return results
