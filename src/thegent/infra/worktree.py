"""Phase 15: Worktree Support implementation.
Includes worktree creation, branch coordination, and cleanup.
"""

import logging
import subprocess
from thegent.infra.shim_subprocess import run as shim_run
import time
from pathlib import Path

logger = logging.getLogger(__name__)


class WorktreeManager:
    """Manages git worktrees for isolated agent environments."""

    def __init__(self, project_root: Path, mesh_dir: Path) -> None:
        self.project_root = project_root
        self.mesh_worktrees_dir = mesh_dir / "worktrees"
        self.mesh_worktrees_dir.mkdir(parents=True, exist_ok=True)

    def create_worktree(self, agent_id: str, branch_name: str | None = None) -> Path | None:
        """Create a new worktree for an agent."""
        wt_path = self.mesh_worktrees_dir / f"agent-{agent_id}"
        if not branch_name:
            branch_name = f"mesh/agent-{agent_id}"

        try:
            shim_run(
                ["git", "rev-parse", "--verify", branch_name], cwd=self.project_root, capture_output=True, check=False
            )

            cmd = ["git", "worktree", "add", str(wt_path), branch_name]
            result = shim_run(cmd, cwd=self.project_root, capture_output=True, text=True, check=False)

            if result.returncode == 0:
                logger.info(f"Created worktree for agent {agent_id} at {wt_path}")
                return wt_path
            logger.error(f"Failed to create worktree: {result.stderr}")
            return None
        except Exception as e:
            logger.error(f"Worktree creation error: {e}")
            return None

    def cleanup_worktree(self, agent_id: str):
        """Remove worktree and prune record."""
        wt_path = self.mesh_worktrees_dir / f"agent-{agent_id}"
        if wt_path.exists():
            try:
                shim_run(["git", "worktree", "remove", "--force", str(wt_path)], cwd=self.project_root, check=True)
                shim_run(["git", "worktree", "prune"], cwd=self.project_root, check=True)
                logger.info(f"Cleaned up worktree for agent {agent_id}")
            except subprocess.CalledProcessError as e:
                logger.error(f"Failed to cleanup worktree: {e}")

    def list_active_worktrees(self) -> list[dict[str, str]]:
        """List current git worktrees."""
        try:
            result = shim_run(
                ["git", "worktree", "list", "--porcelain"],
                cwd=self.project_root,
                capture_output=True,
                text=True,
                check=False,
            )
            worktrees = []
            current = {}
            for line in result.stdout.splitlines():
                if line.startswith("worktree "):
                    if current:
                        worktrees.append(current)
                    current = {"path": line[9:]}
                elif line.startswith("branch "):
                    current["branch"] = line[7:]
            if current:
                worktrees.append(current)
            return worktrees
        except Exception:
            return []


class BranchCoordinator:
    """Coordinates branch naming and collision avoidance."""

    @staticmethod
    def get_safe_branch_name(base: str) -> str:
        timestamp = int(time.time())
        return f"{base}-{timestamp}"
