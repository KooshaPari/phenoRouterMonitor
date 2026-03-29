from __future__ import annotations

import re
import shutil
import subprocess
from pathlib import Path


def run_git(repo_path: Path, args: list[str]) -> str:
    proc = subprocess.run(
        ["git", "-C", str(repo_path), *args],
        capture_output=True,
        text=True,
        check=False,
    )
    if proc.returncode != 0:
        raise RuntimeError(f"git {' '.join(args)} failed: {proc.stderr.strip()}")
    return proc.stdout.strip()


def sanitize_repo_id(value: str) -> str:
    out = re.sub(r"[^A-Za-z0-9._-]+", "-", value).strip("-")
    return out or "repo"


def resolve_ref_to_sha(repo_path: Path, ref: str) -> str:
    return run_git(repo_path, ["rev-parse", f"{ref}^{{commit}}"])


def list_timeline(
    repo_path: Path,
    limit: int = 30,
    branch: str | None = None,
) -> dict[str, list[str] | bool | str]:
    branches = run_git(repo_path, ["for-each-ref", "--format=%(refname:short)", "refs/heads"]).splitlines()
    tags = run_git(repo_path, ["for-each-ref", "--format=%(refname:short)", "refs/tags"]).splitlines()
    selected_ref = "HEAD"
    head_sha = run_git(repo_path, ["rev-parse", "HEAD^{commit}"])
    branch_exists = False
    log_ref = "HEAD"
    if branch:
        branch = branch.strip()
        if not branch:
            raise ValueError("branch must not be empty")
        try:
            head_sha = run_git(repo_path, ["rev-parse", f"{branch}^{{commit}}"])
        except RuntimeError as exc:
            raise ValueError(f"unknown branch: {branch}") from exc
        selected_ref = branch
        branch_exists = True
        log_ref = branch
    recent = run_git(
        repo_path,
        ["log", log_ref, "--oneline", f"--max-count={limit}", "--decorate"],
    ).splitlines()
    return {
        "branches": [b for b in branches if b],
        "tags": [t for t in tags if t],
        "recent": recent,
        "selected_ref": selected_ref,
        "branch": branch,
        "branch_exists": branch_exists,
        "head_sha": head_sha,
    }


def _safe_remove_path(path: Path) -> None:
    if not path.exists():
        return
    if path.is_file() or path.is_symlink():
        path.unlink(missing_ok=True)
        return
    shutil.rmtree(path)


def materialize_repo_checkout(source_repo: Path, checkout_path: Path, resolved_sha: str) -> None:
    checkout_path.parent.mkdir(parents=True, exist_ok=True)

    # Remove existing materialization to keep deterministic state.
    if checkout_path.exists():
        # Attempt to remove as worktree first; ignore if not registered.
        subprocess.run(
            ["git", "-C", str(source_repo), "worktree", "remove", "--force", str(checkout_path)],
            capture_output=True,
            text=True,
            check=False,
        )
        _safe_remove_path(checkout_path)

    proc = subprocess.run(
        ["git", "-C", str(source_repo), "worktree", "add", "--detach", str(checkout_path), resolved_sha],
        capture_output=True,
        text=True,
        check=False,
    )
    if proc.returncode != 0:
        raise RuntimeError(
            "git worktree add failed for "
            f"{source_repo} @ {resolved_sha}: {proc.stderr.strip()}"
        )


def detect_head_branch(checkout_path: Path) -> str | None:
    proc = subprocess.run(
        ["git", "-C", str(checkout_path), "branch", "--show-current"],
        capture_output=True,
        text=True,
        check=False,
    )
    if proc.returncode != 0:
        return None
    value = proc.stdout.strip()
    return value or None
