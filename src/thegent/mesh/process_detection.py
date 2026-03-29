"""Process enumeration and agent detection for heliosShield."""

import platform
import re
import subprocess
import os
from typing import Any


def get_processes() -> list[dict[str, Any]]:
    """Get list of running processes with PIDs and command lines.

    Uses /proc on Linux and ps on macOS.
    """
    processes = []
    system = platform.system().lower()

    if system == "linux":
        # Linux /proc scan
        for pid_str in os.listdir("/proc"):
            if not pid_str.isdigit():
                continue
            pid = int(pid_str)
            try:
                with open(f"/proc/{pid}/cmdline", "rb") as f:
                    cmdline = f.read().replace(b"\x00", b" ").decode("utf-8", errors="ignore").strip()
                if cmdline:
                    processes.append({"pid": pid, "cmd": cmdline})
            except OSError:
                continue
    elif system == "darwin":
        # macOS ps scan
        try:
            output = subprocess.check_output(["ps", "-ax", "-o", "pid,command"], stderr=subprocess.STDOUT).decode(
                "utf-8"
            )
            lines = output.splitlines()[1:]  # skip header
            for line in lines:
                line = line.strip()
                if not line:
                    continue
                match = re.match(r"^(\d+)\s+(.+)$", line)
                if match:
                    processes.append({"pid": int(match.group(1)), "cmd": match.group(2)})
        except subprocess.CalledProcessError:
            pass

    return processes


def detect_agents(patterns: dict[str, str]) -> list[dict[str, Any]]:
    """Detect known agents from running processes using regex patterns.

    Args:
        patterns: Dict of agent name -> regex pattern
    """
    compiled_patterns: list[tuple[str, re.Pattern[str]]] = []
    for name, raw_pattern in patterns.items():
        try:
            compiled_patterns.append((name, re.compile(raw_pattern, re.IGNORECASE)))
        except re.error as exc:
            raise ValueError(f"Invalid regex for agent pattern '{name}': {raw_pattern}") from exc

    processes = get_processes()
    detected: list[dict[str, Any]] = []

    for proc in processes:
        for name, pattern in compiled_patterns:
            if pattern.search(proc["cmd"]):
                detected.append({"agent": name, "pid": proc["pid"], "cmd": proc["cmd"]})
                break  # only detect one agent per PID

    return detected


if __name__ == "__main__":
    # Simple test

    # Example agent patterns
    agent_patterns = {
        "claude": r"claude-code|clode",
        "cursor": r"cursor-agent|cursor",
        "thegent": r"thegent",
        "codex": r"codex",
        "copilot": r"copilot",
    }

    found = detect_agents(agent_patterns)
    for _a in found:
        pass
