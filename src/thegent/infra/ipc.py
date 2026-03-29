"""Phase 11: IPC Primitives implementation.

Includes tmpfs mesh directory, atomic mkdir locks, Maildir queue,
filesystem notification, intent broadcast, conflict detection, and WAL.
"""

from __future__ import annotations

import orjson as json
import logging
import os
import fcntl
import threading
import time
import uuid
from dataclasses import dataclass
from pathlib import Path
from typing import Any

try:
    from watchfiles import watch
except Exception:  # pragma: no cover
    watch = None

logger = logging.getLogger(__name__)


class IPCMesh:
    """Manages the IPC mesh directory and atomic primitives."""

    def __init__(self, mesh_root: Path = Path("/tmp/agent-mesh")) -> None:
        self.mesh_root = mesh_root
        self.locks_dir = self.mesh_root / "locks"
        self._init_mesh()

    def _init_mesh(self) -> None:
        """Initialize tmpfs-like mesh directory."""
        try:
            self.mesh_root.mkdir(parents=True, exist_ok=True, mode=0o1777)
            self.locks_dir.mkdir(parents=True, exist_ok=True, mode=0o1777)
        except PermissionError as exc:
            raise PermissionError(
                f"Unable to initialize IPC mesh directories with mode 1777 at {self.mesh_root}"
            ) from exc

    def acquire_atomic_lock(self, lock_name: str, ttl: int = 60) -> bool:
        """Atomic lock primitive using mkdir (EEXIST)."""
        lock_path = self.locks_dir / lock_name
        lock_path.parent.mkdir(parents=True, exist_ok=True)
        try:
            lock_path.mkdir()
            metadata = {
                "pid": os.getpid(),
                "created_at": time.time(),
                "expires_at": time.time() + ttl,
                "expires_in": ttl,
                "owner": lock_name,
            }
            (lock_path / "metadata").write_text(json.dumps(metadata).decode())
            return True
        except FileExistsError:
            metadata_file = lock_path / "metadata"
            try:
                metadata = json.loads(metadata_file.read_text())
                expires_at = float(metadata.get("expires_at", 0))
                owner_pid = int(metadata.get("pid", 0))
                if time.time() > expires_at:
                    self.release_atomic_lock(lock_name)
                    return self.acquire_atomic_lock(lock_name, ttl)
                if owner_pid > 0:
                    try:
                        os.kill(owner_pid, 0)
                        return False
                    except ProcessLookupError:
                        self.release_atomic_lock(lock_name)
                        return self.acquire_atomic_lock(lock_name, ttl)
                    except PermissionError:
                        return False
                return False
            except (OSError, ValueError, json.JSONDecodeError, KeyError):
                self.release_atomic_lock(lock_name)
                return self.acquire_atomic_lock(lock_name, ttl)
            return False

    def release_atomic_lock(self, lock_name: str):
        """Release atomic lock."""
        lock_path = self.mesh_root / "locks" / lock_name
        if lock_path.exists():
            import shutil

            shutil.rmtree(lock_path, ignore_errors=True)


class MaildirQueue:
    """IPC message queue following Maildir-like tmp -> new -> cur lifecycle."""

    def __init__(self, queue_dir: Path) -> None:
        self.queue_dir = queue_dir
        self.tmp_dir = self.queue_dir / "tmp"
        self.new_dir = self.queue_dir / "new"
        self.cur_dir = self.queue_dir / "cur"
        for d in [self.tmp_dir, self.new_dir, self.cur_dir]:
            d.mkdir(parents=True, exist_ok=True, mode=0o1777)

    def send(self, message: dict[str, Any]) -> str:
        """Send message by writing into ``tmp`` and promoting to ``new``."""
        msg_id = f"{int(time.time())}.{uuid.uuid4().hex}"
        envelope = {
            "id": msg_id,
            "payload": message,
            "created_at": time.time(),
            "sender_pid": os.getpid(),
        }
        tmp_path = self.tmp_dir / msg_id
        new_path = self.new_dir / msg_id

        tmp_path.write_text(json.dumps(envelope).decode(), encoding="utf-8")
        tmp_path.rename(new_path)
        return msg_id

    def receive(self) -> tuple[str, dict[str, Any]] | None:
        """Receive the oldest message from ``new`` and move it to ``cur``."""
        new_msgs = sorted(
            (path for path in self.new_dir.iterdir() if path.is_file()),
            key=lambda path: path.name,
        )
        if not new_msgs:
            return None

        for msg_file in new_msgs:
            msg_id = msg_file.name
            cur_path = self.cur_dir / msg_id
            try:
                msg_file.rename(cur_path)
            except FileNotFoundError:
                continue

            try:
                payload = json.loads(cur_path.read_text(encoding="utf-8"))
                payload_message = payload["payload"]
            except (OSError, json.JSONDecodeError, KeyError) as exc:
                logger.warning("Skipping malformed message in %s: %s", msg_file, exc)
                cur_path.unlink(missing_ok=True)
                continue

            return msg_id, payload_message

        return None

    def ack(self, msg_id: str) -> None:
        """Acknowledge a delivered message by deleting it from ``cur``."""
        cur_path = self.cur_dir / msg_id
        try:
            cur_path.unlink()
        except FileNotFoundError:
            logger.debug("ack called for missing msg_id=%s", msg_id)

    def nack(self, msg_id: str) -> None:
        """Return a previously claimed message to ``new``."""
        cur_path = self.cur_dir / msg_id
        new_path = self.new_dir / msg_id
        try:
            cur_path.rename(new_path)
        except FileNotFoundError:
            logger.debug("nack called for missing msg_id=%s", msg_id)

    def list_pending(self) -> list[str]:
        """List unacked message IDs from ``new`` and ``cur``."""
        return [path.name for path in self.new_dir.iterdir() if path.is_file()] + [
            path.name for path in self.cur_dir.iterdir() if path.is_file()
        ]


@dataclass
class QueueEvent:
    path: str
    is_file: bool
    change_type: str


class QueueNotifier:
    """Filesystem event notification for maildir queues."""

    def __init__(self, queue_dir: Path, poll_interval: float = 0.5) -> None:
        self.queue_dir = Path(queue_dir)
        self.poll_interval = poll_interval

    def _snapshot(self) -> tuple[int, float, int]:
        try:
            new_paths = [p for p in (self.queue_dir / "new").iterdir() if p.is_file()]
        except FileNotFoundError:
            return (0, 0.0, 0)
        queue_token = len(new_paths)
        queue_mtime = 0.0
        if new_paths:
            queue_mtime = max(path.stat().st_mtime for path in new_paths)
        proc_token = self._proc_token()
        return queue_token, queue_mtime, proc_token

    def _proc_token(self) -> int:
        proc_stat = Path("/proc/self/stat")
        try:
            return proc_stat.stat().st_mtime_ns
        except OSError:
            return 0

    def wait_for_message(self, timeout: float = 5.0) -> tuple[QueueEvent, ...] | tuple[()]:
        """Wait for queue activity.

        Uses watchfiles/inotify when available with a /proc-backed polling
        fallback.
        """
        if watch is not None:
            return self._wait_with_watchfiles(timeout)
        return self._wait_with_proc_polling(timeout)

    def _wait_with_watchfiles(self, timeout: float) -> tuple[QueueEvent, ...]:
        watch_fn = watch
        if watch_fn is None:
            return ()
        stop = threading.Event()
        timer = threading.Timer(timeout, stop.set)
        events: list[QueueEvent] = []
        timer.start()
        try:
            for changes in watch_fn(
                self.queue_dir / "new",
                stop_event=stop,
                yield_on_timeout=True,
                recursive=False,
                rust_timeout=100,
            ):
                for change in changes:
                    event_type, path = change
                    events.append(QueueEvent(str(path), bool(Path(path).is_file()), str(event_type)))
                if events:
                    break
                if not stop.is_set():
                    continue
                break
        finally:
            timer.cancel()
        return tuple(events)

    def _wait_with_proc_polling(self, timeout: float) -> tuple[QueueEvent, ...]:
        end_time = time.monotonic() + timeout
        previous = self._snapshot()
        while time.monotonic() < end_time:
            time.sleep(self.poll_interval)
            current = self._snapshot()
            if current != previous:
                # We can't type-narrow exact change deltas without an underlying watcher.
                return (QueueEvent(str(self.queue_dir / "new"), True, "unknown"),)
        return ()


class IntentBroadcaster:
    """Broadcast and track agent intents."""

    def __init__(self, mesh_root: Path = Path("/tmp/agent-mesh")) -> None:
        self.mesh_root = mesh_root
        self.intents_dir = self.mesh_root / "var" / "intents"
        self.intents_dir.mkdir(parents=True, exist_ok=True, mode=0o1777)

    def broadcast(
        self, agent_id: str, intent: str, target: str, operation: str = "read", metadata: dict[str, Any] | None = None
    ) -> str:
        """Write a typed intent record and return the intent ID."""
        intent_id = f"{int(time.time())}.{uuid.uuid4().hex}"
        payload = {
            "id": intent_id,
            "agent_id": agent_id,
            "intent": intent,
            "target": target,
            "operation": operation,
            "metadata": metadata or {},
            "created_at": time.time(),
        }
        intent_path = self.intents_dir / f"{intent_id}.json"
        intent_path.write_text(json.dumps(payload).decode(), encoding="utf-8")
        return intent_id

    def list_active(self, agent_id: str | None = None) -> list[dict[str, Any]]:
        """List all intent records, optionally filtered by ``agent_id``."""
        results: list[dict[str, Any]] = []
        for entry in self.intents_dir.glob("*.json"):
            try:
                payload = json.loads(entry.read_text(encoding="utf-8"))
                if agent_id is None or payload.get("agent_id") == agent_id:
                    results.append(payload)
            except (OSError, json.JSONDecodeError):
                continue
        return results

    def clear(self, intent_id: str) -> bool:
        """Delete a specific intent file if present."""
        path = self.intents_dir / f"{intent_id}.json"
        if not path.exists():
            return False
        path.unlink()
        return True


class IntentConflictDetector:
    """Detect read-write and write-write conflicts between intents."""

    @staticmethod
    def _normalize_operation(operation: str) -> str:
        op = operation.lower()
        if op in {"read", "inspect", "peek", "scan", "list"}:
            return "read"
        return "write"

    @staticmethod
    def detect(intent: dict[str, Any], others: list[dict[str, Any]]) -> list[dict[str, str]]:
        """Return conflicting intents for ``intent`` against ``others``."""
        conflict_pairs: list[dict[str, str]] = []
        intent_target = intent.get("target")
        intent_op = IntentConflictDetector._normalize_operation(str(intent.get("operation", "read")))
        for other in others:
            if other.get("target") != intent_target:
                continue
            other_id = str(other.get("id", ""))
            other_op = IntentConflictDetector._normalize_operation(str(other.get("operation", "read")))
            if (
                (intent_op == "write" and other_op == "write")
                or (intent_op == "write" and other_op == "read")
                or (intent_op == "read" and other_op == "write")
            ):
                conflict_pairs.append(
                    {
                        "intent_id": str(intent.get("id", "")),
                        "conflict_with": other_id,
                        "target": str(intent_target),
                        "reason": f"{intent_op}-{other_op}",
                    }
                )
        return conflict_pairs


class SharedStateManager:
    """Manages high-frequency shared state (metrics, circuit breakers) in the mesh."""

    def __init__(self, mesh_root: Path) -> None:
        self.mesh_root = mesh_root
        self.metrics_dir = mesh_root / "metrics"
        self.metrics_dir.mkdir(parents=True, exist_ok=True)
        from .shm_manager import SHMManager

        self.shm = SHMManager(mesh_root / "state.shm")

    def update_provider_metrics(self, provider: str, metrics: dict[str, Any]):
        """Update metrics for a specific provider."""
        # Update SHM (High Performance)
        req_count = metrics.get("request_count", 0)
        succ_count = metrics.get("success_count", 0)
        lat_ms = int(metrics.get("latency_p50_ms", metrics.get("latency_ms", 0)))
        self.shm.update_provider_metrics(provider, req_count, succ_count, lat_ms)

        # Fallback to file-based (for observability/debugging)
        metric_file = self.metrics_dir / f"{provider}.json"
        tmp_file = metric_file.with_suffix(".tmp")
        tmp_file.write_text(json.dumps(metrics).decode())
        tmp_file.replace(metric_file)

    def get_all_metrics(self) -> dict[str, Any]:
        """Read all provider metrics from the mesh."""
        all_metrics = {}
        # Try reading from SHM first
        # (Note: we need a way to list all providers in SHM,
        # for now we'll merge file-based listing with SHM data)
        for f in self.metrics_dir.glob("*.json"):
            provider = f.stem
            shm_data = self.shm.get_provider_metrics(provider)
            if shm_data:
                all_metrics[provider] = shm_data
            else:
                try:
                    all_metrics[provider] = json.loads(f.read_text())
                except Exception:
                    continue
        return all_metrics


class WriteAheadLog:
    """WAL implementation for crash recovery."""

    def __init__(self, wal_file: Path) -> None:
        self.wal_file = wal_file
        self.wal_file.parent.mkdir(parents=True, exist_ok=True)

    def log(self, operation: str, data: dict[str, Any]):
        """Append entry to WAL before execution."""
        entry = {"timestamp": time.time(), "op": operation, "data": data, "id": uuid.uuid4().hex}
        with open(self.wal_file, "a") as f:
            fcntl.flock(f, fcntl.LOCK_EX)
            f.write(json.dumps(entry).decode() + "\n")
            f.flush()
            os.fsync(f.fileno())
            fcntl.flock(f, fcntl.LOCK_UN)

    def replay(self) -> list[dict[str, Any]]:
        """Read WAL entries for recovery."""
        if not self.wal_file.exists():
            return []

        entries = []
        with open(self.wal_file) as f:
            fcntl.flock(f, fcntl.LOCK_SH)
            for line in f:
                if line.strip():
                    entries.append(json.loads(line))
            fcntl.flock(f, fcntl.LOCK_UN)
        return entries
