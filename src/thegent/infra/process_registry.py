"""Process registry for tracking and cleaning up subprocesses."""

import atexit
import logging
import signal
import subprocess
import threading
import time
from dataclasses import dataclass, field

import psutil

logger = logging.getLogger(__name__)


@dataclass
class ProcessHandle:
    """Handle for a tracked subprocess."""

    pid: int
    proc: subprocess.Popen
    name: str
    started_at: float = field(default_factory=time.time)
    cleanup_on_exit: bool = True
    timeout: float | None = None

    def is_alive(self) -> bool:
        """Check if process is still running."""
        return self.proc.poll() is None

    def terminate(self, timeout: float = 5.0) -> bool:
        """Terminate process gracefully."""
        if not self.is_alive():
            return True

        try:
            self.proc.terminate()
            self.proc.wait(timeout=timeout)
            return True
        except subprocess.TimeoutExpired:
            self.proc.kill()
            try:
                self.proc.wait(timeout=2.0)
            except subprocess.TimeoutExpired:
                return False
            return True
        except Exception:
            logger.warning(f"Error terminating process {self.pid}")
            return False

    def get_psutil_process(self) -> psutil.Process | None:
        """Get psutil Process object for introspection.

        Returns:
            psutil.Process object, or None if process not found.
        """
        try:
            return psutil.Process(self.pid)
        except psutil.NoSuchProcess:
            return None

    def get_resource_usage(self) -> dict | None:
        """Get resource usage using psutil.

        Returns:
            Dictionary with resource usage information, or None if unavailable.
        """
        proc = self.get_psutil_process()
        if not proc:
            return None

        try:
            info = {
                "memory_mb": proc.memory_info().rss / 1024 / 1024,
                "cpu_percent": proc.cpu_percent(interval=0.1),
                "num_threads": proc.num_threads(),
                "open_files": len(proc.open_files()),
                "connections": len(proc.connections()),
            }

            # Add num_fds if available (Linux, macOS)
            if hasattr(proc, "num_fds"):
                info["num_fds"] = proc.num_fds()

            return info
        except (psutil.AccessDenied, AttributeError, psutil.NoSuchProcess):
            return None


class ProcessRegistry:
    """Registry for tracking subprocesses with automatic cleanup."""

    def __init__(self) -> None:
        self._processes: dict[int, ProcessHandle] = {}
        self._lock = threading.Lock()
        self._cleanup_registered = False
        self._register_cleanup()

    def _register_cleanup(self):
        """Register cleanup handlers."""
        if self._cleanup_registered:
            return

        # Register atexit handler
        atexit.register(self.cleanup_all)

        # Register signal handlers
        try:
            signal.signal(signal.SIGTERM, self._signal_handler)
            signal.signal(signal.SIGINT, self._signal_handler)
        except (ValueError, OSError) as exc:
            # Signal handlers may not work in all contexts (e.g., threads)
            logger.debug(f"Signal registration failed: {exc}")

        self._cleanup_registered = True

    def _signal_handler(self, signum, frame):
        """Handle termination signals."""
        logger.info(f"Received signal {signum}, cleaning up processes...")
        self.cleanup_all()
        raise SystemExit(1)

    def register(
        self,
        proc: subprocess.Popen,
        name: str,
        cleanup_on_exit: bool = True,
        timeout: float | None = None,
    ) -> ProcessHandle:
        """Register a process for tracking."""
        if proc.poll() is not None:
            # Process already finished
            return ProcessHandle(
                pid=proc.pid,
                proc=proc,
                name=name,
                cleanup_on_exit=False,
            )

        handle = ProcessHandle(
            pid=proc.pid,
            proc=proc,
            name=name,
            cleanup_on_exit=cleanup_on_exit,
            timeout=timeout,
        )

        with self._lock:
            self._processes[proc.pid] = handle

        logger.debug(f"Registered process {proc.pid} ({name})")
        return handle

    def unregister(self, pid: int) -> ProcessHandle | None:
        """Unregister a process."""
        with self._lock:
            return self._processes.pop(pid, None)

    def get(self, pid: int) -> ProcessHandle | None:
        """Get process handle by PID."""
        with self._lock:
            return self._processes.get(pid)

    def list_alive(self) -> list[ProcessHandle]:
        """List all alive processes."""
        with self._lock:
            return [h for h in self._processes.values() if h.is_alive()]

    def cleanup_all(self, timeout: float = 10.0) -> int:
        """Clean up all registered processes."""
        with self._lock:
            processes = list(self._processes.values())

        cleaned = 0
        for handle in processes:
            if handle.cleanup_on_exit and handle.is_alive():
                if handle.terminate(timeout=timeout):
                    cleaned += 1
                    self.unregister(handle.pid)

        if cleaned > 0:
            logger.info(f"Cleaned up {cleaned} processes")
        return cleaned

    def cleanup_orphaned(self) -> int:
        """Clean up processes that have died but weren't unregistered."""
        with self._lock:
            processes = list(self._processes.values())

        cleaned = 0
        for handle in processes:
            if not handle.is_alive():
                self.unregister(handle.pid)
                cleaned += 1

        if cleaned > 0:
            logger.debug(f"Cleaned up {cleaned} orphaned processes")
        return cleaned

    def cleanup_process_tree(self, pid: int, timeout: float = 10.0) -> int:
        """Clean up process and all children using psutil.

        Args:
            pid: Process ID to clean up.
            timeout: Timeout in seconds for process termination.

        Returns:
            Number of processes cleaned up.
        """
        try:
            proc = psutil.Process(pid)
            children = proc.children(recursive=True)

            cleaned = 0

            def _terminate_child(child: psutil.Process) -> int:
                """Terminate a child process, return 1 if successful."""
                try:
                    child.terminate()
                    return 1
                except psutil.NoSuchProcess:
                    return 0

            def _kill_child(child: psutil.Process) -> int:
                """Kill a child process, return 1 if successful."""
                try:
                    child.kill()
                    return 1
                except psutil.NoSuchProcess:
                    return 0

            # Terminate children first
            for child in children:
                cleaned += _terminate_child(child)

            # Wait for children using psutil.wait_procs
            if children:
                gone, alive = psutil.wait_procs(children, timeout=timeout)

                # Kill any remaining children
                for child in alive:
                    cleaned += _kill_child(child)

                cleaned = len(gone) + len(alive)

            # Terminate parent
            try:
                proc.terminate()
                proc.wait(timeout=timeout)
                cleaned += 1
            except (psutil.NoSuchProcess, psutil.TimeoutExpired):
                try:
                    proc.kill()
                    proc.wait(timeout=2.0)
                    cleaned += 1
                except psutil.NoSuchProcess:
                    pass

            return cleaned
        except psutil.NoSuchProcess:
            return 0
        except Exception:
            logger.warning(f"Error cleaning up process tree for PID {pid}")
            return 0

    def get_stats(self) -> dict:
        """Get registry statistics."""
        with self._lock:
            processes = list(self._processes.values())

        alive = [h for h in processes if h.is_alive()]
        process_info = []
        for h in processes:
            info = {
                "pid": h.pid,
                "name": h.name,
                "alive": h.is_alive(),
                "uptime": time.time() - h.started_at,
            }
            # Add resource usage if available
            resource_usage = h.get_resource_usage()
            if resource_usage:
                info["resource_usage"] = resource_usage
            process_info.append(info)

        return {
            "total": len(processes),
            "alive": len(alive),
            "dead": len(processes) - len(alive),
            "processes": process_info,
        }


# Global registry instance
_registry: ProcessRegistry | None = None


def get_registry() -> ProcessRegistry:
    """Get global process registry."""
    global _registry
    if _registry is None:
        _registry = ProcessRegistry()
    return _registry
