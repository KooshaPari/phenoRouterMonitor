"""Sandboxing and autonomy enforcement for the agent mesh."""

import enum
from pathlib import Path


class AutonomyTier(enum.IntEnum):
    READ_ONLY = 1
    WORKTREE = 2
    GIT_SCOPED = 3
    SHARED_MESH = 4
    PRODUCTION = 5


class Sandboxing:
    """Agent sandboxing and autonomy enforcement (SCLI-P10.1–P10.4)."""

    def __init__(self, project_root: Path, agent_id: str) -> None:
        self.project_root = project_root
        self.agent_id = agent_id

    def get_bubblewrap_args(self, tier: AutonomyTier) -> list[str]:
        """Generate bubblewrap arguments for Linux (SCLI-P10.1)."""
        args = [
            "bwrap",
            "--ro-bind",
            "/usr",
            "/usr",
            "--ro-bind",
            "/lib",
            "/lib",
            "--ro-bind",
            "/bin",
            "/bin",
            "--dev",
            "/dev",
            "--proc",
            "/proc",
        ]

        if tier >= AutonomyTier.WORKTREE:
            # Allow writes to a specific worktree directory
            worktree_dir = self.project_root / ".mesh" / "worktrees" / f"agent-{self.agent_id}"
            worktree_dir.mkdir(parents=True, exist_ok=True)
            args += ["--bind", str(worktree_dir), str(self.project_root)]
        else:
            # Read-only bind of project root
            args += ["--ro-bind", str(self.project_root), str(self.project_root)]

        if tier < AutonomyTier.PRODUCTION:
            args += ["--unshare-net"]

        return args

    def get_seatbelt_profile(self, tier: AutonomyTier) -> str:
        """Generate seatbelt (sandbox-exec) profile for macOS (SCLI-P10.2)."""
        if tier == AutonomyTier.READ_ONLY:
            return f'(version 1)\n(deny default)\n(allow file-read* (subpath "{self.project_root}"))'
        return "(version 1)\n(allow default)\n(deny network-outbound)"

    def classify_operation(self, command: str, target: str) -> AutonomyTier:
        """Classify operation into autonomy tier (SCLI-P10.4)."""
        cmd = command.lower()
        if any(c in cmd for c in ["cat", "ls", "grep", "read"]):
            return AutonomyTier.READ_ONLY
        if "git" in cmd:
            if "commit" in cmd or "push" in cmd:
                return AutonomyTier.SHARED_MESH
            return AutonomyTier.GIT_SCOPED
        if "rm" in cmd or "mv" in cmd:
            return AutonomyTier.PRODUCTION
        return AutonomyTier.WORKTREE

    def check_autonomy(self, tier: AutonomyTier, required: AutonomyTier) -> bool:
        """Enforce autonomy tier limits (SCLI-P10.3)."""
        return tier >= required
