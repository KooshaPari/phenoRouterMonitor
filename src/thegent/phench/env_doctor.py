"""Environment preflight for materialized target checkouts (runners and paths)."""

from __future__ import annotations

import shutil
from pathlib import Path
from typing import Literal

from .models import EnvDoctorReport


def run_env_doctor(target: str, checkouts: list[Path]) -> EnvDoctorReport:
    """Validate materialized checkouts exist and required runner binaries are on PATH.

    When a checkout declares a runner via marker files (Taskfile, justfile, Makefile,
    package.json), the corresponding binary must resolve via ``shutil.which`` or the doctor
    fails and records the runner name in ``missing_requirements``.
    """
    missing: list[str] = []
    detected: list[str] = []
    versions: dict[str, str] = {}

    for checkout in checkouts:
        path = checkout.resolve()
        if not path.exists():
            missing.append(f"missing_checkout:{path}")
            continue
        detected.append(str(path))

        if (path / "Taskfile.yml").exists() or (path / "Taskfile.yaml").exists():
            exe = shutil.which("task")
            if exe:
                versions[f"task@{path.name}"] = exe
            else:
                missing.append("task")

        if (path / "justfile").exists():
            exe = shutil.which("just")
            if exe:
                versions[f"just@{path.name}"] = exe
            else:
                missing.append("just")

        if (path / "Makefile").exists():
            exe = shutil.which("make")
            if exe:
                versions[f"make@{path.name}"] = exe
            else:
                missing.append("make")

        if (path / "package.json").exists():
            found = False
            for name in ("pnpm", "bun", "npm"):
                exe = shutil.which(name)
                if exe:
                    versions[f"{name}@{path.name}"] = exe
                    found = True
                    break
            if not found:
                missing.append("pnpm")

    doctor_status: Literal["pass", "fail"] = "pass" if not missing else "fail"
    return EnvDoctorReport(
        target_name=target,
        doctor_status=doctor_status,
        missing_requirements=missing,
        resolved_versions=versions,
        detected_files=detected,
    )
