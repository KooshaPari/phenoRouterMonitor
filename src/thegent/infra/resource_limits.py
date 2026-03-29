"""Resource limits and enforcement."""

import logging
import resource

logger = logging.getLogger(__name__)


class ResourceLimits:
    """Manage and enforce resource limits."""

    DEFAULT_FD_LIMIT = 1024
    DEFAULT_PROCESS_LIMIT = 100

    def __init__(self) -> None:
        """Initialize resource limits manager."""
        self._original_limits: dict[str, tuple[int, int]] = {}
        self._set_limits()

    def _set_limits(self) -> None:
        """Set resource limits."""
        try:
            # File descriptors
            soft, hard = resource.getrlimit(resource.RLIMIT_NOFILE)
            self._original_limits["nofile"] = (soft, hard)

            # Set higher limit if possible
            new_soft = min(4096, hard)
            if new_soft > soft:
                resource.setrlimit(resource.RLIMIT_NOFILE, (new_soft, hard))
                logger.info(f"Set FD limit to {new_soft}")

            # Process count (only on POSIX systems)
            if hasattr(resource, "RLIMIT_NPROC"):
                try:
                    soft, hard = resource.getrlimit(resource.RLIMIT_NPROC)
                    self._original_limits["nproc"] = (soft, hard)

                    # Set higher limit if possible
                    new_soft = min(200, hard)
                    if new_soft > soft:
                        resource.setrlimit(resource.RLIMIT_NPROC, (new_soft, hard))
                        logger.info(f"Set process limit to {new_soft}")
                except (OSError, ValueError) as e:
                    logger.debug(f"Could not set process limit: {e}")

        except (OSError, ValueError) as e:
            logger.warning(f"Could not set resource limits: {e}")

    def get_fd_limit(self) -> int:
        """Get current FD limit.

        Returns:
            Current file descriptor limit.
        """
        try:
            return resource.getrlimit(resource.RLIMIT_NOFILE)[0]
        except Exception:
            return self.DEFAULT_FD_LIMIT

    def get_process_limit(self) -> int:
        """Get current process limit.

        Returns:
            Current process limit, or default if not available.
        """
        try:
            if hasattr(resource, "RLIMIT_NPROC"):
                return resource.getrlimit(resource.RLIMIT_NPROC)[0]
        except Exception:
            pass
        return self.DEFAULT_PROCESS_LIMIT

    def _restore_single_limit(self, limit_name: str, soft: int, hard: int) -> None:
        """Restore a single resource limit."""
        try:
            if limit_name == "nofile":
                resource.setrlimit(resource.RLIMIT_NOFILE, (soft, hard))
            elif limit_name == "nproc" and hasattr(resource, "RLIMIT_NPROC"):
                resource.setrlimit(resource.RLIMIT_NPROC, (soft, hard))
        except Exception as e:
            logger.warning(f"Could not restore {limit_name} limit: {e}")

    def restore_limits(self) -> None:
        """Restore original limits."""
        for limit_name, (soft, hard) in self._original_limits.items():
            self._restore_single_limit(limit_name, soft, hard)


# Global limits instance
_limits: ResourceLimits | None = None


def get_resource_limits() -> ResourceLimits:
    """Get global resource limits manager."""
    global _limits
    if _limits is None:
        _limits = ResourceLimits()
    return _limits
