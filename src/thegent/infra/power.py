"""Power management utilities (macOS sleep prevention)."""

import platform
import shutil


def wrap_with_caffeinate(cmd: list[str], agent_name: str | None = None) -> list[str]:
    """
    Wrap command with caffeinate on macOS to keep Mac awake during long-running tasks.

    Args:
        cmd: Command to wrap
        agent_name: Name of the agent (claude, codex, etc.) to check against config.
                   If None, it wraps regardless of agent name if mac_keep_awake is True.
    """
    if platform.system() != "Darwin":
        return cmd

    try:
        from thegent.config import ThegentSettings

        settings = ThegentSettings()
    except ImportError:
        # Fallback if config is not available in current context
        return cmd

    if not settings.mac_keep_awake:
        return cmd

    # If agent_name is provided, check if it's in the allowed list
    if agent_name:
        allowed_agents = [a.lower() for a in settings.mac_keep_awake_agents]
        if agent_name.lower() not in allowed_agents:
            return cmd

    caffeinate = shutil.which("caffeinate")
    if not caffeinate:
        return cmd

    # -i: prevent idle sleep
    # -s: prevent system sleep
    # -d: prevent display sleep (optional, but good for interactive sessions)
    # Using -i and -s by default as per existing implementation in direct_agents.py
    return [caffeinate, "-i", "-s", "--", *cmd]


def is_mac_sleep_prevention_enabled() -> bool:
    """Check if macOS sleep prevention is enabled in settings."""
    if platform.system() != "Darwin":
        return False

    try:
        from thegent.config import ThegentSettings

        return ThegentSettings().mac_keep_awake
    except ImportError:
        return False
