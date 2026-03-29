"""Autopilot Doctor Command for system health checks.

WL-172: Autopilot Doctor Command
Provides diagnostic checks and health monitoring for the autopilot system.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import Callable


@dataclass
class DoctorCheck:
    """Result of a single diagnostic check."""

    name: str
    passed: bool
    message: str = ""


class AutopilotDoctor:
    """Doctor command for running system health checks."""

    def __init__(self) -> None:
        """Initialize the autopilot doctor."""
        self._checks: dict[str, tuple[Callable[[], bool], str]] = {}

    def add_check(
        self,
        name: str,
        check_fn: Callable[[], bool],
        message: str = "",
    ) -> None:
        """Register a health check.

        Args:
            name: Unique name for the check.
            check_fn: Callable that returns True if check passed, False otherwise.
            message: Optional message to display with results.
        """
        self._checks[name] = (check_fn, message)

    def run(self) -> list[DoctorCheck]:
        """Run all registered checks and return results.

        Returns:
            List of DoctorCheck results.
        """
        results = []
        for name, (check_fn, message) in self._checks.items():
            passed = check_fn()
            results.append(DoctorCheck(name=name, passed=passed, message=message))
        return results

    @staticmethod
    def all_passed(checks: list[DoctorCheck]) -> bool:
        """Check if all health checks passed.

        Args:
            checks: List of DoctorCheck results.

        Returns:
            True if all checks passed, False otherwise.
        """
        return all(check.passed for check in checks)
