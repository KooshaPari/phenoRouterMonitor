"""Fast file operations with platform-specific optimizations.

This module provides optimized file operations that use platform-specific
optimizations for better performance:
- Linux: os.sendfile() for large file copies (zero-copy)
- All platforms: Optimized shutil operations
- Batch operations where possible

Performance improvements:
- sendfile() on Linux: Zero-copy for large files (10-100MB+)
- Optimized directory operations
- Batch file operations
"""

import os
import shutil
import sys
from collections import defaultdict
from errno import ENOSYS, EOPNOTSUPP, EPERM, ENOTSUP
from logging import getLogger
from pathlib import Path
from typing import Any

SEND_FILE_THRESHOLD_BYTES = 10_000_000
_SEND_FILE_FALLBACK_COUNTS: defaultdict[str, int] = defaultdict(int)
_SEND_FILE_FALLBACK_DIAGNOSTICS: dict[str, Any] = {}
_log = getLogger(__name__)


def _sendfile_fallback_reason(exc: BaseException) -> str:
    if isinstance(exc, AttributeError):
        return "unsupported"
    if isinstance(exc, PermissionError):
        return "permission"
    if isinstance(exc, OSError):
        if exc.errno in {ENOSYS, ENOTSUP, EOPNOTSUPP}:
            return "unsupported"
        if exc.errno == EPERM:
            return "permission"
        return "os_error"
    return type(exc).__name__


def _init_sendfile_diagnostics() -> dict[str, Any]:
    return {
        "status": "unused",
        "reason": None,
        "error_type": None,
        "error_message": None,
        "src": None,
        "dst": None,
        "platform": sys.platform,
        "preserve_metadata": None,
        "src_size": None,
    }


_SEND_FILE_FALLBACK_DIAGNOSTICS.update(_init_sendfile_diagnostics())


def _record_sendfile_fallback(exc: BaseException, src: Path, dst: Path, preserve_metadata: bool) -> None:
    reason = _sendfile_fallback_reason(exc)
    _SEND_FILE_FALLBACK_COUNTS[reason] += 1
    _SEND_FILE_FALLBACK_DIAGNOSTICS.update(
        {
            "status": "fallback",
            "reason": reason,
            "error_type": type(exc).__name__,
            "error_message": str(exc),
            "src": str(src),
            "dst": str(dst),
            "platform": sys.platform,
            "preserve_metadata": preserve_metadata,
            "src_size": src.stat().st_size if src.exists() else None,
        }
    )
    _log.warning(
        "sendfile fallback engaged: reason=%s src=%s dst=%s error=%s",
        reason,
        src,
        dst,
        str(exc)[:200],
    )


def get_sendfile_fallback_counts() -> dict[str, int]:
    return dict(_SEND_FILE_FALLBACK_COUNTS)


def reset_sendfile_fallback_counts() -> None:
    _SEND_FILE_FALLBACK_COUNTS.clear()
    reset_sendfile_fallback_diagnostics()


def get_sendfile_fallback_diagnostics() -> dict[str, Any]:
    return {
        "status": _SEND_FILE_FALLBACK_DIAGNOSTICS.get("status"),
        "reason": _SEND_FILE_FALLBACK_DIAGNOSTICS.get("reason"),
        "error_type": _SEND_FILE_FALLBACK_DIAGNOSTICS.get("error_type"),
        "error_message": _SEND_FILE_FALLBACK_DIAGNOSTICS.get("error_message"),
        "src": _SEND_FILE_FALLBACK_DIAGNOSTICS.get("src"),
        "dst": _SEND_FILE_FALLBACK_DIAGNOSTICS.get("dst"),
        "platform": _SEND_FILE_FALLBACK_DIAGNOSTICS.get("platform"),
        "preserve_metadata": _SEND_FILE_FALLBACK_DIAGNOSTICS.get("preserve_metadata"),
        "src_size": _SEND_FILE_FALLBACK_DIAGNOSTICS.get("src_size"),
        "count_by_reason": get_sendfile_fallback_counts(),
        "total_failures": sum(_SEND_FILE_FALLBACK_COUNTS.values()),
    }


def reset_sendfile_fallback_diagnostics() -> None:
    _SEND_FILE_FALLBACK_DIAGNOSTICS.clear()
    _SEND_FILE_FALLBACK_DIAGNOSTICS.update(_init_sendfile_diagnostics())


class FastFileOps:
    """High-performance file operations with platform-specific optimizations."""

    @staticmethod
    def copy(src: Path | str, dst: Path | str, preserve_metadata: bool = True) -> None:
        """Copy file with optimized method selection.

        Args:
            src: Source file path
            dst: Destination file path
            preserve_metadata: Whether to preserve file metadata

        Performance:
            - Linux: Uses sendfile() for large files (>10MB) - zero-copy
            - Other platforms: Uses optimized shutil.copy2()
        """
        src_path = Path(src)
        dst_path = Path(dst)

        if not src_path.exists():
            raise FileNotFoundError(f"Source file not found: {src_path}")

        # Ensure destination directory exists
        dst_path.parent.mkdir(parents=True, exist_ok=True)

        # Use sendfile() on Linux for large files (zero-copy, much faster)
        if sys.platform == "linux" and src_path.is_file():
            file_size = src_path.stat().st_size
            if file_size > SEND_FILE_THRESHOLD_BYTES:  # > 10MB
                try:
                    with open(src_path, "rb") as fsrc:
                        with open(dst_path, "wb") as fdst:
                            # Use sendfile for zero-copy transfer
                            os.sendfile(fdst.fileno(), fsrc.fileno(), 0, file_size)

                    # Copy metadata if requested
                    if preserve_metadata:
                        stat = src_path.stat()
                        dst_path.chmod(stat.st_mode)
                        os.utime(dst_path, (stat.st_atime, stat.st_mtime))

                    return
                except Exception as exc:
                    # Fallback to shutil if sendfile fails
                    _record_sendfile_fallback(exc, src_path, dst_path, preserve_metadata=preserve_metadata)

        # Standard copy (works on all platforms, preserves metadata)
        if preserve_metadata:
            shutil.copy2(src_path, dst_path)
        else:
            shutil.copy(src_path, dst_path)

    @staticmethod
    def copy_tree(src: Path | str, dst: Path | str, ignore: list[str] | None = None) -> None:
        """Copy directory tree with optimizations.

        Args:
            src: Source directory path
            dst: Destination directory path
            ignore: Optional list of patterns to ignore
        """
        src_path = Path(src)
        dst_path = Path(dst)

        if not src_path.is_dir():
            raise NotADirectoryError(f"Source is not a directory: {src_path}")

        # Use shutil.copytree with optimizations
        def ignore_func(directory: str, files: list[str]) -> list[str]:
            if ignore:
                ignored = []
                for pattern in ignore:
                    ignored.extend([f for f in files if pattern in f])
                return ignored
            return []

        shutil.copytree(src_path, dst_path, ignore=ignore_func if ignore else None, dirs_exist_ok=True)

    @staticmethod
    def move(src: Path | str, dst: Path | str) -> None:
        """Move file or directory (optimized).

        Args:
            src: Source path
            dst: Destination path
        """
        src_path = Path(src)
        dst_path = Path(dst)

        # Ensure destination directory exists
        if dst_path.suffix:  # Has extension, likely a file
            dst_path.parent.mkdir(parents=True, exist_ok=True)
        else:  # Likely a directory
            dst_path.parent.mkdir(parents=True, exist_ok=True)

        shutil.move(str(src_path), str(dst_path))

    @staticmethod
    def remove(path: Path | str, recursive: bool = False) -> None:
        """Remove file or directory (optimized).

        Args:
            path: Path to remove
            recursive: If True, remove directory recursively
        """
        path_obj = Path(path)

        if path_obj.is_file() or path_obj.is_symlink():
            path_obj.unlink()
        elif path_obj.is_dir():
            if recursive:
                shutil.rmtree(path_obj)
            else:
                path_obj.rmdir()
        else:
            raise FileNotFoundError(f"Path not found: {path_obj}")

    @staticmethod
    def get_size(path: Path | str) -> int:
        """Get file or directory size (optimized).

        Args:
            path: Path to file or directory

        Returns:
            Size in bytes
        """
        path_obj = Path(path)

        if path_obj.is_file():
            return path_obj.stat().st_size
        if path_obj.is_dir():
            total = 0
            try:
                # Use os.walk for better performance than Path.rglob()
                for dirpath, _dirnames, filenames in os.walk(path_obj):
                    for filename in filenames:
                        filepath = Path(dirpath) / filename
                        try:
                            total += filepath.stat().st_size
                        except (OSError, FileNotFoundError):
                            continue
            except (OSError, PermissionError):
                pass
            return total
        return 0

    @staticmethod
    def ensure_dir(path: Path | str, mode: int = 0o755) -> Path:
        """Ensure directory exists (create if needed).

        Args:
            path: Directory path
            mode: Directory permissions

        Returns:
            Path object
        """
        path_obj = Path(path)
        path_obj.mkdir(parents=True, exist_ok=True)
        if mode:
            path_obj.chmod(mode)
        return path_obj


# Convenience functions
def copy_file(src: Path | str, dst: Path | str, preserve_metadata: bool = True) -> None:
    """Copy file with optimized method."""
    FastFileOps.copy(src, dst, preserve_metadata)


def copy_tree(src: Path | str, dst: Path | str, ignore: list[str] | None = None) -> None:
    """Copy directory tree with optimizations."""
    FastFileOps.copy_tree(src, dst, ignore)


def move_file(src: Path | str, dst: Path | str) -> None:
    """Move file or directory."""
    FastFileOps.move(src, dst)


def remove_path(path: Path | str, recursive: bool = False) -> None:
    """Remove file or directory."""
    FastFileOps.remove(path, recursive)


def get_path_size(path: Path | str) -> int:
    """Get file or directory size."""
    return FastFileOps.get_size(path)


def ensure_directory(path: Path | str, mode: int = 0o755) -> Path:
    """Ensure directory exists."""
    return FastFileOps.ensure_dir(path, mode)
