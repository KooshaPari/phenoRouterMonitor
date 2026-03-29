"""Per-connector cost accounting for sync budgeting.

# @trace WL-297
"""

from __future__ import annotations

from dataclasses import dataclass
from datetime import UTC, datetime


@dataclass(frozen=True)
class ConnectorUsageEvent:
    """Single connector usage event."""

    connector: str
    operation: str
    requests: int
    tokens_in: int
    tokens_out: int
    usd_cost: float
    timestamp: datetime


@dataclass(frozen=True)
class ConnectorCostSummary:
    """Aggregated connector cost/usage summary."""

    connector: str
    requests: int
    tokens_in: int
    tokens_out: int
    usd_cost: float


class ConnectorCostLedger:
    """In-memory ledger of per-connector usage/cost metrics."""

    def __init__(self) -> None:
        self._events: list[ConnectorUsageEvent] = []

    def record(
        self,
        *,
        connector: str,
        operation: str,
        requests: int,
        tokens_in: int,
        tokens_out: int,
        usd_cost: float,
        timestamp: datetime | None = None,
    ) -> ConnectorUsageEvent:
        if not connector.strip():
            raise ValueError("connector must be non-empty")
        if requests < 0 or tokens_in < 0 or tokens_out < 0 or usd_cost < 0:
            raise ValueError("usage and cost values must be non-negative")

        event = ConnectorUsageEvent(
            connector=connector,
            operation=operation,
            requests=requests,
            tokens_in=tokens_in,
            tokens_out=tokens_out,
            usd_cost=usd_cost,
            timestamp=timestamp or datetime.now(UTC),
        )
        self._events.append(event)
        return event

    def summary_by_connector(self) -> list[ConnectorCostSummary]:
        bucket: dict[str, ConnectorCostSummary] = {}
        for event in self._events:
            prev = bucket.get(event.connector)
            if prev is None:
                bucket[event.connector] = ConnectorCostSummary(
                    connector=event.connector,
                    requests=event.requests,
                    tokens_in=event.tokens_in,
                    tokens_out=event.tokens_out,
                    usd_cost=event.usd_cost,
                )
                continue
            bucket[event.connector] = ConnectorCostSummary(
                connector=event.connector,
                requests=prev.requests + event.requests,
                tokens_in=prev.tokens_in + event.tokens_in,
                tokens_out=prev.tokens_out + event.tokens_out,
                usd_cost=prev.usd_cost + event.usd_cost,
            )

        return sorted(bucket.values(), key=lambda x: x.connector)

    def total_cost(self) -> float:
        return sum(event.usd_cost for event in self._events)
