"""Phase 13: Shell Injection implementation.
Includes tmux session detection, command injection via send-keys, and readiness detection.
"""

import logging
import re
import subprocess
from thegent.infra.shim_subprocess import run as shim_run
import time

logger = logging.getLogger(__name__)


class TmuxInjector:
    """Injects commands into tmux sessions."""

    def __init__(self, session_prefix: str = "mesh-") -> None:
        self.session_prefix = session_prefix

    def list_agent_sessions(self) -> list[str]:
        """List all tmux sessions matching agent prefix."""
        try:
            result = shim_run(["tmux", "ls", "-F", "#S"], capture_output=True, text=True, check=False)
            if result.returncode != 0:
                return []
            sessions = result.stdout.splitlines()
            return [s for s in sessions if s.startswith(self.session_prefix)]
        except FileNotFoundError:
            return []

    def inject_command(self, session_id: str, command: str, wait_for_readiness: bool = True) -> bool:
        """Inject command into tmux session using send-keys."""
        if wait_for_readiness:
            if not self.wait_for_ready(session_id):
                logger.warning(f"Session {session_id} not ready, but injecting anyway.")

        # send-keys -l for literal string, then Enter
        try:
            shim_run(["tmux", "send-keys", "-t", session_id, command, "C-m"], check=True)
            logger.info(f"Injected command into {session_id}: {command}")
            return True
        except subprocess.CalledProcessError:
            return False

    def wait_for_ready(self, session_id: str, timeout: float = 5.0) -> bool:
        """Detect agent readiness by looking for prompt patterns."""
        start = time.time()
        while time.time() - start < timeout:
            if self.is_ready(session_id):
                return True
            time.sleep(0.5)
        return False

    def is_ready(self, session_id: str) -> bool:
        """Check if session is at a prompt (idle)."""
        try:
            # Capture last few lines of the pane
            result = shim_run(["tmux", "capture-pane", "-pt", session_id], capture_output=True, text=True, check=False)
            output = result.stdout.strip()
            if not output:
                return False

            last_line = output.splitlines()[-1]
            # Common prompt patterns: $, %, #, >, or agent-specific prompts
            prompt_patterns = [r"\$ $", r"% $", r"# $", r"> $", r"aider>", r"claude>"]
            return any(re.search(pattern, last_line) for pattern in prompt_patterns)
        except Exception:
            return False


class AgentReadinessDetector:
    """Advanced readiness detection using process state and output analysis."""

    @staticmethod
    def get_agent_state(pid: int) -> str:
        """Determine agent state: idle, busy, error."""
        import psutil

        try:
            proc = psutil.Process(pid)
            if proc.status() == psutil.STATUS_SLEEPING:
                # Likely waiting for input/idle
                return "idle"
            if proc.status() == psutil.STATUS_RUNNING:
                return "busy"
            return "unknown"
        except (psutil.NoSuchProcess, psutil.AccessDenied):
            return "error"
