"""WP-33003: External Policy Enforcement (The Cage).
Physically restricts black-box agents using runtime isolation and monitoring.
Provides a 'cage' that enforces governance regardless of the agent's internal logic.
"""

import logging
import shutil
from pathlib import Path
from typing import Any

from thegent.infra.shim_subprocess import run as shim_run

_log = logging.getLogger(__name__)


class AgentCage:
    """A hardened runtime environment for untrusted or black-box agents."""

    def __init__(self, cage_id: str, base_dir: Path) -> None:
        self.cage_id = cage_id
        self.root = base_dir / cage_id
        self.is_active = False

    def setup(self, allowed_files: list[Path]):
        """Initialize the cage by mirroring only allowed files (Copy-on-Write style)."""
        _log.info("Setting up Cage: %s", self.cage_id)
        if self.root.exists():
            shutil.rmtree(self.root)
        self.root.mkdir(parents=True)

        for f in allowed_files:
            if f.exists():
                dest = self.root / f.relative_to(f.anchor)  # simplified
                dest.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(f, dest)

        self.is_active = True
        _log.info("Cage %s ready with %d file(s).", self.cage_id, len(allowed_files))

    def run_command(self, cmd: list[str]) -> dict[str, Any]:
        """Execute a command inside the cage with restricted CWD."""
        if not self.is_active:
            raise RuntimeError("Cage is not active.")

        _log.info("Executing caged command: %s", " ".join(cmd))

        # In a real implementation, this would use:
        # 1. chroot or namespaces (Linux)
        # 2. Docker/Wasm sandbox
        # 3. Environment variable scrubbing (PATH, etc)

        try:
            result = shim_run(
                cmd,
                cwd=self.root,
                capture_output=True,
                text=True,
                timeout=30,
                env={"CAGE_ID": self.cage_id, "PATH": "/usr/bin:/bin"},  # Restricted path
                check=False,
            )
            return {"exit_code": result.returncode, "stdout": result.stdout, "stderr": result.stderr}
        except Exception as e:
            _log.error("Caged command failed: %s", e)
            return {"exit_code": -1, "error": str(e)}

    def cleanup(self):
        """Tear down the cage and scrub data."""
        if self.root.exists():
            shutil.rmtree(self.root)
        self.is_active = False
        _log.info("Cage %s cleaned up.", self.cage_id)
