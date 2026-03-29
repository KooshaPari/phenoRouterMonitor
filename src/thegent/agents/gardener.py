"""Automated documentation synthesis agent (Gardener).

# @trace WL-060

Reads from memory logs, conversation dumps, and governance events;
detects stale documentation; synthesizes rule-based updates and writes
them back to the relevant docs.
"""

from __future__ import annotations

import logging
import re
import time
from dataclasses import dataclass
from pathlib import Path

_log = logging.getLogger(__name__)

# ---------------------------------------------------------------------------
# Data classes
# ---------------------------------------------------------------------------


@dataclass
class SourceDocument:
    """A source document read by the gardener.

    # @trace WL-060
    """

    path: Path
    content: str
    last_modified: float


@dataclass
class StaleDoc:
    """A documentation file detected as stale.

    # @trace WL-060
    """

    path: Path
    reason: str
    suggested_action: str


@dataclass
class GardenResult:
    """Result of a full garden cycle.

    # @trace WL-060
    """

    docs_checked: int
    docs_updated: int
    items_found: list[str]
    dry_run: bool


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

_WL_COMPLETED_RE = re.compile(
    r"WL-(\d+).*?(?:COMPLETED|completed|Complete|Status.*?completed)",
    re.IGNORECASE | re.DOTALL,
)
_WL_STATUS_PENDING_RE = re.compile(
    r"\[WL-(\d+)\].*?\*\*Status:\*\*\s*pending",
    re.IGNORECASE | re.DOTALL,
)

_MAX_GOVERNANCE_EVENTS = 100


def _extract_completed_wl_ids(content: str) -> set[str]:
    """Return WL item IDs mentioned as completed in *content*."""
    completed: set[str] = set()

    # Pattern: "WL-NNN ... COMPLETED" within a reasonable span
    for match in re.finditer(r"\[WL-(\d+)\]", content, re.IGNORECASE):
        wl_id = match.group(1)
        window = content[match.start() : match.start() + 300]
        if re.search(r"COMPLETED|Status.*?completed", window, re.IGNORECASE):
            completed.add(wl_id)

    # Also catch "WL-NNN: COMPLETED" shorthand
    for match in re.finditer(r"WL-(\d+)[^\n]*?COMPLETED", content, re.IGNORECASE):
        completed.add(match.group(1))

    return completed


def _extract_pending_wl_ids(content: str) -> set[str]:
    """Return WL item IDs whose Status is pending in *content*."""
    pending: set[str] = set()
    for match in re.finditer(
        r"\[WL-(\d+)\].*?\*\*Status:\*\*\s*pending",
        content,
        re.IGNORECASE | re.DOTALL,
    ):
        pending.add(match.group(1))
    return pending


# ---------------------------------------------------------------------------
# GardenerAgent
# ---------------------------------------------------------------------------


class GardenerAgent:
    """Automated documentation synthesis agent.

    # @trace WL-060

    Reads memory logs, conversation dumps, and governance events; detects
    stale documentation entries; synthesizes rule-based updates; and
    writes (or dry-runs) patches back to the relevant docs.
    """

    def __init__(
        self,
        dry_run: bool = False,
        project_root: Path | None = None,
        memory_dir: Path | None = None,
    ) -> None:
        """Initialise the gardener.

        Args:
            dry_run: When True, detect and synthesise but do not write files.
            project_root: Root of the project.  Defaults to cwd.
            memory_dir: Override for the agent memory directory.
                Defaults to ``~/.thegent/memory``.
        """
        self.dry_run = dry_run
        self.project_root = project_root or Path.cwd()
        self.memory_dir = memory_dir or (Path.home() / ".thegent" / "memory")

    # ------------------------------------------------------------------
    # Source reading
    # ------------------------------------------------------------------

    def read_sources(self) -> list[SourceDocument]:
        """Read all source documents for synthesis.

        Sources:
        - ``~/.thegent/memory/*.jsonl`` — agent memory logs
        - ``docs/research/CONVERSATION_DUMP_*.md`` — conversation dumps
        - ``~/.thegent/governance_events.jsonl`` — governance events (last 100 lines)
        - ``docs/reference/WORK_STREAM.md`` — current work stream state

        Returns:
            List of :class:`SourceDocument` instances.

        Raises:
            FileNotFoundError: If memory_dir does not exist.
        """
        if not self.memory_dir.exists():
            raise FileNotFoundError(f"Memory directory not found: {self.memory_dir}")

        sources: list[SourceDocument] = []

        # 1. Agent memory logs: ~/.thegent/memory/*.jsonl
        for jsonl_path in sorted(self.memory_dir.glob("*.jsonl")):
            content = jsonl_path.read_text(encoding="utf-8", errors="replace")
            sources.append(
                SourceDocument(
                    path=jsonl_path,
                    content=content,
                    last_modified=jsonl_path.stat().st_mtime,
                )
            )
        _log.info(
            "GardenerAgent.read_sources: loaded %d memory log(s)",
            sum(1 for s in sources if s.path.suffix == ".jsonl"),
        )

        # 2. Conversation dumps: docs/research/CONVERSATION_DUMP_*.md
        research_dir = self.project_root / "docs" / "research"
        if research_dir.exists():
            for dump_path in sorted(research_dir.glob("CONVERSATION_DUMP_*.md")):
                content = dump_path.read_text(encoding="utf-8", errors="replace")
                sources.append(
                    SourceDocument(
                        path=dump_path,
                        content=content,
                        last_modified=dump_path.stat().st_mtime,
                    )
                )
            _log.info(
                "GardenerAgent.read_sources: loaded %d conversation dump(s)",
                len([s for s in sources if s.path.name.startswith("CONVERSATION_DUMP_")]),
            )
        else:
            _log.info(
                "GardenerAgent.read_sources: research dir not found, skipping dumps: %s",
                research_dir,
            )

        # 3. Governance events: ~/.thegent/governance_events.jsonl (optional)
        gov_path = Path.home() / ".thegent" / "governance_events.jsonl"
        if gov_path.exists():
            lines = gov_path.read_text(encoding="utf-8", errors="replace").splitlines()
            tail = "\n".join(lines[-_MAX_GOVERNANCE_EVENTS:])
            sources.append(
                SourceDocument(
                    path=gov_path,
                    content=tail,
                    last_modified=gov_path.stat().st_mtime,
                )
            )
            _log.info(
                "GardenerAgent.read_sources: loaded governance events (%d lines)",
                min(len(lines), _MAX_GOVERNANCE_EVENTS),
            )
        else:
            _log.info(
                "GardenerAgent.read_sources: governance_events.jsonl not found, skipping: %s",
                gov_path,
            )

        # 4. WORK_STREAM.md
        work_stream_path = self.project_root / "docs" / "reference" / "WORK_STREAM.md"
        if work_stream_path.exists():
            content = work_stream_path.read_text(encoding="utf-8", errors="replace")
            sources.append(
                SourceDocument(
                    path=work_stream_path,
                    content=content,
                    last_modified=work_stream_path.stat().st_mtime,
                )
            )
            _log.info("GardenerAgent.read_sources: loaded WORK_STREAM.md")
        else:
            _log.info(
                "GardenerAgent.read_sources: WORK_STREAM.md not found: %s",
                work_stream_path,
            )

        return sources

    # ------------------------------------------------------------------
    # Stale detection
    # ------------------------------------------------------------------

    def detect_stale_docs(self, max_age_days: int = 7) -> list[StaleDoc]:
        """Detect stale documentation files.

        Checks ``docs/reference/`` and ``docs/context/`` for:
        - Files older than *max_age_days* with no recent source references.
        - ``WORK_STREAM.md`` items marked **pending** that appear as
          COMPLETED in conversation dump files.

        Args:
            max_age_days: Age threshold in days.

        Returns:
            List of :class:`StaleDoc` instances.
        """
        stale: list[StaleDoc] = []
        cutoff = time.time() - max_age_days * 86400

        # Collect all completed WL IDs from conversation dumps
        completed_ids: set[str] = set()
        research_dir = self.project_root / "docs" / "research"
        if research_dir.exists():
            for dump_path in research_dir.glob("CONVERSATION_DUMP_*.md"):
                content = dump_path.read_text(encoding="utf-8", errors="replace")
                completed_ids.update(_extract_completed_wl_ids(content))

        # Check docs/reference/ and docs/context/ for stale files
        for subdir_name in ("reference", "context"):
            subdir = self.project_root / "docs" / subdir_name
            if not subdir.exists():
                continue
            for doc_path in subdir.glob("*.md"):
                mtime = doc_path.stat().st_mtime
                if mtime < cutoff:
                    stale.append(
                        StaleDoc(
                            path=doc_path,
                            reason=f"Not modified in >{max_age_days} days",
                            suggested_action="Review and update or mark as archived",
                        )
                    )

        # Check WORK_STREAM.md for pending items that are completed in sources
        work_stream_path = self.project_root / "docs" / "reference" / "WORK_STREAM.md"
        if work_stream_path.exists() and completed_ids:
            ws_content = work_stream_path.read_text(encoding="utf-8", errors="replace")
            pending_ids = _extract_pending_wl_ids(ws_content)
            stale_pending = pending_ids & completed_ids
            if stale_pending:
                sorted_ids = sorted(stale_pending, key=int)
                stale.append(
                    StaleDoc(
                        path=work_stream_path,
                        reason=(
                            "WL items marked pending but completed in sources: "
                            + ", ".join(f"WL-{i}" for i in sorted_ids)
                        ),
                        suggested_action=("Mark items as COMPLETED: " + ", ".join(f"WL-{i}" for i in sorted_ids)),
                    )
                )

        _log.info(
            "GardenerAgent.detect_stale_docs: found %d stale doc(s)",
            len(stale),
        )
        return stale

    # ------------------------------------------------------------------
    # Synthesis
    # ------------------------------------------------------------------

    def synthesize_update(self, stale_doc: StaleDoc, sources: list[SourceDocument]) -> str:
        """Generate a rule-based status update for a stale doc.

        For ``WORK_STREAM.md``: extracts completed/pending items and
        returns patch content (lines to mark as COMPLETED).
        For other docs: returns a summary note to append.

        No LLM calls are made — this is purely rule-based pattern
        matching on the source documents.

        Args:
            stale_doc: The stale document to update.
            sources: Source documents to draw information from.

        Returns:
            Non-empty update string describing the proposed change.
        """
        is_work_stream = stale_doc.path.name == "WORK_STREAM.md"

        if is_work_stream:
            return self._synthesize_work_stream_update(stale_doc, sources)

        return self._synthesize_generic_update(stale_doc, sources)

    def _synthesize_work_stream_update(self, stale_doc: StaleDoc, sources: list[SourceDocument]) -> str:
        """Build a patch note for WORK_STREAM.md."""
        # Extract WL IDs from the reason field
        wl_ids = re.findall(r"WL-(\d+)", stale_doc.reason)

        lines = [
            f"## Gardener Update — {stale_doc.path.name}",
            "",
            "**Reason:** " + stale_doc.reason,
            "**Action:** " + stale_doc.suggested_action,
            "",
        ]

        if wl_ids:
            lines.append("### Items to mark COMPLETED")
            for wl_id in sorted(set(wl_ids), key=int):
                lines.append(f"- Change `**Status:** pending` → `**Status:** COMPLETED` for [WL-{wl_id}]")
            lines.append("")

        # Cross-reference sources that mention these IDs
        refs: list[str] = []
        for src in sources:
            for wl_id in wl_ids:
                if f"WL-{wl_id}" in src.content:
                    refs.append(str(src.path))
                    break
        if refs:
            lines.append("### Evidence sources")
            for ref in sorted(set(refs)):
                lines.append(f"- {ref}")

        return "\n".join(lines)

    def _synthesize_generic_update(self, stale_doc: StaleDoc, sources: list[SourceDocument]) -> str:
        """Build a generic append note for non-WORK_STREAM docs."""
        import datetime

        today = datetime.datetime.now(tz=datetime.UTC).date().isoformat()
        lines = [
            f"<!-- Gardener note: {today} -->",
            f"<!-- {stale_doc.reason} -->",
            f"<!-- Suggested: {stale_doc.suggested_action} -->",
        ]
        return "\n".join(lines)

    # ------------------------------------------------------------------
    # Full garden cycle
    # ------------------------------------------------------------------

    def run(self, max_age_days: int = 7) -> GardenResult:
        """Execute a full garden cycle.

        Steps:
        1. Read sources.
        2. Detect stale docs.
        3. Synthesize updates for each stale doc.
        4. Write updates (unless dry_run=True).

        Args:
            max_age_days: Age threshold forwarded to
                :meth:`detect_stale_docs`.

        Returns:
            :class:`GardenResult` with cycle statistics.
        """
        _log.info(
            "GardenerAgent.run: starting garden cycle (dry_run=%s, max_age_days=%d)",
            self.dry_run,
            max_age_days,
        )

        sources = self.read_sources()
        stale_docs = self.detect_stale_docs(max_age_days=max_age_days)

        docs_checked = len(stale_docs)
        docs_updated = 0
        items_found: list[str] = []

        for stale_doc in stale_docs:
            update_text = self.synthesize_update(stale_doc, sources)
            items_found.append(f"{stale_doc.path.name}: {stale_doc.reason}")

            if self.dry_run:
                _log.info(
                    "GardenerAgent.run [dry_run]: would update %s",
                    stale_doc.path,
                )
                continue

            self._apply_update(stale_doc, update_text)
            docs_updated += 1
            _log.info(
                "GardenerAgent.run: updated %s",
                stale_doc.path,
            )

        result = GardenResult(
            docs_checked=docs_checked,
            docs_updated=docs_updated,
            items_found=items_found,
            dry_run=self.dry_run,
        )
        _log.info(
            "GardenerAgent.run: cycle complete — checked=%d, updated=%d, dry_run=%s",
            result.docs_checked,
            result.docs_updated,
            result.dry_run,
        )
        return result

    def _apply_update(self, stale_doc: StaleDoc, update_text: str) -> None:
        """Write the synthesized update to the stale document.

        For WORK_STREAM.md: applies in-place status transitions.
        For other docs: appends the update note.

        Args:
            stale_doc: Target document.
            update_text: Text produced by :meth:`synthesize_update`.
        """
        is_work_stream = stale_doc.path.name == "WORK_STREAM.md"

        if is_work_stream:
            self._apply_work_stream_patch(stale_doc.path, stale_doc.reason)
        else:
            existing = stale_doc.path.read_text(encoding="utf-8", errors="replace")
            stale_doc.path.write_text(
                existing + "\n\n" + update_text + "\n",
                encoding="utf-8",
            )

    def _apply_work_stream_patch(self, path: Path, reason: str) -> None:
        """Mark pending WL items as COMPLETED in WORK_STREAM.md.

        Args:
            path: Path to WORK_STREAM.md.
            reason: Reason string containing the WL IDs to mark.
        """
        wl_ids = re.findall(r"WL-(\d+)", reason)
        if not wl_ids:
            return

        content = path.read_text(encoding="utf-8", errors="replace")
        updated = content

        for wl_id in wl_ids:
            # Replace **Status:** pending within the WL-NNN block
            pattern = re.compile(
                r"(\[WL-" + re.escape(wl_id) + r"\].*?\*\*Status:\*\*\s*)pending",
                re.IGNORECASE | re.DOTALL,
            )
            updated = pattern.sub(r"\g<1>COMPLETED", updated, count=1)

        if updated != content:
            path.write_text(updated, encoding="utf-8")
