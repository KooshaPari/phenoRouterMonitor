"""Phase 17: Resource Management implementation.
Includes memory limits, process limits, and FD budget monitoring.
"""

import logging
import resource
from typing import Any

import psutil

logger = logging.getLogger(__name__)


class ResourceManager:
    """Manages resource limits for agent processes."""

    def __init__(self) -> None:
        self._applied_limits: dict[str, tuple[int, int]] = {}

    def apply_limits(self, memory_mb: int = 512, proc_limit: int = 100) -> dict[str, tuple[int, int]]:
        """Apply soft/hard limits using resource module."""
        if memory_mb <= 0:
            raise ValueError("memory_mb must be > 0")
        if proc_limit <= 0:
            raise ValueError("proc_limit must be > 0")

        mem_bytes = memory_mb * 1024 * 1024
        nofile_limit = 1024

        self._applied_limits["memory"] = self._enforce_limit(resource.RLIMIT_AS, mem_bytes, "memory")
        self._applied_limits["processes"] = self._enforce_limit(resource.RLIMIT_NPROC, proc_limit, "processes")
        self._applied_limits["nofile"] = self._enforce_limit(resource.RLIMIT_NOFILE, nofile_limit, "nofile")

        logger.info(
            "Applied resource limits: memory=%sB processes=%s nofile=%s",
            self._applied_limits["memory"][0],
            self._applied_limits["processes"][0],
            self._applied_limits["nofile"][0],
        )
        return dict(self._applied_limits)

    def _enforce_limit(self, kind: int, target: int, label: str) -> tuple[int, int]:
        """Set soft limit to target (or clamp to hard limit when required)."""
        hard_limit = resource.getrlimit(kind)[1]
        if hard_limit == resource.RLIM_INFINITY:
            soft_limit = target
        else:
            soft_limit = min(target, hard_limit)

        if soft_limit <= 0:
            raise RuntimeError(f"invalid computed {label} soft limit: {soft_limit}")

        resource.setrlimit(kind, (soft_limit, hard_limit))
        current_soft, current_hard = resource.getrlimit(kind)
        if current_soft != soft_limit:
            raise RuntimeError(f"failed to enforce {label} limit: expected {soft_limit}, got {current_soft}")
        return current_soft, current_hard

    def get_applied_limits(self) -> dict[str, tuple[int, int]]:
        """Return the most recently applied resource limits."""
        return dict(self._applied_limits)

    def monitor_usage(self, pid: int) -> dict[str, Any]:
        """Monitor current resource usage of a process and record to SHM."""
        try:
            proc = psutil.Process(pid)
            mem_info = proc.memory_info()
            cpu_percent = proc.cpu_percent(interval=0.1)
            fd_count = proc.num_fds()
            children = proc.children(recursive=True)

            # Record to SHM for global observability
            try:
                from thegent_shm import record_resource_usage

                record_resource_usage(pid, cpu_percent, mem_info.rss // 1024)
            except (ImportError, RuntimeError):
                pass

            return {
                "pid": pid,
                "memory_rss": mem_info.rss,
                "cpu_percent": cpu_percent,
                "fd_count": fd_count,
                "child_count": len(children),
                "status": proc.status(),
            }
        except (psutil.NoSuchProcess, psutil.AccessDenied):
            return {"error": "Process not accessible"}


class ResourcePredictionEngine:
    """Predicts future resource usage based on historical Harness Cards and trends."""

    def __init__(self) -> None:
        self.history: list[dict[str, Any]] = []

    def predict_spawn_impact(self, harness_type: str, current_free_mb: int) -> bool:
        """Return True if spawning the agent is safe based on predicted impact."""
        # Simple heuristic: T3 isolation adds ~128MB overhead
        predicted_usage = 512  # Default
        if "claude" in harness_type.lower():
            predicted_usage = 1024
        elif "cursor" in harness_type.lower():
            predicted_usage = 768

        # Safety buffer
        if (current_free_mb - predicted_usage) < 256:
            logger.warning(f"Predictive throttle: Spawning {harness_type} may cause OOM.")
            return False
        return True

    def record_actual(self, harness_type: str, actual_mb: int):
        """Record actual usage for future predictions."""
        self.history.append({"harness": harness_type, "mb": actual_mb})
        # Keep last 100 entries
        if len(self.history) > 100:
            self.history = self.history[-100:]


class FDBudget:
    """Monitors and alerts on File Descriptor budget."""

    def __init__(self, threshold: float = 0.8) -> None:
        self.threshold = threshold

    def check(self, current: int, limit: int) -> bool:
        usage = current / limit
        if usage > self.threshold:
            logger.warning(f"FD usage critical: {current}/{limit} ({usage:.1%})")
            return False
        return True
