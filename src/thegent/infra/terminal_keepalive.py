"""Terminal keepalive mechanism to prevent timeout in long-running commands.

Detects the calling terminal and sends keepalive input (Enter key) periodically
to prevent timeout (e.g., Cursor's 4-minute guard).

This module provides a robust keepalive mechanism that:
- Detects parent terminal processes using multiple methods
- Sends keepalive signals via stdin or tmux
- Handles errors gracefully without affecting the main process
- Provides detailed logging for debugging
"""

from typing import Any

import logging
import os
import sys
import threading
import time

from thegent.infra.shim_subprocess import run as shim_run

logger = logging.getLogger(__name__)

try:
    import psutil

    PSUTIL_AVAILABLE = True
except ImportError:
    PSUTIL_AVAILABLE = False
    logger.debug("psutil not available, terminal detection will be limited")


def _get_parent_terminal_info() -> dict[str, Any] | None:
    """Get information about the parent terminal process.

    Uses multiple detection methods for robustness:
    1. Direct parent PID inspection
    2. Process tree traversal
    3. Environment variable detection
    4. TTY detection

    Returns:
        Dict with 'pid', 'name', 'is_cursor', 'is_ide', 'tty', 'stdin_tty',
        'detection_method', or None if not detectable
    """
    info = {
        "pid": None,
        "name": None,
        "is_cursor": False,
        "is_ide": False,
        "tty": None,
        "stdin_tty": False,
        "detection_method": None,
        "process": None,
    }

    # Method 1: Check stdin TTY
    try:
        info["stdin_tty"] = sys.stdin.isatty() if sys.stdin else False
    except (AttributeError, OSError):
        info["stdin_tty"] = False

    # Method 2: Environment variable detection (fastest, most reliable)
    cursor_env_vars = ["CURSOR_SANDBOX", "CURSOR_ASKPASS", "CURSOR_AGENT"]
    if any(os.environ.get(var) for var in cursor_env_vars):
        info["is_cursor"] = True
        info["is_ide"] = True
        info["detection_method"] = "environment"
        logger.debug("Detected Cursor via environment variables")
        return info

    # Method 3: Process inspection (requires psutil)
    if not PSUTIL_AVAILABLE:
        logger.debug("psutil not available, skipping process detection")
        return info if info["stdin_tty"] else None

    try:
        ppid = os.getppid()
        if ppid <= 1:
            # Parent is init/systemd, not a terminal
            return None

        parent = psutil.Process(ppid)
        info["pid"] = ppid
        info["process"] = parent

        # Get process name
        try:
            name = parent.name().lower()
            info["name"] = name
        except (psutil.AccessDenied, psutil.NoSuchProcess):
            name = None

        # Check if it's Cursor or similar IDE terminal
        ide_keywords = ["cursor", "code", "vscode", "claude", "clode", "cursor-agent"]
        if name:
            info["is_cursor"] = any(keyword in name for keyword in ide_keywords)
            info["is_ide"] = info["is_cursor"] or any(kw in name for kw in ["code", "vscode", "idea", "pycharm"])

        # Try to get TTY from process connections
        try:
            connections = parent.connections()
            for conn in connections:
                if hasattr(conn, "type") and conn.type == 1:  # socket
                    if hasattr(conn, "laddr") and conn.laddr:
                        info["tty"] = str(conn.laddr)
                        break
        except (psutil.AccessDenied, AttributeError, psutil.NoSuchProcess):
            pass

        # Method 4: Check process tree for IDE processes
        if not info["is_ide"] and name:
            try:
                # Walk up process tree to find IDE
                current = parent
                for _ in range(5):  # Check up to 5 levels
                    try:
                        parent_name = current.name().lower()
                        if any(kw in parent_name for kw in ide_keywords):
                            info["is_cursor"] = "cursor" in parent_name or "claude" in parent_name
                            info["is_ide"] = True
                            info["detection_method"] = "process_tree"
                            logger.debug(f"Detected IDE in process tree: {parent_name}")
                            break
                        current = current.parent()
                        if not current or current.pid == 1:
                            break
                    except (psutil.NoSuchProcess, psutil.AccessDenied):
                        break
            except Exception as e:
                logger.debug(f"Error traversing process tree: {e}")

        if not info["detection_method"]:
            info["detection_method"] = "direct_parent"

        return info

    except (psutil.NoSuchProcess, psutil.AccessDenied, AttributeError, OSError) as e:
        logger.debug(f"Error getting parent terminal info: {e}")
        return info if info["stdin_tty"] else None


def _send_keepalive_to_stdin() -> bool:
    """Send keepalive input (Enter) to stdin.

    Uses multiple methods for robustness:
    1. Direct write to sys.stdin
    2. Write to /dev/tty if available
    3. Use termios if available (Unix)

    Returns:
        True if successful, False otherwise
    """
    if not sys.stdin:
        return False

    # Method 1: Check if stdin is a TTY
    try:
        if not sys.stdin.isatty():
            return False
    except (OSError, AttributeError):
        return False

    # Method 2: Try direct write to stdin
    try:
        if hasattr(sys.stdin, "write") and hasattr(sys.stdin, "flush"):
            sys.stdin.write("\n")
            sys.stdin.flush()
            return True
    except (OSError, AttributeError, ValueError) as e:
        logger.debug(f"Failed to write to stdin: {e}")

    # Method 3: Try writing to /dev/tty (Unix fallback)
    if sys.platform != "win32":
        try:
            with open("/dev/tty", "w") as tty:
                tty.write("\n")
                tty.flush()
            return True
        except (OSError, PermissionError, FileNotFoundError):
            pass

    return False


def _send_keepalive_via_tmux() -> bool:
    """Try to send keepalive via tmux if we're in a tmux session.

    Supports multiple tmux detection methods:
    1. TMUX_PANE environment variable
    2. TMUX environment variable (extract pane)
    3. Detect tmux socket from process tree

    Returns:
        True if successful, False otherwise
    """
    import subprocess

    # Method 1: Use TMUX_PANE environment variable
    tmux_pane = os.environ.get("TMUX_PANE")
    if not tmux_pane:
        # Method 2: Try to extract from TMUX variable
        tmux_var = os.environ.get("TMUX")
        if tmux_var:
            # TMUX format: /path/to/socket,pid,session_id
            # We can use session_id or try to detect current pane
            try:
                # Try to get current pane from tmux
                result = shim_run(
                    ["tmux", "display-message", "-p", "#{pane_id}"],
                    capture_output=True,
                    text=True,
                    timeout=0.5,
                    check=False,
                )
                if result.returncode == 0:
                    tmux_pane = result.stdout.strip()
            except Exception:
                pass

    if not tmux_pane:
        return False

    # Send Enter key to tmux pane
    try:
        result = shim_run(
            ["tmux", "send-keys", "-t", tmux_pane, "C-m"],
            capture_output=True,
            timeout=1.0,
            check=False,  # Don't raise on error
        )
        if result.returncode == 0:
            return True
        logger.debug(f"tmux send-keys failed with return code {result.returncode}")
    except subprocess.TimeoutExpired:
        logger.debug("tmux send-keys timed out")
    except FileNotFoundError:
        logger.debug("tmux command not found")
    except Exception as e:
        logger.debug(f"Error sending keepalive via tmux: {e}")

    return False


class TerminalKeepalive:
    """Manages keepalive mechanism for long-running commands.

    Thread-safe, robust keepalive that handles errors gracefully and provides
    detailed logging for debugging. Automatically detects the best keepalive
    method based on the environment.
    """

    def __init__(
        self,
        interval: float = 180.0,  # 3 minutes (under 4min timeout)
        enabled: bool = True,
        max_failures: int = 3,
    ) -> None:
        """Initialize keepalive.

        Args:
            interval: Seconds between keepalive signals (default: 180s)
            enabled: Whether keepalive is enabled
            max_failures: Maximum consecutive failures before disabling (default: 3)
        """
        self.interval = max(30.0, interval)  # Minimum 30 seconds
        self.enabled = enabled
        self.max_failures = max_failures
        self._thread: threading.Thread | None = None
        self._stop_event = threading.Event()
        self._lock = threading.Lock()
        self._failure_count = 0
        self._success_count = 0
        self._parent_info = _get_parent_terminal_info()
        self._last_success_time: float | None = None

    def should_enable(self) -> bool:
        """Check if keepalive should be enabled based on environment.

        Uses multiple detection methods for robustness:
        1. Environment variable detection (fastest)
        2. Process inspection (most accurate)
        3. TTY detection (fallback)

        Returns:
            True if keepalive should be enabled, False otherwise
        """
        if not self.enabled:
            logger.debug("Keepalive disabled by user")
            return False

        # Method 1: Check for Cursor-specific environment variables (fastest)
        cursor_env_vars = ["CURSOR_SANDBOX", "CURSOR_ASKPASS", "CURSOR_AGENT"]
        if any(os.environ.get(var) for var in cursor_env_vars):
            logger.debug("Keepalive enabled: Cursor environment detected")
            return True

        # Method 2: Check if we're in an interactive terminal
        try:
            if not sys.stdin or not sys.stdin.isatty():
                logger.debug("Keepalive disabled: not in interactive terminal")
                return False
        except (OSError, AttributeError):
            logger.debug("Keepalive disabled: cannot check stdin")
            return False

        # Method 3: Check parent process info
        if self._parent_info:
            if self._parent_info.get("is_cursor") or self._parent_info.get("is_ide"):
                logger.debug(f"Keepalive enabled: IDE detected ({self._parent_info.get('name', 'unknown')})")
                return True

            if self._parent_info.get("stdin_tty"):
                # We have a TTY, might be useful
                logger.debug("Keepalive enabled: TTY detected")
                return True

        # Method 4: Check if we're in tmux (might be Cursor's terminal)
        if os.environ.get("TMUX"):
            logger.debug("Keepalive enabled: tmux session detected")
            return True

        logger.debug("Keepalive disabled: no suitable environment detected")
        return False

    def _keepalive_loop(self) -> None:
        """Background thread that sends keepalive signals.

        Tries multiple methods in order of preference:
        1. Stdin (most direct)
        2. Tmux (if in tmux session)

        Handles errors gracefully and tracks success/failure rates.
        Automatically disables if too many failures occur.
        """
        logger.debug(f"Keepalive thread started (interval={self.interval}s)")

        while not self._stop_event.wait(self.interval):
            if self._stop_event.is_set():
                break

            success = False
            method_used = None

            # Method 1: Try stdin first (most direct)
            try:
                if _send_keepalive_to_stdin():
                    success = True
                    method_used = "stdin"
            except Exception as e:
                logger.debug(f"Error sending keepalive via stdin: {e}")

            # Method 2: Try tmux if stdin failed
            if not success:
                try:
                    if _send_keepalive_via_tmux():
                        success = True
                        method_used = "tmux"
                except Exception as e:
                    logger.debug(f"Error sending keepalive via tmux: {e}")

            # Update statistics
            with self._lock:
                if success:
                    self._success_count += 1
                    self._failure_count = 0
                    self._last_success_time = time.time()

                    # Debug logging
                    from thegent.config import ThegentSettings

                    if ThegentSettings().debug_keepalive:
                        logger.info(f"Keepalive sent via {method_used} at {time.time():.2f}")
                else:
                    self._failure_count += 1
                    self._success_count = 0

                    # Disable if too many failures
                    if self._failure_count >= self.max_failures:
                        logger.warning(
                            f"Keepalive disabled after {self._failure_count} consecutive failures. "
                            "This is normal if not in an interactive terminal."
                        )
                        break

        logger.debug("Keepalive thread stopped")

    def start(self) -> bool:
        """Start keepalive thread.

        Thread-safe: can be called multiple times safely.

        Returns:
            True if started, False otherwise
        """
        with self._lock:
            if not self.should_enable():
                logger.debug("Keepalive not started: should_enable() returned False")
                return False

            if self._thread and self._thread.is_alive():
                logger.debug("Keepalive already running")
                return True  # Already running

            # Reset state
            self._stop_event.clear()
            self._failure_count = 0
            self._success_count = 0
            self._last_success_time = None

            # Create and start thread
            try:
                self._thread = threading.Thread(target=self._keepalive_loop, daemon=True, name="TerminalKeepalive")
                self._thread.start()
                logger.debug(f"Keepalive thread started (interval={self.interval}s)")
                return True
            except Exception as e:
                logger.error(f"Failed to start keepalive thread: {e}")
                return False

    def stop(self) -> None:
        """Stop keepalive thread.

        Thread-safe: can be called multiple times safely.
        Waits up to 2 seconds for thread to finish.
        """
        with self._lock:
            if not self._thread or not self._thread.is_alive():
                return

            self._stop_event.set()

            # Wait for thread to finish (with timeout)
            try:
                self._thread.join(timeout=2.0)
                if self._thread.is_alive():
                    logger.warning("Keepalive thread did not stop within timeout")
                else:
                    logger.debug("Keepalive thread stopped")
            except Exception as e:
                logger.error(f"Error stopping keepalive thread: {e}")

    def get_stats(self) -> dict[str, Any]:
        """Get keepalive statistics.

        Returns:
            Dict with success_count, failure_count, last_success_time, is_running
        """
        with self._lock:
            return {
                "success_count": self._success_count,
                "failure_count": self._failure_count,
                "last_success_time": self._last_success_time,
                "is_running": self._thread is not None and self._thread.is_alive(),
                "interval": self.interval,
                "parent_info": self._parent_info,
            }

    def __enter__(self):
        """Context manager entry."""
        self.start()
        return self

    def __exit__(self, exc_type, _exc_val, _exc_tb):
        """Context manager exit."""
        self.stop()


def create_keepalive(interval: float = 180.0, enabled: bool = True, max_failures: int = 3) -> TerminalKeepalive:
    """Create a keepalive instance.

    Factory function for creating TerminalKeepalive instances with
    sensible defaults.

    Args:
        interval: Seconds between keepalive signals (default: 180s, min: 30s)
        enabled: Whether keepalive is enabled (default: True)
        max_failures: Max consecutive failures before disabling (default: 3)

    Returns:
        TerminalKeepalive instance

    Example:
        >>> keepalive = create_keepalive(interval=120.0)
        >>> if keepalive.start():
        ...     try:
        ...         # Long-running operation
        ...         pass
        ...     finally:
        ...         keepalive.stop()
    """
    return TerminalKeepalive(interval=interval, enabled=enabled, max_failures=max_failures)
