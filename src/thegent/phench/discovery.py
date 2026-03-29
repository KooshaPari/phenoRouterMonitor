"""Repository discovery helpers for project orchestration workflows."""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

from .paths import phenotype_repos_root, should_include_repo
from .git_ops import sanitize_repo_id


@dataclass(slots=True)
class RepoCandidate:
    """Discovered repository candidate in a workspace root."""

    repo_id: str
    path: Path


def discover_local_git_repos(
    root: Path | None = None,
    include: list[str] | None = None,
    exclude: list[str] | None = None,
) -> list[RepoCandidate]:
    """Discover local git checkouts under `root` (shallow, one level deep)."""
    root_path = root or phenotype_repos_root()
    candidates: list[RepoCandidate] = []
    if not root_path.exists():
        return candidates

    for child in sorted(root_path.iterdir(), key=lambda item: item.name.lower()):
        if not child.is_dir() or child.name.startswith("."):
            continue
        git_dir = child / ".git"
        if not git_dir.exists():
            continue
        if not should_include_repo(child.name, include_patterns=include, exclude_patterns=exclude):
            continue
        candidates.append(RepoCandidate(repo_id=sanitize_repo_id(child.name), path=child))
    return candidates
