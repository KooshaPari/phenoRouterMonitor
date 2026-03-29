"""Compatibility wrapper for Git parallelism ops domain module."""

import logging
import hashlib
import json
import os
import random
import shutil
import subprocess
from thegent.infra.shim_subprocess import run as shim_run
import time
from pathlib import Path

logger = logging.getLogger(__name__)


try:
    import thegent_git
except ImportError:
    thegent_git = None


def _thegent_git_has(name: str) -> bool:
    return thegent_git is not None and hasattr(thegent_git, name)


class GitParallelismManager:
    """Manages parallel git operations using per-agent index files and plumbing (SCLI-P4.1–P4.2)."""

    def __init__(self, project_root: Path, agent_id: str, mesh_root: Path = Path("/tmp/agent-mesh")) -> None:  # noqa: S108 -- intentional platform temp dir for agent mesh IPC
        self.project_root = project_root
        self.agent_id = agent_id
        self.git_dir = project_root / ".git"
        self.mesh_root = mesh_root
        self.project_tag = hashlib.sha256(str(project_root.resolve()).encode("utf-8")).hexdigest()[:12]
        self.agent_index = mesh_root / "indices" / self.project_tag / f"index-{agent_id}"
        self.staging_map = mesh_root / "scoped-staging" / f"{self.project_tag}.json"
        self.agent_index.parent.mkdir(parents=True, exist_ok=True, mode=0o1777)
        self.staging_map.parent.mkdir(parents=True, exist_ok=True, mode=0o1777)

    def _run_git(
        self,
        args: list[str],
        *,
        use_index: bool = True,
        input_text: str | None = None,
        check: bool = False,
        env: dict[str, str] | None = None,
    ) -> subprocess.CompletedProcess[str]:
        process_env = os.environ.copy()
        if use_index:
            process_env["GIT_INDEX_FILE"] = str(self.agent_index)
        if env:
            process_env.update(env)
        return shim_run(
            ["git", *args],
            cwd=self.project_root,
            env=process_env,
            input=input_text,
            capture_output=True,
            text=True,
            check=check,
        )

    def _get_ref_hash(self, ref: str) -> str | None:
        """Get current hash for a ref."""
        if _thegent_git_has("rev_parse"):
            sha = thegent_git.rev_parse(str(self.project_root), ref)
            return sha.strip() if sha else None

        ref_proc = self._run_git(["rev-parse", ref], use_index=False)
        return ref_proc.stdout.strip() if ref_proc.returncode == 0 else None

    def _index_lock_path(self) -> Path:
        """Return .git/index.lock path for this repository."""
        return self.git_dir / "index.lock"

    def _has_open_lock_holder(self, lock_path: Path) -> bool:
        """Return True when the lock file appears to be open in another process."""
        try:
            check = shim_run(
                ["lsof", str(lock_path)],
                capture_output=True,
                text=True,
                check=False,
                timeout=1.0,
            )
        except (FileNotFoundError, subprocess.TimeoutExpired):
            return False

        # On macOS/Linux lsof returns 0 and output when a file is open by at least one process.
        return check.returncode == 0 and bool(check.stdout and check.stdout.strip())

    def _is_stale_lock(self, lock_path: Path, stale_after_s: float) -> bool:
        """Return True when lock appears stale and safe to remove."""
        try:
            mtime = lock_path.stat().st_mtime
        except OSError:
            return False

        age = time.time() - mtime
        if age < stale_after_s:
            return False

        if self._has_open_lock_holder(lock_path):
            return False

        return True

    def index_lock_status(self, stale_after_s: float = 90.0) -> dict[str, object]:
        """Return index lock state summary."""
        lock_path = self._index_lock_path()
        if not lock_path.exists():
            return {
                "exists": False,
                "path": str(lock_path),
                "age_seconds": None,
                "stale_after_seconds": stale_after_s,
                "is_stale": False,
                "open_holder_detected": False,
            }

        try:
            mtime = lock_path.stat().st_mtime
        except OSError:
            return {
                "exists": True,
                "path": str(lock_path),
                "age_seconds": None,
                "stale_after_seconds": stale_after_s,
                "is_stale": False,
                "open_holder_detected": False,
            }

        age = time.time() - mtime
        open_holder = self._has_open_lock_holder(lock_path)
        is_stale = age >= stale_after_s and not open_holder
        return {
            "exists": True,
            "path": str(lock_path),
            "age_seconds": age,
            "stale_after_seconds": stale_after_s,
            "is_stale": is_stale,
            "open_holder_detected": open_holder,
        }

    def _load_staging_map(self) -> dict[str, list[str]]:
        if not self.staging_map.exists():
            return {}
        raw = self.staging_map.read_text(encoding="utf-8")
        data = json.loads(raw) if raw.strip() else {}
        return data if isinstance(data, dict) else {}

    def _save_staging_map(self, mapping: dict[str, list[str]]) -> None:
        serializable = {key: sorted(set(value)) for key, value in mapping.items()}
        self.staging_map.parent.mkdir(parents=True, exist_ok=True, mode=0o1777)
        with self.staging_map.open("w", encoding="utf-8") as fh:
            json.dump(serializable, fh, sort_keys=True)

    def _normalise_files(self, files: list[str]) -> list[str]:
        normalized: set[str] = set()
        for file in files:
            candidate = file.strip()
            if not candidate:
                continue
            path = Path(candidate)
            if path.is_absolute():
                try:
                    candidate = str(path.relative_to(self.project_root))
                except ValueError:
                    candidate = path.name
            normalized.add(candidate)
        return sorted(normalized)

    def wait_for_index_lock(
        self,
        timeout_s: float = 8.0,
        poll_s: float = 0.2,
        *,
        stale_after_s: float = 90.0,
        allow_stale_cleanup: bool = True,
    ) -> bool:
        """Wait briefly for index.lock to clear, optionally cleaning stale locks."""
        lock_path = self._index_lock_path()
        deadline = time.time() + timeout_s
        while lock_path.exists():
            if allow_stale_cleanup and stale_after_s >= 0 and self._is_stale_lock(lock_path, stale_after_s):
                try:
                    lock_path.unlink()
                    logger.info("Removed stale git index lock at %s", lock_path)
                    return True
                except OSError:
                    logger.warning("Failed to remove stale git index lock at %s", lock_path)
                    break
            if time.time() >= deadline:
                return False
            time.sleep(poll_s)
        return True

    def ensure_index(self) -> Path:
        """Create or refresh the per-agent index file."""
        system_index = self.git_dir / "index"
        if not self.agent_index.exists() or (
            system_index.exists() and system_index.stat().st_mtime > self.agent_index.stat().st_mtime
        ):
            if system_index.exists():
                shutil.copy2(system_index, self.agent_index)
            else:
                open(self.agent_index, "wb").close()
        return self.agent_index

    def stage_files(self, files: list[str]) -> bool:
        """Stage specific files using the agent's index."""
        self.ensure_index()
        staged_files = self._normalise_files(files)
        if not staged_files:
            return True

        result = self._run_git(["add", "--"] + staged_files)
        if result.returncode != 0:
            return False

        mapping = self._load_staging_map()
        mapping[self.agent_id] = sorted(set(mapping.get(self.agent_id, [])).union(staged_files))
        self._save_staging_map(mapping)
        return True

    def create_commit_from_index(
        self,
        message: str,
        parent_ref: str = "HEAD",
        *,
        author_env: dict[str, str] | None = None,
    ) -> str | None:
        """Build commit from private index with plumbing commands."""
        self.ensure_index()
        parent_resolve = self._run_git(["rev-parse", parent_ref], use_index=False, check=False)
        if parent_resolve.returncode != 0:
            return None
        parent_hash = parent_resolve.stdout.strip()

        tree_res = self._run_git(["write-tree"])
        if tree_res.returncode != 0:
            return None
        tree_sha = tree_res.stdout.strip()

        commit_args = (
            ["commit-tree", tree_sha, "-m", message, "-p", parent_hash]
            if parent_hash
            else [
                "commit-tree",
                tree_sha,
                "-m",
                message,
            ]
        )
        commit_res = self._run_git(commit_args, env=author_env)
        if commit_res.returncode != 0:
            return None

        return commit_res.stdout.strip()

    def update_ref_cas(self, ref: str, new_hash: str, old_hash: str) -> bool:
        """Compare-And-Swap ref update with exponential backoff + jitter."""
        max_retries = 5
        base_delay = 0.1
        expected = old_hash

        for attempt in range(max_retries):
            update = shim_run(
                ["git", "update-ref", ref, new_hash, expected],
                cwd=self.project_root,
                capture_output=True,
                text=True,
            )
            if update.returncode == 0:
                return True

            current = self._get_ref_hash(ref)
            if current is None:
                return False
            if current != expected:
                expected = current

            if attempt == max_retries - 1:
                return False
            delay = base_delay * (2**attempt) + random.uniform(0, 0.1)
            time.sleep(delay)

        return False

    def staged_files(self) -> list[str]:
        """Return staged files from this agent's private index."""
        self.ensure_index()
        diff_proc = self._run_git(["diff", "--cached", "--name-only"])
        if diff_proc.returncode != 0:
            return []
        return sorted({line.strip() for line in diff_proc.stdout.splitlines() if line.strip()})

    def changed_files_between(self, older: str, newer: str) -> list[str]:
        """Return files changed between two refs/hashes."""
        diff_proc = self._run_git(["diff", "--name-only", f"{older}..{newer}"], use_index=False)
        if diff_proc.returncode != 0:
            return []
        return sorted({line.strip() for line in diff_proc.stdout.splitlines() if line.strip()})

    def related_overlap(self, ours: list[str], theirs: list[str]) -> list[str]:
        """Return sorted overlap between two file lists."""
        return sorted(set(ours).intersection(theirs))

    def queue_commit_conflict(
        self,
        ref: str,
        reason: str,
        ours: list[str],
        theirs: list[str],
        overlap: list[str],
        old_hash: str | None = None,
        new_hash: str | None = None,
    ) -> Path:
        """Append a conflict record to per-project git conflict queue."""
        queue_path = self.project_root / ".thegent" / "git-conflict-queue.jsonl"
        queue_path.parent.mkdir(parents=True, exist_ok=True)
        entry = {
            "ts": int(time.time()),
            "agent_id": self.agent_id,
            "ref": ref,
            "reason": reason,
            "ours": ours,
            "theirs": theirs,
            "overlap": overlap,
            "old_hash": old_hash or "",
            "new_hash": new_hash or "",
        }
        with queue_path.open("a", encoding="utf-8") as fh:
            fh.write(json.dumps(entry) + "\n")
        return queue_path

    def try_auto_merge_commit(
        self,
        ours_commit: str,
        theirs_commit: str,
        message: str,
        *,
        author_env: dict[str, str] | None = None,
    ) -> str | None:
        """Attempt to create a synthetic 3-way merge commit."""
        if (
            not (
                _thegent_git_has("merge_base")
                and _thegent_git_has("create_commit")
                and _thegent_git_has("diff_stat")
            )
            or author_env
        ):
            probe = self._run_git(["merge-tree", ours_commit, theirs_commit], use_index=False)
            if probe.returncode != 0 or "CONFLICT" in probe.stdout:
                return None

            tree_proc = self._run_git(["merge-tree", "--write-tree", ours_commit, theirs_commit], use_index=False)
            if tree_proc.returncode != 0:
                return None

            tree_sha = tree_proc.stdout.splitlines()[0].strip() if tree_proc.stdout else ""
            if not tree_sha:
                return None

            commit = self._run_git(
                [
                    "commit-tree",
                    tree_sha,
                    "-p",
                    ours_commit,
                    "-p",
                    theirs_commit,
                    "-m",
                    f"merge(auto): {message}",
                ],
                use_index=False,
                env=author_env,
            )
            if commit.returncode != 0:
                logger.debug("merge-tree fallback commit-tree failed: %s", commit.stderr)
                return None
            return commit.stdout.strip()

        base = thegent_git.merge_base(str(self.project_root), ours_commit, theirs_commit)
        if not base:
            return None

        diff = thegent_git.diff_stat(str(self.project_root), f"{ours_commit}..{theirs_commit}")
        if not diff:
            return None

        commit_hash = thegent_git.create_commit(
            str(self.project_root),
            base,
            f"merge(auto): {message}",
            [theirs_commit, ours_commit],
        )
        return commit_hash

    def get_agent_status(self) -> str:
        """Show per-agent staged changes."""
        staged = self.staged_files()
        if not staged:
            return ""
        branch_proc = self._run_git(["rev-parse", "--abbrev-ref", "HEAD"], use_index=False)
        sha_proc = self._run_git(["rev-parse", "--short=7", "HEAD"], use_index=False)
        branch = branch_proc.stdout.strip() if branch_proc.returncode == 0 else "unknown"
        sha = sha_proc.stdout.strip() if sha_proc.returncode == 0 else "unknown"
        lines = [f"[{self.agent_id}] {branch} ({sha})"]
        lines.extend(staged)
        return "\n".join(lines)


def harness_git_status_view(agent_id: str) -> None:
    """Display git status for a specific agent."""
    logger.info("Agent: %s", agent_id)
    manager = GitParallelismManager(Path.cwd(), agent_id)
    status = manager.get_agent_status()
    if status:
        for line in status.split("\n"):
            logger.info("%s", line)
    else:
        logger.info("No staged changes")
