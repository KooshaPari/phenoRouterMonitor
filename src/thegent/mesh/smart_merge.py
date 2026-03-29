"""AST-aware smart merge driver for concurrent agent edits (heliosShield Phase 7).

Integrates Mergiraf as a git merge driver for .py, .rs, .ts, .js files.
Falls back to diff3 / git merge-file when Mergiraf is unavailable.

Also provides the SmartMerger class-based API for integration with WorktreePool
(heliosShield-smart-merge, FR-MESH-007).
"""

from __future__ import annotations

import os
import contextlib
import shutil
import subprocess
from thegent.infra.shim_subprocess import run as shim_run
from dataclasses import dataclass, field
from pathlib import Path

# File extensions that Mergiraf understands structurally.
MERGIRAF_EXTENSIONS: frozenset[str] = frozenset(
    {".py", ".rs", ".ts", ".js", ".tsx", ".jsx", ".java", ".go", ".c", ".cpp", ".h"}
)

# ---------------------------------------------------------------------------
# Data types for the SmartMerger API
# ---------------------------------------------------------------------------


@dataclass
class SmartMergeConfig:
    """Configuration for SmartMerger.

    Attributes:
        mergiraf_binary: Path to the mergiraf binary.  When *None*, the value
            of the ``THGENT_MERGIRAF_BINARY`` environment variable is used, or
            ``mergiraf`` is searched on ``PATH``.
        fallback_to_git: When True (default), fall back to ``git merge-file``
            if mergiraf is unavailable or returns a hard error.
        timeout_s: Maximum wall-clock seconds to wait for the merge subprocess.
    """

    mergiraf_binary: str | None = None
    fallback_to_git: bool = True
    timeout_s: int = 30


@dataclass
class MergeResult:
    """Result of a smart merge operation.

    Attributes:
        success: True when the merge produced no unresolved conflicts.
        conflicts: List of conflicting file paths (empty on a clean merge).
        output: Captured stdout/stderr from the merge tool.
        used_mergiraf: True when the merge was performed by mergiraf (as
            opposed to the git merge-file fallback).
    """

    success: bool
    conflicts: list[str] = field(default_factory=list)
    output: str = ""
    used_mergiraf: bool = False


def is_mergiraf_available() -> bool:
    """Return True if the mergiraf binary is discoverable on PATH."""
    return shutil.which("mergiraf") is not None


def configure_mergiraf_driver(
    repo_root: Path | None = None,
    *,
    global_config: bool = False,
) -> bool:
    """Register mergiraf as a git merge driver.

    Writes the ``[merge "mergiraf"]`` stanza to ``.git/config`` (local) or
    ``~/.gitconfig`` (global) and creates / updates ``.gitattributes`` at
    *repo_root* with entries for the supported extensions.

    Args:
        repo_root: Repository working directory.  When *None*, the current
            working directory is used.  Ignored when *global_config* is True.
        global_config: Write to the user-global ``~/.gitconfig`` instead of
            the repo-local ``.git/config``.

    Returns:
        True on success, False if git or mergiraf are not available.
    """
    mergiraf_bin = shutil.which("mergiraf")
    if not mergiraf_bin:
        return False

    # ------------------------------------------------------------------ #
    # 1.  Register the merge driver in git config                          #
    # ------------------------------------------------------------------ #
    # The driver receives: %O = base, %A = ours (overwritten in place),
    #                      %B = theirs, %P = path in repo.
    driver_cmd = f"{mergiraf_bin} merge --git %O %A %B -p %P"

    git_config_args_name = [
        "git",
        "config",
        *(["--global"] if global_config else []),
        "merge.mergiraf.name",
        "mergiraf",
    ]
    git_config_args_driver = [
        "git",
        "config",
        *(["--global"] if global_config else []),
        "merge.mergiraf.driver",
        driver_cmd,
    ]

    cwd = None if global_config else (repo_root or Path.cwd())

    try:
        shim_run(git_config_args_name, check=True, capture_output=True, cwd=cwd)
        shim_run(git_config_args_driver, check=True, capture_output=True, cwd=cwd)
    except (subprocess.CalledProcessError, FileNotFoundError):
        return False

    # ------------------------------------------------------------------ #
    # 2.  Write .gitattributes entries (repo-local only)                  #
    # ------------------------------------------------------------------ #
    if global_config:
        return True

    root = repo_root or Path.cwd()
    gitattributes = root / ".gitattributes"

    # Collect existing lines so we do not duplicate entries.
    existing_lines: list[str] = []
    if gitattributes.exists():
        existing_lines = gitattributes.read_text(encoding="utf-8").splitlines()

    entries = {
        "*.py": "merge=mergiraf",
        "*.rs": "merge=mergiraf",
        "*.ts": "merge=mergiraf",
        "*.tsx": "merge=mergiraf",
        "*.js": "merge=mergiraf",
        "*.jsx": "merge=mergiraf",
        "*.java": "merge=mergiraf",
        "*.go": "merge=mergiraf",
    }

    added: list[str] = []
    for pattern, attr in entries.items():
        line = f"{pattern} {attr}"
        if not any(line in existing for existing in existing_lines):
            added.append(line)

    if added:
        with gitattributes.open("a", encoding="utf-8") as fh:
            if existing_lines and existing_lines[-1] != "":
                fh.write("\n")
            fh.write("# mergiraf AST-aware merge driver\n")
            for entry in added:
                fh.write(entry + "\n")

    return True


def merge_files(
    base: Path,
    ours: Path,
    theirs: Path,
    output: Path,
    *,
    path_hint: str | None = None,
) -> bool:
    """Perform an AST-aware three-way merge, writing the result to *output*.

    Tries Mergiraf first.  If Mergiraf is not installed or returns a non-zero
    exit code for a reason other than unresolved conflicts, falls back to
    ``git merge-file`` (diff3 strategy) writing to *output*.

    Args:
        base:      Common ancestor file.
        ours:      Our version of the file.
        theirs:    Their version of the file.
        output:    Destination for the merged result.
        path_hint: Logical file path (used by Mergiraf for language detection
                   when the temp-file names lose the original extension).

    Returns:
        True  – merge produced a clean result (no unresolved conflicts).
        False – merge completed with conflicts left in the file, or the merge
                tool itself failed unexpectedly.  The *output* file may still
                be written with conflict markers for manual resolution.
    """
    mergiraf_bin = shutil.which("mergiraf")

    if mergiraf_bin:
        return _merge_with_mergiraf(mergiraf_bin, base, ours, theirs, output, path_hint=path_hint)

    return _merge_with_git_merge_file(base, ours, theirs, output)


# --------------------------------------------------------------------------- #
# Private helpers                                                               #
# --------------------------------------------------------------------------- #


def _merge_with_mergiraf(
    mergiraf_bin: str,
    base: Path,
    ours: Path,
    theirs: Path,
    output: Path,
    *,
    path_hint: str | None,
) -> bool:
    """Run ``mergiraf merge`` and capture result to *output*.

    Mergiraf exit codes:
        0  – clean merge
        1  – conflicts present (output still written with markers)
        2+ – hard error
    """
    cmd: list[str] = [mergiraf_bin, "merge", str(base), str(ours), str(theirs)]

    if output:
        cmd += ["-o", str(output)]

    if path_hint:
        cmd += ["-p", path_hint]

    try:
        result = shim_run(
            cmd,
            capture_output=True,
            text=True,
            check=False,
        )

        # Exit 0 = clean, 1 = conflicts written with markers (still usable).
        if result.returncode in (0, 1):
            # If mergiraf wrote to stdout instead of --output, capture it.
            if not output.exists() and result.stdout:
                output.write_text(result.stdout, encoding="utf-8")
            return result.returncode == 0

        # Hard failure (exit >=2): fall through to diff3 fallback.
    except FileNotFoundError:
        pass

    return _merge_with_git_merge_file(base, ours, theirs, output)


def _merge_with_git_merge_file(
    base: Path,
    ours: Path,
    theirs: Path,
    output: Path,
) -> bool:
    """Fall back to ``git merge-file --diff3`` for a text-level three-way merge.

    ``git merge-file`` overwrites *ours* in place, so we work on a temp copy
    then move it to *output*.
    """
    import tempfile

    # Work on a scratch copy of ours so the original is preserved.
    with tempfile.NamedTemporaryFile(suffix=ours.suffix, delete=False, dir=output.parent) as tmp:
        tmp_path = Path(tmp.name)

    try:
        tmp_path.write_bytes(ours.read_bytes())

        output_dir = output.parent
        if base.parent == output_dir:
            base_arg = base.name
        else:
            base_arg = str(base)
        if theirs.parent == output_dir:
            theirs_arg = theirs.name
        else:
            theirs_arg = str(theirs)

        result = shim_run(
            [
                "git",
                "merge-file",
                "--diff3",
                tmp_path.name,
                base_arg,
                theirs_arg,
            ],
            cwd=str(output_dir),
            capture_output=True,
            text=True,
            check=False,
        )

        # git merge-file: 0 = clean, >0 = conflicts or error.
        # Either way, copy whatever was written to output.
        output.write_bytes(tmp_path.read_bytes())
        return result.returncode == 0

    except ((FileNotFoundError, OSError)):
        # git not available - last resort: just copy ours.
        with contextlib.suppress(OSError):
            output.write_bytes(ours.read_bytes())
        return False
    finally:
        with contextlib.suppress(OSError):
            tmp_path.unlink(missing_ok=True)


# ---------------------------------------------------------------------------
# SmartMerger — class-based API for WorktreePool integration
# ---------------------------------------------------------------------------


class SmartMerger:
    """Thin coordinator for AST-aware merges via Mergiraf with git fallback.

    This class wraps subprocess calls to mergiraf (or ``git merge-file``) and
    provides a structured :class:`MergeResult` rather than a bare bool.  It is
    intentionally thin: it does *not* implement merge logic itself.

    Typical usage::

        merger = make_smart_merger()
        result = merger.merge(base, ours, theirs, output)
        if not result.success:
            print("Conflicts:", result.conflicts)

    @trace heliosShield-smart-merge / FR-MESH-007
    """

    def __init__(self, config: SmartMergeConfig | None = None) -> None:
        self._config = config or SmartMergeConfig()
        self._binary: str | None = self._resolve_binary()

    # ------------------------------------------------------------------
    # Public API
    # ------------------------------------------------------------------

    def is_available(self) -> bool:
        """Return True if the mergiraf binary can be located.

        @trace FR-MESH-007
        """
        return self._binary is not None

    def merge(
        self,
        base: str,
        ours: str,
        theirs: str,
        output: str,
        *,
        path_hint: str | None = None,
    ) -> MergeResult:
        """Perform a three-way merge of three file paths.

        Calls mergiraf when available; otherwise falls back to
        ``git merge-file`` (if :attr:`SmartMergeConfig.fallback_to_git`
        is True).

        Args:
            base:      Common ancestor file path (string).
            ours:      Our version file path (string).
            theirs:    Their version file path (string).
            output:    Destination file path for the merged result (string).
            path_hint: Logical repository path for language detection.

        Returns:
            :class:`MergeResult` describing whether the merge was clean.

        @trace FR-MESH-007
        """
        base_p = Path(base)
        ours_p = Path(ours)
        theirs_p = Path(theirs)
        output_p = Path(output)

        if self._binary:
            return self._run_mergiraf(base_p, ours_p, theirs_p, output_p, path_hint=path_hint)

        if self._config.fallback_to_git:
            return self._run_git_fallback(base_p, ours_p, theirs_p, output_p)

        return MergeResult(success=False, output="mergiraf unavailable and fallback disabled")

    def merge_worktree_changes(
        self,
        worktree_path: Path,
        target_branch: str,
    ) -> MergeResult:
        """Merge all uncommitted changes in *worktree_path* into *target_branch*.

        This method integrates with :class:`~thegent_gitops.worktree.WorktreePool`:
        it performs a ``git merge --no-ff`` of the worktree's branch into
        *target_branch*, using mergiraf as the merge driver when available.

        The merge driver is activated via git config only if mergiraf is
        available; otherwise a standard ``git merge --no-ff`` is attempted.

        Args:
            worktree_path: Path to the checked-out worktree directory.
            target_branch: Branch name to merge changes into.

        Returns:
            :class:`MergeResult` with ``used_mergiraf`` indicating whether
            the merge driver was active.

        @trace FR-MESH-007
        """
        # Determine the current branch in the worktree.
        try:
            branch_result = shim_run(
                ["git", "rev-parse", "--abbrev-ref", "HEAD"],
                cwd=str(worktree_path),
                capture_output=True,
                text=True,
                check=False,
                timeout=self._config.timeout_s,
            )
            if branch_result.returncode != 0:
                return MergeResult(success=False, output="Failed to resolve worktree branch")
            worktree_branch = branch_result.stdout.strip()
            if not worktree_branch:
                return MergeResult(success=False, output="Worktree branch is empty")

            git_dir_result = shim_run(
                ["git", "rev-parse", "--git-dir"],
                cwd=str(worktree_path),
                capture_output=True,
                text=True,
                check=False,
                timeout=self._config.timeout_s,
            )
            if git_dir_result.returncode != 0 or not git_dir_result.stdout.strip():
                return MergeResult(success=False, output="Failed to resolve git dir for worktree")

            git_dir = Path(git_dir_result.stdout.strip())
            if not git_dir.is_absolute():
                git_dir = (worktree_path / git_dir).resolve()

            # Worktree git dirs are typically <repo>/.git/worktrees/<agent-id>.
            if git_dir.parent.name == "worktrees" and git_dir.parent.parent.parent is not None:
                project_root = git_dir.parent.parent.parent
            else:
                project_root = git_dir.parent

            head_result = shim_run(
                ["git", "rev-parse", "--abbrev-ref", "HEAD"],
                cwd=str(project_root),
                capture_output=True,
                text=True,
                check=False,
                timeout=self._config.timeout_s,
            )
            if head_result.returncode != 0:
                return MergeResult(success=False, output="Failed to resolve project root branch")
            original_branch = head_result.stdout.strip()
        except (subprocess.TimeoutExpired, FileNotFoundError) as exc:
            return MergeResult(success=False, output=str(exc))

        # Ensure merge executes against the resolved target branch.
        checkout_success = False
        try:
            if original_branch != target_branch:
                checkout_target = shim_run(
                    ["git", "checkout", target_branch],
                    cwd=str(project_root),
                    capture_output=True,
                    text=True,
                    check=False,
                    timeout=self._config.timeout_s,
                )
                if checkout_target.returncode != 0:
                    return MergeResult(
                        success=False,
                        output=f"Failed to checkout target branch {target_branch}: {checkout_target.stderr.strip()}",
                    )
                checkout_success = True
        except (subprocess.TimeoutExpired, FileNotFoundError) as exc:
            return MergeResult(success=False, output=str(exc))

        # Set the mergiraf driver in git config if available
        used_mergiraf = False
        if self._binary and self._config.fallback_to_git:
            try:
                shim_run(
                    [
                        "git",
                        "config",
                        "merge.mergiraf.driver",
                        f"{self._binary} merge --git %O %A %B -p %P",
                    ],
                    cwd=str(project_root),
                    capture_output=True,
                    check=False,
                    timeout=self._config.timeout_s,
                )
                used_mergiraf = True
            except (subprocess.TimeoutExpired, FileNotFoundError):
                pass

        # Perform the merge
        try:
            checkout_result = shim_run(
                ["git", "checkout", target_branch],
                cwd=str(project_root),
                capture_output=True,
                text=True,
                check=False,
                timeout=self._config.timeout_s,
            )
            if checkout_result.returncode != 0:
                return MergeResult(
                    success=False,
                    output=f"failed to checkout target branch {target_branch}: {checkout_result.stderr or checkout_result.stdout}",
                    used_mergiraf=used_mergiraf,
                )

            result = shim_run(
                [
                    "git",
                    "merge",
                    "--no-ff",
                    "-m",
                    f"Merge {worktree_branch} into {target_branch} (smart-merge)",
                    worktree_branch,
                ],
                cwd=str(project_root),
                capture_output=True,
                text=True,
                check=False,
                timeout=self._config.timeout_s,
            )
            merged = result.returncode == 0
        except subprocess.TimeoutExpired:
            return MergeResult(
                success=False,
                output=f"git merge timed out after {self._config.timeout_s}s",
                used_mergiraf=used_mergiraf,
            )
        except FileNotFoundError as exc:
            return MergeResult(success=False, output=str(exc), used_mergiraf=used_mergiraf)
        finally:
            if checkout_success:
                with contextlib.suppress(subprocess.TimeoutExpired, FileNotFoundError):
                    shim_run(
                        ["git", "checkout", original_branch],
                        cwd=str(project_root),
                        capture_output=True,
                        text=True,
                        check=False,
                        timeout=self._config.timeout_s,
                    )

        combined_output = (result.stdout or "") + (result.stderr or "")
        success = merged

        # Collect conflict file paths from output
        conflicts: list[str] = []
        if not success:
            for line in combined_output.splitlines():
                if line.startswith("CONFLICT"):
                    # e.g. "CONFLICT (content): Merge conflict in src/foo.py"
                    parts = line.split(" in ", maxsplit=1)
                    if len(parts) == 2:
                        conflicts.append(parts[1].strip())

        return MergeResult(
            success=success,
            conflicts=conflicts,
            output=combined_output,
            used_mergiraf=used_mergiraf,
        )

    # ------------------------------------------------------------------
    # Private helpers
    # ------------------------------------------------------------------

    def _resolve_binary(self) -> str | None:
        """Determine the path to the mergiraf binary."""
        if self._config.mergiraf_binary:
            return self._config.mergiraf_binary
        from thegent.config import ThegentSettings

        settings = ThegentSettings()
        env_bin = getattr(settings, "mergiraf_binary", None)
        if not env_bin:
            env_bin = os.getenv("THGENT_MERGIRAF_BINARY")
        if env_bin:
            return env_bin
        return shutil.which("mergiraf")

    def _run_mergiraf(
        self,
        base: Path,
        ours: Path,
        theirs: Path,
        output: Path,
        *,
        path_hint: str | None,
    ) -> MergeResult:
        """Invoke mergiraf and parse the result into a MergeResult."""
        assert self._binary is not None
        cmd: list[str] = [self._binary, "merge", str(base), str(ours), str(theirs)]
        if output:
            cmd += ["-o", str(output)]
        if path_hint:
            cmd += ["-p", path_hint]

        try:
            result = shim_run(
                cmd,
                capture_output=True,
                text=True,
                check=False,
                timeout=self._config.timeout_s,
            )
        except subprocess.TimeoutExpired:
            if self._config.fallback_to_git:
                return self._run_git_fallback(base, ours, theirs, output)
            return MergeResult(success=False, output="mergiraf timed out")
        except FileNotFoundError:
            if self._config.fallback_to_git:
                return self._run_git_fallback(base, ours, theirs, output)
            return MergeResult(success=False, output="mergiraf binary not found")

        combined = (result.stdout or "") + (result.stderr or "")

        if result.returncode == 0:
            if not output.exists() and result.stdout:
                output.write_text(result.stdout, encoding="utf-8")
            return MergeResult(success=True, output=combined, used_mergiraf=True)

        if result.returncode == 1:
            # Conflicts present but output written with markers
            if not output.exists() and result.stdout:
                output.write_text(result.stdout, encoding="utf-8")
            return MergeResult(success=False, output=combined, used_mergiraf=True)

        # Hard failure (exit >=2): fall through to diff3 fallback
        if self._config.fallback_to_git:
            return self._run_git_fallback(base, ours, theirs, output)

        return MergeResult(success=False, output=combined, used_mergiraf=True)

    def _run_git_fallback(
        self,
        base: Path,
        ours: Path,
        theirs: Path,
        output: Path,
    ) -> MergeResult:
        """Fall back to ``git merge-file --diff3``."""
        success = _merge_with_git_merge_file(base, ours, theirs, output)
        return MergeResult(success=success, used_mergiraf=False)


# ---------------------------------------------------------------------------
# Factory function
# ---------------------------------------------------------------------------


def make_smart_merger(config: SmartMergeConfig | None = None) -> SmartMerger:
    """Create a :class:`SmartMerger` instance, reading env var for binary path.

    When *config* is ``None``, a default :class:`SmartMergeConfig` is used.
    The mergiraf binary path is resolved from (in priority order):

    1. ``config.mergiraf_binary`` if provided.
    2. The ``THGENT_MERGIRAF_BINARY`` environment variable.
    3. ``shutil.which("mergiraf")`` (PATH search).

    @trace heliosShield-smart-merge / FR-MESH-007
    """
    return SmartMerger(config)
