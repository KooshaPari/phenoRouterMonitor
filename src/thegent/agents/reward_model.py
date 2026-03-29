"""WP-39003: Recursive Reward Modeling Optimization.

This module implements recursive reward modeling for agent optimization,
integrating with heliosShield bridge (WP-16003) for task coordination.
"""

from dataclasses import dataclass
from datetime import datetime
from typing import Any

from thegent.governance.heliosShield_bridge import heliosShieldBridge


@dataclass
class RewardSignal:
    """Represents a reward signal for model optimization."""

    agent_id: str
    task_id: str
    reward_value: float
    timestamp: datetime
    metadata: dict[str, Any]


class RecursiveRewardModel:
    """WP-39003: Recursive Reward Modeling Optimization.

    Implements recursive reward modeling that learns from agent performance
    and optimizes reward signals over time. Integrates with heliosShield bridge
    for task coordination.
    """

    def __init__(self) -> None:
        """Initialize the recursive reward model."""
        self._reward_history: list[RewardSignal] = []
        self._bridge = heliosShieldBridge()
        self._optimization_epoch = 0

    def record_reward(
        self,
        agent_id: str,
        task_id: str,
        reward_value: float,
        metadata: dict[str, Any] | None = None,
    ) -> None:
        """Record a reward signal for optimization.

        Args:
            agent_id: Identifier for the agent
            task_id: Identifier for the task
            reward_value: Reward value (higher is better)
            metadata: Optional metadata about the reward
        """
        signal = RewardSignal(
            agent_id=agent_id,
            task_id=task_id,
            reward_value=reward_value,
            timestamp=datetime.now(),
            metadata=metadata or {},
        )
        self._reward_history.append(signal)

        # Integrate with heliosShield bridge if available
        if self._bridge.is_available():
            self._bridge.broadcast_intent(
                agent_id=agent_id,
                intent_type="reward_recorded",
                target=task_id,
            )

    def optimize(self) -> dict[str, Any]:
        """Perform recursive optimization of reward model.

        Returns:
            Dictionary with optimization results and metrics
        """
        if not self._reward_history:
            return {"status": "no_data", "epoch": self._optimization_epoch}

        # Calculate average reward
        avg_reward = sum(s.reward_value for s in self._reward_history) / len(self._reward_history)

        # Find best performing agent
        agent_rewards: dict[str, list[float]] = {}
        for signal in self._reward_history:
            if signal.agent_id not in agent_rewards:
                agent_rewards[signal.agent_id] = []
            agent_rewards[signal.agent_id].append(signal.reward_value)

        best_agent = (
            max(
                agent_rewards.items(),
                key=lambda x: sum(x[1]) / len(x[1]) if x[1] else 0,
            )[0]
            if agent_rewards
            else None
        )

        self._optimization_epoch += 1

        result = {
            "status": "optimized",
            "epoch": self._optimization_epoch,
            "avg_reward": avg_reward,
            "total_signals": len(self._reward_history),
            "best_agent": best_agent,
            "agent_count": len(agent_rewards),
        }

        return result

    def get_reward_statistics(self) -> dict[str, Any]:
        """Get statistics about recorded rewards.

        Returns:
            Dictionary with reward statistics
        """
        if not self._reward_history:
            return {"total": 0, "average": 0.0, "min": 0.0, "max": 0.0}

        rewards = [s.reward_value for s in self._reward_history]
        return {
            "total": len(rewards),
            "average": sum(rewards) / len(rewards),
            "min": min(rewards),
            "max": max(rewards),
            "epoch": self._optimization_epoch,
        }
