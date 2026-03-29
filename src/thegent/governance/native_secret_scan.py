"""Native secret scanning integration for the hook-dispatcher binary.

Provides ``scan_secrets(content)`` which delegates to the
``hook-dispatcher scan-secrets --stdin`` subcommand (BKM-07).  Falls back to a
pure-Python regex implementation when the binary is not found, so the module
is always functional regardless of whether the Rust toolchain has been
compiled.

Traces to: FR-SEC-001 (secret detection), FR-GOV-006 (native binary integration)
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
class SecretMatch:
    """A single secret detected in content."""

    kind: str
    """Human-readable type label, e.g. ``"aws_access_key_id"``."""

    line: int
    """1-based line number of the match."""

    masked: str
    """Masked representation of the matched text (never the raw secret)."""


# ---------------------------------------------------------------------------
# Binary resolution
# ---------------------------------------------------------------------------

_BINARY_NAME: Final = "hook-dispatcher"

# Search order for the binary.  The build artifact is preferred when present.
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
    # Also check PATH
    return shutil.which(_BINARY_NAME)


# ---------------------------------------------------------------------------
# Python fallback patterns
# ---------------------------------------------------------------------------

_FALLBACK_PATTERNS: Final[list[tuple[str, re.Pattern[str]]]] = [
    ("openai_api_key", re.compile(r"sk-[a-zA-Z0-9]{48}")),
    ("openai_proj_key", re.compile(r"sk-proj-[a-zA-Z0-9_\-]{48,}")),
    ("anthropic_api_key", re.compile(r"sk-ant-[a-zA-Z0-9_\-]{90,}")),
    ("google_cloud_key", re.compile(r"AIza[0-9A-Za-z\-_]{35}")),
    ("slack_token", re.compile(r"xox[baprs]-[0-9A-Za-z\-]{10,}")),
    ("private_key_block", re.compile(r"-----BEGIN [A-Z ]+ PRIVATE KEY-----")),
    ("square_access_token", re.compile(r"sq0atp-[0-9A-Za-z\-_]{22}")),
    ("aws_access_key_id", re.compile(r"AKIA[0-9A-Z]{16}")),
    ("aws_secret_key_context", re.compile(r"(?i)(aws_secret_access_key|secret_access_key)\s*[=:]\s*\S{20,}")),
    ("github_pat", re.compile(r"ghp_[a-zA-Z0-9]{36}")),
    ("github_oauth", re.compile(r"gho_[a-zA-Z0-9]{36}")),
    ("github_app_token", re.compile(r"ghs_[a-zA-Z0-9]{36}")),
    ("generic_hex_secret", re.compile(r"(?i)(password|secret|token|api[_\-]?key)\s*[=:]\s*[0-9a-f]{20,}")),
    (
        "generic_base64_secret",
        re.compile(r"(?i)(password|secret|token|api[_\-]?key)\s*[=:]\s*[A-Za-z0-9+/]{32,}={0,2}"),
    ),
]


def _mask(text: str) -> str:
    """Mask a secret string: keep first 4 + last 2 chars, hide the rest."""
    if len(text) <= 8:
        return "****"
    return f"{text[:4]}****{text[-2:]}"


def _python_scan(content: str) -> list[SecretMatch]:
    """Pure-Python fallback: scan content for secrets using regex patterns."""
    matches: list[SecretMatch] = []
    for line_idx, line in enumerate(content.splitlines(), start=1):
        for kind, pattern in _FALLBACK_PATTERNS:
            m = pattern.search(line)
            if m:
                matches.append(SecretMatch(kind=kind, line=line_idx, masked=_mask(m.group(0))))
                break  # one match per pattern set per line is sufficient
    return matches


def _run_binary(binary: str, content: str) -> list[SecretMatch]:
    """Invoke the hook-dispatcher binary and parse JSON output.

    Args:
        binary: Absolute path to hook-dispatcher.
        content: Text to scan via stdin.

    Returns:
        Parsed list of :class:`SecretMatch`, or raises on failure.

    Raises:
        subprocess.TimeoutExpired: Binary did not respond within 30 s.
        json.JSONDecodeError: Binary returned non-JSON output.
        OSError: Binary could not be executed.
    """
    proc = shim_run(
        [binary, "scan-secrets", "--stdin"],
        input=content,
        capture_output=True,
        text=True,
        timeout=30,
        check=False,
    )
    data = json.loads(proc.stdout)
    return [SecretMatch(kind=m["kind"], line=m["line"], masked=m["masked"]) for m in data.get("matches", [])]


# ---------------------------------------------------------------------------
# Public API
# ---------------------------------------------------------------------------


def scan_secrets(content: str) -> list[SecretMatch]:
    """Scan *content* for secrets.

    Delegates to the Rust ``hook-dispatcher scan-secrets --stdin`` binary when
    available.  Falls back to the pure-Python implementation otherwise.

    Args:
        content: Raw text (file content or diff) to inspect.

    Returns:
        A list of :class:`SecretMatch` objects.  Empty list means no secrets
        were detected.

    Example::

        matches = scan_secrets(Path("config.env").read_text())
        for m in matches:
            print(f"  {m.kind} at line {m.line}: {m.masked}")
    """
    binary = _find_binary()
    if binary is None:
        _log.debug("hook-dispatcher binary not found; using Python fallback for secret scan")
        return _python_scan(content)

    try:
        return _run_binary(binary, content)
    except (subprocess.TimeoutExpired, json.JSONDecodeError, KeyError, OSError) as exc:
        _log.warning("hook-dispatcher scan-secrets failed (%s); using Python fallback", exc)
        return _python_scan(content)


def scan_secrets_file(path: str | Path) -> list[SecretMatch]:
    """Convenience wrapper: read *path* and call :func:`scan_secrets`.

    Args:
        path: Path to the file to scan.

    Returns:
        List of :class:`SecretMatch` objects.

    Raises:
        OSError: If the file cannot be read.
    """
    content = Path(path).read_text(errors="replace")
    return scan_secrets(content)
