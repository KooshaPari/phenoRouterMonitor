from __future__ import annotations

"""GW-71: DLP guardrail — GDPR, HIPAA, PCI DSS pre-built compliance profiles.

Scans text for sensitive data matching compliance-specific regex patterns.
Returns matches with category labels and a compliance violation verdict.

# @trace FR-GUARD-071
"""

import logging
import re
from dataclasses import dataclass
from enum import Enum

_log = logging.getLogger(__name__)


# ---------------------------------------------------------------------------
# Enums and data structures
# ---------------------------------------------------------------------------


class DlpProfile(str, Enum):
    GDPR = "gdpr"
    HIPAA = "hipaa"
    PCI_DSS = "pci_dss"
    CUSTOM = "custom"


@dataclass
class DlpPattern:
    category: str  # e.g. "eu_national_id", "credit_card"
    profile: DlpProfile  # which profile this belongs to
    pattern: str  # regex
    severity: str  # "low", "medium", "high"


@dataclass
class DlpMatch:
    category: str
    severity: str
    snippet: str  # first 50 chars of matched text (for audit logs)
    offset: int  # char offset in original text


@dataclass
class DlpResult:
    profile: DlpProfile
    matches: list[DlpMatch]
    violation: bool  # True if any high-severity match
    categories_found: list[str]  # unique category names matched


@dataclass
class DlpConfig:
    enabled: bool = True
    profile: DlpProfile = DlpProfile.GDPR
    block_on_violation: bool = True
    custom_patterns: list[DlpPattern] | None = None  # used when profile=CUSTOM


# ---------------------------------------------------------------------------
# Pre-built compliance pattern sets
# ---------------------------------------------------------------------------

GDPR_PATTERNS: list[DlpPattern] = [
    DlpPattern(
        category="eu_email",
        profile=DlpProfile.GDPR,
        pattern=r"\b[a-zA-Z0-9._%+\-]+@[a-zA-Z0-9.\-]+\.[a-zA-Z]{2,}\b",
        severity="medium",
    ),
    DlpPattern(
        category="eu_phone",
        profile=DlpProfile.GDPR,
        pattern=r"\+?(?:3[01]|32|33|34|39|40|41|43|44|45|46|47|48|49)\s?\d[\d\s\-]{6,12}\d",
        severity="medium",
    ),
    DlpPattern(
        category="eu_national_id",  # generic: 8-12 alphanumeric
        profile=DlpProfile.GDPR,
        pattern=r"\b[A-Z]{1,3}[\s\-]?\d{6,9}\b",
        severity="high",
    ),
    DlpPattern(
        category="personal_name_context",  # "Name: FirstName LastName" pattern
        profile=DlpProfile.GDPR,
        pattern=r"(?i)\bname\s*:\s*[A-Z][a-z]+(?:\s[A-Z][a-z]+)+",
        severity="low",
    ),
]

HIPAA_PATTERNS: list[DlpPattern] = [
    DlpPattern(
        category="ssn",
        profile=DlpProfile.HIPAA,
        pattern=r"\b(?!000|666|9\d{2})\d{3}[- ]?(?!00)\d{2}[- ]?(?!0000)\d{4}\b",
        severity="high",
    ),
    DlpPattern(
        category="medical_record_number",
        profile=DlpProfile.HIPAA,
        pattern=r"(?i)(?:MRN|medical\s+record)\s*[:#]?\s*\d{6,12}",
        severity="high",
    ),
    DlpPattern(
        category="diagnosis_code",  # ICD-10 codes
        profile=DlpProfile.HIPAA,
        pattern=r"\b[A-Z]\d{2}(?:\.\d{1,4})?\b",
        severity="medium",
    ),
    DlpPattern(
        category="dob",
        profile=DlpProfile.HIPAA,
        pattern=r"(?i)(?:dob|date\s+of\s+birth)\s*[:#]?\s*\d{1,2}[/\-]\d{1,2}[/\-]\d{2,4}",
        severity="high",
    ),
]

PCI_DSS_PATTERNS: list[DlpPattern] = [
    DlpPattern(
        category="credit_card_visa",
        profile=DlpProfile.PCI_DSS,
        pattern=r"\b4\d{3}[\s\-]?\d{4}[\s\-]?\d{4}[\s\-]?\d{4}\b",
        severity="high",
    ),
    DlpPattern(
        category="credit_card_mastercard",
        profile=DlpProfile.PCI_DSS,
        pattern=r"\b5[1-5]\d{2}[\s\-]?\d{4}[\s\-]?\d{4}[\s\-]?\d{4}\b",
        severity="high",
    ),
    DlpPattern(
        category="credit_card_amex",
        profile=DlpProfile.PCI_DSS,
        pattern=r"\b3[47]\d{2}[\s\-]?\d{6}[\s\-]?\d{5}\b",
        severity="high",
    ),
    DlpPattern(
        category="cvv",
        profile=DlpProfile.PCI_DSS,
        pattern=r"(?i)(?:cvv|cvc|csc)\s*[:#]?\s*\d{3,4}",
        severity="high",
    ),
    DlpPattern(
        category="track_data",  # Mag stripe track data
        profile=DlpProfile.PCI_DSS,
        pattern=r"%B\d{13,19}\^[A-Z /]{2,26}\^\d{4}",
        severity="high",
    ),
]


# ---------------------------------------------------------------------------
# Compiled pattern cache
# ---------------------------------------------------------------------------

_compiled_cache: dict[DlpProfile, list[tuple[DlpPattern, re.Pattern[str]]]] = {}


def _get_compiled(profile: DlpProfile) -> list[tuple[DlpPattern, re.Pattern[str]]]:
    """Return compiled (DlpPattern, regex) pairs for the given profile (cached)."""
    global _compiled_cache  # noqa: PLW0603
    if profile not in _compiled_cache:
        patterns_map: dict[DlpProfile, list[DlpPattern]] = {
            DlpProfile.GDPR: GDPR_PATTERNS,
            DlpProfile.HIPAA: HIPAA_PATTERNS,
            DlpProfile.PCI_DSS: PCI_DSS_PATTERNS,
        }
        raw = patterns_map.get(profile, [])
        _compiled_cache[profile] = [(p, re.compile(p.pattern, re.IGNORECASE)) for p in raw]
    return _compiled_cache[profile]


# ---------------------------------------------------------------------------
# Public API
# ---------------------------------------------------------------------------


def scan_dlp(text: str, config: DlpConfig | None = None) -> DlpResult:
    """Scan text against the configured DLP profile. Returns DlpResult."""
    cfg = config or DlpConfig()

    if not cfg.enabled:
        return DlpResult(profile=cfg.profile, matches=[], violation=False, categories_found=[])

    if cfg.profile == DlpProfile.CUSTOM:
        patterns = cfg.custom_patterns or []
        compiled: list[tuple[DlpPattern, re.Pattern[str]]] = [
            (p, re.compile(p.pattern, re.IGNORECASE)) for p in patterns
        ]
    else:
        compiled = _get_compiled(cfg.profile)

    matches: list[DlpMatch] = []
    seen_categories: set[str] = set()

    for pat, regex in compiled:
        for m in regex.finditer(text):
            snippet = text[m.start() : min(m.start() + 50, m.end())]
            matches.append(
                DlpMatch(
                    category=pat.category,
                    severity=pat.severity,
                    snippet=snippet,
                    offset=m.start(),
                )
            )
            seen_categories.add(pat.category)

    violation = any(m.severity == "high" for m in matches)

    _log.debug(
        "scan_dlp: profile=%s matches=%d violation=%s",
        cfg.profile,
        len(matches),
        violation,
    )

    return DlpResult(
        profile=cfg.profile,
        matches=matches,
        violation=violation,
        categories_found=sorted(seen_categories),
    )


def should_block_dlp(result: DlpResult, config: DlpConfig | None = None) -> bool:
    """Return True if the DLP result warrants blocking."""
    cfg = config or DlpConfig()
    return cfg.block_on_violation and result.violation
