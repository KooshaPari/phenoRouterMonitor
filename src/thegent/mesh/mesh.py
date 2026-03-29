"""Phase 1 & 2: Process Detection, Mesh Init, and IPC Primitives for Agent Mesh Coordination."""

from __future__ import annotations

import json
import logging
import re
import threading
import time
import uuid
from pathlib import Path
from typing import Any

import psutil
from thegent.infra.fast_yaml_parser import yaml_dump

from thegent.mesh.task_queue import MaildirQueue

_log = logging.getLogger(__name__)


class MeshManager:
    """Core mesh initialization and process detection for agent coordination."""

    HEARTBEAT_INTERVAL_SECONDS = 5
    STALE_THRESHOLD_SECONDS = 15

    def __init__(self, mesh_root: Path = Path("/tmp/agent-mesh")) -> None:
        self.mesh_root = mesh_root
        self.agents_dir = self.mesh_root / "agents"
        self.queue_dir = self.mesh_root / "queue"
        self.tasks_dir = self.mesh_root / "var" / "tasks"
        self.intents_dir = self.mesh_root / "var" / "intents"
        self.locks_dir = self.mesh_root / "locks"
        self.metrics_dir = self.mesh_root / "metrics"
        self._init_mesh()

    def _init_mesh(self) -> None:
        """Initialize mesh directories with explicit permissions (SCLI-P1.4)."""
        (self.mesh_root / "var").mkdir(parents=True, exist_ok=True)
        mode_map = {
            self.agents_dir: 0o755,
            self.queue_dir: 0o1777,
            self.tasks_dir: 0o755,
            self.intents_dir: 0o755,
            self.locks_dir: 0o1777,
            self.metrics_dir: 0o755,
        }
        for path, mode in mode_map.items():
            path.mkdir(parents=True, exist_ok=True)
            path.chmod(mode)

    def _check_process(self, proc: psutil.Process, patterns: list[str]) -> dict[str, Any] | None:
        """Check if a process matches patterns. Returns process info or None."""
        try:
            cmdline = " ".join(proc.info["cmdline"] or [])
            if any(re.search(pattern, cmdline, flags=re.IGNORECASE) for pattern in patterns):
                return {
                    "pid": proc.info["pid"],
                    "name": proc.info["name"],
                    "cmdline": cmdline,
                    "discovered_at": time.time(),
                }
        except (psutil.NoSuchProcess, psutil.AccessDenied):
            pass
        return None

    def discover_agents(self, patterns: list[str]) -> list[dict[str, Any]]:
        """Discover agents based on process patterns."""
        discovered = []
        for proc in psutil.process_iter(["pid", "name", "cmdline"]):
            info = self._check_process(proc, patterns)
            if info is not None:
                discovered.append(info)
        return discovered

    def register_agent(self, agent_id: str, metadata: dict[str, Any]) -> None:
        """Create agent manifest in mesh."""
        manifest_path = self.agents_dir / f"{agent_id}.yaml"
        with open(manifest_path, "w") as f:
            yaml_dump({"id": agent_id, "registered_at": time.time(), **metadata}, f)

    def heartbeat(self, agent_id: str) -> None:
        """Touch heartbeat file."""
        hb_file = self.agents_dir / f"{agent_id}.heartbeat"
        hb_file.touch()

    def start_heartbeat_loop(self, agent_id: str, stop_event: threading.Event | None = None) -> threading.Event:
        """Start a background heartbeat loop for *agent_id* (5 seconds by default)."""
        if stop_event is None:
            stop_event = threading.Event()

        def _run() -> None:
            while not stop_event.is_set():
                self.heartbeat(agent_id)
                stop_event.wait(self.HEARTBEAT_INTERVAL_SECONDS)

        threading.Thread(target=_run, daemon=True).start()
        return stop_event

    def cleanup_stale(self, threshold: int = STALE_THRESHOLD_SECONDS) -> int:
        """Cleanup agents whose heartbeats are older than threshold."""
        now = time.time()
        reclaimed = 0
        for hb in self.agents_dir.glob("*.heartbeat"):
            if now - hb.stat().st_mtime > threshold:
                agent_id = hb.stem
                _log.info("Cleaning up stale agent %s", agent_id)
                hb.unlink()
                manifest = self.agents_dir / f"{agent_id}.yaml"
                if manifest.exists():
                    manifest.unlink()
                reclaimed += self._reclaim_tasks(agent_id)
        return reclaimed

    def _reclaim_tasks(self, agent_id: str) -> int:
        """Reclaim in-flight tasks currently owned by a stale agent."""
        queue = MaildirQueue(self.queue_dir)
        return queue.reclaim_owner(agent_id)


class MeshIPC:
    """IPC primitives for agent mesh: mkdir locks and Maildir queue."""

    def __init__(self, mesh_root: Path) -> None:
        self.mesh_root = mesh_root
        self.locks_dir = mesh_root / "locks"
        self.queue_dir = mesh_root / "queue"

    def acquire_lock(self, name: str) -> bool:
        """Atomic mkdir lock."""
        lock_path = self.locks_dir / name
        try:
            lock_path.mkdir()
            return True
        except FileExistsError:
            return False

    def release_lock(self, name: str) -> None:
        """Release mkdir lock."""
        lock_path = self.locks_dir / name
        if lock_path.exists():
            lock_path.rmdir()

    def send_message(self, agent_id: str, message: dict[str, Any]) -> None:
        """Send message via Maildir-like queue."""
        agent_queue = self.queue_dir / agent_id
        for d in ["tmp", "new", "cur"]:
            (agent_queue / d).mkdir(parents=True, exist_ok=True)

        msg_id = str(uuid.uuid4())
        tmp_path = agent_queue / "tmp" / msg_id
        new_path = agent_queue / "new" / msg_id

        with open(tmp_path, "w") as f:
            json.dump(message, f)
        tmp_path.rename(new_path)
