"""Resource management and limits for the agent mesh."""

import contextlib
import os
import resource
from pathlib import Path


class ResourceManager:
    """Enforces resource limits on agent processes (SCLI-P12.1–P12.3)."""

    def __init__(self, agent_id: str) -> None:
        self.agent_id = agent_id

    def set_limits(self, memory_mb: int = 512, proc_limit: int = 50, fd_limit: int = 1024):
        """Set resource limits for the current process (SCLI-P12.1–P12.3)."""

        # Memory limit (RSS) (SCLI-P12.1)
        # Note: RLIMIT_AS or RLIMIT_RSS
        mem_bytes = memory_mb * 1024 * 1024
        with contextlib.suppress(ValueError, resource.error):
            resource.setrlimit(resource.RLIMIT_AS, (mem_bytes, mem_bytes))

        # Process count limit (SCLI-P12.2)
        with contextlib.suppress(ValueError, resource.error):
            resource.setrlimit(resource.RLIMIT_NPROC, (proc_limit, proc_limit))

        # File descriptor limit (SCLI-P12.3)
        with contextlib.suppress(ValueError, resource.error):
            resource.setrlimit(resource.RLIMIT_NOFILE, (fd_limit, fd_limit))

    def get_cgroup_path(self) -> Path | None:
        """Get Linux cgroup path for memory enforcement (SCLI-P12.1)."""
        if os.path.exists("/sys/fs/cgroup/memory"):
            path = Path(f"/sys/fs/cgroup/memory/agent-{self.agent_id}")
            return path
        return None

    def apply_cgroup_limits(self, memory_mb: int):
        """Apply cgroup-based memory limits (Linux only)."""
        path = self.get_cgroup_path()
        if not path:
            return

        try:
            path.mkdir(parents=True, exist_ok=True)
            with open(path / "memory.limit_in_bytes", "w") as f:
                f.write(str(memory_mb * 1024 * 1024))
            # Current process PID
            with open(path / "tasks", "w") as f:
                f.write(str(os.getpid()))
        except (PermissionError, OSError):
            pass  # Requires sudo or specific cgroup setup
