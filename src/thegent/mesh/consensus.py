"""Consensus and escalation protocols for the agent mesh."""

import enum
import json
import time
from collections import defaultdict
from pathlib import Path
from typing import Any


class ConsensusStatus(enum.Enum):
    PENDING = "pending"
    AGREED = "agreed"
    REJECTED = "rejected"
    ESCALATED = "escalated"


class ConsensusProtocol:
    """Crash-Preventing Weighted Byzantine Fault Tolerance (CP-WBFT) (ADR-013, SCLI-P3.1)."""

    IMPLEMENTATION_THRESHOLD = 0.5
    ARCHITECTURE_THRESHOLD = 2 / 3
    DEFAULT_MAX_DEBATE_ROUNDS = 3

    def __init__(self, mesh_root: Path) -> None:
        self.mesh_root = mesh_root
        self.proposals_dir = mesh_root / "proposals"
        self.votes_dir = mesh_root / "votes"
        self.decisions_dir = mesh_root / "decisions"
        for d in [self.proposals_dir, self.votes_dir, self.decisions_dir]:
            d.mkdir(parents=True, exist_ok=True, mode=0o1777)

    def propose(
        self,
        proposal_id: str,
        agent_id: str,
        topic: str,
        content: dict,
        decision_type: str = "implementation",
        max_debate_rounds: int = DEFAULT_MAX_DEBATE_ROUNDS,
    ) -> None:
        """Phase 1: PROPOSE (ADR-013). Initial proposal by a leader."""
        proposal_data = {
            "proposal_id": proposal_id,
            "leader_id": agent_id,
            "topic": topic,
            "content": content,
            "decision_type": decision_type,
            "phase": "propose",
            "round": 1,
            "max_debate_rounds": max_debate_rounds,
            "timestamp": time.time(),
        }
        with open(self.proposals_dir / f"{proposal_id}.json", "w") as f:
            json.dump(proposal_data, f)

    def draft(self, proposal_id: str, agent_id: str, refinement: dict) -> None:
        """Phase 2: DRAFT (ADR-013). Agents can provide refinements or counter-proposals."""
        draft_dir = self.proposals_dir / f"{proposal_id}.drafts"
        draft_dir.mkdir(parents=True, exist_ok=True, mode=0o1777)
        with open(draft_dir / f"agent-{agent_id}.json", "w") as f:
            json.dump({"agent_id": agent_id, "refinement": refinement, "ts": time.time()}, f)

    def share(self, proposal_id: str) -> None:
        """Phase 3: SHARE (ADR-013). Finalize the proposal after drafting period."""
        proposal_file = self.proposals_dir / f"{proposal_id}.json"
        if not proposal_file.exists():
            return

        with open(proposal_file) as f:
            data = json.load(f)

        data["phase"] = "share"
        data["finalized_at"] = time.time()

        with open(proposal_file, "w") as f:
            json.dump(data, f)

    def _vote_round_dir(self, proposal_id: str, vote_round: int) -> Path:
        return self.votes_dir / proposal_id / f"round-{vote_round}"

    def _load_json(self, path: Path) -> dict[str, Any] | None:
        if not path.exists():
            return None
        try:
            with open(path) as f:
                return json.load(f)
        except (OSError, json.JSONDecodeError):
            return None

    def _load_proposal(self, proposal_id: str) -> dict[str, Any] | None:
        return self._load_json(self.proposals_dir / f"{proposal_id}.json")

    def _required_majority(self, decision_type: str) -> float:
        if decision_type == "architecture":
            return self.ARCHITECTURE_THRESHOLD
        return self.IMPLEMENTATION_THRESHOLD

    def cast_vote(
        self,
        proposal_id: str,
        agent_id: str,
        vote: bool,
        confidence: float = 0.8,
        vote_round: int = 1,
    ) -> None:
        """Phase 4: VOTE (ADR-013). Cast a weighted vote for the finalized proposal."""
        proposal = self._load_proposal(proposal_id) or {}
        max_rounds = int(proposal.get("max_debate_rounds", self.DEFAULT_MAX_DEBATE_ROUNDS))
        if vote_round < 1 or vote_round > max_rounds:
            return

        vote_data = {
            "agent_id": agent_id,
            "vote": bool(vote),
            "confidence": confidence,
            "vote_round": vote_round,
            "timestamp": time.time(),
        }
        proposal_dir = self._vote_round_dir(proposal_id, vote_round)
        proposal_dir.mkdir(parents=True, exist_ok=True, mode=0o1777)

        with open(proposal_dir / f"agent-{agent_id}.json", "w") as f:
            json.dump(vote_data, f)

        if proposal:
            proposal["round"] = vote_round
            proposal["phase"] = "vote"
            proposal["last_vote_at"] = time.time()
            with open(self.proposals_dir / f"{proposal_id}.json", "w") as f:
                json.dump(proposal, f)

    def _load_round_votes(self, proposal_id: str, vote_round: int) -> list[dict[str, Any]]:
        votes = []
        legacy_dir = self.votes_dir / proposal_id
        if legacy_dir.exists() and not (legacy_dir / "round-1").exists():
            for vote_file in legacy_dir.glob("*.json"):
                vote = self._load_json(vote_file)
                if vote is not None:
                    vote_round_value = int(vote.get("vote_round", 1))
                    if vote_round_value == vote_round:
                        vote["vote_round"] = vote_round_value
                        votes.append(vote)

        round_dir = self._vote_round_dir(proposal_id, vote_round)
        if round_dir.exists():
            for vote_file in round_dir.glob("*.json"):
                vote = self._load_json(vote_file)
                if vote is not None:
                    vote["vote_round"] = vote_round
                    votes.append(vote)
        return votes

    def advance_debate_round(self, proposal_id: str) -> int:
        """Move a proposal to the next debate round, capped by configured max rounds."""
        proposal_file = self.proposals_dir / f"{proposal_id}.json"
        proposal = self._load_json(proposal_file)
        if proposal is None:
            return 1

        current_round = int(proposal.get("round", 1))
        max_rounds = int(proposal.get("max_debate_rounds", self.DEFAULT_MAX_DEBATE_ROUNDS))
        next_round = min(current_round + 1, max_rounds)
        if next_round != current_round:
            proposal["round"] = next_round
            proposal["phase"] = "debate"
            proposal["round_started_at"] = time.time()
            with open(proposal_file, "w") as f:
                json.dump(proposal, f)
        return next_round

    def get_consensus(
        self, proposal_id: str, required_majority: float | None = None, vote_round: int | None = None
    ) -> tuple[ConsensusStatus, float]:
        """Phase 5 & 6: TALLY & DECIDE (ADR-013). Check if consensus is reached."""
        proposal = self._load_proposal(proposal_id)
        if proposal is None:
            return ConsensusStatus.PENDING, 0.0

        decision_type = str(proposal.get("decision_type", "implementation"))
        max_rounds = int(proposal.get("max_debate_rounds", self.DEFAULT_MAX_DEBATE_ROUNDS))
        current_round = int(vote_round or proposal.get("round", 1))
        current_round = min(max(1, current_round), max_rounds)
        if required_majority is None:
            required_majority = self._required_majority(decision_type)

        votes = self._load_round_votes(proposal_id, current_round)
        if not votes:
            return ConsensusStatus.PENDING, 0.0

        total_weight = sum(float(v.get("confidence", 0.0)) for v in votes)
        if total_weight <= 0:
            return ConsensusStatus.PENDING, 0.0

        weighted_votes = sum(float(v.get("confidence", 0.0)) if v.get("vote") else 0.0 for v in votes)
        ratio = weighted_votes / total_weight

        status = ConsensusStatus.PENDING
        action = "debate"
        if ratio >= required_majority:
            status = ConsensusStatus.AGREED
            action = "finalize"
        elif ratio <= (1.0 - required_majority):
            status = ConsensusStatus.REJECTED
            action = "finalize"
        elif current_round >= max_rounds:
            status = ConsensusStatus.ESCALATED
            action = "human_review"

        decision_file = self.decisions_dir / f"{proposal_id}.json"
        with open(decision_file, "w") as f:
            json.dump(
                {
                    "proposal_id": proposal_id,
                    "status": status.value,
                    "ratio": ratio,
                    "total_weight": total_weight,
                    "round": current_round,
                    "max_rounds": max_rounds,
                    "required_majority": required_majority,
                    "decision_type": decision_type,
                    "action": action,
                    "ts": time.time(),
                },
                f,
            )

        return status, ratio


class CausalInfluenceTracker:
    """Shapley-value causal influence tracking (SCLI-P3.2)."""

    def __init__(self, mesh_root: Path) -> None:
        self.influence_log = mesh_root / "influence.jsonl"

    def record_influence(self, agent_id: str, action_id: str, contribution: float) -> None:
        """Log contribution for later analysis."""
        entry = {
            "agent_id": agent_id,
            "action_id": action_id,
            "contribution": contribution,
            "timestamp": time.time(),
        }
        with open(self.influence_log, "a") as f:
            f.write(json.dumps(entry) + "\n")

    def compute_shapley_values(self, action_id: str) -> dict[str, float]:
        """Compute per-agent causal influence using Shapley-style normalized attribution."""
        if not self.influence_log.exists():
            return {}

        totals: dict[str, float] = defaultdict(float)
        with open(self.influence_log) as f:
            for line in f:
                try:
                    entry = json.loads(line)
                except json.JSONDecodeError:
                    continue
                if entry.get("action_id") != action_id:
                    continue
                totals[str(entry.get("agent_id", "unknown"))] += float(entry.get("contribution", 0.0))

        if not totals:
            return {}

        total_abs = sum(abs(value) for value in totals.values())
        if total_abs == 0:
            return dict.fromkeys(totals, 0.0)
        return {agent: value / total_abs for agent, value in totals.items()}


class EscalationWorkflow:
    """5-tier escalation workflow (SCLI-P3.3)."""

    def __init__(self, mesh_root: Path) -> None:
        self.mesh_root = mesh_root
        self.escalation_queue = mesh_root / "escalation-queue"
        self.human_escalation = mesh_root / "human-escalation"
        self.tiers = {
            1: "self",
            2: "peer",
            3: "lead",
            4: "committee",
            5: "human",
        }
        self.escalation_queue.mkdir(parents=True, exist_ok=True, mode=0o1777)
        self.human_escalation.mkdir(parents=True, exist_ok=True, mode=0o1777)

    def _load_json(self, path: Path) -> dict[str, Any] | None:
        if not path.exists():
            return None
        try:
            with open(path) as f:
                return json.load(f)
        except (OSError, json.JSONDecodeError):
            return None

    def _next_tier(self, current_tier: int) -> int:
        return 5 if current_tier >= 5 else current_tier + 1

    def escalate(
        self,
        proposal_id: str,
        current_tier: int = 1,
        reason: str = "",
        metadata: dict[str, Any] | None = None,
    ) -> bool:
        """Escalate to next tier (SCLI-P3.3)."""
        next_tier = self._next_tier(current_tier)
        if current_tier >= 5:
            # Enqueue for human intervention (SCLI-P3.4)
            self._enqueue_human_escalation(proposal_id, reason=reason, metadata=metadata)
            return True

        escalation_data = {
            "proposal_id": proposal_id,
            "from_tier": current_tier,
            "to_tier": next_tier,
            "from_label": self.tiers.get(current_tier, "unknown"),
            "to_label": self.tiers.get(next_tier, "unknown"),
            "reason": reason,
            "metadata": metadata or {},
            "status": "pending",
            "timestamp": time.time(),
        }
        with open(self.escalation_queue / f"escalation-{proposal_id}.json", "w") as f:
            json.dump(escalation_data, f)
        return True

    def _enqueue_human_escalation(
        self, proposal_id: str, reason: str = "", metadata: dict[str, Any] | None = None
    ) -> None:
        """SCLI-P3.4 Async human escalation queue."""
        with open(self.human_escalation / f"human-{proposal_id}.json", "w") as f:
            json.dump(
                {
                    "proposal_id": proposal_id,
                    "reason": reason,
                    "status": "pending",
                    "metadata": metadata or {},
                    "timestamp": time.time(),
                },
                f,
            )

    def list_pending_human_escalations(self) -> list[dict[str, Any]]:
        """List pending asynchronous human escalations (SCLI-P3.4)."""
        pending = []
        for item_path in self.human_escalation.glob("*.json"):
            item = self._load_json(item_path)
            if item is not None and item.get("status") == "pending":
                pending.append(item)
        pending.sort(key=lambda item: float(item.get("timestamp", 0.0)))
        return pending

    def resolve_human_escalation(self, proposal_id: str, status: str = "resolved") -> bool:
        """Resolve a queued human escalation item."""
        item_path = self.human_escalation / f"human-{proposal_id}.json"
        item = self._load_json(item_path)
        if item is None:
            return False

        item["status"] = status
        item["resolved_at"] = time.time()
        with open(item_path, "w") as f:
            json.dump(item, f)
        return True
