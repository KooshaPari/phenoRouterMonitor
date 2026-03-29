"""Universal Multi-Runtime Coordinator.

This module allows a single program to orchestrate tasks across multiple Python runtimes
(PyPy, CPython 3.13, CPython 3.14) simultaneously using a shared state bridge.
"""

import asyncio
from asyncio import subprocess
import contextlib
import logging
import os
import platform
import tempfile
import time
from dataclasses import dataclass
from enum import Enum
from pathlib import Path
from typing import Any

from thegent.infra.ipc import IPCMesh, MaildirQueue

logger = logging.getLogger(__name__)


class RuntimeType(Enum):
    PYPY = "pypy"
    CPYTHON_313 = "3.13"
    CPYTHON_314 = "3.14"


@dataclass
class RuntimeTask:
    task_id: str
    runtime: RuntimeType
    module: str
    function: str
    args: list[Any]
    kwargs: dict[str, Any]
    timeout: float = 30.0


class MultiRuntimeBridge:
    """Orchestrates tasks across multiple specialized Python processes with network-aware reliability."""

    def __init__(self, mesh_root: Path | None = None) -> None:
        self.mesh_root = mesh_root or (Path(tempfile.gettempdir()) / "thegent-bridge")
        self.mesh = IPCMesh(self.mesh_root)
        self.active_workers: dict[RuntimeType, Any] = {}
        self.worker_heartbeats: dict[RuntimeType, float] = {}
        self._log_forwarder_tasks: set[asyncio.Task[None]] = set()

        # Network-aware tuning
        # macOS on Wi-Fi needs larger buffers and longer timeouts
        is_mac = platform.system() == "Darwin"
        self.default_timeout = 60.0 if is_mac else 30.0
        self.heartbeat_interval = 5.0 if is_mac else 2.0

        # Start background health monitor
        self._monitor_task = None

    async def _ensure_monitor_running(self):
        if self._monitor_task is None:
            self._monitor_task = asyncio.create_task(self._health_monitor_loop())

    async def _health_monitor_loop(self):
        """Monitor worker health and restart if crashed."""
        while True:
            for runtime, process in list(self.active_workers.items()):
                # Check if process is still running
                if process.returncode is not None:
                    del self.active_workers[runtime]
                    await self.start_worker(runtime)

                # Check heartbeat (future implementation: workers write to SHM/file)
                last_seen = self.worker_heartbeats.get(runtime, 0)
                if last_seen > 0 and (time.time() - last_seen) > self.heartbeat_interval * 3:
                    process.terminate()

            await asyncio.sleep(self.heartbeat_interval)

    async def start_worker(self, runtime: RuntimeType):
        """Start a long-running worker on a specific runtime."""
        if runtime in self.active_workers:
            return

        await self._ensure_monitor_running()

        worker_script = Path(__file__).parent / "worker_node.py"
        src_root = str(Path(__file__).parent.parent.parent)  # .../src
        venv_name = f".venv-{runtime.name.lower()}"

        # Build command using uv
        cmd = [
            "uv",
            "run",
            "--python",
            runtime.value,
            "python",
            str(worker_script),
            "--mesh-root",
            str(self.mesh_root),
            "--runtime",
            runtime.name,
        ]

        # Environment with platform-specific tuning
        env = os.environ.copy()
        env["PYTHONPATH"] = src_root
        env["UV_PROJECT_ENVIRONMENT"] = str(Path(src_root).parent / venv_name)

        # Network robustness: Increase socket timeouts for workers on Wi-Fi
        if platform.system() == "Darwin":
            env["THEGENT_NETWORK_TIER"] = "wifi"
            env["THEGENT_IPC_TIMEOUT"] = "10.0"
        else:
            env["THEGENT_NETWORK_TIER"] = "ethernet"
            env["THEGENT_IPC_TIMEOUT"] = "2.0"

        try:
            process = await asyncio.create_subprocess_exec(
                *cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, env=env
            )
            self.active_workers[runtime] = process
            self.worker_heartbeats[runtime] = time.time()

            # Log forwarding
            log_forwarder_task = asyncio.create_task(self._forward_logs(runtime, process))
            self._log_forwarder_tasks.add(log_forwarder_task)
            log_forwarder_task.add_done_callback(self._log_forwarder_tasks.discard)
        except Exception:
            raise

    async def _forward_logs(self, runtime: RuntimeType, process: Any):
        """Forward worker logs and track heartbeats."""
        while True:
            line = await process.stdout.readline()
            if not line:
                break
            text = line.decode().strip()
            if "[HEARTBEAT]" in text:
                self.worker_heartbeats[runtime] = time.time()
            else:
                logger.debug("Worker %s log: %s", runtime.name, text)

        err_data = await process.stderr.read()
        if err_data:
            try:
                stderr_text = err_data.decode("utf-8", errors="replace").strip()
            except Exception as exc:
                logger.warning("Failed decoding stderr for %s: %s", runtime.name, exc)
            else:
                logger.warning("Worker %s stderr: %s", runtime.name, stderr_text)

        await process.wait()
        if runtime in self.active_workers:
            del self.active_workers[runtime]

    async def dispatch(self, task: RuntimeTask) -> str:
        """Dispatch a task with automatic failover and status tracking."""
        try:
            await self.start_worker(task.runtime)
        except Exception:
            # Fallback to CPython 3.14 if PyPy fails, or vice versa
            fallback = RuntimeType.CPYTHON_314 if task.runtime == RuntimeType.PYPY else RuntimeType.PYPY
            await self.start_worker(fallback)
            task.runtime = fallback

        queue_path = self.mesh_root / "queue" / task.runtime.name.lower()
        runtime_queue = MaildirQueue(queue_path)

        message = {
            "type": "task",
            "task_id": task.task_id,
            "runtime": task.runtime.name,
            "module": task.module,
            "function": task.function,
            "args": task.args,
            "kwargs": task.kwargs,
            "dispatched_at": time.time(),
        }
        return runtime_queue.send(message)

    async def _shutdown_worker(self, process: subprocess.Process) -> None:
        """Gracefully shutdown a single worker process."""
        try:
            process.terminate()
            await asyncio.wait_for(process.wait(), timeout=2.0)
        except Exception:
            with contextlib.suppress(Exception):
                process.kill()

    async def shutdown(self):
        """Gracefully shutdown all workers and monitor."""
        if self._monitor_task:
            self._monitor_task.cancel()

        for task in list(self._log_forwarder_tasks):
            task.cancel()
        if self._log_forwarder_tasks:
            await asyncio.gather(*self._log_forwarder_tasks, return_exceptions=True)
            self._log_forwarder_tasks.clear()

        for _runtime, process in list(self.active_workers.items()):
            await self._shutdown_worker(process)

        self.active_workers.clear()
