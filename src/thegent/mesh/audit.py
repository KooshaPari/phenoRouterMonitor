"""Audit trail and recovery for the agent mesh."""

import shutil
import subprocess
from thegent.infra.shim_subprocess import run as shim_run
import time
from pathlib import Path


class AuditManager:
    """Shadow repo and audit trail (SCLI-P14.1–P14.3)."""

    def __init__(self, project_root: Path, mesh_root: Path) -> None:
        self.project_root = project_root
        self.shadow_root = mesh_root / "shadow-repo"
        self.audit_log = mesh_root / "audit.log"

    def ensure_shadow_repo(self) -> Path:
        """Create a shadow git repo for recovery (SCLI-P14.1)."""
        if not self.shadow_root.exists():
            self.shadow_root.mkdir(parents=True, exist_ok=True)
            # Initialize as a git repo or clone current
            try:
                shim_run(
                    ["git", "clone", "--shared", str(self.project_root), str(self.shadow_root)],
                    check=True,
                    capture_output=True,
                )
            except subprocess.CalledProcessError:
                shim_run(["git", "init"], cwd=self.shadow_root, check=True)
        return self.shadow_root

    def record_action(self, agent_id: str, action: str, details: str):
        """Log agent actions to audit trail (SCLI-P14.2)."""
        timestamp = time.ctime()
        entry = f"[{timestamp}] Agent:{agent_id} Action:{action} Details:{details}\n"
        with open(self.audit_log, "a") as f:
            f.write(entry)

    def sync_to_shadow(self, file_path: Path):
        """Backup a file to the shadow repo before modification (SCLI-P14.2)."""
        self.ensure_shadow_repo()
        rel_path = file_path.relative_to(self.project_root)
        dest = self.shadow_root / rel_path
        dest.parent.mkdir(parents=True, exist_ok=True)

        if file_path.exists():
            shutil.copy2(file_path, dest)
            # Commit to shadow repo for version history
            try:
                shim_run(["git", "add", str(rel_path)], cwd=self.shadow_root, check=True)
                shim_run(["git", "commit", "-m", f"Audit backup of {rel_path}"], cwd=self.shadow_root, check=True)
            except subprocess.CalledProcessError:
                pass

    def recover_file(self, rel_path: str, timestamp: str) -> bool:
        """Recover a file from shadow repo (SCLI-P14.3)."""
        # In a real implementation, this would use git checkout from shadow repo
        src = self.shadow_root / rel_path
        dest = self.project_root / rel_path
        if src.exists():
            shutil.copy2(src, dest)
            return True
        return False
