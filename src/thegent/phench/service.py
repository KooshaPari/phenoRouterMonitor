from __future__ import annotations

import ast
from concurrent.futures import ThreadPoolExecutor
from copy import deepcopy
from dataclasses import asdict
from fnmatch import fnmatch
from pathlib import Path
from datetime import UTC, datetime
import hashlib
import json
import os
import secrets
import re
from typing import Any

from .env_doctor import run_env_doctor
from .discovery import RepoCandidate, discover_local_git_repos
from .git_ops import (
    detect_head_branch,
    list_timeline,
    materialize_repo_checkout,
    resolve_ref_to_sha,
    sanitize_repo_id,
)
from .models import ModuleManifest, RepoSelection, RuntimeRepo, RuntimeState, RunnerCatalog, TargetLock, TargetMode
from .models import ModuleManifest, RepoSelection, RuntimeRepo, RuntimeState, RunnerCatalog, TargetLock, TargetMode
from .paths import (
    _load_json_file,
    mirror_target_state_root,
    module_manifests_root,
    module_manifest_path,
    phenotype_repos_root,
    phenotype_root,
    projects_modules_root,
    projects_root,
    repository_root_candidates,
    target_repos_root,
    target_root,
    target_state_root,
    validate_family_name,
)
from .runner import build_runner_catalog, pick_command_interactive, run_command
from .store import dual_write, read_dual, sync_dual, utc_now_iso
LOCK_FILE = "target.lock.json"
RUNTIME_FILE = "runtime.json"
ENV_FILE = "env.snapshot.json"
RUNNER_FILE = "runner.catalog.json"
PROFILE_FILE = "env.profile.json"
SNAPSHOT_DIR = "snapshots"
SUPPORTED_MODULE_MANIFEST_SCHEMA_VERSIONS = {1}
DEFAULT_MODULE_REFRESH_CADENCE = "never"
_REFRESH_CADENCE_RE = re.compile(
    r"^(never|manual|daily|weekly|monthly|yearly|hourly|every-\d+[smhdwy])$"
)
DEFAULT_SHARED_MODULE_REPO_EXCLUDE = {"4sgm", "trace", "parpour", "civ"}
DEFAULT_EXCLUDED_REPOS = frozenset(DEFAULT_SHARED_MODULE_REPO_EXCLUDE)
SCAN_SHARED_REPOS_SCHEMA_VERSION = 1
SCAN_SHARED_REPOS_ROOT_MODE_REPOS = "repos"
SCAN_SHARED_REPOS_ROOT_MODE_WORKTREES = "worktrees"
SCAN_SHARED_REPOS_DEFAULT_MODULE_PREFIX = "shared-module"
SCAN_SHARED_REPOS_MAX_NAME_LENGTH = 60
SCAN_SHARED_REPOS_RECOMMENDED_MODULE_COUNT_LIMIT = 10
SCAN_SHARED_REPOS_MANIFEST_INDEX_FILENAME = "index.json"
SCAN_SHARED_REPOS_MANIFEST_INDEX_SUMMARY_FILENAME = "index-summary.json"
SCAN_SHARED_REPOS_MANIFEST_AUDIT_FILENAME = "manifest-audit.jsonl"


def _manifest_payload_repo_ids(payload: Any) -> list[str] | None:
    if not isinstance(payload, dict):
        return None

    raw_ids = payload.get("repo_ids")
    if not isinstance(raw_ids, list):
        return None

    repo_ids: list[str] = []
    for value in raw_ids:
        if not isinstance(value, str):
            return None
        repo_ids.append(value)
    return repo_ids


def _compatible_module_manifest(existing_payload: Any, next_payload: Any) -> bool:
    if existing_payload == next_payload:
        return True

    existing_ids = _manifest_payload_repo_ids(existing_payload)
    next_ids = _manifest_payload_repo_ids(next_payload)
    if existing_ids is None or next_ids is None:
        return False

    existing_core = dict(existing_payload)
    next_core = dict(next_payload)
    existing_core.pop("repo_ids", None)
    next_core.pop("repo_ids", None)
    if existing_core != next_core:
        return False

    return set(existing_ids).issubset(next_ids) or set(next_ids).issubset(existing_ids)


def _normalize_name_list(values: list[str] | None) -> list[str]:
    if values is None:
        return []
    normalized: list[str] = []
    seen: set[str] = set()
    for value in values:
        name = value.strip()
        if not name:
            continue
        if name in seen:
            continue
        seen.add(name)
        normalized.append(name)
    return normalized


def _normalize_name_set(values: list[str] | None) -> set[str]:
    return set(_normalize_name_list(values))


def _resolve_module_manifest_path(module: str) -> Path:
    normalized_module = module.strip()
    if not normalized_module:
        raise ValueError("module manifest not found: <empty>")

    is_path_like = ("/" in normalized_module) or ("\\" in normalized_module)
    candidates: list[Path] = []

    def _add(path: Path) -> None:
        manifest_path = path if path.name == "manifest.json" else path / "manifest.json"
        if manifest_path not in candidates:
            candidates.append(manifest_path)

    if is_path_like:
        explicit_path = Path(normalized_module).expanduser()
        _add(explicit_path)
        if not explicit_path.is_absolute():
            _add(phenotype_root() / explicit_path)

        if explicit_path.is_absolute() and explicit_path.exists():
            if explicit_path.is_file() and explicit_path.name == "manifest.json":
                return explicit_path
            if explicit_path.is_dir() and (explicit_path / "manifest.json").exists():
                return explicit_path / "manifest.json"

        normalized_parts = [part.lower() for part in explicit_path.as_posix().split("/")]
        if "modules" in normalized_parts:
            modules_index = normalized_parts.index("modules")
            if modules_index + 1 < len(normalized_parts):
                _add(projects_modules_root() / normalized_parts[modules_index + 1])
        else:
            _add(projects_modules_root() / explicit_path.name)

        for candidate in candidates:
            if candidate.exists():
                return candidate
    else:
        module_name = validate_family_name(normalized_module)
        module_path = module_manifest_path(module_name)
        _add(module_path)
        if module_path.exists():
            return module_path

    raise ValueError(f"module manifest not found: {module}")


def _snapshot_id() -> str:
    return f"{datetime.now(UTC).strftime('%Y%m%dT%H%M%SZ')}-{secrets.token_hex(4)}"


def _stable_payload_hash(payload: Any) -> str:
    raw = json.dumps(payload, sort_keys=True, separators=(",", ":"))
    return hashlib.sha256(raw.encode("utf-8")).hexdigest()


def import_repos(
    target: str,
    family: str | None = None,
    source_root: Path | None = None,
    selected_ref: str = "HEAD",
    preferred_runner: str | None = None,
    preferred_command: str | None = None,
    preferred_ref: str | None = None,
    include: list[str] | None = None,
    exclude: list[str] | None = None,
    repo_ids: list[str] | None = None,
    auto_lock: bool = True,
) -> TargetLock:
    """Import discovered repositories into an existing target."""
    repo_root = source_root or phenotype_repos_root()
    candidates = discover_local_git_repos(root=repo_root, include=include, exclude=exclude)
    if not candidates:
        raise ValueError(f"no repos discovered under: {repo_root}")

    by_id = {item.repo_id: item.path for item in candidates}
    if repo_ids:
        selected = []
        for repo_id in repo_ids:
            path = by_id.get(repo_id)
            if not path:
                raise ValueError(f"repo_id not discovered: {repo_id}")
            selected.append(RepoCandidate(repo_id=repo_id, path=path))
    else:
        selected = candidates

    for item in selected:
        add_repo(
            target,
            family=family,
            repo_path=str(item.path),
            selected_ref=selected_ref,
            repo_id=item.repo_id,
            preferred_runner=preferred_runner,
            preferred_command=preferred_command,
            preferred_ref=preferred_ref,
        )

    if auto_lock:
        return lock_target(target, family=family)
    return load_target_lock(target, family=family)


def create_target_snapshot(
    target: str,
    family: str | None = None,
    snapshot_id: str | None = None,
) -> dict[str, Any]:
    lock = load_target_lock(target, family=family)
    snapshot_id = snapshot_id or _snapshot_id()
    filename = f"{SNAPSHOT_DIR}/{snapshot_id}.json"

    runtime_payload: dict[str, Any] | None = None
    env_payload: dict[str, Any] | None = None
    runner_payload: dict[str, Any] | None = None

    try:
        runtime_payload = read_dual(target, RUNTIME_FILE, family=family)
    except FileNotFoundError:
        runtime_payload = None
    try:
        env_payload = read_dual(target, ENV_FILE, family=family)
    except FileNotFoundError:
        env_payload = None
    try:
        runner_payload = read_dual(target, RUNNER_FILE, family=family)
    except FileNotFoundError:
        runner_payload = None

    snapshot = {
        "snapshot_id": snapshot_id,
        "created_at_utc": utc_now_iso(),
        "target_name": target,
        "lock": asdict(lock),
        "lock_hash": lock.lock_hash,
        "runtime": runtime_payload,
        "env": env_payload,
        "runner_catalog": runner_payload,
    }
    snapshot["runtime_hash"] = _stable_payload_hash(runtime_payload) if isinstance(runtime_payload, dict) else ""
    snapshot["env_hash"] = _stable_payload_hash(env_payload) if isinstance(env_payload, dict) else ""
    snapshot["runner_catalog_hash"] = (
        _stable_payload_hash(runner_payload) if isinstance(runner_payload, dict) else ""
    )
    snapshot["snapshot_hash"] = _stable_payload_hash({k: snapshot[k] for k in snapshot if k != "snapshot_hash"})
    result = dual_write(target, filename, snapshot, family=family)
    return {
        "snapshot_id": snapshot_id,
        "filename": filename,
        "target": target,
        "written_at_utc": snapshot["created_at_utc"],
        **result,
    }


def list_target_snapshots(target: str, family: str | None = None) -> list[dict[str, Any]]:
    directories = [
        target_state_root(target, family=family) / SNAPSHOT_DIR,
        mirror_target_state_root(target, family=family) / SNAPSHOT_DIR,
    ]
    snapshots: list[dict[str, Any]] = []
    files: dict[str, Path] = {}
    for directory in directories:
        if not directory.exists():
            continue
        for path in directory.glob("*.json"):
            filename = path.name
            files.setdefault(filename, path)

    for filename in sorted(files):
        rel_filename = f"{SNAPSHOT_DIR}/{filename}"
        try:
            payload = read_dual(target, rel_filename, family=family)
        except FileNotFoundError:
            continue
        if not isinstance(payload, dict):
            continue
        snapshots.append(
            {
                "snapshot_id": str(payload.get("snapshot_id", Path(filename).stem)),
                "filename": rel_filename,
                "created_at_utc": str(payload.get("created_at_utc", "")),
                "target_name": str(payload.get("target_name", "")),
                "lock_hash": str((payload.get("lock") or {}).get("lock_hash", "")),
                "runtime_hash": str(payload.get("runtime_hash", "")),
                "snapshot_hash": str(payload.get("snapshot_hash", "")),
            }
        )
    return snapshots


def show_target_snapshot(target: str, snapshot_id: str, family: str | None = None) -> dict[str, Any]:
    filename = f"{SNAPSHOT_DIR}/{snapshot_id}.json"
    return read_dual(target, filename, family=family)


def discover_repos(
    root: Path | None = None,
    include: list[str] | None = None,
    exclude: list[str] | None = None,
) -> list[RepoCandidate]:
    """Discover available local repositories for bootstrap workflows."""
    return discover_local_git_repos(root=root, include=include, exclude=exclude)


def bootstrap_target(
    target: str,
    mode: TargetMode,
    family: str | None = None,
    source_root: Path | None = None,
    selected_ref: str = "HEAD",
    preferred_runner: str | None = None,
    preferred_command: str | None = None,
    preferred_ref: str | None = None,
    include: list[str] | None = None,
    exclude: list[str] | None = None,
    repo_ids: list[str] | None = None,
    auto_lock: bool = True,
) -> TargetLock:
    """Create a target and add discovered repositories from a workspace."""
    if mode not in {"repo", "stack"}:
        raise ValueError("mode must be one of: repo, stack")

    repo_root = source_root or phenotype_repos_root()
    candidates = discover_local_git_repos(root=repo_root, include=include, exclude=exclude)
    if not candidates:
        raise ValueError(f"no repos discovered under: {repo_root}")

    by_id = {item.repo_id: item.path for item in candidates}
    if repo_ids:
        selected = []
        for repo_id in repo_ids:
            path = by_id.get(repo_id)
            if not path:
                raise ValueError(f"repo_id not discovered: {repo_id}")
            selected.append(RepoCandidate(repo_id=repo_id, path=path))
    else:
        selected = candidates

    lock = init_target(target, mode=mode, family=family)
    for item in selected:
        add_repo(
            target,
            family=family,
            repo_path=str(item.path),
            selected_ref=selected_ref,
            repo_id=item.repo_id,
            preferred_runner=preferred_runner,
            preferred_command=preferred_command,
            preferred_ref=preferred_ref,
        )
    if auto_lock:
        lock = lock_target(target, family=family)
    return lock


def set_repo_ref(
    target: str,
    repo_id: str,
    selected_ref: str,
    family: str | None = None,
) -> TargetLock:
    """Update a single repo selection ref in a target and relock the target."""
    lock = load_target_lock(target, family=family)
    updated = False
    for repo in lock.repos:
        if repo.repo_id == repo_id:
            repo.selected_ref = selected_ref
            repo.resolved_sha = None
            updated = True
            break
    if not updated:
        raise ValueError(f"repo_id not in target: {repo_id}")
    lock.created_at_utc = utc_now_iso()
    lock.lock_hash = _lock_hash(lock)
    dual_write(target, LOCK_FILE, lock, family=family)
    return lock_target(target, family=family)


def _parse_lock(payload: dict[str, Any]) -> TargetLock:
    repos = [RepoSelection(**repo) for repo in payload.get("repos", [])]
    return TargetLock(
        schema_version=int(payload.get("schema_version", 1)),
        target_name=str(payload.get("target_name", "")),
        mode=payload.get("mode", "repo"),
        repos=repos,
        lock_hash=str(payload.get("lock_hash", "")),
        created_at_utc=str(payload.get("created_at_utc", "")),
    )


def _lock_hash(lock: TargetLock) -> str:
    payload = asdict(lock)
    payload["lock_hash"] = ""
    raw = json.dumps(payload, sort_keys=True, separators=(",", ":"))
    return hashlib.sha256(raw.encode("utf-8")).hexdigest()


def init_target(target: str, mode: TargetMode, family: str | None = None) -> TargetLock:
    root = target_root(target, family=family)
    root.mkdir(parents=True, exist_ok=True)
    (root / "repos").mkdir(parents=True, exist_ok=True)

    lock = TargetLock(
        schema_version=1,
        target_name=target,
        mode=mode,
        repos=[],
        lock_hash="",
        created_at_utc=utc_now_iso(),
    )
    lock.lock_hash = _lock_hash(lock)
    dual_write(target, LOCK_FILE, lock, family=family)
    return lock


def load_target_lock(target: str, family: str | None = None) -> TargetLock:
    payload = read_dual(target, LOCK_FILE, family=family)
    return _parse_lock(payload)


def add_repo(
    target: str,
    repo_path: str,
    selected_ref: str,
    family: str | None = None,
    repo_id: str | None = None,
    worktree_path: str | None = None,
    preferred_runner: str | None = None,
    preferred_command: str | None = None,
    preferred_ref: str | None = None,
) -> TargetLock:
    lock = load_target_lock(target, family=family)
    repo = Path(repo_path).expanduser().resolve()
    if not (repo / ".git").exists() and not (repo / ".git").is_file():
        raise ValueError(f"repo is not a git checkout: {repo}")

    rid = repo_id or sanitize_repo_id(repo.name)
    lock.repos = [entry for entry in lock.repos if entry.repo_id != rid]
    lock.repos.append(
        RepoSelection(
            repo_id=rid,
            repo_path=str(repo),
            selected_ref=selected_ref,
            source_worktree_path=worktree_path,
            resolved_sha=None,
            preferred_runner=preferred_runner,
            preferred_command=preferred_command,
            preferred_ref=preferred_ref,
        )
    )
    lock.created_at_utc = utc_now_iso()
    lock.lock_hash = _lock_hash(lock)
    dual_write(target, LOCK_FILE, lock, family=family)
    return lock


def add_module_to_target(
    target: str,
    module_name: str,
    selected_ref: str | None = None,
    exclude_repos: set[str] | None = None,
    family: str | None = None,
) -> TargetLock:
    if not module_name.strip():
        raise ValueError("module name cannot be empty")
    if "/" in module_name or "\\" in module_name or ".." in module_name:
        raise ValueError(f"invalid module name: {module_name}")

    manifest = _load_module_manifest_dataclass(module_name)
    lock = load_target_lock(target, family=family)

    candidates = _select_module_repos(manifest, exclude_repos=exclude_repos)
    if not candidates:
        raise ValueError(f"module {module_name} selected no matching repos")

    fallback_ref = selected_ref or manifest.default_ref
    for candidate in candidates:
        repo_id = _repo_id_from_path(candidate)
        _append_repo_selection(
            lock,
            candidate,
            selected_ref=manifest.repo_ref_overrides.get(repo_id, fallback_ref),
            module_name=module_name,
            selected_runner=manifest.repo_runner_overrides.get(repo_id),
            selected_command=manifest.repo_command_overrides.get(repo_id),
            selected_env_profile=manifest.repo_env_profile_overrides.get(repo_id),
        )

    lock.created_at_utc = utc_now_iso()
    lock.lock_hash = _lock_hash(lock)
    dual_write(target, LOCK_FILE, lock, family=family)
    return lock


def lock_target(target: str, family: str | None = None) -> TargetLock:
    lock = load_target_lock(target, family=family)
    if not lock.repos:
        raise ValueError("target has no repos; add at least one repo before lock")

    for repo in lock.repos:
        resolved = resolve_ref_to_sha(Path(repo.repo_path), repo.selected_ref)
        repo.resolved_sha = resolved

    lock.created_at_utc = utc_now_iso()
    lock.lock_hash = _lock_hash(lock)
    dual_write(target, LOCK_FILE, lock, family=family)
    return lock


def materialize_target(target: str, family: str | None = None) -> RuntimeState:
    lock = load_target_lock(target, family=family)
    if not lock.repos:
        raise ValueError("target has no repos")

    runtime_repos: list[RuntimeRepo] = []
    repos_root = target_repos_root(target, family=family)
    repos_root.mkdir(parents=True, exist_ok=True)

    for repo in lock.repos:
        if not repo.resolved_sha:
            raise ValueError(f"repo {repo.repo_id} is not locked; run target lock first")
        checkout_path = repos_root / repo.repo_id
        materialize_repo_checkout(Path(repo.repo_path), checkout_path, repo.resolved_sha)
        runtime_repos.append(
            RuntimeRepo(
                repo_id=repo.repo_id,
                checkout_path=str(checkout_path),
                resolved_sha=repo.resolved_sha,
                head_branch=detect_head_branch(checkout_path),
            )
        )

    runtime = RuntimeState(
        target_name=target,
        materialized_root=str(target_root(target, family=family)),
        repo_materializations=runtime_repos,
        materialized_at_utc=utc_now_iso(),
    )
    dual_write(target, RUNTIME_FILE, runtime, family=family)

    report = run_env_doctor(target, [Path(r.checkout_path) for r in runtime_repos])
    dual_write(target, ENV_FILE, report, family=family)

    return runtime


def target_status(target: str, family: str | None = None) -> dict[str, Any]:
    lock = load_target_lock(target, family=family)
    runtime_payload: dict[str, Any] | None = None
    env_payload: dict[str, Any] | None = None
    try:
        runtime_payload = read_dual(target, RUNTIME_FILE, family=family)
    except FileNotFoundError:
        runtime_payload = None
    try:
        env_payload = read_dual(target, ENV_FILE, family=family)
    except FileNotFoundError:
        env_payload = None

    snapshots = list_target_snapshots(target, family=family)
    latest_snapshot = snapshots[0] if snapshots else None

    if latest_snapshot is not None:
        latest_snapshot = dict(latest_snapshot)
        latest_snapshot = {
            "snapshot_id": latest_snapshot.get("snapshot_id"),
            "filename": latest_snapshot.get("filename"),
            "created_at_utc": latest_snapshot.get("created_at_utc"),
            "lock_hash": latest_snapshot.get("lock_hash"),
            "runtime_hash": latest_snapshot.get("runtime_hash"),
            "snapshot_hash": latest_snapshot.get("snapshot_hash"),
        }

    return {
        "target": target,
        "mode": lock.mode,
        "repos": [asdict(r) for r in lock.repos],
        "lock_hash": lock.lock_hash,
        "created_at_utc": lock.created_at_utc,
        "runtime": runtime_payload,
        "env": env_payload,
        "latest_snapshot": latest_snapshot,
        "snapshot": latest_snapshot,
    }


def target_timeline(
    target: str,
    family: str | None = None,
    repo_id: str | None = None,
    limit: int = 30,
    branch: str | None = None,
) -> dict[str, Any]:
    lock = load_target_lock(target, family=family)
    if not lock.repos:
        raise ValueError("target has no repos")

    if repo_id:
        chosen = next((repo for repo in lock.repos if repo.repo_id == repo_id), None)
        if not chosen:
            raise ValueError(f"repo_id not found in target: {repo_id}")
    else:
        chosen = lock.repos[0]

    timeline = list_timeline(Path(chosen.repo_path), limit=limit, branch=branch)
    return {
        "target": target,
        "repo_id": chosen.repo_id,
        "repo_path": chosen.repo_path,
        "selected_ref": timeline["selected_ref"],
        "resolved_sha": chosen.resolved_sha,
        **timeline,
    }


def audit_shared_modules(target: str, family: str | None = None) -> dict[str, Any]:
    lock = load_target_lock(target, family=family)
    module_map: dict[str, list[str]] = {}
    for repo in lock.repos:
        src_root = Path(repo.repo_path) / "src"
        if not src_root.exists() or not src_root.is_dir():
            continue
        for child in src_root.iterdir():
            if not child.is_dir():
                continue
            if not (child / "__init__.py").exists():
                continue
            owners = module_map.setdefault(child.name, [])
            owners.append(repo.repo_id)

    shared = {module: sorted(set(owners)) for module, owners in module_map.items() if len(set(owners)) >= 2}
    return {
        "target": target,
        "shared_modules": shared,
        "repo_count": len(lock.repos),
        "module_count": len(module_map),
    }


def audit_shared_modules_across_repos(
    *,
    source_root: Path | None = None,
    include_repos: list[str] | None = None,
    exclude_repos: list[str] | None = None,
    min_repo_count: int = 2,
    exclude_modules: list[str] | None = None,
    include_modules: list[str] | None = None,
    include_repo_modules_root: bool = True,
    skip_repos: list[str] | None = None,
) -> dict[str, Any]:
    source = source_root or phenotype_repos_root()
    include_specs = _normalize_name_list(include_repos)
    exclude_specs = _normalize_name_set(exclude_repos)
    additional_excludes = _normalize_name_set(skip_repos)
    effective_excludes = sorted(exclude_specs | DEFAULT_SHARED_MODULE_REPO_EXCLUDE | additional_excludes)
    excluded_modules = _normalize_name_set(exclude_modules)
    included_modules = _normalize_name_set(include_modules)

    candidates = discover_local_git_repos(
        root=source,
        include=include_specs or None,
        exclude=effective_excludes or None,
    )

    repo_paths: list[tuple[str, Path]] = []
    for candidate in candidates:
        if candidate.repo_id in additional_excludes:
            continue
        repo_paths.append((candidate.repo_id, candidate.path))

    discovered_module_map: dict[str, set[str]] = {}
    shared_candidate_module_map: dict[str, set[str]] = {}
    for repo_id, repo_path in repo_paths:
        src_root = repo_path / "src"
        if src_root.exists() and src_root.is_dir():
            for child in src_root.iterdir():
                if not child.is_dir():
                    continue
                if not (child / "__init__.py").exists():
                    continue
                module = child.name
                if included_modules and module not in included_modules:
                    continue
                if module in excluded_modules:
                    continue
                discovered_module_map.setdefault(module, set()).add(repo_id)

        if include_repo_modules_root:
            modules_root = repo_path / "modules"
            if not (modules_root.exists() and modules_root.is_dir()):
                continue
            for child in modules_root.iterdir():
                if not child.is_dir() or not (child / "manifest.json").is_file():
                    continue
                module = child.name
                if included_modules and module not in included_modules:
                    continue
                if module in excluded_modules:
                    continue
                shared_candidate_module_map.setdefault(module, set()).add(repo_id)

    shared_modules = {
        module: sorted(repos)
        for module, repos in (
            shared_candidate_module_map if include_repo_modules_root else discovered_module_map
        ).items()
        if len(repos) >= max(min_repo_count, 2)
    }

    discovered_modules = {module: set(repos) for module, repos in discovered_module_map.items()}
    if include_repo_modules_root:
        for module, repos in shared_candidate_module_map.items():
            discovered_modules.setdefault(module, set()).update(repos)
        discovered_modules = {module: sorted(repos) for module, repos in discovered_modules.items()}
    existing = set(list_modules())
    candidate_modules = sorted(
        module for module, repos in shared_modules.items() if include_repo_modules_root and module not in existing
    )

    return {
        "source_root": str(source),
        "repo_count": len(repo_paths),
        "module_count": len(discovered_modules),
        "shared_modules": shared_modules,
        "shared_module_count": len(shared_modules),
        "moduleization_candidates": candidate_modules,
        "excluded_repos": effective_excludes,
        "excluded_modules": sorted(excluded_modules),
        "module_repo_map": discovered_modules,
        "include_repo_modules_root": bool(include_repo_modules_root),
    }


def sync_project_modules_from_repos(
    *,
    source_root: Path | None = None,
    destination_root: Path | None = None,
    include_repos: list[str] | None = None,
    exclude_repos: list[str] | None = None,
    include_modules: list[str] | None = None,
    exclude_modules: list[str] | None = None,
    overwrite: bool = False,
    dry_run: bool = False,
) -> dict[str, Any]:
    source = source_root or phenotype_repos_root()
    destination = destination_root or projects_modules_root()
    destination = destination_root
    if destination is None:
        candidate = source.parent / "projects" / "modules" if source_root is not None else None
        if (
            os.environ.get("THGENT_PHENOTYPE_ROOT") is None
            and candidate is not None
            and candidate.as_posix()
            != projects_modules_root().as_posix()
        ):
            destination = candidate
        else:
            destination = projects_modules_root()
    include_specs = _normalize_name_list(include_repos)
    exclude_specs = _normalize_name_set(exclude_repos)
    effective_excludes = sorted(exclude_specs | DEFAULT_SHARED_MODULE_REPO_EXCLUDE)
    included_modules = _normalize_name_set(include_modules)
    excluded_modules = _normalize_name_set(exclude_modules)

    discovered_repos = discover_local_git_repos(
        root=source,
        include=include_specs or None,
        exclude=effective_excludes or None,
    )

    discovered: dict[str, Path] = {}
    discovered_from: dict[str, str] = {}
    for repo_candidate in discovered_repos:
        modules_root = repo_candidate.path / "modules"
        if not modules_root.exists() or not modules_root.is_dir():
            continue
        for entry in modules_root.iterdir():
            if not entry.is_dir():
                continue
            module_name = entry.name
            if included_modules and module_name not in included_modules:
                continue
            if module_name in excluded_modules:
                continue
            manifest = entry / "manifest.json"
            if not manifest.is_file():
                continue
            payload = manifest.read_text(encoding="utf-8")
            try:
                parsed = json.loads(payload)
            except Exception as exc:  # noqa: BLE001
                raise ValueError(f"invalid module manifest in {repo_candidate.repo_id}: {entry.name}") from exc

            if not isinstance(parsed, dict):
                raise ValueError(
                    f"invalid module manifest in {repo_candidate.repo_id}: {entry.name} payload must be an object"
                )

            existing_repo = discovered.get(module_name)
            if existing_repo is not None:
                with open(existing_repo, encoding="utf-8") as existing_stream:
                    existing_payload = json.load(existing_stream)
                if _compatible_module_manifest(existing_payload, parsed):
                    if set(_manifest_payload_repo_ids(existing_payload) or []) < set(
                        _manifest_payload_repo_ids(parsed) or []
                    ):
                        discovered[module_name] = manifest
                        discovered_from[module_name] = repo_candidate.repo_id
                    continue
                if not dry_run:
                    raise ValueError(
                        f"conflicting manifests for module '{module_name}': "
                        f"{discovered_from[module_name]} and {repo_candidate.repo_id}"
                    )
                continue

            discovered[module_name] = manifest
            discovered_from[module_name] = repo_candidate.repo_id

    created: list[str] = []
    updated: list[str] = []
    skipped: list[str] = []

    for module_name, manifest in sorted(discovered.items()):
        destination_dir = destination / module_name
        destination_manifest = destination_dir / "manifest.json"
        payload_text = manifest.read_text(encoding="utf-8")
        if destination_manifest.exists() and not overwrite:
            skipped.append(module_name)
            continue
        if dry_run:
            if destination_manifest.exists():
                updated.append(module_name)
            else:
                created.append(module_name)
            continue

        destination_dir.mkdir(parents=True, exist_ok=True)
        parsed = json.loads(payload_text)
        is_new = not destination_manifest.exists()
        destination_manifest.write_text(
            json.dumps(parsed, sort_keys=True, indent=2) + "\n",
            encoding="utf-8",
        )
        if is_new:
            created.append(module_name)
        else:
            updated.append(module_name)

    return {
        "source_root": str(source),
        "destination_root": str(destination),
        "discovered_modules": sorted(discovered),
        "created": sorted(created),
        "updated": sorted(updated),
        "skipped": sorted(skipped),
        "dry_run": dry_run,
        "overwrite": overwrite,
    }


def build_catalog(target: str, repo_id: str | None = None, family: str | None = None) -> RunnerCatalog:
    runtime = read_dual(target, RUNTIME_FILE, family=family)
    materializations = runtime.get("repo_materializations")
    if not isinstance(materializations, list) or not materializations:
        raise ValueError("target has no runtime materialization; run target materialize")

    selected = materializations[0]
    if repo_id:
        selected = next((item for item in materializations if item.get("repo_id") == repo_id), None)
        if not selected:
            raise ValueError(f"repo_id not materialized: {repo_id}")
    checkout = Path(str(selected.get("checkout_path", ""))).resolve()
    catalog = build_runner_catalog(target, checkout)
    dual_write(target, RUNNER_FILE, catalog, family=family)
    return catalog


def list_targets(family: str | None = None) -> list[str]:
    if family is not None:
        root = projects_root() / validate_family_name(family)
        if not root.exists():
            return []
        return sorted(
            entry.name
            for entry in root.iterdir()
            if entry.is_dir() and (entry / ".phench" / LOCK_FILE).exists()
        )

    root = projects_root()
    if not root.exists():
        return []
    targets: list[str] = []
    for entry in root.iterdir():
        if not entry.is_dir():
            continue
        if (entry / ".phench" / LOCK_FILE).exists():
            targets.append(entry.name)
        else:
            targets.extend(_list_targets_in_root(entry, family_prefix=entry.name))
    unique: list[str] = sorted(set(targets), key=lambda value: ("/" in value, value))
    return unique


def list_modules() -> list[str]:
    root = projects_modules_root()
    if not root.exists():
        return []

    return sorted(
        entry.name
        for entry in root.iterdir()
        if entry.is_dir() and (entry / "manifest.json").is_file()
    )


def _list_targets_in_root(root: Path, family_prefix: str | None) -> list[str]:
    found: list[str] = []
    for entry in root.iterdir():
        if not entry.is_dir():
            continue
        if (entry / ".phench" / LOCK_FILE).exists():
            if family_prefix is None:
                found.append(entry.name)
            else:
                found.append(f"{family_prefix}/{entry.name}")
    return sorted(found)


def set_env_profile(target: str, profile: str, values: dict[str, str], family: str | None = None) -> dict[str, Any]:
    if not profile.strip():
        raise ValueError("profile name cannot be empty")
    normalized = {str(k): str(v) for k, v in values.items()}
    try:
        state = read_dual(target, PROFILE_FILE, family=family)
    except FileNotFoundError:
        state = {"active_profile": profile, "profiles": {}}
    profiles = state.get("profiles")
    if not isinstance(profiles, dict):
        profiles = {}
    profiles[profile] = normalized
    state = {"active_profile": profile, "profiles": profiles}
    dual_write(target, PROFILE_FILE, state, family=family)
    return state


def get_env_profile(target: str, profile: str | None = None, family: str | None = None) -> dict[str, str]:
    try:
        state = read_dual(target, PROFILE_FILE, family=family)
    except FileNotFoundError:
        return {}
    profiles = state.get("profiles")
    if not isinstance(profiles, dict):
        return {}
    selected = profile or str(state.get("active_profile", ""))
    payload = profiles.get(selected)
    if not isinstance(payload, dict):
        return {}
    return {str(key): str(value) for key, value in payload.items()}


def _run_single_repo_target(
    checkout_path: Path,
    catalog: RunnerCatalog,
    runner: str | None,
    command_name: str | None,
    non_interactive: bool = False,
    env_overrides: dict[str, str] | None = None,
) -> int:
    if command_name and not runner:
        raise ValueError("--command requires --runner")
    if runner and command_name:
        return run_command(checkout_path, runner, command_name, env_overrides=env_overrides)

    if runner and not command_name:
        options = [command for command in catalog.commands if command.runner == runner]
        if not options:
            raise ValueError(f"runner has no discovered commands: {runner}")
        return run_command(checkout_path, runner, options[0].name, env_overrides=env_overrides)

    if non_interactive:
        raise ValueError(
            "runner policy is required for non-interactive execution; pass --runner or configure preferred_runner"
        )

    selected = pick_command_interactive(catalog)
    return run_command(checkout_path, selected.runner, selected.name, env_overrides=env_overrides)


def _resolve_repo_runner_and_command(
    repo_id: str,
    repo: RepoSelection,
    catalog: RunnerCatalog,
    cli_runner: str | None,
    cli_command: str | None,
    repo_runner_override: str | None = None,
    repo_command_override: str | None = None,
    enforce_no_interactive: bool = False,
) -> tuple[str | None, str | None]:
    runner = repo_runner_override or cli_runner or repo.preferred_runner
    command_name = repo_command_override or cli_command or repo.preferred_command

    if command_name and not runner:
        raise ValueError(f"repo '{repo_id}' requires a runner for command '{command_name}'")

    if not runner:
        if enforce_no_interactive:
            raise ValueError(
                f"repo '{repo_id}' requires runner policy (--runner or preferred_runner) "
                "for non-interactive/all-repos execution."
            )
        return None, command_name

    if command_name:
        return runner, command_name

    available = [command for command in catalog.commands if command.runner == runner]
    if not available:
        raise ValueError(f"runner '{runner}' has no discovered commands in repo '{repo_id}'")
    if enforce_no_interactive and len(available) > 1:
        raise ValueError(
            f"repo '{repo_id}' with runner '{runner}' requires explicit command "
            "for non-interactive/all-repos execution."
        )
    return runner, available[0].name


def _materialization_entry(item: dict[str, Any]) -> tuple[str, Path]:
    repo_id = item.get("repo_id")
    checkout_path = item.get("checkout_path")
    if not isinstance(repo_id, str) or not repo_id.strip():
        raise ValueError("invalid runtime materialization entry: missing repo_id")
    if not isinstance(checkout_path, str) or not checkout_path.strip():
        raise ValueError(f"invalid runtime materialization entry for {repo_id}: missing checkout_path")
    return repo_id, Path(checkout_path).resolve()


def _materialization_checkouts(
    materializations: list[dict[str, Any]],
) -> list[Path]:
    return [checkout for _, checkout in (_materialization_entry(item) for item in materializations)]


def _run_env_doctor_for_materializations(
    target: str,
    materializations: list[dict[str, Any]],
    family: str | None = None,
) -> dict[str, Any]:
    report = run_env_doctor(target, _materialization_checkouts(materializations))
    dual_write(target, ENV_FILE, report, family=family)
    return asdict(report)


def _materialization_lookup(
    repo_id: str | None,
    repo_ids: list[str] | None,
    all_repos: bool,
    materializations: list[dict[str, Any]],
) -> list[dict[str, Any]]:
    if not materializations:
        raise ValueError("target has no runtime materialization; run target materialize")

    if all_repos:
        return [dict(item) for item in materializations]

    if repo_id is not None:
        selected = next((item for item in materializations if item.get("repo_id") == repo_id), None)
        if selected is None:
            raise ValueError(f"repo_id not materialized: {repo_id}")
        return [dict(selected)]

    if repo_ids is not None:
        index = {repo_id: item for item in materializations for repo_id in [item.get("repo_id")] if isinstance(repo_id, str)}
        if not index:
            raise ValueError("target has no runtime materialization; run target materialize")
        requested: list[str] = []
        seen: set[str] = set()
        for requested_repo_id in repo_ids:
            if requested_repo_id in seen:
                continue
            seen.add(requested_repo_id)
            if requested_repo_id not in index:
                raise ValueError(f"repo_id not materialized: {requested_repo_id}")
            requested.append(requested_repo_id)
        return [dict(index[rid]) for rid in requested]

    return [dict(item) for item in materializations[:1]]


def build_project_execution_matrix(
    target: str,
    family: str | None = None,
    snapshot_id: str | None = None,
    repo_id: str | None = None,
    repo_ids: list[str] | None = None,
    repo_ref_overrides: dict[str, str] | None = None,
    repo_runner_overrides: dict[str, str] | None = None,
    repo_command_overrides: dict[str, str] | None = None,
    repo_env_profile_overrides: dict[str, str] | None = None,
    runner: str | None = None,
    command_name: str | None = None,
    selected_ref: str | None = None,
    all_repos: bool = False,
    env_profile: str | None = None,
    non_interactive: bool = False,
    validate_commands: bool = False,
    sort_repos: bool = False,
) -> dict[str, Any]:
    lock_hash = None
    snapshot_hash = None
    runtime_hash = None

    if snapshot_id is not None:
        snapshot = show_target_snapshot(target, snapshot_id, family=family)
        runtime = snapshot.get("runtime")
        if not isinstance(runtime, dict):
            raise ValueError(f"snapshot '{snapshot_id}' has no runtime payload")
        materializations = runtime.get("repo_materializations")
        if not isinstance(materializations, list) or not materializations:
            raise ValueError(
                f"snapshot '{snapshot_id}' has no runtime materialization; create snapshot after materialize"
            )
        snapshot_lock = snapshot.get("lock")
        if not isinstance(snapshot_lock, dict):
            raise ValueError(f"snapshot '{snapshot_id}' has invalid lock payload")
        lock = _parse_lock(snapshot_lock)
        report = _run_env_doctor_for_materializations(target, materializations, family=family)
        lock_hash = str((snapshot_lock or {}).get("lock_hash", ""))
        snapshot_hash = str(snapshot.get("snapshot_hash", ""))
        runtime_hash = str(snapshot.get("runtime_hash", ""))
        if not runtime_hash:
            runtime_hash = _stable_payload_hash(runtime)
    else:
        report = run_env_doctor_for_target(target, family=family)
        runtime = read_dual(target, RUNTIME_FILE, family=family)
        materializations = runtime.get("repo_materializations")
        lock = load_target_lock(target, family=family)
        lock_hash = lock.lock_hash

    if report["doctor_status"] != "pass":
        missing = ", ".join(report["missing_requirements"])
        raise RuntimeError(f"env doctor failed, missing requirements: {missing}")

    if repo_id is not None and repo_ids is not None:
        raise ValueError("repo_id and repo_ids are mutually exclusive")
    if repo_ids is not None:
        repo_ids = [repo for repo in repo_ids if repo]
        if not repo_ids:
            raise ValueError("repo_ids cannot be empty")

    if repo_ref_overrides is None:
        repo_ref_overrides_normalized = {}
    else:
        repo_ref_overrides_normalized = _normalize_repo_map(repo_ref_overrides, label="repo_ref_overrides")

    repo_runner_overrides_normalized = _normalize_repo_map(
        repo_runner_overrides,
        label="repo_runner_overrides",
    )
    repo_command_overrides_normalized = _normalize_repo_map(
        repo_command_overrides,
        label="repo_command_overrides",
    )
    repo_env_profile_overrides_normalized = _normalize_repo_map(
        repo_env_profile_overrides,
        label="repo_env_profile_overrides",
    )

    if all_repos and repo_ids is not None:
        raise ValueError("repo_ids is not compatible with --all-repos")

    if repo_id is not None and not repo_id.strip():
        raise ValueError("repo_id cannot be empty")

    if repo_ids is not None:
        repo_index = {item.strip(): item.strip() for item in repo_ids}
        repo_ids = list(repo_index.values())
        unknown_override = [item for item in repo_ref_overrides_normalized if item not in repo_index]
        if unknown_override:
            raise ValueError(f"repo_id not materialized: {unknown_override[0]}")
        unknown_runner_override = [item for item in repo_runner_overrides_normalized if item not in repo_index]
        if unknown_runner_override:
            raise ValueError(f"repo_id not materialized: {unknown_runner_override[0]}")
        unknown_command_override = [item for item in repo_command_overrides_normalized if item not in repo_index]
        if unknown_command_override:
            raise ValueError(f"repo_id not materialized: {unknown_command_override[0]}")
        unknown_env_profile_override = [
            item
            for item in repo_env_profile_overrides_normalized
            if item not in repo_index
        ]
        if unknown_env_profile_override:
            raise ValueError(f"repo_id not materialized: {unknown_env_profile_override[0]}")

    if not isinstance(materializations, list) or not materializations:
        raise ValueError("target has no runtime materialization; run target materialize")

    if all_repos:
        selected_items = [dict(item) for item in materializations]
    else:
        selected_items = _materialization_lookup(
            repo_id,
            repo_ids=repo_ids,
            all_repos=False,
            materializations=materializations,
        )

    if repo_id is not None:
        repo_ids = [repo_id]

    lock_index = {entry.repo_id: entry for entry in lock.repos}
    enforce_no_interactive = non_interactive or all_repos

    plans: list[dict[str, Any]] = []
    for item in selected_items:
        item_repo_id, item_checkout = _materialization_entry(item)
        lock_repo = lock_index.get(item_repo_id)
        if lock_repo is None:
            raise ValueError(f"repo_id not in target lock: {item_repo_id}")

        if (repo_ref := repo_ref_overrides_normalized.get(item_repo_id)) is not None:
            ref_source = "repo_override"
        elif selected_ref is not None:
            repo_ref = selected_ref
            ref_source = "cli_ref"
        elif lock_repo.preferred_ref is not None:
            repo_ref = lock_repo.preferred_ref
            ref_source = "preferred_ref"
        elif lock_repo.selected_ref is not None:
            repo_ref = lock_repo.selected_ref
            ref_source = "selected_ref"
        else:
            repo_ref = None
            ref_source = "materialized_ref"

        if repo_ref is not None:
            resolved = resolve_ref_to_sha(Path(lock_repo.repo_path), repo_ref)
            materialize_repo_checkout(Path(lock_repo.repo_path), item_checkout, resolved)
            item["resolved_sha"] = resolved
        else:
            resolved = item.get("resolved_sha")

        repo_runner_override = repo_runner_overrides_normalized.get(item_repo_id)
        repo_command_override = repo_command_overrides_normalized.get(item_repo_id)
        repo_env_profile = repo_env_profile_overrides_normalized.get(item_repo_id, env_profile)
        env_overrides = get_env_profile(target, profile=repo_env_profile, family=family)

        if validate_commands:
            catalog = build_runner_catalog(target, item_checkout)
            repo_runner, repo_command = _resolve_repo_runner_and_command(
                item_repo_id,
                lock_repo,
                catalog,
                cli_runner=runner,
                cli_command=command_name,
                repo_runner_override=repo_runner_override,
                repo_command_override=repo_command_override,
                enforce_no_interactive=enforce_no_interactive,
            )
        else:
            repo_runner = repo_runner_override or runner or lock_repo.preferred_runner
            repo_command = repo_command_override or command_name or lock_repo.preferred_command
            if repo_command and not repo_runner:
                raise ValueError(f"repo '{item_repo_id}' requires a runner for command '{repo_command}'")

        plans.append(
            {
                "repo_id": item_repo_id,
                "repo_path": lock_repo.repo_path,
                "checkout_path": item["checkout_path"],
                "lock_selected_ref": lock_repo.selected_ref,
                "lock_preferred_ref": lock_repo.preferred_ref,
                "effective_ref": repo_ref,
                "effective_ref_source": ref_source,
                "resolved_sha": resolved,
                "effective_runner": repo_runner,
                "effective_command": repo_command,
                "effective_env_profile": repo_env_profile,
                "env_overrides": env_overrides,
                "preferred_runner": lock_repo.preferred_runner,
                "preferred_command": lock_repo.preferred_command,
            }
        )

    if sort_repos:
        plans.sort(key=lambda item: str(item["repo_id"]))

    return {
        "target": target,
        "family": family,
        "lock_hash": lock_hash,
        "snapshot_hash": snapshot_hash,
        "runtime_hash": runtime_hash,
        "snapshot_id": snapshot_id,
        "all_repos": all_repos,
        "non_interactive": non_interactive,
        "repo_count": len(plans),
        "repos": plans,
    }


def _validate_module_repos_payload_field(
    payload: dict[str, Any],
    *,
    field: str,
) -> list[str]:
    raw = payload.get(field)
    if raw is None:
        return []

    if not isinstance(raw, list):
        raise ValueError(f"module manifest field '{field}' must be a list")

    values: list[str] = []
    for value in raw:
        if not isinstance(value, str):
            raise ValueError(f"module manifest field '{field}' contains non-string entry")
        repo_id = value.strip()
        if not repo_id:
            continue
        values.append(repo_id)
    return values


def _validate_module_repos_payload_map(
    payload: dict[str, Any],
    *,
    field: str,
) -> dict[str, str]:
    raw = payload.get(field)
    if raw is None:
        return {}

    if not isinstance(raw, dict):
        raise ValueError(f"module manifest field '{field}' must be an object")

    items: dict[str, str] = {}
    for key, value in raw.items():
        if not isinstance(key, str):
            raise ValueError(f"module manifest field '{field}' contains non-string key")

        repo_id = key.strip()
        if not repo_id:
            raise ValueError(f"module manifest field '{field}' contains empty repo id")

        if not isinstance(value, str):
            raise ValueError(f"module manifest field '{field}' requires string values")
        item = value.strip()
        if not item:
            raise ValueError(
                f"module manifest field '{field}' for repo '{repo_id}' must not be empty"
            )

        items[repo_id] = item

    return items


def _validate_module_manifest_schema_version(
    module: str,
    payload: dict[str, Any],
) -> int:
    schema_version = payload.get("schema_version")
    if schema_version is None:
        return 1
    if not isinstance(schema_version, int) or isinstance(schema_version, bool):
        raise ValueError(
            f"module manifest '{module}' schema_version must be an integer"
        )

    if schema_version not in SUPPORTED_MODULE_MANIFEST_SCHEMA_VERSIONS:
        raise ValueError(
            f"module manifest '{module}' has unsupported schema_version: {schema_version}"
        )

    return schema_version


def _normalize_repo_id_list(values: list[str] | None) -> list[str]:
    if values is None:
        return []

    normalized: list[str] = []
    seen: set[str] = set()
    for value in values:
        repo_id = value.strip()
        if not repo_id:
            continue
        if repo_id in seen:
            continue
        seen.add(repo_id)
        normalized.append(repo_id)
    return normalized


def _validate_module_owners(payload: dict[str, Any], *, field: str) -> list[str]:
    raw = payload.get(field)
    if raw is None:
        return []

    if not isinstance(raw, list):
        raise ValueError(f"module manifest field '{field}' must be a list")

    owners: list[str] = []
    seen: set[str] = set()
    for item in raw:
        if not isinstance(item, str):
            raise ValueError(f"module manifest field '{field}' contains non-string entry")
        owner = item.strip()
        if not owner:
            continue
        normalized_owner = owner.lower()
        if normalized_owner in seen:
            continue
        seen.add(normalized_owner)
        owners.append(normalized_owner)
    return owners


def _validate_module_refresh_cadence(payload: dict[str, Any], *, field: str) -> str:
    raw = payload.get(field, DEFAULT_MODULE_REFRESH_CADENCE)
    if not isinstance(raw, str):
        raise ValueError(f"module manifest field '{field}' must be a string")
    cadence = raw.strip().lower()
    if not cadence:
        raise ValueError(f"module manifest field '{field}' cannot be empty")
    if not _REFRESH_CADENCE_RE.match(cadence):
        raise ValueError(
            f"module manifest field '{field}' must be one of "
            f"never, manual, daily, weekly, monthly, yearly, hourly, every-<duration>"
        )
    return cadence


def _normalize_repo_map(values: dict[str, str] | None, *, label: str) -> dict[str, str]:
    if values is None:
        return {}
    normalized: dict[str, str] = {}
    for repo_id, value in values.items():
        if not isinstance(repo_id, str):
            raise ValueError(f"{label} keys must be strings")
        normalized_key = repo_id.strip()
        if not normalized_key:
            raise ValueError(f"{label} contains empty key")

        if not isinstance(value, str):
            raise ValueError(f"{label} values must be strings")
        normalized_value = value.strip()
        if not normalized_value:
            raise ValueError(f"{label} value for repo '{normalized_key}' cannot be empty")
        normalized[normalized_key] = normalized_value
    return normalized


def _load_module_manifest_dataclass(module: str) -> ModuleManifest:
    manifest_path = _resolve_module_manifest_path(module)
    try:
        payload = json.loads(manifest_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        raise ValueError(f"invalid module manifest for {module}: malformed json") from exc

    if not isinstance(payload, dict):
        raise ValueError(f"invalid module manifest for {module}: payload must be an object")

    schema_version = int(payload.get("schema_version", 1))
    if schema_version not in {1}:
        raise ValueError(f"unsupported schema version {schema_version} for manifest: {module}")

    owners: list[str] = []
    if "owners" in payload:
        _validate_module_owners(payload, field="owners")
        raw_owners = payload["owners"]
        if isinstance(raw_owners, list):
            seen: set[str] = set()
            for o in raw_owners:
                normalized = o.lower().strip()
                if normalized and normalized not in seen:
                    seen.add(normalized)
                    owners.append(normalized)

    refresh_cadence: str | None = None
    if "refresh_cadence" in payload:
        _validate_module_refresh_cadence(payload, field="refresh_cadence")
        refresh_cadence = str(payload["refresh_cadence"]).lower().strip()

    raw_patterns = payload.get("repo_patterns")
    if raw_patterns is None:
        repo_patterns: list[str] = ["*"]
    elif isinstance(raw_patterns, list) and all(isinstance(p, str) for p in raw_patterns):
        repo_patterns = raw_patterns
    else:
        raise ValueError(f"invalid repo_patterns in manifest: {module}")

    # Compute repo_ids from patterns against candidates
    temp_manifest = ModuleManifest(schema_version=schema_version, repo_patterns=repo_patterns)
    selected_repos = _select_module_repos(temp_manifest)
    repo_ids = sorted(r.name for r in selected_repos)

    def _load_str_dict(key: str) -> dict[str, str]:
        raw = payload.get(key)
        if raw is None:
            return {}
        if not isinstance(raw, dict):
            raise ValueError(f"invalid {key} in manifest: {module}")
        return {str(k): str(v) for k, v in raw.items() if isinstance(k, str) and isinstance(v, str)}

    return ModuleManifest(
        schema_version=schema_version,
        repo_patterns=repo_patterns,
        owners=owners,
        refresh_cadence=refresh_cadence,
        default_ref=str(payload.get("default_ref", "HEAD")),
        repo_ids=repo_ids,
        repo_ref_overrides=_load_str_dict("repo_ref_overrides"),
        repo_runner_overrides=_load_str_dict("repo_runner_overrides"),
        repo_command_overrides=_load_str_dict("repo_command_overrides"),
        repo_env_profile_overrides=_load_str_dict("repo_env_profile_overrides"),
    )


def load_module_repos(
    module: str,
    *,
    available_repo_ids: list[str] | None = None,
) -> list[str]:
    return load_module_manifest(module, available_repo_ids=available_repo_ids)["repo_ids"]


def run_target(
    target: str,
    family: str | None = None,
    snapshot_id: str | None = None,
    repo_id: str | None = None,
    repo_ids: list[str] | None = None,
    repo_ref_overrides: dict[str, str] | None = None,
    repo_runner_overrides: dict[str, str] | None = None,
    repo_command_overrides: dict[str, str] | None = None,
    repo_env_profile_overrides: dict[str, str] | None = None,
    runner: str | None = None,
    command_name: str | None = None,
    selected_ref: str | None = None,
    all_repos: bool = False,
    execution_mode: str = "serial",
    env_profile: str | None = None,
    non_interactive: bool = False,
) -> int:
    matrix = build_project_execution_matrix(
        target=target,
        family=family,
        snapshot_id=snapshot_id,
        repo_id=repo_id,
        repo_ids=repo_ids,
        repo_ref_overrides=repo_ref_overrides,
        repo_runner_overrides=repo_runner_overrides,
        repo_command_overrides=repo_command_overrides,
        repo_env_profile_overrides=repo_env_profile_overrides,
        runner=runner,
        command_name=command_name,
        selected_ref=selected_ref,
        all_repos=all_repos,
        env_profile=env_profile,
        non_interactive=non_interactive,
        validate_commands=True,
        sort_repos=False,
    )

    if execution_mode not in {"serial", "parallel"}:
        raise ValueError("execution_mode must be one of: serial, parallel")

    if command_name is not None and command_name.startswith("-"):
        raise ValueError("command names cannot start with '-'")

    runs: list[tuple[Path, RunnerCatalog, str | None, str | None, dict[str, str] | None]] = []
    for item in matrix["repos"]:
        checkout_path = Path(item["checkout_path"])
        catalog = build_runner_catalog(target, checkout_path)
        runs.append(
            (
                checkout_path,
                catalog,
                item["effective_runner"],
                item["effective_command"],
                item.get("env_overrides"),
            )
        )

    if execution_mode == "parallel" and len(runs) > 1:
        with ThreadPoolExecutor(max_workers=len(runs)) as pool:
            futures = [
                pool.submit(
                    _run_single_repo_target,
                    checkout_path,
                    effective_catalog,
                    effective_runner,
                    effective_command,
                    non_interactive,
                    env_overrides,
                )
                for checkout_path, effective_catalog, effective_runner, effective_command, env_overrides in runs
            ]
            results = [future.result() for future in futures]
        nonzero = [code for code in results if code != 0]
        return nonzero[0] if nonzero else 0

    for checkout, effective_catalog, effective_runner, effective_command, env_overrides in runs:
        code = _run_single_repo_target(
            checkout,
            effective_catalog,
            effective_runner,
            effective_command,
            non_interactive,
            env_overrides,
        )
        if code != 0:
            return code
    return 0


def run_env_doctor_for_target(target: str, family: str | None = None) -> dict[str, Any]:
    runtime = read_dual(target, RUNTIME_FILE, family=family)
    materializations = runtime.get("repo_materializations")
    if not isinstance(materializations, list) or not materializations:
        raise ValueError("target has no runtime materialization; run target materialize")
    checkouts = [Path(str(item.get("checkout_path", ""))).resolve() for item in materializations]
    report = run_env_doctor(target, checkouts)
    dual_write(target, ENV_FILE, report, family=family)
    return asdict(report)


def sync_target(target: str, prefer: str | None = None, family: str | None = None) -> dict[str, Any]:
    results = {}
    for filename in (LOCK_FILE, RUNTIME_FILE, ENV_FILE, RUNNER_FILE, PROFILE_FILE):
        try:
            results[filename] = sync_dual(target, filename, prefer=prefer, family=family)
        except FileNotFoundError:
            continue
    if not results:
        raise FileNotFoundError("No state files to sync")
    return results
def _normalize_candidate_module_name(value: str) -> str:
    slug = "".join(ch.lower() if ch.isalnum() else "-" for ch in value.strip())
    while "--" in slug:
        slug = slug.replace("--", "-")
    slug = slug.strip("-")
    return slug or "module"


def _truncate_module_name(value: str, *, max_length: int = SCAN_SHARED_REPOS_MAX_NAME_LENGTH) -> str:
    return value[:max_length] if len(value) > max_length else value


def _candidate_conflict_suffix(value: str) -> str:
    return hashlib.sha1(value.encode("utf-8")).hexdigest()[:8]


def _build_candidate_name(base: str, *, suffix: str | None = None) -> str:
    if suffix:
        suffix_fragment = f"-{suffix}"
        allowed = max(SCAN_SHARED_REPOS_MAX_NAME_LENGTH - len(suffix_fragment), 1)
        base_prefix = _truncate_module_name(base, max_length=allowed)
        return f"{base_prefix}{suffix_fragment}"
    return _truncate_module_name(base)


def _build_scannable_candidate_name(base: str, used_names: set[str], *, max_attempts: int = 25) -> str:
    attempt = 0
    candidate_name = _build_candidate_name(base)
    while candidate_name in used_names:
        attempt += 1
        if attempt > max_attempts:
            raise RuntimeError(f"unable to resolve unique candidate name for: {base}")
        candidate_name = _build_candidate_name(base, suffix=_candidate_conflict_suffix(f"{base}:{attempt}"))
    return candidate_name


def _build_recommended_modules(
    shared_modules: dict[str, list[str]],
    depends_on_by_module: dict[str, set[str]] | None = None,
) -> list[dict[str, Any]]:
    recommendations: list[dict[str, Any]] = []
    for module_name, repos in shared_modules.items():
        if not repos:
            continue
        depends_on = sorted(depends_on_by_module.get(module_name, set())) if depends_on_by_module else []
        recommendations.append(
            {
                "module": module_name,
                "repo_count": len(repos),
                "repo_ids": sorted(repos),
                "depends_on_count": len(depends_on),
                "depends_on": depends_on,
            }
        )

    return sorted(
        recommendations,
        key=lambda item: (-item["repo_count"], -item["depends_on_count"], item["module"]),
    )


def _append_warning(warnings: list[str], message: str) -> None:
    if message not in warnings:
        warnings.append(message)


def _sorted_repo_paths(repo_paths: dict[str, str]) -> dict[str, str]:
    return {repo_id: repo_paths[repo_id] for repo_id in sorted(repo_paths)}


def _read_index_payload(path: Path) -> list[dict[str, Any]]:
    if not path.exists():
        return []
    existing = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(existing, list):
        raise ValueError(f"invalid manifest index format: {path}")
    payload: list[dict[str, Any]] = []
    for item in existing:
        if not isinstance(item, dict):
            continue
        payload.append(item)
    return payload


def _write_json_lines(path: Path, payload: dict[str, Any]) -> None:
    _ensure_non_symlink_path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("a", encoding="utf-8") as stream:
        stream.write(json.dumps(payload) + "\n")


def _build_candidate_manifest_index_summary(entries: list[dict[str, Any]]) -> dict[str, Any]:
    module_names = sorted({str(item.get("module_name", "")) for item in entries if isinstance(item, dict)})
    module_manifest_paths = {
        item.get("module_name", ""): str(item.get("manifest_path", ""))
        for item in entries
        if isinstance(item, dict) and item.get("module_name")
    }
    return {
        "generated_at": utc_now_iso(),
        "module_count": len(module_names),
        "manifest_count": len(entries),
        "module_names": module_names,
        "module_manifest_paths": module_manifest_paths,
    }


def _validate_candidate_name_regex(pattern: str) -> re.Pattern[str]:
    try:
        return re.compile(pattern)
    except re.error as exc:
        raise ValueError(f"invalid candidate-name regex: {pattern}") from exc


def _iter_repo_python_files(repo_path: Path) -> list[Path]:
    files: list[Path] = []
    src_root = repo_path / "src"
    if not src_root.exists():
        return files
    for file_path in src_root.rglob("*.py"):
        if file_path.is_file():
            files.append(file_path)
    return files


def _extract_imports_from_python(code: str) -> set[str]:
    imports: set[str] = set()
    try:
        tree = ast.parse(code)
    except SyntaxError as exc:
        raise ValueError(str(exc)) from exc

    for node in ast.walk(tree):
        if isinstance(node, ast.Import):
            for alias in node.names:
                if alias.name:
                    imports.add(alias.name)
        elif isinstance(node, ast.ImportFrom):
            # Skip relative imports (indicated by node.level > 0)
            if node.level > 0:
                continue
            if node.module:
                imports.add(node.module)
    return imports


def _ensure_non_symlink_path(path: Path) -> None:
    current = path
    while current != current.parent:
        if current.exists() and current.is_symlink():
            raise ValueError(f"path component is a symlink: {current}")
        current = current.parent


def _resolve_candidate_dependency(
    imported_name: str,
    known_modules: set[str],
    current_module: str,
) -> str | None:
    parts = imported_name.split(".")
    if not parts:
        return None
    candidate = parts[0]
    if candidate == current_module:
        return None
    if candidate in known_modules:
        return candidate
    return None


def _find_repo_module_dependencies(
    repo_path: Path,
) -> tuple[dict[str, set[str]], list[dict[str, str]]]:
    modules = _find_repo_modules(repo_path)
    if not modules:
        return {}, []

    dependencies: dict[str, set[str]] = {module: set() for module in modules}
    warnings: list[dict[str, str]] = []

    for py_file in _iter_repo_python_files(repo_path):
        try:
            imports = _extract_imports_from_python(py_file.read_text(encoding="utf-8"))
        except (OSError, ValueError) as exc:
            warnings.append(
                {
                    "repo_file": str(py_file),
                    "reason": "python-parse-error",
                    "detail": str(exc),
                }
            )
            continue

        module_name = py_file.relative_to(repo_path).parts
        if len(module_name) < 2 or module_name[0] != "src":
            continue
        current_module = module_name[1]
        if current_module not in modules:
            continue
        for imported in imports:
            dep = _resolve_candidate_dependency(imported, modules, current_module)
            if dep is not None:
                dependencies[current_module].add(dep)

    return dependencies, warnings


def build_module_manifest_payload(module_name: str, repo_ids: list[str]) -> dict[str, Any]:
    sorted_repos = sorted(repo_ids)
    return {
        "schema_version": SCAN_SHARED_REPOS_SCHEMA_VERSION,
        "repo_patterns": sorted_repos,
        "default_ref": "HEAD",
        "repo_ref_overrides": {},
        "repo_runner_overrides": {},
        "repo_command_overrides": {},
        "repo_env_profile_overrides": {},
        "matched_repos": sorted_repos,
    }


def build_scan_candidates(
    shared_modules: dict[str, list[str]],
    *,
    module_prefix: str = SCAN_SHARED_REPOS_DEFAULT_MODULE_PREFIX,
    depends_on_by_module: dict[str, set[str]] | None = None,
) -> list[dict[str, Any]]:
    candidates: list[dict[str, Any]] = []
    used_names: set[str] = set()

    for module_name, repos in sorted(shared_modules.items(), key=lambda item: item[0]):
        if not repos:
            continue
        sorted_repos = sorted(repos)
        prefix = _normalize_candidate_module_name(module_prefix)
        base = _normalize_candidate_module_name(module_name)
        candidate_name = _build_scannable_candidate_name(f"{prefix}-{base}-{len(sorted_repos)}", used_names)
        used_names.add(candidate_name)

        depends_on = sorted(depends_on_by_module.get(module_name, set())) if depends_on_by_module else []

        manifest_template = build_module_manifest_payload(module_name, sorted_repos)
        candidates.append(
            {
                "module": module_name,
                "module_name": candidate_name,
                "repo_ids": sorted_repos,
                "repo_count": len(sorted_repos),
                "depends_on_count": len(depends_on),
                "depends_on": depends_on,
                "manifest_template": manifest_template,
            }
        )

    return sorted(
        candidates,
        key=lambda item: (
            -item["repo_count"],
            -item["depends_on_count"],
            item["module"],
            item["module_name"],
        ),
    )


def _candidate_manifest_path(output_dir: Path, module_name: str) -> Path:
    return output_dir / module_name / "manifest.json"


def _read_json_dict(path: Path) -> dict[str, Any]:
    payload = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(payload, dict):
        raise ValueError(f"manifest content must be a JSON object: {path}")
    return payload


def _write_manifest(path: Path, payload: dict[str, Any]) -> None:
    _ensure_non_symlink_path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")


def materialize_scan_candidate_manifest(
    candidate: dict[str, Any],
    *,
    output_dir: Path,
    dry_run: bool = False,
    update_index: bool = True,
) -> dict[str, Any]:
    module_name = str(candidate.get("module_name", "")).strip()
    repo_ids = candidate.get("repo_ids", [])
    if not module_name:
        raise ValueError("candidate module_name cannot be empty")
    if not isinstance(repo_ids, list):
        raise ValueError("candidate repo_ids must be a list")
    if not isinstance(module_name, str) or "/" in module_name or "\\" in module_name or ".." in module_name:
        raise ValueError(f"invalid module_name: {module_name}")

    manifest_path = _candidate_manifest_path(output_dir, module_name)
    manifest_before: dict[str, Any] | None = None
    if manifest_path.exists():
        manifest_before = _read_json_dict(manifest_path)
    manifest_payload = candidate.get("manifest_template")
    if not isinstance(manifest_payload, dict):
        manifest_payload = build_module_manifest_payload(
            str(candidate.get("module", "")), [str(repo_id) for repo_id in repo_ids]
        )

    if not dry_run and manifest_before is not None:
        if manifest_before != manifest_payload:
            raise ValueError(f"manifest conflict for {module_name}; existing file differs: {manifest_path}")

    if not dry_run:
        _write_manifest(manifest_path, manifest_payload)

    index_path = output_dir / SCAN_SHARED_REPOS_MANIFEST_INDEX_FILENAME
    existing_index = _read_index_payload(index_path) if update_index else []
    index_entry = {
        "module_name": module_name,
        "manifest_path": str(manifest_path),
        "repo_count": len(repo_ids),
        "generated_at": utc_now_iso(),
    }
    next_index = deepcopy(existing_index)
    if update_index:
        updated = False
        for entry in next_index:
            if not isinstance(entry, dict):
                continue
            if entry.get("module_name") == module_name:
                entry.update(deepcopy(index_entry))
                updated = True
                break
        if not updated:
            next_index.append(index_entry)
        next_index = sorted(next_index, key=lambda item: str(item.get("module_name")))

    index_summary_path = output_dir / SCAN_SHARED_REPOS_MANIFEST_INDEX_SUMMARY_FILENAME
    index_summary_payload = _build_candidate_manifest_index_summary(next_index if update_index else existing_index)

    audit_path = output_dir / SCAN_SHARED_REPOS_MANIFEST_AUDIT_FILENAME
    _ensure_non_symlink_path(manifest_path)
    _ensure_non_symlink_path(index_path)
    _ensure_non_symlink_path(index_summary_path)
    _ensure_non_symlink_path(audit_path)
    audit_entry = {
        "event": "manifest-materialization",
        "timestamp": utc_now_iso(),
        "module": str(candidate.get("module", "")),
        "module_name": module_name,
        "manifest_path": str(manifest_path),
        "repo_ids": sorted([str(repo_id) for repo_id in repo_ids]),
        "dry_run": dry_run,
        "index_updated": bool(update_index),
    }

    if not dry_run and update_index:
        index_path.write_text(json.dumps(next_index, indent=2) + "\n", encoding="utf-8")
        index_summary_path.write_text(json.dumps(index_summary_payload, indent=2) + "\n", encoding="utf-8")
        _write_json_lines(audit_path, audit_entry)

    result = {
        "module": str(candidate.get("module", "")),
        "module_name": module_name,
        "manifest_path": str(manifest_path),
        "repos": sorted([str(repo_id) for repo_id in repo_ids]),
        "manifest_payload": manifest_payload,
        "manifest_after": manifest_payload,
        "dry_run": dry_run,
        "manifest_before": manifest_before,
        "index_before": existing_index if update_index else None,
        "index_after": next_index if update_index else None,
        "index_summary": index_summary_payload,
        "index_summary_path": str(index_summary_path),
    }

    return result


def _resolve_candidate_inclusion(
    *,
    candidates: bool,
    omit_candidates: bool,
) -> bool:
    if candidates and omit_candidates:
        raise ValueError("cannot set both --candidates and --omit-candidates")
    return bool(candidates) and not omit_candidates


def _repo_id_from_path(repo_path: Path) -> str:
    return sanitize_repo_id(repo_path.name)


def _validate_excluded_repo_ids(exclude_repos: Iterable[str]) -> set[str]:
    normalized: set[str] = set()
    for repo_name in exclude_repos:
        candidate = repo_name.strip()
        if not candidate:
            raise ValueError("exclude repo id cannot be blank")
        sanitized = sanitize_repo_id(candidate)
        if sanitized != candidate:
            raise ValueError(f"invalid repo id: {candidate}")
        normalized.add(sanitized)
    return normalized


def _collect_candidate_repos(root: Path) -> list[Path]:
    if not root.exists():
        return []
    if not root.is_dir():
        return []
    return sorted([entry for entry in root.iterdir() if entry.is_dir() and not entry.name.startswith(".")])


def _resolve_worktree_roots() -> list[Path]:
    base_root = phenotype_root()
    return sorted([entry for entry in base_root.iterdir() if entry.is_dir() and entry.name.endswith("-wtrees")])


def _collect_worktree_repos(root: Path) -> list[Path]:
    if root.name == "src":
        return [root.parent]
    if root.name.endswith("-wtrees"):
        return _collect_candidate_repos(root)
    if (root / "src").exists():
        return [root]
    return _collect_candidate_repos(root)


def _repos_root_metadata(
    repos_root: Path | None,
    root_mode: str,
) -> tuple[Path, list[Path], str, list[str]]:
    if root_mode not in {SCAN_SHARED_REPOS_ROOT_MODE_REPOS, SCAN_SHARED_REPOS_ROOT_MODE_WORKTREES}:
        raise ValueError(
            f"repos_root_mode must be one of: {SCAN_SHARED_REPOS_ROOT_MODE_REPOS}, {SCAN_SHARED_REPOS_ROOT_MODE_WORKTREES}"
        )

    warnings: list[str] = []
    phenotype = phenotype_root()
    base_root = repos_root if repos_root is not None else None
    if base_root is None:
        if root_mode == SCAN_SHARED_REPOS_ROOT_MODE_REPOS:
            base_root = phenotype / "repos"
            hint = (
                f"defaulted repos_root to {base_root} using repos_root_mode={root_mode}; pass --repos-root to override"
            )
            return base_root, _collect_candidate_repos(base_root), hint, []

        worktree_roots = _resolve_worktree_roots()
        candidate_paths: list[Path] = []
        for worktree_root in worktree_roots:
            candidate_paths.extend(_collect_worktree_repos(worktree_root))
        hint = "resolved worktree scan roots from phenotype_root/*-wtrees"
        if not candidate_paths:
            _append_warning(warnings, "no worktree roots found under phenotype_root; verify -wtrees directories exist")
        return phenotype, sorted(candidate_paths), hint, warnings

    explicit_root = base_root.expanduser().resolve()
    if not explicit_root.exists():
        _append_warning(warnings, f"repos_root does not exist: {explicit_root}")
        return explicit_root, [], f"explicit repos_root provided but path missing: {explicit_root}", warnings
    if root_mode == SCAN_SHARED_REPOS_ROOT_MODE_REPOS:
        hint = f"scanning repos mode from explicit root: {explicit_root}"
        if phenotype not in explicit_root.parents and explicit_root != phenotype:
            _append_warning(
                warnings,
                "scan configured outside THGENT_PHENOTYPE_ROOT; explicit repo path provenance retained in result",
            )
        return explicit_root, _collect_candidate_repos(explicit_root), hint, warnings

    hint = f"scanning worktrees from explicit root: {explicit_root}"
    if explicit_root.name.endswith("-wtrees"):
        _append_warning(warnings, "scanning nested worktree directory with -wtrees suffix")
    if (explicit_root / "src").exists() and explicit_root.name != "src":
        hint = f"explicit worktree-style repo root provided: {explicit_root}"
    return explicit_root, _collect_worktree_repos(explicit_root), hint, warnings


def _select_module_repos(
    manifest: ModuleManifest,
    exclude_repos: set[str] | None = None,
) -> list[Path]:
    excluded = {sanitize_repo_id(repo_name) for repo_name in (exclude_repos or set())}
    selected: list[Path] = []
    for repo_path in repository_root_candidates():
        repo_id = _repo_id_from_path(repo_path)
        if repo_id in DEFAULT_EXCLUDED_REPOS or repo_id in excluded:
            continue
        if not any(fnmatch(repo_id, pattern) for pattern in manifest.repo_patterns):
            continue
        # Verify directory is a git checkout
        if not (repo_path / ".git").exists():
            continue
        selected.append(repo_path)
    return selected


def _find_repo_modules(repo_path: Path) -> set[str]:
    src_root = repo_path / "src"
    if not src_root.exists() or not src_root.is_dir():
        return set()

    modules: set[str] = set()
    for child in src_root.iterdir():
        if not child.is_dir():
            continue
        if not (child / "__init__.py").exists():
            continue
        modules.add(child.name)
    return modules


def _collect_shared_modules(
    repo_modules: dict[str, set[str]],
    *,
    min_repo_count: int = 2,
) -> dict[str, list[str]]:
    return {module_name: sorted(repos) for module_name, repos in repo_modules.items() if len(repos) >= min_repo_count}


def _append_repo_selection(
    lock: TargetLock,
    repo_path: Path,
    selected_ref: str,
    *,
    module_name: str | None = None,
    selected_runner: str | None = None,
    selected_command: str | None = None,
    selected_env_profile: str | None = None,
    repo_id: str | None = None,
    worktree_path: str | None = None,
) -> None:
    rid = repo_id or _repo_id_from_path(repo_path)
    lock.repos = [entry for entry in lock.repos if entry.repo_id != rid]
    lock.repos.append(
        RepoSelection(
            repo_id=rid,
            repo_path=str(repo_path.resolve()),
            selected_ref=selected_ref,
            module_name=module_name,
            selected_runner=selected_runner,
            selected_command=selected_command,
            selected_env_profile=selected_env_profile,
            source_worktree_path=worktree_path,
            resolved_sha=None,
        )
    )


def scan_shared_modules_across_repos(
    repos_root: Path | None = None,
    *,
    exclude_repos: set[str] | None = None,
    min_repo_count: int = 2,
    repos_root_mode: str | None = None,
    candidate_name_regex: str | None = None,
    candidates: bool = False,
    omit_candidates: bool = False,
) -> dict[str, Any]:
    if min_repo_count < 2:
        raise ValueError("min_repo_count must be at least 2")

    root_mode = repos_root_mode or SCAN_SHARED_REPOS_ROOT_MODE_REPOS
    root, candidate_paths, root_mode_hint, warnings = _repos_root_metadata(repos_root, root_mode)
    if not candidate_paths:
        return {
            "scan_schema_version": SCAN_SHARED_REPOS_SCHEMA_VERSION,
            "repos_root": str(root),
            "repos_root_mode": root_mode,
            "root_mode_hint": root_mode_hint,
            "warnings": warnings,
            "repo_paths": {},
            "shared_modules": {},
            "shared_count": 0,
            "module_count": 0,
            "repo_count": 0,
            "excluded_repos": sorted(DEFAULT_EXCLUDED_REPOS),
            "examined_repos": [],
            "min_repo_count": min_repo_count,
            "module_candidates": [],
            "recommended_modules": [],
        }

    excluded = _validate_excluded_repo_ids(exclude_repos or set())
    effective_excludes = sorted(DEFAULT_EXCLUDED_REPOS | excluded)
    repo_modules: dict[str, set[str]] = {}
    repo_dependencies: dict[str, set[str]] = {}
    warnings = list(warnings)

    if candidate_name_regex is not None:
        compiled_name_regex = _validate_candidate_name_regex(candidate_name_regex)
    else:
        compiled_name_regex = None

    examined_repos: list[str] = []
    repo_paths: dict[str, str] = {}
    for repo_path in candidate_paths:
        repo_id = _repo_id_from_path(repo_path)
        if repo_id in effective_excludes:
            continue
        examined_repos.append(repo_id)
        repo_paths[repo_id] = str(repo_path)
        modules = _find_repo_modules(repo_path)
        for module_name in modules:
            repo_modules.setdefault(module_name, set()).add(repo_id)

        module_dependencies, dep_warnings = _find_repo_module_dependencies(repo_path)
        for dep_module, deps in module_dependencies.items():
            repo_dependencies.setdefault(dep_module, set()).update(deps)
        for dep_warning in dep_warnings:
            _append_warning(warnings, f"{dep_warning['repo_file']}: {dep_warning['reason']}")

    shared_modules = _collect_shared_modules(repo_modules, min_repo_count=min_repo_count)
    include_candidates = _resolve_candidate_inclusion(
        candidates=candidates,
        omit_candidates=omit_candidates,
    )
    candidates = (
        [] if not include_candidates else build_scan_candidates(shared_modules, depends_on_by_module=repo_dependencies)
    )
    if compiled_name_regex is not None:
        if not include_candidates:
            _append_warning(warnings, "candidate-name-regex ignored when candidates are omitted")
        else:
            candidates = [item for item in candidates if compiled_name_regex.search(str(item.get("module", "")))]

    recommendations = _build_recommended_modules(shared_modules, depends_on_by_module=repo_dependencies)[
        :SCAN_SHARED_REPOS_RECOMMENDED_MODULE_COUNT_LIMIT
    ]

    return {
        "scan_schema_version": SCAN_SHARED_REPOS_SCHEMA_VERSION,
        "repos_root": str(root),
        "repos_root_mode": root_mode,
        "shared_modules": shared_modules,
        "root_mode_hint": root_mode_hint,
        "warnings": warnings,
        "repo_paths": _sorted_repo_paths(repo_paths),
        "shared_count": len(shared_modules),
        "module_count": len(repo_modules),
        "repo_count": len(examined_repos),
        "excluded_repos": effective_excludes,
        "examined_repos": examined_repos,
        "min_repo_count": min_repo_count,
        "module_candidates": candidates,
        "recommended_modules": recommendations,
    }


def materialize_module_candidate_manifest(
    module: str,
    *,
    repos_root: Path | None = None,
    repos_root_mode: str | None = None,
    repos: list[str] | None = None,
    min_repo_count: int = 2,
    module_prefix: str = SCAN_SHARED_REPOS_DEFAULT_MODULE_PREFIX,
    output_dir: Path | None = None,
    dry_run: bool = False,
    update_index: bool = True,
) -> dict[str, Any]:
    if not module.strip():
        raise ValueError("module name cannot be empty")
    if min_repo_count < 2:
        raise ValueError("min_repo_count must be at least 2")

    state = scan_shared_modules_across_repos(
        repos_root=repos_root,
        repos_root_mode=repos_root_mode,
        min_repo_count=min_repo_count,
    )
    shared_modules = state.get("shared_modules")
    if not isinstance(shared_modules, dict) or module not in shared_modules:
        raise ValueError(f"module {module} is not shared at min_repo_count={min_repo_count}")

    selected_repos = sorted(shared_modules[module])
    if repos:
        selected_repos = sorted(set(repos) & set(selected_repos))
        if len(selected_repos) < min_repo_count:
            raise ValueError(
                f"module {module} has insufficient pinned repos after filtering: {len(selected_repos)} < {min_repo_count}"
            )

    candidates = build_scan_candidates({module: selected_repos}, module_prefix=module_prefix)
    if not candidates:
        raise ValueError(f"module {module} has no qualifying repos")
    candidate = candidates[0]
    target_output_dir = (output_dir or module_manifests_root()).expanduser().resolve()
    return materialize_scan_candidate_manifest(
        candidate,
        output_dir=target_output_dir,
        dry_run=dry_run,
        update_index=update_index,
    )


def list_modules() -> list[str]:
    root = module_manifests_root()
    if not root.exists():
        return []
    return sorted(d.name for d in root.iterdir() if d.is_dir() and (d / "manifest.json").exists())


def load_module_manifest(
    module: str,
    available_repo_ids: list[str] | None = None,
) -> dict[str, Any]:
    dm = _load_module_manifest_dataclass(module)
    # If available_repo_ids provided, filter repo_ids via patterns against the given list
    if available_repo_ids is not None:
        repo_ids = sorted(
            r for r in available_repo_ids if any(fnmatch(r, p) for p in dm.repo_patterns)
        )
    else:
        repo_ids = dm.repo_ids

    # Validate repo_ref_overrides keys are in repo_ids
    if dm.repo_ref_overrides:
        for key in dm.repo_ref_overrides:
            if key not in repo_ids:
                raise ValueError(
                    f"unknown repo_ref_overrides key '{key}' not in resolved repo_ids"
                )

    return {
        "schema_version": dm.schema_version,
        "repo_ids": repo_ids,
        "repo_patterns": dm.repo_patterns,
        "owners": dm.owners,
        "refresh_cadence": dm.refresh_cadence if dm.refresh_cadence is not None else "never",
        "default_ref": dm.default_ref,
        "repo_ref_overrides": dm.repo_ref_overrides,
        "repo_runner_overrides": dm.repo_runner_overrides,
        "repo_command_overrides": dm.repo_command_overrides,
        "repo_env_profile_overrides": dm.repo_env_profile_overrides,
    }


def audit_shared_modules_across_repos(
    source_root: Path | None = None,
    *,
    include_repo_modules_root: bool = False,
    include_modules: list[str] | None = None,
    exclude_modules: list[str] | None = None,
    min_repo_count: int = 2,
) -> dict[str, Any]:
    root = (source_root or phenotype_root() / "repos").expanduser().resolve()
    candidate_paths = sorted(root.iterdir()) if root.exists() else []

    exclude_set = set(exclude_modules or [])
    include_set = set(include_modules) if include_modules is not None else None
    excluded_repos: list[str] = []
    repo_modules: dict[str, set[str]] = {}

    for repo_path in candidate_paths:
        if not repo_path.is_dir():
            continue
        repo_id = repo_path.name
        if repo_id in DEFAULT_EXCLUDED_REPOS or (repo_id and repo_id[0].isdigit()):
            excluded_repos.append(repo_id)
            continue

        if include_repo_modules_root:
            modules_dir = repo_path / "modules"
            if modules_dir.is_dir():
                for mod_dir in modules_dir.iterdir():
                    if mod_dir.is_dir() and (mod_dir / "manifest.json").exists():
                        mod_name = mod_dir.name
                        if mod_name in exclude_set:
                            continue
                        if include_set is not None and mod_name not in include_set:
                            continue
                        repo_modules.setdefault(mod_name, set()).add(repo_id)
        else:
            src_dir = repo_path / "src"
            if src_dir.is_dir():
                for mod_dir in src_dir.iterdir():
                    if mod_dir.is_dir():
                        mod_name = mod_dir.name
                        if mod_name in exclude_set:
                            continue
                        if include_set is not None and mod_name not in include_set:
                            continue
                        repo_modules.setdefault(mod_name, set()).add(repo_id)

    shared_modules: dict[str, list[str]] = {
        mod: sorted(repos)
        for mod, repos in repo_modules.items()
        if len(repos) >= min_repo_count
    }

    # Moduleization candidates: shared modules not already in module_manifests_root()
    existing: set[str] = set()
    mroot = module_manifests_root()
    if mroot.exists():
        existing = {d.name for d in mroot.iterdir() if d.is_dir() and (d / "manifest.json").exists()}

    moduleization_candidates = [mod for mod in shared_modules if mod not in existing]

    return {
        "shared_module_count": len(shared_modules),
        "module_count": len(repo_modules),
        "shared_modules": shared_modules,
        "excluded_repos": excluded_repos,
        "moduleization_candidates": moduleization_candidates,
    }


def sync_project_modules_from_repos(
    source_root: Path,
    destination_root: Path | None = None,
    include_modules: list[str] | None = None,
    dry_run: bool = False,
    overwrite: bool = False,
) -> dict[str, Any]:
    dest = (destination_root or module_manifests_root()).expanduser().resolve()
    include_set = set(include_modules) if include_modules is not None else None

    # Scan source_root for repos, skip excluded
    discovered: dict[str, tuple[str, dict[str, Any]]] = {}  # mod_name -> (repo_id, payload)
    for repo_path in sorted(source_root.iterdir()):
        if not repo_path.is_dir():
            continue
        repo_id = repo_path.name
        if repo_id in DEFAULT_EXCLUDED_REPOS or (repo_id and repo_id[0].isdigit()):
            continue
        modules_dir = repo_path / "modules"
        if not modules_dir.is_dir():
            continue
        for mod_dir in sorted(modules_dir.iterdir()):
            if not mod_dir.is_dir():
                continue
            mod_name = mod_dir.name
            if include_set is not None and mod_name not in include_set:
                continue
            manifest_file = mod_dir / "manifest.json"
            if not manifest_file.exists():
                continue
            raw = manifest_file.read_text(encoding="utf-8").strip()
            if not raw:
                raise ValueError(f"invalid module manifest in {repo_id}: empty JSON")
            try:
                payload = json.loads(raw)
            except json.JSONDecodeError as exc:
                raise ValueError(f"invalid module manifest in {repo_id}: malformed json") from exc
            if not isinstance(payload, dict):
                raise ValueError(f"invalid module manifest in {repo_id}: payload must be an object")

            if mod_name in discovered:
                existing_payload = discovered[mod_name][1]
                if existing_payload != payload:
                    raise ValueError(f"conflicting manifests for module '{mod_name}'")
            else:
                discovered[mod_name] = (repo_id, payload)

    created: list[str] = []
    updated: list[str] = []
    skipped: list[str] = []

    for mod_name, (_, payload) in sorted(discovered.items()):
        target_dir = dest / mod_name
        target_file = target_dir / "manifest.json"
        if target_file.exists():
            if overwrite:
                if not dry_run:
                    target_file.write_text(json.dumps(payload, indent=2), encoding="utf-8")
                updated.append(mod_name)
            else:
                skipped.append(mod_name)
        else:
            if not dry_run:
                target_dir.mkdir(parents=True, exist_ok=True)
                target_file.write_text(json.dumps(payload, indent=2), encoding="utf-8")
            created.append(mod_name)

    return {
        "dry_run": dry_run,
        "created": created,
        "updated": updated,
        "skipped": skipped,
        "discovered_modules": sorted(discovered.keys()),
    }
