"""Ghostty terminal emulator integration for thegent.

Provides detection, configuration management, and feature access for the
Ghostty terminal emulator (https://ghostty.org).

FR traceability: FR-IDE-002 (Ghostty terminal integration)
"""

from __future__ import annotations

import logging
import os
import subprocess
from thegent.infra.shim_subprocess import run as shim_run
from dataclasses import dataclass, field
from pathlib import Path

logger = logging.getLogger(__name__)

# ---------------------------------------------------------------------------
# Exceptions
# ---------------------------------------------------------------------------


class GhosttyError(Exception):
    """Raised when a Ghostty operation fails in an unrecoverable way."""


# ---------------------------------------------------------------------------
# Data types
# ---------------------------------------------------------------------------


@dataclass
class GhosttyConfig:
    """Configuration record for a Ghostty terminal installation.

    Attributes:
        socket_path: Path to the Ghostty IPC socket, or None if not configured.
        theme:       Active color theme name (default: "dark").
        font_size:   Font size in points (default: 14).
        raw:         Raw key->value pairs parsed from the config file.
                     Excluded from equality checks and repr.
    """

    socket_path: str | None = None
    theme: str = "dark"
    font_size: int = 14

    # Internal: raw key->value pairs parsed from the config file.
    # Named without leading underscore so external code (e.g. get_config)
    # can access it without triggering SLF001.
    raw: dict[str, str] = field(default_factory=dict, init=False, repr=False, compare=False)


# ---------------------------------------------------------------------------
# Config file helpers
# ---------------------------------------------------------------------------

_DEFAULT_CONFIG_PATH = Path.home() / ".config" / "ghostty" / "config"

# Key names as they appear in the Ghostty config file.
_KEY_THEME = "theme"
_KEY_FONT_SIZE = "font-size"
_KEY_SOCKET_PATH = "socket-path"


def _parse_config_file(path: Path) -> dict[str, str]:
    """Parse a Ghostty config file into a key->value dict.

    Lines with ``#`` as the first non-whitespace character are treated as
    comments.  Each setting line has the form ``key = value`` (spaces around
    ``=`` are optional).  Only the first occurrence of each key is kept.

    Args:
        path: Absolute path to the Ghostty config file.

    Returns:
        Mapping of config key to value string.  Empty dict on read failure.
    """
    result: dict[str, str] = {}
    try:
        text = path.read_text(encoding="utf-8")
    except OSError as exc:
        logger.debug("Cannot read Ghostty config at %s: %s", path, exc)
        return result

    for line in text.splitlines():
        stripped = line.strip()
        if not stripped or stripped.startswith("#"):
            continue
        if "=" not in stripped:
            continue
        key, _, value = stripped.partition("=")
        key = key.strip()
        value = value.strip()
        if key and key not in result:
            result[key] = value

    return result


def _write_config_key(path: Path, key: str, value: str) -> None:
    """Write or update a single key in the Ghostty config file.

    If the file already contains the key the line is replaced.  Otherwise
    the key is appended.  The parent directory is created if needed.

    Args:
        path:  Path to the config file.
        key:   Config key to set (e.g. "theme").
        value: Value string to assign.

    Raises:
        OSError: If the file cannot be read or written.
    """
    path.parent.mkdir(parents=True, exist_ok=True)

    existing_lines: list[str] = []
    if path.exists():
        existing_lines = path.read_text(encoding="utf-8").splitlines(keepends=True)

    new_line = f"{key} = {value}\n"
    replaced = False
    output_lines: list[str] = []

    for line in existing_lines:
        stripped = line.strip()
        if not stripped.startswith("#") and "=" in stripped:
            line_key = stripped.partition("=")[0].strip()
            if line_key == key:
                output_lines.append(new_line)
                replaced = True
                continue
        output_lines.append(line)

    if not replaced:
        output_lines.append(new_line)

    path.write_text("".join(output_lines), encoding="utf-8")


# ---------------------------------------------------------------------------
# Subprocess helpers (extracted to avoid S603/S607 in loops)
# ---------------------------------------------------------------------------

_GHOSTTY_OPEN_TAB_CMD = "ghostty"
_OSASCRIPT_CMD = "osascript"


def _run_ghostty_open_tab(command: str | None) -> subprocess.CompletedProcess[str]:
    """Run ``ghostty +open-tab [-- command]`` and return the result.

    Extracted from GhosttyIntegration.open_tab to keep the public method
    within the 40-line limit.

    Args:
        command: Optional shell command to pass after ``--``.

    Returns:
        CompletedProcess with returncode, stdout, stderr.
    """
    cmd: list[str] = [_GHOSTTY_OPEN_TAB_CMD, "+open-tab"]
    if command:
        cmd += ["--", command]
    return shim_run(cmd, check=False, capture_output=True, text=True, timeout=10)


def _run_osascript_notification(title: str, body: str) -> subprocess.CompletedProcess[str]:
    """Run ``osascript`` to display a macOS notification.

    Extracted from GhosttyIntegration.send_notification to keep the public
    method within the 40-line limit.

    Args:
        title: AppleScript-escaped notification title.
        body:  AppleScript-escaped notification body.

    Returns:
        CompletedProcess with returncode, stdout, stderr.
    """
    script = f'display notification "{body}" with title "{title}"'
    return shim_run(
        [_OSASCRIPT_CMD, "-e", script],
        check=False,
        capture_output=True,
        text=True,
        timeout=10,
    )


# ---------------------------------------------------------------------------
# Main integration class
# ---------------------------------------------------------------------------


class GhosttyIntegration:
    """Detect Ghostty and provide access to its terminal features.

    Usage::

        integration = GhosttyIntegration()
        if integration.is_available():
            cfg = integration.get_config()
            integration.set_theme("light")
            integration.send_notification("thegent", "Task complete")
    """

    def __init__(self, config_path: Path | None = None) -> None:
        """Initialise the integration.

        Args:
            config_path: Override the default Ghostty config path
                         (~/.config/ghostty/config).  Useful for tests.
        """
        self._config_path: Path = config_path or _DEFAULT_CONFIG_PATH

    # ------------------------------------------------------------------
    # Detection
    # ------------------------------------------------------------------

    def is_available(self) -> bool:
        """Return True when the current process is running inside Ghostty.

        Detection is based on the ``TERM_PROGRAM`` environment variable being
        set to ``"ghostty"``.

        Returns:
            True if ``TERM_PROGRAM == "ghostty"``; False otherwise.
        """
        return os.environ.get("TERM_PROGRAM", "").lower() == "ghostty"

    # ------------------------------------------------------------------
    # Configuration
    # ------------------------------------------------------------------

    def get_config(self) -> GhosttyConfig:
        """Read the Ghostty configuration file and return a GhosttyConfig.

        If the config file does not exist or cannot be parsed the returned
        GhosttyConfig contains default values.

        Returns:
            GhosttyConfig populated from ``~/.config/ghostty/config``
            (or the custom path provided at construction time).
        """
        raw = _parse_config_file(self._config_path)
        cfg = GhosttyConfig()
        cfg.raw = raw

        if _KEY_THEME in raw:
            cfg.theme = raw[_KEY_THEME]

        if _KEY_FONT_SIZE in raw:
            try:
                cfg.font_size = int(raw[_KEY_FONT_SIZE])
            except ValueError:
                logger.debug("Invalid font-size value in Ghostty config: %r", raw[_KEY_FONT_SIZE])

        if _KEY_SOCKET_PATH in raw:
            cfg.socket_path = raw[_KEY_SOCKET_PATH] or None

        return cfg

    def set_theme(self, theme: str) -> bool:
        """Write the theme setting to the Ghostty config file.

        Args:
            theme: Theme name to set (e.g. ``"dark"``, ``"light"``,
                   ``"Dracula"``).

        Returns:
            True on success; False if the write failed.
        """
        if not theme:
            logger.warning("set_theme called with empty theme string; ignoring")
            return False

        try:
            _write_config_key(self._config_path, _KEY_THEME, theme)
            logger.debug("Ghostty theme set to %r in %s", theme, self._config_path)
            return True
        except OSError as exc:
            logger.error("Failed to write Ghostty theme to %s: %s", self._config_path, exc)
            return False

    # ------------------------------------------------------------------
    # Actions
    # ------------------------------------------------------------------

    def open_tab(self, command: str | None = None) -> bool:
        """Open a new tab in the current Ghostty window.

        Uses the ``ghostty +open-tab`` CLI command.  The Ghostty binary must
        be on ``PATH``.

        Args:
            command: Optional shell command to run in the new tab.
                     If None, the default shell is used.

        Returns:
            True if the command exited successfully; False otherwise.
        """
        try:
            result = _run_ghostty_open_tab(command)
            if result.returncode != 0:
                logger.debug(
                    "ghostty +open-tab exited %d: %s",
                    result.returncode,
                    result.stderr.strip(),
                )
                return False
            return True
        except FileNotFoundError:
            logger.debug("'ghostty' binary not found on PATH; cannot open tab")
            return False
        except subprocess.TimeoutExpired:
            logger.warning("ghostty +open-tab timed out")
            return False
        except OSError as exc:
            logger.error("Error running ghostty +open-tab: %s", exc)
            return False

    def send_notification(self, title: str, body: str) -> bool:
        """Send a macOS desktop notification via osascript.

        Uses ``osascript`` (AppleScript) which is available on macOS.
        Returns False on non-macOS platforms or when osascript is unavailable.

        Args:
            title: Notification title.
            body:  Notification body text.

        Returns:
            True if the notification was delivered; False otherwise.
        """
        safe_title = title.replace('"', '\\"')
        safe_body = body.replace('"', '\\"')

        try:
            result = _run_osascript_notification(safe_title, safe_body)
            if result.returncode != 0:
                logger.debug(
                    "osascript notification failed (exit %d): %s",
                    result.returncode,
                    result.stderr.strip(),
                )
                return False
            return True
        except FileNotFoundError:
            logger.debug("'osascript' not found; notifications unavailable on this platform")
            return False
        except subprocess.TimeoutExpired:
            logger.warning("osascript timed out sending notification")
            return False
        except OSError as exc:
            logger.error("Error running osascript: %s", exc)
            return False

    # ------------------------------------------------------------------
    # Environment introspection
    # ------------------------------------------------------------------

    def get_env_info(self) -> dict[str, str]:
        """Return a dict of terminal-related environment variables.

        The following variables are included (value is empty string when not
        set in the current environment):

        - ``TERM_PROGRAM``
        - ``TERM``
        - ``COLORTERM``
        - ``TERM_PROGRAM_VERSION``
        - ``GHOSTTY_RESOURCES_DIR``
        - ``GHOSTTY_BIN_DIR``

        Returns:
            Mapping of env-var name to its current value (or empty string).
        """
        keys = (
            "TERM_PROGRAM",
            "TERM",
            "COLORTERM",
            "TERM_PROGRAM_VERSION",
            "GHOSTTY_RESOURCES_DIR",
            "GHOSTTY_BIN_DIR",
        )
        return {k: os.environ.get(k, "") for k in keys}
