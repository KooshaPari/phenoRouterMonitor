"""OS User Management for Layer 1 Isolation.

Handles creation, deletion, and management of real system accounts
across macOS, Linux, and Windows.
"""

import logging
import platform
from thegent.infra.shim_subprocess import run as shim_run
from dataclasses import dataclass
from pathlib import Path

logger = logging.getLogger(__name__)


@dataclass
class OSUser:
    username: str
    uid: int | None = None
    gid: int | None = None
    home_dir: str | None = None
    is_created: bool = False


class OSUserManager:
    """
    Manages real OS-level user accounts for L1 identity.
    Requires administrative privileges for most operations.
    """

    def __init__(self, prefix: str = "tg_") -> None:
        self.prefix = prefix
        self.os_type = platform.system().lower()

    def create_user(self, name: str, home_base: str | None = None) -> OSUser:
        """Create a new system user if it doesn't exist."""
        username = f"{self.prefix}{name}"

        if self._user_exists(username):
            logger.debug(f"User {username} already exists.")
            return self._get_user_info(username)

        logger.info(f"Creating OS user: {username} on {self.os_type}")

        try:
            if self.os_type == "linux":
                self._create_linux_user(username, home_base)
            elif self.os_type == "darwin":
                self._create_macos_user(username, home_base)
            elif self.os_type == "windows":
                self._create_windows_user(username, home_base)
            else:
                raise RuntimeError(f"OS User creation not supported on {self.os_type}")

            return self._get_user_info(username)
        except Exception as e:
            logger.error(f"Failed to create user {username}: {e}")
            raise

    def delete_user(self, username: str, delete_home: bool = True) -> bool:
        """Remove a system user."""
        if not self._user_exists(username):
            return True

        logger.info(f"Deleting OS user: {username}")
        try:
            if self.os_type == "linux":
                cmd = ["userdel", "-r" if delete_home else "", username]
            elif self.os_type == "darwin":
                # macOS deletion is multi-step or uses sysadminctl
                cmd = ["sysadminctl", "-deleteUser", username]
            elif self.os_type == "windows":
                cmd = ["powershell.exe", "-Command", f"Remove-LocalUser -Name '{username}'"]

            shim_run(cmd, check=True, capture_output=True)
            return True
        except Exception as e:
            logger.error(f"Failed to delete user {username}: {e}")
            return False

    def _user_exists(self, username: str) -> bool:
        """Check if user exists in the system."""
        try:
            if self.os_type == "windows":
                res = shim_run(
                    ["powershell.exe", "-Command", f"Get-LocalUser -Name '{username}'"],
                    capture_output=True,
                    check=False,
                )
                return res.returncode == 0
            import pwd

            pwd.getpwnam(username)
            return True
        except (KeyError, ImportError):
            return False

    def _get_user_info(self, username: str) -> OSUser:
        """Fetch UID/GID/Home for an existing user."""
        if self.os_type == "windows":
            # For Windows, we don't have UIDs in the POSIX sense, but SIDs
            # We'll map the SID to a pseudo-UID later if needed for WSL2
            return OSUser(username=username, is_created=True)

        import pwd

        info = pwd.getpwnam(username)
        return OSUser(username=username, uid=info.pw_uid, gid=info.pw_gid, home_dir=info.pw_dir, is_created=True)

    def _create_linux_user(self, username: str, home_base: str | None) -> None:
        """Linux-specific user creation."""
        # -r: system account, -m: create home, -s: shell
        cmd = ["useradd", "-r", "-m", "-s", "/bin/bash"]
        if home_base:
            cmd.extend(["-d", str(Path(home_base) / username)])
        cmd.append(username)

        shim_run(cmd, check=True, capture_output=True)

    def _create_macos_user(self, username: str, home_base: str | None) -> None:
        """macOS-specific user creation using sysadminctl or dscl."""
        # Using sysadminctl is cleaner on modern macOS
        cmd = ["sysadminctl", "-addUser", username, "-fullName", f"TheGent Agent {username}", "-type", "standard"]
        # Note: In a real system, we'd also handle password/secure-token if needed
        # but for internal agent accounts, we might want them hidden.
        shim_run(cmd, check=True, capture_output=True)

        # Hide the user from the login screen
        shim_run(["dscl", ".", "create", f"/Users/{username}", "IsHidden", "1"], check=True)

    def _create_windows_user(self, username: str, home_base: str | None) -> None:
        """Windows-specific user creation."""
        # -NoPassword for simple local accounts (requires elevated PS)
        ps_cmd = f"New-LocalUser -Name '{username}' -Description 'TheGent Agent Identity' -NoPassword"
        shim_run(["powershell.exe", "-Command", ps_cmd], check=True, capture_output=True)
