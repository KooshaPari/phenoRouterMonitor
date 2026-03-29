"""Shell and context injection for the agent mesh."""

import subprocess
from thegent.infra.shim_subprocess import run as shim_run
import time
from pathlib import Path


class ShellInjection:
    """Tmux-based shell injection (SCLI-P9.1–P9.3)."""

    def __init__(self, agent_id: str) -> None:
        self.agent_id = agent_id
        self.session_name = f"mesh-{agent_id}"

    def find_session(self) -> bool:
        """Detect if agent tmux session exists (SCLI-P9.1)."""
        try:
            shim_run(["tmux", "has-session", "-t", self.session_name], check=True, capture_output=True)
            return True
        except subprocess.CalledProcessError:
            return False

    def send_command(self, command: str, wait: float = 1.5) -> bool:
        """Inject command into tmux session (SCLI-P9.2)."""
        if not self.find_session():
            return False

        try:
            # send-keys with delay for reliability
            shim_run(["tmux", "send-keys", "-t", self.session_name, "-l", command], check=True)
            time.sleep(wait)
            shim_run(["tmux", "send-keys", "-t", self.session_name, "Enter"], check=True)
            return True
        except subprocess.CalledProcessError:
            return False

    def is_ready(self, prompt_pattern: str = r"(\$|#|>>>) ") -> bool:
        """Detect if agent shell is ready for input (SCLI-P9.3)."""
        try:
            # Capture last few lines of tmux session
            output = subprocess.check_output(["tmux", "capture-pane", "-p", "-t", self.session_name], text=True)
            lines = output.splitlines()
            if not lines:
                return False

            import re

            return bool(re.search(prompt_pattern, lines[-1]))
        except subprocess.CalledProcessError:
            return False


class ContextInjection:
    """Dynamic context injection (SCLI-P9.4–P9.5)."""

    def __init__(self, project_root: Path, mesh_root: Path) -> None:
        self.project_root = project_root
        self.mesh_root = mesh_root

    def update_agent_md(self, mesh_state: dict) -> Path:
        """Render AGENT.md from template with current mesh state (SCLI-P9.4)."""
        agent_md = self.project_root / "AGENT.md"

        # Simple template rendering
        content = [
            "# Agent Mesh Status",
            f"Last Update: {time.ctime()}",
            "\n## Active Agents",
        ]
        for agent in mesh_state.get("agents", []):
            content.append(f"- {agent['id']} ({agent['type']}) - {agent['status']}")

        content.append("\n## Coordination Rules")
        content.append("- Always use per-agent index for git operations")
        content.append("- Respect file leases in /tmp/agent-mesh/claims")

        with open(agent_md, "w") as f:
            f.write("\n".join(content))

        return agent_md

    def create_tool_symlinks(self, agent_id: str) -> list[Path]:
        """Create tool-specific symlinks to AGENT.md (SCLI-P9.5)."""
        links = []
        target = self.project_root / "AGENT.md"

        for name in [".cursorrules", ".clinerules", "CLAUDE.md"]:
            link_path = self.project_root / name
            if not link_path.exists():
                try:
                    link_path.symlink_to(target)
                    links.append(link_path)
                except OSError:
                    pass
        return links
