"""WSL2 interop and path translation utilities."""

import os
import platform
import re
from thegent.infra.shim_subprocess import run as shim_run
from hashlib import sha256


class WslInterop:
    """
    Utilities for seamless interop between native Windows and WSL2.

    Provides high-performance path translation and identity mapping.
    """

    def __init__(self) -> None:
        self.is_wsl = self._detect_wsl()
        self.is_windows = platform.system().lower() == "windows"
        self._sid_uid_namespace = "thegent:wsl-sid-to-uid:v1"
        self._sid_uid_min = 2000
        self._sid_uid_max = 9000
        self._uid_assignments: dict[str, int] = {}
        self._used_uids: dict[int, str] = {}

        # Pre-calculated drive mount points (default /mnt/c/)
        self.wsl_mount_root = "/mnt/"
        self.drive_regex_win = re.compile(r"^([A-Za-z]):\\(.*)")
        self.drive_regex_wsl = re.compile(r"^/mnt/([a-z])/(.*)")

    def _detect_wsl(self) -> bool:
        """Detect if running inside WSL."""
        if platform.system().lower() != "linux":
            return False

        # Check /proc/version for Microsoft/WSL string
        try:
            with open("/proc/version") as f:
                content = f.read().lower()
                return "microsoft" in content or "wsl" in content
        except Exception:
            return False

    def to_wsl_path(self, windows_path: str) -> str:
        """
        Convert a Windows path to a WSL path.

        Uses fast-path regex if possible, falls back to wslpath.
        """
        if not self.is_windows and not self.is_wsl:
            return windows_path

        # Fast-path: regex conversion for common drive letters
        match = self.drive_regex_win.match(windows_path)
        if match:
            drive = match.group(1).lower()
            rest = match.group(2).replace("\\", "/")
            return f"{self.wsl_mount_root}{drive}/{rest}"

        # Fallback: call wslpath if available
        if self.is_wsl:
            try:
                result = shim_run(["wslpath", "-a", windows_path], capture_output=True, text=True, check=True)
                return result.stdout.strip()
            except Exception:
                pass

        return windows_path

    def to_windows_path(self, wsl_path: str) -> str:
        """
        Convert a WSL path to a Windows path.

        Uses fast-path regex if possible, falls back to wslpath.
        """
        if not self.is_windows and not self.is_wsl:
            return wsl_path

        # Fast-path: regex conversion for /mnt/x/ style paths
        match = self.drive_regex_wsl.match(wsl_path)
        if match:
            drive = match.group(1).upper()
            rest = match.group(2).replace("/", "\\")
            return f"{drive}:\\{rest}"

        # Fallback: call wslpath if available
        if self.is_wsl:
            try:
                result = shim_run(["wslpath", "-w", wsl_path], capture_output=True, text=True, check=True)
                return result.stdout.strip()
            except Exception:
                pass

        return wsl_path

    def get_windows_user_profile(self) -> str | None:
        """Get the Windows user profile path (C:\\Users\\Name)."""
        if self.is_windows:
            return os.environ.get("USERPROFILE")

        if self.is_wsl:
            # Under WSL, we can often find it via /mnt/c/Users/...
            # or by calling powershell.exe $env:USERPROFILE
            try:
                result = shim_run(
                    ["powershell.exe", "-Command", "$env:USERPROFILE"], capture_output=True, text=True, check=True
                )
                win_path = result.stdout.strip()
                return self.to_wsl_path(win_path)
            except Exception:
                pass

        return None

    def map_sid_to_uid(self, sid: str) -> int:
        """
        Map a Windows SID to a WSL2 UID.

        Implementation logic:
        1. deterministic SID fingerprinting with bounded UID search.
        2. collision-aware probing when a UID collision occurs.
        3. deterministic fallback namespace for reproducible outputs.
        """
        if not sid:
            raise ValueError("sid must be a non-empty string")

        existing = self._uid_assignments.get(sid)
        if existing is not None:
            return existing

        uid_span = self._sid_uid_max - self._sid_uid_min + 1
        for probe in range(uid_span):
            fingerprint = self._sid_fingerprint(sid, probe)
            uid = self._sid_uid_min + (fingerprint % uid_span)
            owner_sid = self._used_uids.get(uid)
            if owner_sid is None:
                self._uid_assignments[sid] = uid
                self._used_uids[uid] = sid
                return uid
            if owner_sid == sid:
                return uid

        raise RuntimeError(f"Unable to allocate a stable UID for SID {sid}")

    def _sid_fingerprint(self, sid: str, probe: int) -> int:
        payload = f"{self._sid_uid_namespace}|{sid}|{probe}".encode()
        return int.from_bytes(sha256(payload).digest()[:8], "big")
