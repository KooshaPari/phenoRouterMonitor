"""Resource isolation for the agent mesh."""

import os
import shutil
import socket
from pathlib import Path


class ResourceIsolation:
    """Resource isolation for agents (SCLI-P8.1–P8.3, FR-ISOL-001, FR-ISOL-003)."""

    def __init__(self, mesh_root: Path, agent_id: str, project_root: Path | None = None) -> None:
        self.mesh_root = mesh_root
        self.project_root = project_root if project_root is not None else mesh_root
        self.agent_id = agent_id
        self.agent_tmp = mesh_root / "tmp" / agent_id
        self.port_registry = mesh_root / "ports.json"
        self.work_dir = self.project_root / ".mesh" / "worktrees" / f"agent-{agent_id}"

    def allocate_tmpdir(self) -> Path:
        """Allocate private TMPDIR for the agent (SCLI-P8.1, FR-ISOL-003)."""
        if self.agent_tmp.exists():
            shutil.rmtree(self.agent_tmp)
        self.agent_tmp.mkdir(parents=True, exist_ok=True, mode=0o700)
        return self.agent_tmp

    def allocate_workdir(self, branch: str | None = None) -> Path:
        """Allocate isolated work directory for the agent (FR-ISOL-001).

        If a branch is provided, it uses a git worktree. Otherwise, it
        ensures the agent's work directory exists.
        """
        if branch:
            from thegent.mesh.worktree import WorktreeManager

            wm = WorktreeManager(self.project_root, self.mesh_root)
            return wm.create_worktree(self.agent_id, branch)

        self.work_dir.mkdir(parents=True, exist_ok=True, mode=0o700)
        return self.work_dir

    def allocate_port(self, start_port: int = 10000, end_port: int = 20000) -> int | None:
        """Dynamically allocate an available port (SCLI-P8.2)."""
        for port in range(start_port, end_port):
            with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
                try:
                    s.bind(("127.0.0.1", port))
                    return port
                except OSError:
                    continue
        return None

    def get_isolated_env(self, base_env: dict[str, str] | None = None) -> dict[str, str]:
        """Get environment variables isolated for the agent (SCLI-P8.3, FR-ISOL-003)."""
        env = (base_env or os.environ).copy()
        env["TMPDIR"] = str(self.agent_tmp)
        env["TEMP"] = str(self.agent_tmp)
        env["TMP"] = str(self.agent_tmp)
        env["AGENT_ID"] = self.agent_id
        env["MESH_ROOT"] = str(self.mesh_root)
        env["AGENT_WORK_DIR"] = str(self.work_dir)

        # Prevent agents from seeing other agents' temp files
        env["PYTHONTEMP"] = str(self.agent_tmp)

        return env

    def get_process_tree(self) -> list[int]:
        """Identify process tree for the current agent (FR-ISOL-002)."""
        import psutil

        try:
            current_process = psutil.Process(os.getpid())
            children = current_process.children(recursive=True)
            return [os.getpid()] + [p.pid for p in children]
        except (psutil.NoSuchProcess, ImportError):
            return [os.getpid()]

    def cleanup(self) -> None:
        """Cleanup agent resources on exit."""
        if self.agent_tmp.exists():
            shutil.rmtree(self.agent_tmp)
