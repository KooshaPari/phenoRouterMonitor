"""WP-5005: Usage spike circuit breakers."""

import orjson as json
import logging
from datetime import UTC, datetime
from pathlib import Path

_log = logging.getLogger(__name__)


class CircuitBreaker:
    """Breaks the flow when usage spikes are detected."""

    def __init__(self, session_dir: Path) -> None:
        self.session_dir = session_dir
        self.breaker_file = session_dir / "circuit_breakers.jsonl"
        self.threshold_usd_per_min = 1.0  # $1 per minute spike

    def check_spike(self, current_batch_cost: float) -> bool:
        """Check if the current cost batch causes a spike."""
        # This is a simplification; a real impl would check moving average
        if current_batch_cost > self.threshold_usd_per_min:
            self.trip("Usage spike detected", current_batch_cost)
            return True
        return False

    def trip(self, reason: str, value: float):
        """Trip the circuit breaker."""
        _log.critical("CIRCUIT BREAKER TRIPPED: %s (value: %s)", reason, value)
        event = {
            "timestamp": datetime.now(UTC).isoformat(),
            "event": "tripped",
            "reason": reason,
            "value": value,
        }
        self.session_dir.mkdir(parents=True, exist_ok=True)
        with self.breaker_file.open("a", encoding="utf-8") as f:
            f.write(json.dumps(event).decode() + "\n")

    def is_tripped(self) -> bool:
        """Return True if any active breaker is tripped."""
        # Check last event in file
        if not self.breaker_file.exists():
            return False

        lines = self.breaker_file.read_text(encoding="utf-8").splitlines()
        if not lines:
            return False

        last_event = json.loads(lines[-1])
        # Simple policy: once tripped, it stays tripped until reset
        return last_event.get("event") == "tripped"
