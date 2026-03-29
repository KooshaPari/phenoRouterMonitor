"""WP-Y8-rel: Provider scoring with learning."""

import json
import logging

from thegent.config import ThegentSettings

_log = logging.getLogger(__name__)


class ProviderScorer:
    """Scores providers based on historical performance and learning."""

    def __init__(self, settings: ThegentSettings) -> None:
        self.settings = settings
        self.scores_file = settings.session_dir / "provider_scores.json"
        self.scores = self._load_scores()

    def get_score(self, provider_id: str) -> float:
        """Get the current score for a provider (0.0 to 1.0)."""
        return self.scores.get(provider_id, 0.8)  # Default 0.8

    def update_score(self, provider_id: str, latency_s: float, success: bool):
        """Update provider score based on a new result."""
        current = self.get_score(provider_id)
        learning_rate = 0.05

        # Reward success, penalize failure
        result_score = 1.0 if success else 0.0
        # Penalize high latency (simple threshold)
        if latency_s > 10.0:
            result_score *= 0.8

        new_score = current + (result_score - current) * learning_rate
        self.scores[provider_id] = new_score
        self._save_scores()

    def _load_scores(self) -> dict[str, float]:
        if not self.scores_file.exists():
            return {}
        try:
            return json.loads(self.scores_file.read_text(encoding="utf-8"))
        except Exception:
            return {}

    def _save_scores(self):
        self.settings.session_dir.mkdir(parents=True, exist_ok=True)
        self.scores_file.write_text(json.dumps(self.scores, indent=2), encoding="utf-8")
