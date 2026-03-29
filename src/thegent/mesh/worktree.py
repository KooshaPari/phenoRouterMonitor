"""Git worktree management for the agent mesh.

Implements TGNT-P15.1 (worktree creation), TGNT-P15.2 (branch coordination),
and TGNT-P15.3 (worktree cleanup with orphan detection).
"""

import json
import logging
import shutil
import subprocess
from thegent.infra.shim_subprocess import run as shim_run
import time
from pathlib import Path

_log = logging.getLogger(__name__)

_DEFAULT_GRACE_SECONDS = 30


class BranchCollisionError(Exception):
    """Raised when a branch is already claimed by another agent."""


class WorktreeManager:
    """Manages per-agent git worktrees (TGNT-P15.1 -- P15.3)."""

    def __init__(self, project_root: Path, mesh_root: Path) -> None:
        self.project_root = project_root
        self.mesh_root = mesh_root
        self.worktree_base = project_root / ".mesh" / "worktrees"
        self._registry_path = self.worktree_base / "branch_registry.json"

    # ------------------------------------------------------------------
    # TGNT-P15.1: Worktree creation
    # ------------------------------------------------------------------

    def create_worktree(self, agent_id: str, branch: str) -> Path:
        """Create a private worktree for the agent (TGNT-P15.1).

        The worktree is placed at ``.mesh/worktrees/agent-{agent_id}``.
        Registers the branch in the branch registry (TGNT-P15.2).

        Raises:
            BranchCollisionError: If *branch* is already claimed by a
                different agent.
        """
        self._check_branch_collision(agent_id, branch)

        worktree_path = self.worktree_base / f"agent-{agent_id}"
        if worktree_path.exists():
            self.remove_worktree(agent_id)

        try:
            shim_run(
                ["git", "worktree", "add", str(worktree_path), branch],
                cwd=self.project_root,
                check=True,
                capture_output=True,
            )
        except subprocess.CalledProcessError:
            pass  # Return path anyway; may exist but be detached

        self._register_branch(agent_id, branch)
        return worktree_path

    # ------------------------------------------------------------------
    # TGNT-P15.2: Branch coordination
    # ------------------------------------------------------------------

    def _load_registry(self) -> dict[str, dict[str, str]]:
        """Load branch registry from disk.

        Returns a dict mapping ``agent_id`` to ``{"branch": ..., "ts": ...}``.
        """
        if not self._registry_path.exists():
            return {}
        try:
            return json.loads(self._registry_path.read_text(encoding="utf-8"))
        except (json.JSONDecodeError, OSError):
            return {}

    def _save_registry(self, registry: dict[str, dict[str, str]]) -> None:
        """Persist the branch registry to disk."""
        self._registry_path.parent.mkdir(parents=True, exist_ok=True)
        self._registry_path.write_text(json.dumps(registry, indent=2), encoding="utf-8")

    def _register_branch(self, agent_id: str, branch: str) -> None:
        """Register a branch as owned by *agent_id*."""
        registry = self._load_registry()
        registry[agent_id] = {
            "branch": branch,
            "ts": str(time.time()),
        }
        self._save_registry(registry)

    def _unregister_branch(self, agent_id: str) -> None:
        """Remove *agent_id* from the branch registry."""
        registry = self._load_registry()
        registry.pop(agent_id, None)
        self._save_registry(registry)

    def _check_branch_collision(self, agent_id: str, branch: str) -> None:
        """Raise :class:`BranchCollisionError` if *branch* belongs to another agent."""
        registry = self._load_registry()
        for other_id, entry in registry.items():
            if other_id != agent_id and entry.get("branch") == branch:
                raise BranchCollisionError(f"Branch {branch!r} is already claimed by agent {other_id!r}")

    def get_branch_status(self) -> dict[str, dict[str, str]]:
        """Return the full branch registry (TGNT-P15.2)."""
        return self._load_registry()

    # ------------------------------------------------------------------
    # TGNT-P15.3: Worktree removal and cleanup
    # ------------------------------------------------------------------

    def remove_worktree(self, agent_id: str) -> bool:
        """Remove and cleanup agent worktree (TGNT-P15.3)."""
        worktree_path = self.worktree_base / f"agent-{agent_id}"
        self._unregister_branch(agent_id)

        if not worktree_path.exists():
            return True

        try:
            shim_run(
                ["git", "worktree", "remove", "--force", str(worktree_path)],
                cwd=self.project_root,
                check=True,
                capture_output=True,
            )
            return True
        except subprocess.CalledProcessError:
            if worktree_path.exists():
                shutil.rmtree(worktree_path, ignore_errors=True)
            return False

    def list_worktrees(self) -> list[dict[str, str]]:
        """List active mesh worktrees."""
        try:
            output = subprocess.check_output(
                ["git", "worktree", "list", "--porcelain"],
                cwd=self.project_root,
                text=True,
            )
            worktrees: list[dict[str, str]] = []
            current: dict[str, str] = {}
            for line in output.splitlines():
                if line.startswith("worktree "):
                    if current:
                        worktrees.append(current)
                    current = {"path": line.split(" ", 1)[1]}
                elif line.startswith("branch "):
                    current["branch"] = line.split(" ", 1)[1]
            if current:
                worktrees.append(current)
            return [w for w in worktrees if ".mesh/worktrees" in w.get("path", "")]
        except subprocess.CalledProcessError:
            return []

    def cleanup_orphans(
        self,
        grace_seconds: float = _DEFAULT_GRACE_SECONDS,
    ) -> list[str]:
        """Detect and clean orphan worktrees (TGNT-P15.3).

        A worktree is considered orphaned when its directory exists under
        ``.mesh/worktrees/`` but has no corresponding entry in the branch
        registry, **and** the directory is older than *grace_seconds*.

        Returns a list of removed agent IDs.
        """
        removed: list[str] = []
        if not self.worktree_base.exists():
            return removed

        registry = self._load_registry()
        now = time.time()

        for entry in self.worktree_base.iterdir():
            if not entry.is_dir():
                continue
            if not entry.name.startswith("agent-"):
                continue
            agent_id = entry.name[len("agent-") :]
            if agent_id in registry:
                continue

            try:
                mtime = entry.stat().st_mtime
            except OSError:
                continue

            age = now - mtime
            if age < grace_seconds:
                _log.debug(
                    "Orphan worktree agent-%s is %.1fs old (grace=%s); skipping",
                    agent_id,
                    age,
                    grace_seconds,
                )
                continue

            _log.info("Cleaning orphan worktree agent-%s (age=%.1fs)", agent_id, age)
            self.remove_worktree(agent_id)
            removed.append(agent_id)

        return removed

    def health_check(self) -> dict[str, object]:
        """Return health summary for monitoring (TGNT-P15.3)."""
        registry = self._load_registry()
        worktree_dirs = []
        if self.worktree_base.exists():
            worktree_dirs = [d.name for d in self.worktree_base.iterdir() if d.is_dir() and d.name.startswith("agent-")]
        orphans = [d for d in worktree_dirs if d[len("agent-") :] not in registry]
        return {
            "registered_agents": len(registry),
            "worktree_dirs": len(worktree_dirs),
            "orphan_dirs": len(orphans),
            "orphan_names": orphans,
        }
