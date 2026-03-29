"""Native governance scanning integration for the hook-dispatcher binary.

Provides ``NativeGovernanceScanner`` which delegates to the
``hook-dispatcher governance`` subcommand (BKM-11).  Falls back to a
pure-Python regex implementation when the binary is not found, so the module
is always functional regardless of whether the Rust toolchain has been
compiled.

Traces to: FR-GOV-007 (governance violation detection), FR-GOV-006 (native binary integration)
"""

from __future__ import annotations

import orjson as json
import logging
import re
import shutil
import subprocess
from thegent.infra.shim_subprocess import run as shim_run
from dataclasses import dataclass
from pathlib import Path
from typing import Final

_log = logging.getLogger(__name__)

# ---------------------------------------------------------------------------
# Public types
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class GovernanceViolation:
    """A single governance violation detected in source content."""

    rule: str
    """Rule ID that was violated (e.g. ``"noqa-no-justification"``)."""

    severity: str
    """Severity level: ``"error"``, ``"warning"``, or ``"info"``."""

    line: int
    """1-based line number of the violation."""

    message: str
    """Human-readable description of the violation."""


# ---------------------------------------------------------------------------
# Binary resolution (reuses same paths as native_secret_scan)
# ---------------------------------------------------------------------------

_BINARY_NAME: Final = "hook-dispatcher"

_BINARY_SEARCH_PATHS: Final[tuple[str, ...]] = (
    str(
        Path(__file__).parent.parent.parent.parent
        / "hooks"
        / "hook-dispatcher"
        / "target"
        / "release"
        / "hook-dispatcher"
    ),
    str(Path(__file__).parent.parent.parent.parent / "hooks" / "bin" / "hook-dispatcher"),
)


def _find_binary() -> str | None:
    """Return the absolute path to the hook-dispatcher binary, or None."""
    for candidate in _BINARY_SEARCH_PATHS:
        if Path(candidate).is_file():
            return candidate
    return shutil.which(_BINARY_NAME)


# ---------------------------------------------------------------------------
# Python fallback implementations
# ---------------------------------------------------------------------------

# Suppression annotation detection: bare suppression or suppression with code,
# but without the required justification marker " -- reason".
_SUPPRESSION_BARE_RE: Final = re.compile(r"#\s*noqa", re.IGNORECASE)
_SUPPRESSION_JUSTIFIED_RE: Final = re.compile(r"#\s*noqa(?::\s*\S+)?\s+--\s", re.IGNORECASE)

# TODO/FIXME/HACK/XXX keywords
_TODO_KEYWORD_RE: Final = re.compile(r"\b(TODO|FIXME|HACK|XXX)\b", re.IGNORECASE)

# Ticket reference patterns: #123, PROJ-456, [#789]
_TICKET_REF_RE: Final = re.compile(r"#\d+|[A-Z][A-Z0-9]+-\d+|\[#\d+\]")

# Hardcoded credential pattern
_HARDCODED_CRED_RE: Final = re.compile(
    r"""(?i)(password|passwd|pwd|secret|api_key|apikey|access_token|auth_token)\s*=\s*["'][^"']{4,}["']"""
)

# Maximum function length in lines
_MAX_FUNCTION_LINES: Final = 40

# Python `def` line detection
_DEF_RE: Final = re.compile(r"^(\s*)(?:async\s+)?def\s+\w+")
_CLASS_OR_DEF_RE: Final = re.compile(r"^(\s*)(?:class|(?:async\s+)?def)\s+\w+")


def _python_scan_noqa(content: str) -> list[GovernanceViolation]:
    """Detect suppression annotations without inline justification."""
    violations: list[GovernanceViolation] = []
    for line_idx, line in enumerate(content.splitlines(), start=1):
        if _SUPPRESSION_BARE_RE.search(line) and not _SUPPRESSION_JUSTIFIED_RE.search(line):
            violations.append(
                GovernanceViolation(
                    rule="noqa-no-justification",
                    severity="error",
                    line=line_idx,
                    message=(f"Suppression annotation at line {line_idx} lacks inline justification (`-- reason`)."),
                )
            )
    return violations


def _python_scan_todo_no_ticket(content: str) -> list[GovernanceViolation]:
    """Detect TODO/FIXME/HACK comments without a ticket reference."""
    violations: list[GovernanceViolation] = []
    for line_idx, line in enumerate(content.splitlines(), start=1):
        m = _TODO_KEYWORD_RE.search(line)
        if m and not _TICKET_REF_RE.search(line):
            keyword = m.group(0).upper()
            violations.append(
                GovernanceViolation(
                    rule="todo-no-ticket",
                    severity="warning",
                    line=line_idx,
                    message=(f"{keyword} at line {line_idx} has no ticket reference (e.g. #123 or PROJ-456)."),
                )
            )
    return violations


def _python_scan_function_length(
    content: str,
    max_lines: int = _MAX_FUNCTION_LINES,
) -> list[GovernanceViolation]:
    """Detect Python functions that exceed *max_lines* lines."""
    lines = content.splitlines()
    violations: list[GovernanceViolation] = []

    i = 0
    while i < len(lines):
        m = _DEF_RE.match(lines[i])
        if m:
            def_indent = len(m.group(1))
            def_line_no = i + 1
            func_start = i

            j = i + 1
            while j < len(lines):
                stripped = lines[j].strip()
                # Skip blank lines and comments
                if not stripped or stripped.startswith("#"):
                    j += 1
                    continue
                indent = len(lines[j]) - len(lines[j].lstrip())
                mc = _CLASS_OR_DEF_RE.match(lines[j])
                if mc and indent <= def_indent:
                    break
                j += 1

            func_len = j - func_start
            if func_len > max_lines:
                violations.append(
                    GovernanceViolation(
                        rule="function-too-long",
                        severity="warning",
                        line=def_line_no,
                        message=(f"Function at line {def_line_no} is {func_len} lines long (max {max_lines})."),
                    )
                )
            i = j
        else:
            i += 1

    return violations


def _python_scan_hardcoded_creds(content: str) -> list[GovernanceViolation]:
    """Detect hardcoded credential patterns."""
    violations: list[GovernanceViolation] = []
    for line_idx, line in enumerate(content.splitlines(), start=1):
        if _HARDCODED_CRED_RE.search(line):
            violations.append(
                GovernanceViolation(
                    rule="hardcoded-credential",
                    severity="error",
                    line=line_idx,
                    message=f"Possible hardcoded credential at line {line_idx}.",
                )
            )
    return violations


def _python_scan_all(content: str) -> list[GovernanceViolation]:
    """Run all Python-fallback governance rules against *content*."""
    violations: list[GovernanceViolation] = []
    violations.extend(_python_scan_noqa(content))
    violations.extend(_python_scan_todo_no_ticket(content))
    violations.extend(_python_scan_function_length(content))
    violations.extend(_python_scan_hardcoded_creds(content))
    violations.sort(key=lambda v: v.line)
    return violations


# Contract -> rule-subset mapping (mirrors Rust implementation)
_CONTRACT_RULE_MAP: Final[dict[str, str]] = {
    "P2-PRIVACY": "secret-detection",
    "secret-detection": "secret-detection",
    "suppression-policy": "noqa-policy",
    "noqa-policy": "noqa-policy",
    "todo-policy": "todo-policy",
    "complexity-policy": "function-length",
    "function-length": "function-length",
}


def _python_check_contract(contract_id: str, content: str) -> list[GovernanceViolation]:
    """Run the Python-fallback rules for a specific *contract_id*."""
    rule_set = _CONTRACT_RULE_MAP.get(contract_id)
    if rule_set == "secret-detection":
        violations = _python_scan_hardcoded_creds(content)
    elif rule_set == "noqa-policy":
        violations = _python_scan_noqa(content)
    elif rule_set == "todo-policy":
        violations = _python_scan_todo_no_ticket(content)
    elif rule_set == "function-length":
        violations = _python_scan_function_length(content)
    else:
        violations = _python_scan_all(content)
    violations.sort(key=lambda v: v.line)
    return violations


# ---------------------------------------------------------------------------
# Binary invocation helpers
# ---------------------------------------------------------------------------


def _parse_binary_output(stdout: str) -> list[GovernanceViolation]:
    """Parse JSON output from the hook-dispatcher governance subcommand."""
    data = json.loads(stdout)
    return [
        GovernanceViolation(
            rule=v["rule"],
            severity=v["severity"],
            line=v["line"],
            message=v["message"],
        )
        for v in data.get("violations", [])
    ]


def _run_binary_scan(binary: str, content: str) -> list[GovernanceViolation]:
    """Invoke ``hook-dispatcher governance scan --stdin`` and parse output.

    Args:
        binary: Absolute path to hook-dispatcher.
        content: Text to scan via stdin.

    Returns:
        Parsed list of :class:`GovernanceViolation`.

    Raises:
        subprocess.TimeoutExpired: Binary did not respond within 30 s.
        json.JSONDecodeError: Binary returned non-JSON output.
        OSError: Binary could not be executed.
    """
    proc = shim_run(
        [binary, "governance", "scan", "--stdin"],
        input=content,
        capture_output=True,
        text=True,
        timeout=30,
        check=False,
    )
    return _parse_binary_output(proc.stdout)


def _run_binary_check_contract(
    binary: str,
    contract_id: str,
    content: str,
) -> list[GovernanceViolation]:
    """Invoke ``hook-dispatcher governance check-contract`` and parse output.

    Args:
        binary: Absolute path to hook-dispatcher.
        contract_id: Contract identifier (e.g. ``"P2-PRIVACY"``).
        content: Text to scan via stdin.

    Returns:
        Parsed list of :class:`GovernanceViolation`.

    Raises:
        subprocess.TimeoutExpired: Binary did not respond within 30 s.
        json.JSONDecodeError: Binary returned non-JSON output.
        OSError: Binary could not be executed.
    """
    proc = shim_run(
        [binary, "governance", "check-contract", contract_id, "--stdin"],
        input=content,
        capture_output=True,
        text=True,
        timeout=30,
        check=False,
    )
    return _parse_binary_output(proc.stdout)


# ---------------------------------------------------------------------------
# Public API
# ---------------------------------------------------------------------------


class NativeGovernanceScanner:
    """Governance scanner that uses the Rust hook-dispatcher binary when available.

    Falls back to a pure-Python implementation when the binary is absent or
    returns an error, ensuring the scanner is always operational.

    Example::

        scanner = NativeGovernanceScanner()
        violations = scanner.scan_file(Path("src/foo.py"))
        for v in violations:
            print(f"  [{v.severity}] {v.rule} at line {v.line}: {v.message}")
    """

    def scan_file(self, path: Path) -> list[GovernanceViolation]:
        """Scan a single file for governance violations.

        Args:
            path: Path to the file to scan.

        Returns:
            List of :class:`GovernanceViolation` objects.  Empty list means
            no violations were detected.

        Raises:
            OSError: If the file cannot be read.
        """
        content = Path(path).read_text(errors="replace")
        return self.scan_content(content)

    def scan_content(self, content: str) -> list[GovernanceViolation]:
        """Scan raw *content* for governance violations.

        Delegates to the Rust binary when available; falls back to Python
        otherwise.

        Args:
            content: Raw text to inspect.

        Returns:
            List of :class:`GovernanceViolation` objects.
        """
        binary = _find_binary()
        if binary is None:
            _log.debug("hook-dispatcher binary not found; using Python fallback for governance scan")
            return _python_scan_all(content)

        try:
            return _run_binary_scan(binary, content)
        except (subprocess.TimeoutExpired, json.JSONDecodeError, KeyError, OSError) as exc:
            _log.warning(
                "hook-dispatcher governance scan failed (%s); using Python fallback",
                exc,
            )
            return _python_scan_all(content)

    def check_contract(
        self,
        contract_id: str,
        path: Path,
    ) -> list[GovernanceViolation]:
        """Check a file against a specific governance contract.

        Args:
            contract_id: Contract identifier (e.g. ``"P2-PRIVACY"``).
            path: Path to the file to check.

        Returns:
            List of :class:`GovernanceViolation` objects relevant to the
            specified contract.  Empty list means no violations.

        Raises:
            OSError: If the file cannot be read.
        """
        content = Path(path).read_text(errors="replace")
        return self.check_contract_content(contract_id, content)

    def check_contract_content(
        self,
        contract_id: str,
        content: str,
    ) -> list[GovernanceViolation]:
        """Check raw *content* against a specific governance contract.

        Args:
            contract_id: Contract identifier (e.g. ``"P2-PRIVACY"``).
            content: Raw text to inspect.

        Returns:
            List of :class:`GovernanceViolation` objects.
        """
        binary = _find_binary()
        if binary is None:
            _log.debug("hook-dispatcher binary not found; using Python fallback for contract check")
            return _python_check_contract(contract_id, content)

        try:
            return _run_binary_check_contract(binary, contract_id, content)
        except (subprocess.TimeoutExpired, json.JSONDecodeError, KeyError, OSError) as exc:
            _log.warning(
                "hook-dispatcher governance check-contract failed (%s); using Python fallback",
                exc,
            )
            return _python_check_contract(contract_id, content)
