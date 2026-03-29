"""Compatibility re-export for worktree parallelism domain module."""

from thegent_agents.mesh.git_parallelism import (
    WorktreeContext,
    WorktreePool,
    _atomic_write,
    _git_available,
    _PoolStateLock,
    _project_hash,
    _worktrees_supported,
)

__all__ = [
    "WorktreeContext",
    "WorktreePool",
    "_atomic_write",
    "_git_available",
    "_PoolStateLock",
    "_project_hash",
    "_worktrees_supported",
]
