"""Resource-aware subprocess management with automatic cleanup."""

import contextlib
import logging
import subprocess
import threading
from collections.abc import Iterator

from thegent.infra.process_registry import ProcessHandle, get_registry

logger = logging.getLogger(__name__)


class SubprocessManager:
    """Manager for subprocess lifecycle with resource tracking."""

    MAX_CONCURRENT_PROCESSES = 300  # Increased from 50 to support 30-300 concurrent sessions
    MAX_PROCESS_UPTIME = 3600  # 1 hour

    def __init__(self) -> None:
        self.registry = get_registry()
        self._active_count = 0
        self._lock = threading.Lock()

    @contextlib.contextmanager
    def popen(
        self,
        args: list[str],
        name: str,
        **kwargs,
    ) -> Iterator[subprocess.Popen]:
        """Context manager for subprocess.Popen with automatic cleanup."""
        # Check resource limits
        with self._lock:
            if self._active_count >= self.MAX_CONCURRENT_PROCESSES:
                raise RuntimeError(f"Maximum concurrent processes ({self.MAX_CONCURRENT_PROCESSES}) exceeded")
            self._active_count += 1

        proc: subprocess.Popen | None = None
        handle: ProcessHandle | None = None

        try:
            # Ensure stdout/stderr are handled
            if "stdout" not in kwargs:
                kwargs["stdout"] = subprocess.DEVNULL
            if "stderr" not in kwargs:
                kwargs["stderr"] = subprocess.DEVNULL
            if "stdin" not in kwargs:
                kwargs["stdin"] = subprocess.DEVNULL

            proc = subprocess.Popen(args, **kwargs)
            handle = self.registry.register(
                proc=proc,
                name=name,
                cleanup_on_exit=True,
            )

            yield proc

        finally:
            with self._lock:
                self._active_count -= 1

            # Cleanup
            if proc is not None:
                # Drain stdout/stderr to prevent blocking
                if proc.stdout and proc.stdout != subprocess.DEVNULL:
                    with contextlib.suppress(Exception):
                        proc.stdout.close()
                if proc.stderr and proc.stderr != subprocess.DEVNULL:
                    with contextlib.suppress(Exception):
                        proc.stderr.close()

                # Wait for process or terminate
                if proc.poll() is None:
                    try:
                        proc.wait(timeout=5.0)
                    except subprocess.TimeoutExpired:
                        proc.terminate()
                        try:
                            proc.wait(timeout=2.0)
                        except subprocess.TimeoutExpired:
                            proc.kill()
                            try:
                                proc.wait(timeout=1.0)
                            except subprocess.TimeoutExpired:
                                logger.warning(f"Process {proc.pid} did not terminate")

            if handle:
                self.registry.unregister(handle.pid)

    def run(
        self,
        args: list[str],
        name: str,
        timeout: float | None = None,
        **kwargs,
    ) -> subprocess.CompletedProcess:
        """Run subprocess with resource tracking."""
        with self.popen(args, name, **kwargs) as proc:
            try:
                stdout, stderr = proc.communicate(timeout=timeout)
                return subprocess.CompletedProcess(
                    args=args,
                    returncode=proc.returncode,
                    stdout=stdout,
                    stderr=stderr,
                )
            except subprocess.TimeoutExpired:
                proc.kill()
                stdout, stderr = proc.communicate()
                raise subprocess.TimeoutExpired(
                    cmd=args,
                    timeout=timeout if timeout is not None else 0.0,
                    output=stdout,
                    stderr=stderr,
                )


# Global manager instance
_manager: SubprocessManager | None = None


def get_subprocess_manager() -> SubprocessManager:
    """Get global subprocess manager."""
    global _manager
    if _manager is None:
        _manager = SubprocessManager()
    return _manager
