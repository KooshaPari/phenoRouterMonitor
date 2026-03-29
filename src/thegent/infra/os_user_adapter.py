"""OS-level user management adapter for cross-platform isolation."""

import os
import platform
import shutil
from thegent.infra.shim_subprocess import run as shim_run
from pathlib import Path


class OSUserAdapter:
    """Adapter for creating and managing OS-level users for agent isolation."""

    def __init__(self) -> None:
        self.system = platform.system()

    def _run_privileged(self, cmd: list[str]) -> tuple[bool, str]:
        """Run a command with elevated privileges (sudo/admin)."""
        # 1. On Unix-like systems, try sudo if not already root
        if self.system in ["Linux", "Darwin"]:
            if os.geteuid() != 0:
                if shutil.which("sudo"):
                    cmd = ["sudo", "-n"] + cmd  # -n for non-interactive
                else:
                    return False, "Not root and sudo not found"

        # 2. On Windows, assume the process is already elevated or the command handles elevation
        # (Windows doesn't have a direct 'sudo' in standard shell, needs ShellExecute with 'runas')

        try:
            result = shim_run(cmd, capture_output=True, text=True, check=False)
            if result.returncode == 0:
                return True, result.stdout.strip()
            return False, result.stderr.strip()
        except Exception as e:
            return False, str(e)

    def create_os_user(self, username: str, home_dir: Path | None = None) -> tuple[bool, str]:
        """Create a new OS user for agent isolation."""
        if self.system == "Linux":
            return self._create_linux_user(username, home_dir)
        if self.system == "Darwin":
            return self._create_macos_user(username, home_dir)
        if self.system == "Windows":
            return self._create_windows_user(username, home_dir)
        return False, f"Unsupported OS: {self.system}"

    def _create_linux_user(self, username: str, home_dir: Path | None) -> tuple[bool, str]:
        """Create a user on Linux using useradd."""
        cmd = ["useradd", "-m"]
        if home_dir:
            cmd.extend(["-d", str(home_dir)])
        cmd.append(username)

        return self._run_privileged(cmd)

    def _create_macos_user(self, username: str, home_dir: Path | None) -> tuple[bool, str]:
        """Create a user on macOS using dscl."""
        # Note: macOS user creation via dscl is complex and needs a unique UID
        # This is a simplified version; in production, we'd need to find the next available UID
        uid = "501"  # Placeholder for discovery logic

        success, next_uid = self._run_privileged(["dscl", ".", "-list", "/Users", "UniqueID"])
        if success:
            uids = []
            for value in next_uid.splitlines():
                parts = value.split()
                if len(parts) < 2:
                    continue
                try:
                    uids.append(int(parts[1]))
                except ValueError:
                    continue
            uid = str(max(uids) + 1) if uids else "505"

        commands = [
            ["dscl", ".", "-create", f"/Users/{username}"],
            ["dscl", ".", "-create", f"/Users/{username}", "UserShell", "/bin/zsh"],
            ["dscl", ".", "-create", f"/Users/{username}", "RealName", f"Agent User {username}"],
            ["dscl", ".", "-create", f"/Users/{username}", "UniqueID", uid],
            ["dscl", ".", "-create", f"/Users/{username}", "PrimaryGroupID", "20"],  # staff group
            ["dscl", ".", "-create", f"/Users/{username}", "NFSHomeDirectory", str(home_dir or f"/Users/{username}")],
            ["dscl", ".", "-passwd", f"/Users/{username}", "*"],  # No password
        ]

        for cmd in commands:
            ok, msg = self._run_privileged(cmd)
            if not ok:
                return False, f"macOS user creation failed at step {cmd}: {msg}"

        # Create home directory if needed
        if home_dir:
            home_dir.mkdir(parents=True, exist_ok=True)
            self._run_privileged(["chown", f"{username}:staff", str(home_dir)])

        return True, f"User {username} created on macOS with UID {uid}"

    def _create_windows_user(self, username: str, home_dir: Path | None) -> tuple[bool, str]:
        """Create a local user on Windows using PowerShell."""
        # New-LocalUser -Name "username" -NoPassword
        ps_cmd = f'New-LocalUser -Name "{username}" -NoPassword'
        if home_dir:
            # Setting home dir on Windows is usually done via WMI or Registry,
            # New-LocalUser doesn't have a direct parameter for it.
            safe_home = str(home_dir).replace('"', '\\"')
            ps_cmd = (
                f'{ps_cmd} ; New-Item -ItemType Directory -Path "{safe_home}" -Force | Out-Null ; '
                f'Set-LocalUser -Name "{username}" -HomeDirectory "{safe_home}"'
            )

        cmd = ["powershell.exe", "-NoProfile", "-NonInteractive", "-Command", ps_cmd]

        # Windows doesn't have 'sudo', we expect to be running as admin.
        try:
            result = shim_run(cmd, capture_output=True, text=True, check=False)
            if result.returncode == 0:
                return True, result.stdout.strip()
            return False, result.stderr.strip()
        except Exception as e:
            return False, str(e)


def create_os_user(username: str, home_dir: Path | None = None) -> tuple[bool, str]:
    """Factory function for create_os_user."""
    return OSUserAdapter().create_os_user(username, home_dir)
