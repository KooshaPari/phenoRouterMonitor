from __future__ import annotations

import json
import os
import re
import shutil
import subprocess
import sys
from pathlib import Path

import typer

from .models import RunnerCatalog, RunnerCommand


_TARGET_RE = re.compile(r"^([A-Za-z0-9_.-]+):(?:\s|$)")


def _read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def _makefile_targets(path: Path) -> list[str]:
    targets: list[str] = []
    for line in _read_text(path).splitlines():
        if line.startswith("\t") or line.startswith("#"):
            continue
        match = _TARGET_RE.match(line)
        if not match:
            continue
        target = match.group(1)
        if target.startswith("."):
            continue
        targets.append(target)
    return sorted(set(targets))


def _task_targets(path: Path) -> list[str]:
    # Taskfile is yaml; do a conservative parse for top-level keys ending with ':'.
    targets: list[str] = []
    for raw in _read_text(path).splitlines():
        line = raw.rstrip()
        if not line or line.startswith(" ") or line.startswith("\t"):
            continue
        if line.startswith("#"):
            continue
        if (
            line.endswith(":")
            and not line.startswith("version:")
            and not line.startswith("tasks:")
        ):
            targets.append(line[:-1].strip())
    return sorted({t for t in targets if t})


def _just_targets(path: Path) -> list[str]:
    targets: list[str] = []
    for line in _read_text(path).splitlines():
        if line.startswith("#") or not line.strip() or line.startswith(" "):
            continue
        match = _TARGET_RE.match(line)
        if match:
            targets.append(match.group(1))
    return sorted(set(targets))


def _npm_scripts(path: Path) -> list[str]:
    parsed = json.loads(_read_text(path))
    scripts = parsed.get("scripts") if isinstance(parsed, dict) else None
    if not isinstance(scripts, dict):
        return []
    return sorted(k for k in scripts if isinstance(k, str))


def build_runner_catalog(target: str, repo_checkout: Path) -> RunnerCatalog:
    runners: list[str] = []
    commands: list[RunnerCommand] = []

    taskfile = repo_checkout / "Taskfile.yml"
    if taskfile.exists() and shutil.which("task"):
        runners.append("task")
        for name in _task_targets(taskfile):
            commands.append(RunnerCommand("task", name, f"task {name}", str(taskfile)))

    justfile = repo_checkout / "justfile"
    if justfile.exists() and shutil.which("just"):
        runners.append("just")
        for name in _just_targets(justfile):
            commands.append(RunnerCommand("just", name, f"just {name}", str(justfile)))

    makefile = repo_checkout / "Makefile"
    if makefile.exists() and shutil.which("make"):
        runners.append("make")
        for name in _makefile_targets(makefile):
            commands.append(RunnerCommand("make", name, f"make {name}", str(makefile)))

    package_json = repo_checkout / "package.json"
    if package_json.exists() and (
        shutil.which("pnpm") or shutil.which("npm") or shutil.which("bun")
    ):
        if shutil.which("pnpm"):
            runner = "pnpm"
        elif shutil.which("bun"):
            runner = "bun"
        else:
            runner = "npm"
        runners.append(runner)
        for name in _npm_scripts(package_json):
            if runner == "pnpm":
                cmd = f"pnpm run {name}"
            elif runner == "bun":
                cmd = f"bun run {name}"
            else:
                cmd = f"npm run {name}"
            commands.append(RunnerCommand(runner, name, cmd, str(package_json)))

    default = commands[0].description if commands else ""
    return RunnerCatalog(
        target_name=target,
        runners_detected=sorted(set(runners)),
        commands=commands,
        default_command=default,
    )


def pick_command_interactive(catalog: RunnerCatalog) -> RunnerCommand:
    if not catalog.commands:
        raise ValueError("No runnable commands found in runner catalog")
    if not sys.stdin.isatty() or not sys.stdout.isatty():
        raise ValueError(
            "Interactive command selection requires a TTY; pass --runner and --command explicitly"
        )

    typer.echo("Select command to run:")
    for idx, command in enumerate(catalog.commands, start=1):
        typer.echo(f"{idx}. [{command.runner}] {command.description}")

    raw = input("Enter selection number: ").strip()
    if not raw.isdigit():
        raise ValueError("Invalid selection; expected a number")
    index = int(raw)
    if index < 1 or index > len(catalog.commands):
        raise ValueError("Selection out of range")
    return catalog.commands[index - 1]


def run_command(repo_checkout: Path, runner: str, command_name: str, env_overrides: dict[str, str] | None = None) -> int:
    if runner == "task":
        cmd = ["task", command_name]
    elif runner == "just":
        cmd = ["just", command_name]
    elif runner == "make":
        cmd = ["make", command_name]
    elif runner == "pnpm":
        cmd = ["pnpm", "run", command_name]
    elif runner == "npm":
        cmd = ["npm", "run", command_name]
    elif runner == "bun":
        cmd = ["bun", "run", command_name]
    else:
        raise ValueError(f"Unsupported runner: {runner}")

    env = os.environ.copy()
    if env_overrides:
        env.update(env_overrides)

    proc = subprocess.run(cmd, cwd=str(repo_checkout), check=False, env=env)
    return proc.returncode
