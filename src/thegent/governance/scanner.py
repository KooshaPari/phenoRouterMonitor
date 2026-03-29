"""Codebase scanner producing structured dimension measurements.

Python port of hooks/gardener-scan.sh.  Runs 8 scan dimensions
(test coverage, lint violations, doc organisation, fragmented research,
missing specs, technical debt, stale items, agent failure) and returns
pydantic models consumable by the health-score computer.
"""

from __future__ import annotations

import orjson as json
import logging
import re
import subprocess
from thegent.infra.shim_subprocess import run as shim_run
import time
from datetime import UTC, datetime
from typing import TYPE_CHECKING

from pydantic import BaseModel, Field

if TYPE_CHECKING:
    from pathlib import Path
else:
    Path = str

_log = logging.getLogger(__name__)

_RAW_OUTPUT_LIMIT = 500


class DimensionScan(BaseModel):
    """Result of a single scan dimension."""

    dimension: str
    current_value: float
    target_value: float
    delta: float
    raw_output: str = ""
    affected_files: list[str] = Field(default_factory=list)
    scan_duration_s: float = 0.0


class ScanResult(BaseModel):
    """Aggregated result of all dimension scans."""

    dimensions: dict[str, DimensionScan]
    scanned_at: str = Field(default_factory=lambda: datetime.now(UTC).isoformat())
    duration_s: float = 0.0
    project_dir: str = ""


def _truncate(text: str, limit: int = _RAW_OUTPUT_LIMIT) -> str:
    if len(text) <= limit:
        return text
    return text[:limit] + "...[truncated]"


def _run_tool(
    args: list[str],
    *,
    cwd: Path,
    timeout: int = 120,
) -> subprocess.CompletedProcess[str]:
    """Run an external tool, returning the CompletedProcess unconditionally."""
    return shim_run(  # noqa: S603 -- args constructed internally
        args,
        capture_output=True,
        text=True,
        cwd=cwd,
        timeout=timeout,
        check=False,
    )


class CodebaseScanner:
    """Scans the codebase across 8 governance dimensions."""

    _DIMENSIONS = (
        "test_coverage",
        "lint_violations",
        "doc_disorganization",
        "fragmented_research",
        "missing_specs",
        "technical_debt",
        "stale_items",
        "agent_failure",
    )

    def __init__(self, project_dir: Path, session_dir: Path) -> None:
        self.project_dir = project_dir.resolve()
        self.session_dir = session_dir.resolve()

    # ------------------------------------------------------------------
    # public API
    # ------------------------------------------------------------------

    def scan_all(self) -> ScanResult:
        """Run every dimension scan and return the aggregated result."""
        t0 = time.monotonic()
        dimensions: dict[str, DimensionScan] = {}
        for dim in self._DIMENSIONS:
            dimensions[dim] = self.scan_dimension(dim)
        elapsed = time.monotonic() - t0
        return ScanResult(
            dimensions=dimensions,
            duration_s=round(elapsed, 3),
            project_dir=str(self.project_dir),
        )

    scan = scan_all  # Alias for AgilePlus compatibility

    def scan_dimension(self, dimension: str) -> DimensionScan:
        """Run a single dimension scan by name."""
        method = getattr(self, f"_scan_{dimension}", None)
        if method is None:
            msg = f"unknown dimension: {dimension}"
            raise ValueError(msg)
        return method()

    # ------------------------------------------------------------------
    # individual dimension scans
    # ------------------------------------------------------------------

    def _scan_test_coverage(self) -> DimensionScan:
        target = 80.0
        t0 = time.monotonic()
        try:
            proc = _run_tool(
                ["pytest", "--cov=src", "--cov-report=term", "-q", "--no-header"],
                cwd=self.project_dir,
                timeout=120,
            )
            output = proc.stdout + proc.stderr
            match = re.search(r"(\d+)%", output)
            current = float(match.group(1)) if match else 0.0
        except FileNotFoundError:
            _log.debug("pytest not found, skipping test_coverage scan")
            output = "pytest not found"
            current = 0.0
        except subprocess.TimeoutExpired:
            _log.warning("pytest timed out during test_coverage scan")
            output = "pytest timed out"
            current = 0.0

        return DimensionScan(
            dimension="test_coverage",
            current_value=current,
            target_value=target,
            delta=current - target,
            raw_output=_truncate(output),
            scan_duration_s=round(time.monotonic() - t0, 3),
        )

    def _scan_lint_violations(self) -> DimensionScan:
        target = 0.0
        t0 = time.monotonic()
        try:
            proc = _run_tool(
                ["ruff", "check", "."],
                cwd=self.project_dir,
                timeout=60,
            )
            output = proc.stdout
            lines = [ln for ln in output.splitlines() if ln.strip()]
            current = float(len(lines))
        except FileNotFoundError:
            _log.debug("ruff not found, skipping lint_violations scan")
            output = "ruff not found"
            current = 0.0
        except subprocess.TimeoutExpired:
            _log.warning("ruff timed out during lint_violations scan")
            output = "ruff timed out"
            current = 0.0

        return DimensionScan(
            dimension="lint_violations",
            current_value=current,
            target_value=target,
            delta=target - current,
            raw_output=_truncate(output),
            scan_duration_s=round(time.monotonic() - t0, 3),
        )

    def _scan_doc_disorganization(self) -> DimensionScan:
        target = 0.0
        t0 = time.monotonic()
        required_dirs = ["docs/guides", "docs/reference", "docs/reports"]
        missing: list[str] = []
        for d in required_dirs:
            full = self.project_dir / d
            if not full.is_dir():
                missing.append(d)
        current = float(len(missing))

        return DimensionScan(
            dimension="doc_disorganization",
            current_value=current,
            target_value=target,
            delta=target - current,
            raw_output=f"missing dirs: {missing}" if missing else "all required dirs present",
            affected_files=missing,
            scan_duration_s=round(time.monotonic() - t0, 3),
        )

    def _scan_fragmented_research(self) -> DimensionScan:
        target = 0.0
        t0 = time.monotonic()
        docs_dir = self.project_dir / "docs"
        research_dir = self.project_dir / "docs" / "research"
        fragmented: list[str] = []

        if docs_dir.is_dir():
            for p in docs_dir.rglob("*research*"):
                if p.is_file() and not _is_under(p, research_dir):
                    fragmented.append(str(p.relative_to(self.project_dir)))

        current = float(len(fragmented))
        return DimensionScan(
            dimension="fragmented_research",
            current_value=current,
            target_value=target,
            delta=target - current,
            raw_output=f"{len(fragmented)} research file(s) outside docs/research/",
            affected_files=fragmented,
            scan_duration_s=round(time.monotonic() - t0, 3),
        )

    def _scan_missing_specs(self) -> DimensionScan:
        target = 0.0
        t0 = time.monotonic()
        approved_dir = self.project_dir / "specs" / "approved"
        missing_specs: list[str] = []

        if approved_dir.is_dir():
            for child in approved_dir.iterdir():
                if child.is_dir() and not (child / "SPEC.md").exists():
                    missing_specs.append(str(child.relative_to(self.project_dir)))

        current = float(len(missing_specs))
        return DimensionScan(
            dimension="missing_specs",
            current_value=current,
            target_value=target,
            delta=target - current,
            raw_output=f"{len(missing_specs)} approved feature(s) without SPEC.md",
            affected_files=missing_specs,
            scan_duration_s=round(time.monotonic() - t0, 3),
        )

    def _scan_technical_debt(self) -> DimensionScan:
        target = 10.0
        t0 = time.monotonic()
        try:
            proc = _run_tool(
                ["radon", "cc", ".", "-a", "-n", "C"],
                cwd=self.project_dir,
                timeout=120,
            )
            output = proc.stdout + proc.stderr
            match = re.search(r"Average complexity:\s*[A-F]?\s*\(?([\d.]+)\)?", output)
            current = float(match.group(1)) if match else 0.0
        except FileNotFoundError:
            _log.debug("radon not found, skipping technical_debt scan")
            output = "radon not found"
            current = 0.0
        except subprocess.TimeoutExpired:
            _log.warning("radon timed out during technical_debt scan")
            output = "radon timed out"
            current = 0.0

        return DimensionScan(
            dimension="technical_debt",
            current_value=current,
            target_value=target,
            delta=target - current,
            raw_output=_truncate(output),
            scan_duration_s=round(time.monotonic() - t0, 3),
        )

    def _scan_stale_items(self) -> DimensionScan:
        target = 0.0
        stale_days = 7
        t0 = time.monotonic()
        specs_dir = self.project_dir / "specs"
        stale: list[str] = []

        if specs_dir.is_dir():
            cutoff = time.time() - stale_days * 86400
            for p in specs_dir.rglob("*"):
                if p.is_file() and p.stat().st_mtime < cutoff:
                    stale.append(str(p.relative_to(self.project_dir)))

        current = float(len(stale))
        return DimensionScan(
            dimension="stale_items",
            current_value=current,
            target_value=target,
            delta=target - current,
            raw_output=f"{len(stale)} file(s) in specs/ not modified in {stale_days}+ days",
            affected_files=stale,
            scan_duration_s=round(time.monotonic() - t0, 3),
        )

    def _scan_agent_failure(self) -> DimensionScan:
        target = 0.0
        t0 = time.monotonic()
        cb_path = self.session_dir / "circuit_breakers.jsonl"
        open_count = 0
        affected: list[str] = []

        if cb_path.is_file():
            for raw_line in cb_path.read_text().splitlines():
                line = raw_line.strip()
                if not line:
                    continue
                try:
                    record = json.loads(line)
                except json.JSONDecodeError:
                    continue
                if record.get("status") == "OPEN":
                    open_count += 1
                    name = record.get("name", record.get("agent", "unknown"))
                    affected.append(str(name))

        current = float(open_count)
        return DimensionScan(
            dimension="agent_failure",
            current_value=current,
            target_value=target,
            delta=target - current,
            raw_output=f"{open_count} open circuit breaker(s)",
            affected_files=affected,
            scan_duration_s=round(time.monotonic() - t0, 3),
        )


def _is_under(path: Path, parent: Path) -> bool:
    """Return True if *path* is inside *parent* (resolved)."""
    try:
        path.resolve().relative_to(parent.resolve())
        return True
    except ValueError:
        return False
