"""Resource monitoring and leak detection using psutil."""

import logging
import resource
import threading
import time
from dataclasses import dataclass

import psutil

logger = logging.getLogger(__name__)


@dataclass
class ResourceStats:
    """Resource usage statistics."""

    fd_count: int
    fd_limit: int
    fd_usage_percent: float
    process_count: int
    memory_mb: float
    cpu_percent: float
    timestamp: float

    def is_critical(self) -> bool:
        """Check if resource usage is critical.

        Critical thresholds:
        - FD usage > 80% (file descriptor exhaustion risk)
        - Process count > 500 (very high, may indicate leak)
        - Memory > 2048MB (2GB) for this process
        """
        return (
            self.fd_usage_percent > 80.0
            or self.process_count > 500  # Increased from 100 - 611 processes may be normal for dev systems
            or self.memory_mb > 2048  # 2GB
        )

    def get_suspicion_level(self) -> tuple[str, str]:
        """Get suspicion level and optimization suggestions.

        Returns (level, suggestions) where level is one of:
        - "low": Normal usage, no concerns
        - "medium": Elevated usage, monitor
        - "high": High usage, investigate
        - "critical": Critical usage, immediate action needed
        """
        issues = []
        suggestions = []

        if self.fd_usage_percent > 80.0:
            issues.append("FD usage critical")
            suggestions.append("Check for file descriptor leaks: lsof | wc -l")
            return ("critical", "; ".join(suggestions))
        if self.fd_usage_percent > 60.0:
            issues.append("FD usage elevated")
            suggestions.append("Monitor FD usage: watch -n 1 'lsof | wc -l'")

        if self.process_count > 500:
            issues.append("Very high process count")
            suggestions.append("Check for process leaks: ps aux | wc -l")
            return ("high", "; ".join(suggestions))
        if self.process_count > 300:
            issues.append("Elevated process count")
            suggestions.append("Monitor process count: watch -n 5 'ps aux | wc -l'")

        if self.memory_mb > 2048:
            issues.append("Memory usage high")
            suggestions.append("Check memory usage: ps aux --sort=-%mem | head -10")
            return ("critical", "; ".join(suggestions))
        if self.memory_mb > 1024:
            issues.append("Memory usage elevated")
            suggestions.append("Monitor memory: top -o mem")

        if not issues:
            return ("low", "Resource usage is normal")
        if len(issues) == 1:
            return ("medium", "; ".join(suggestions))
        return ("high", "; ".join(suggestions))


class ResourceMonitor:
    """Monitor system resources using psutil and detect leaks."""

    def __init__(self, check_interval: float = 60.0) -> None:
        """Initialize resource monitor.

        Args:
            check_interval: Interval in seconds between monitoring checks.
        """
        self.check_interval = check_interval
        self._running = False
        self._thread: threading.Thread | None = None
        self._stats_history: list[ResourceStats] = []
        self._max_history = 100

    def start(self) -> None:
        """Start monitoring thread."""
        if self._running:
            return

        self._running = True
        self._thread = threading.Thread(target=self._monitor_loop, daemon=True)
        self._thread.start()
        logger.info("Resource monitor started")

    def stop(self) -> None:
        """Stop monitoring thread."""
        self._running = False
        if self._thread:
            self._thread.join(timeout=5.0)
        logger.info("Resource monitor stopped")

    def _monitor_loop(self) -> None:
        """Monitoring loop."""
        while self._running:
            try:
                stats = self.get_stats()
                self._stats_history.append(stats)
                if len(self._stats_history) > self._max_history:
                    self._stats_history.pop(0)

                # Only warn if usage is actually high (>80% FD or >2GB memory)
                # Don't warn just for high process count (normal on dev systems)
                if stats.fd_usage_percent > 80.0 or stats.memory_mb > 2048:
                    logger.warning(
                        f"Critical resource usage detected: "
                        f"FDs: {stats.fd_count}/{stats.fd_limit} ({stats.fd_usage_percent:.1f}%), "
                        f"Processes: {stats.process_count}, "
                        f"Memory: {stats.memory_mb:.1f}MB"
                    )
                elif stats.fd_usage_percent > 0.1:  # Only log if there's actual usage
                    logger.debug(
                        f"Resource usage: "
                        f"FDs: {stats.fd_count}/{stats.fd_limit} ({stats.fd_usage_percent:.1f}%), "
                        f"Processes: {stats.process_count}, "
                        f"Memory: {stats.memory_mb:.1f}MB"
                    )

            except Exception as e:
                logger.error(f"Error in resource monitor: {e}")

            time.sleep(self.check_interval)

    def get_stats(self) -> ResourceStats:
        """Get current resource statistics using psutil."""
        process = psutil.Process()

        # File descriptors using psutil
        try:
            # Try psutil's num_fds() first (Linux, macOS)
            if hasattr(process, "num_fds"):
                fd_count = process.num_fds()
            else:
                # Fallback: count open files and connections
                fd_count = len(process.open_files()) + len(process.net_connections())
        except (psutil.AccessDenied, AttributeError, psutil.NoSuchProcess):
            fd_count = 0

        # FD limit using resource module
        try:
            fd_limit = resource.getrlimit(resource.RLIMIT_NOFILE)[0]
        except Exception:
            fd_limit = 1024

        fd_usage_percent = (fd_count / fd_limit * 100) if fd_limit > 0 else 0

        # Process count via psutil
        process_count = len(psutil.pids())

        # Memory using psutil (RSS)
        try:
            memory_mb = process.memory_info().rss / 1024 / 1024
        except (psutil.AccessDenied, psutil.NoSuchProcess):
            memory_mb = 0

        # CPU using psutil
        try:
            cpu_percent = process.cpu_percent(interval=0.1)
        except (psutil.AccessDenied, psutil.NoSuchProcess):
            cpu_percent = 0

        return ResourceStats(
            fd_count=fd_count,
            fd_limit=fd_limit,
            fd_usage_percent=fd_usage_percent,
            process_count=process_count,
            memory_mb=memory_mb,
            cpu_percent=cpu_percent,
            timestamp=time.time(),
        )

    def get_process_info(self, pid: int) -> dict | None:
        """Get detailed process information using psutil.

        Args:
            pid: Process ID.

        Returns:
            Dictionary with process information, or None if process not found.
        """
        try:
            proc = psutil.Process(pid)
            info = {
                "pid": proc.pid,
                "name": proc.name(),
                "status": proc.status(),
                "memory_mb": proc.memory_info().rss / 1024 / 1024,
                "cpu_percent": proc.cpu_percent(interval=0.1),
                "num_threads": proc.num_threads(),
                "open_files": len(proc.open_files()),
                "connections": len(proc.net_connections()),
            }

            # Add num_fds if available (Linux, macOS)
            if hasattr(proc, "num_fds"):
                info["num_fds"] = proc.num_fds()

            return info
        except (psutil.NoSuchProcess, psutil.AccessDenied):
            return None

    def get_history(self) -> list[ResourceStats]:
        """Get resource usage history."""
        return self._stats_history.copy()

    def detect_leak(self) -> str | None:
        """Detect potential resource leaks from history.

        Returns:
            Description of detected leak, or None if no leak detected.
        """
        if len(self._stats_history) < 10:
            return None

        recent = self._stats_history[-10:]

        # Check for increasing FD count
        fd_trend = [s.fd_count for s in recent]
        if all(fd_trend[i] < fd_trend[i + 1] for i in range(len(fd_trend) - 1)):
            return "File descriptor leak detected (increasing trend)"

        # Check for increasing process count
        proc_trend = [s.process_count for s in recent]
        if all(proc_trend[i] < proc_trend[i + 1] for i in range(len(proc_trend) - 1)):
            return "Process leak detected (increasing trend)"

        # Check for increasing memory
        mem_trend = [s.memory_mb for s in recent]
        if all(mem_trend[i] < mem_trend[i + 1] for i in range(len(mem_trend) - 1)):
            return "Memory leak detected (increasing trend)"

        return None


# Global monitor instance
_monitor: ResourceMonitor | None = None


def get_resource_monitor() -> ResourceMonitor:
    """Get global resource monitor."""
    global _monitor
    if _monitor is None:
        _monitor = ResourceMonitor()
    return _monitor
