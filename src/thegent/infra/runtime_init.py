"""Runtime infrastructure initialization and cleanup."""

import atexit
import logging
import signal
import sys
from typing import TYPE_CHECKING, Optional

if TYPE_CHECKING:
    from thegent.infra.resource_limits import ResourceLimits
    from thegent.infra.resource_monitor import ResourceMonitor, ResourceStats

logger = logging.getLogger(__name__)

# Global instances
_resource_limits: Optional["ResourceLimits"] = None
_resource_monitor: Optional["ResourceMonitor"] = None
_initialized = False


def initialize_runtime_infrastructure() -> None:
    """Initialize runtime infrastructure (resource limits and monitoring).

    This function:
    1. Sets up resource limits (FD, process count)
    2. Starts resource monitoring in background thread
    3. Registers cleanup handlers for graceful shutdown

    Safe to call multiple times (idempotent).
    """
    global _resource_limits, _resource_monitor, _initialized

    if _initialized:
        logger.debug("Runtime infrastructure already initialized")
        return

    try:
        # Import here to avoid circular imports
        from thegent.infra.resource_limits import get_resource_limits
        from thegent.infra.resource_monitor import get_resource_monitor

        # Initialize resource limits (sets higher limits if possible)
        _resource_limits = get_resource_limits()
        logger.debug("Resource limits initialized")

        # Initialize and start resource monitor
        _resource_monitor = get_resource_monitor()
        _resource_monitor.start()
        logger.debug("Resource monitor started")

        # Register cleanup handlers
        atexit.register(_cleanup_runtime_infrastructure)
        signal.signal(signal.SIGTERM, _signal_handler)
        signal.signal(signal.SIGINT, _signal_handler)

        # On Windows, also handle SIGBREAK
        if sys.platform == "win32":
            signal.signal(signal.SIGBREAK, _signal_handler)

        _initialized = True
        logger.info("Runtime infrastructure initialized successfully")

    except Exception as e:
        logger.warning(f"Failed to initialize runtime infrastructure: {e}", exc_info=True)
        # Don't raise - allow application to continue even if monitoring fails


def _cleanup_runtime_infrastructure() -> None:
    """Cleanup runtime infrastructure on exit."""
    global _resource_limits, _resource_monitor, _initialized

    if not _initialized:
        return

    try:
        if _resource_monitor:
            _resource_monitor.stop()
            logger.debug("Resource monitor stopped")

        if _resource_limits:
            restore_fn = getattr(_resource_limits, "restore", None)
            reset_fn = getattr(_resource_limits, "reset", None)
            if restore_fn is not None:
                restore_fn()
                logger.debug("Resource limits restored")
            elif reset_fn is not None:
                reset_fn()
                logger.debug("Resource limits reset")

        _initialized = False
        logger.info("Runtime infrastructure cleaned up")

    except Exception as e:
        logger.error(f"Error during runtime infrastructure cleanup: {e}", exc_info=True)


def _signal_handler(signum: int, frame) -> None:
    """Handle termination signals."""
    logger.info(f"Received signal {signum}, cleaning up runtime infrastructure")
    _cleanup_runtime_infrastructure()
    # Re-raise signal to allow normal signal handling
    signal.signal(signum, signal.SIG_DFL)
    sys.exit(0)


def get_resource_stats() -> Optional["ResourceStats"]:
    """Get current resource statistics.

    Returns:
        ResourceStats if monitoring is active, None otherwise.
    """
    global _resource_monitor
    if _resource_monitor:
        return _resource_monitor.get_stats()
    return None


def is_initialized() -> bool:
    """Check if runtime infrastructure is initialized."""
    return _initialized
