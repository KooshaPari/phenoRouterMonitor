"""Board-ID-first reconciliation policy for deterministic conflict resolution.

Implements reconciliation that always uses board-id as the primary key,
never title-based matching, with configurable fallback behavior.

FR traceability: WL-161 (Board-ID-First Reconciliation Policy)
"""

from __future__ import annotations

import logging
from dataclasses import dataclass
from enum import Enum

logger = logging.getLogger(__name__)


class ReconciliationMode(str, Enum):
    """Reconciliation modes for conflict resolution."""

    BOARD_ID_FIRST = "board_id_first"
    STRICT = "strict"


@dataclass
class ReconciliationPolicy:
    """Policy defining reconciliation behavior.

    Attributes:
        mode: The reconciliation strategy (default: BOARD_ID_FIRST).
        title_fallback_enabled: If True, allows title-based matching as fallback
            (default: False). When False, title-only records are rejected.
    """

    mode: ReconciliationMode = ReconciliationMode.BOARD_ID_FIRST
    title_fallback_enabled: bool = False

    def validate_record(self, record: dict) -> bool:
        """Validate a record against this policy.

        Args:
            record: A record dict to validate.

        Returns:
            True if the record is valid, False otherwise.

        Raises:
            ValueError: If title_fallback_enabled is False and record has no board_id.
        """
        if not isinstance(record, dict):
            raise ValueError("Record must be a dictionary")

        has_board_id = "board_id" in record and record["board_id"]
        has_title = "title" in record and record["title"]

        # If board_id is present, always valid
        if has_board_id:
            return True

        # If no board_id, check fallback policy
        if not self.title_fallback_enabled:
            if not has_board_id:
                raise ValueError(
                    "Record has no board_id and title_fallback_enabled=False; title-only records are rejected"
                )

        # If fallback enabled, title-only records are acceptable
        if has_title:
            return True

        raise ValueError("Record has neither board_id nor title")

    def requires_board_id(self, record: dict) -> bool:
        """Check if a record requires a board_id for reconciliation.

        Args:
            record: A record dict.

        Returns:
            True if the record must have a board_id.
        """
        if self.mode == ReconciliationMode.STRICT:
            return True
        # BOARD_ID_FIRST mode: board_id is required
        return True


def create_default_policy() -> ReconciliationPolicy:
    """Create a default reconciliation policy.

    Returns:
        A policy with board_id_first mode and no title fallback.
    """
    return ReconciliationPolicy(mode=ReconciliationMode.BOARD_ID_FIRST, title_fallback_enabled=False)
