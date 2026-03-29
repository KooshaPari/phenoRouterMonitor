"""Team coordination and cross-team collaboration."""

import logging
from typing import Any

from .agent_hierarchy import (
    AgentHierarchyManager,
    AgentNode,
    AgentRelationship,
    CoordinationMode,
    RelationshipType,
)

_log = logging.getLogger(__name__)


class TeamCoordinator:
    """Coordinates team activities and cross-team collaboration."""

    def __init__(self, hierarchy_manager: AgentHierarchyManager) -> None:
        """
        Initialize team coordinator.

        Args:
            hierarchy_manager: AgentHierarchyManager instance
        """
        self.hierarchy = hierarchy_manager

    def delegate_within_team(
        self,
        from_agent_id: str,
        to_agent_id: str,
        task: str,
        context: dict[str, Any] | None = None,
    ) -> AgentRelationship:
        """
        Delegate task within same team.

        Args:
            from_agent_id: Source agent run_id
            to_agent_id: Target agent run_id
            task: Task description
            context: Optional context

        Returns:
            Created AgentRelationship

        Raises:
            ValueError: If agents not in same team
        """
        from_agent = self.hierarchy.get_agent(from_agent_id)
        to_agent = self.hierarchy.get_agent(to_agent_id)

        if not from_agent or not to_agent:
            raise ValueError("Agent not found")

        if from_agent.team_id != to_agent.team_id:
            raise ValueError("Agents not in same team")

        # Create delegation relationship
        relationship = self.hierarchy.create_relationship(
            parent_id=from_agent_id,
            child_id=to_agent_id,
            relationship_type=RelationshipType.TEAM_MEMBERSHIP,
            delegation_prompt=task,
            handoff_context=context,
        )

        return relationship

    def delegate_cross_team(
        self,
        from_agent_id: str,
        to_agent_id: str,
        task: str,
        context: dict[str, Any] | None = None,
        mediator_id: str | None = None,
    ) -> AgentRelationship:
        """
        Delegate task across teams (requires coordination).

        Args:
            from_agent_id: Source agent run_id
            to_agent_id: Target agent run_id
            task: Task description
            context: Optional context
            mediator_id: Optional mediator agent run_id (defaults to orchestrator)

        Returns:
            Created AgentRelationship

        Raises:
            ValueError: If agents in same team or not found
        """
        from_agent = self.hierarchy.get_agent(from_agent_id)
        to_agent = self.hierarchy.get_agent(to_agent_id)

        if not from_agent or not to_agent:
            raise ValueError("Agent not found")

        if from_agent.team_id == to_agent.team_id:
            raise ValueError("Agents in same team, use delegate_within_team")

        # Find orchestrator if mediator not specified
        if not mediator_id:
            mediator_id = self._find_orchestrator()

        # Create cross-team relationship
        handoff_context = {
            "mediator_id": mediator_id,
            "cross_team": True,
            **(context or {}),
        }

        relationship = self.hierarchy.create_relationship(
            parent_id=from_agent_id,
            child_id=to_agent_id,
            relationship_type=RelationshipType.CROSS_TEAM_COLLABORATION,
            delegation_prompt=task,
            handoff_context=handoff_context,
        )

        return relationship

    def _find_orchestrator(self) -> str | None:
        """Find orchestrator agent (Executive role with no parent)."""
        for agent in self.hierarchy.list_all_agents():
            if agent.role.value == "executive" and agent.parent_id is None:
                return agent.run_id
        return None

    def get_team_coordination_status(self, team_id: str) -> dict[str, Any]:
        """
        Get coordination status for a team.

        Args:
            team_id: Team identifier

        Returns:
            Coordination status dictionary
        """
        team = self.hierarchy.get_team(team_id)
        if not team:
            return {}

        members = self.hierarchy.get_team_members(team_id)
        active_members = [m for m in members if m.status == "active"]

        # Get active relationships within team
        team_relationships = [
            rel
            for rel in self.hierarchy.list_all_relationships()
            if rel.status == "active"
            and rel.relationship_type == RelationshipType.TEAM_MEMBERSHIP
            and self._is_team_relationship(rel, team_id)
        ]

        return {
            "team_id": team_id,
            "team_name": team.name,
            "team_type": team.team_type.value,
            "coordination_mode": team.coordination_mode.value,
            "lead_id": team.lead_id,
            "total_members": len(members),
            "active_members": len(active_members),
            "active_relationships": len(team_relationships),
            "status": team.status,
        }

    def _is_team_relationship(self, relationship: AgentRelationship, team_id: str) -> bool:
        """Check if relationship involves team members."""
        parent = self.hierarchy.get_agent(relationship.parent_id)
        child = self.hierarchy.get_agent(relationship.child_id)

        if not parent or not child:
            return False

        return parent.team_id == team_id and child.team_id == team_id

    def coordinate_team_task(
        self,
        team_id: str,
        task: str,
        context: dict[str, Any] | None = None,
    ) -> dict[str, Any]:
        """
        Coordinate a task within a team based on coordination mode.

        Args:
            team_id: Team identifier
            task: Task description
            context: Optional context

        Returns:
            Coordination result
        """
        team = self.hierarchy.get_team(team_id)
        if not team:
            raise ValueError(f"Team {team_id} not found")

        members = self.hierarchy.get_team_members(team_id)
        active_members = [m for m in members if m.status == "active"]

        if not active_members:
            return {"status": "error", "message": "No active members in team"}

        coordination_mode = team.coordination_mode

        if coordination_mode == CoordinationMode.HIERARCHICAL:
            return self.coordinate_team_task_hierarchical(team_id, task, context, active_members)

        if coordination_mode == CoordinationMode.COLLABORATIVE:
            return self.coordinate_team_task_collaborative(team_id, task, context, active_members)

        if coordination_mode == CoordinationMode.SWARM:
            # All members work independently
            assignments = []
            for member in active_members:
                # Each member gets the task independently
                # In swarm mode, we don't create relationships, just track assignments
                assignments.append(member.run_id)

            return {
                "status": "success",
                "coordination_mode": "swarm",
                "assignments": assignments,
                "participants": [m.run_id for m in active_members],
            }

        if coordination_mode == CoordinationMode.ADAPTIVE:
            # Choose mode based on task complexity
            complexity = self._evaluate_task_complexity(task, context)
            target_mode = CoordinationMode.HIERARCHICAL if complexity >= 0.5 else CoordinationMode.COLLABORATIVE

            _log.info(
                "Adaptive coordination: task complexity %.2f -> %s",
                complexity,
                target_mode.value,
            )

            # Delegate to appropriate mode logic
            # For simplicity, we just call the same logic as above but with the target_mode
            if target_mode == CoordinationMode.HIERARCHICAL:
                return self.coordinate_team_task_hierarchical(team_id, task, context, active_members)
            return self.coordinate_team_task_collaborative(team_id, task, context, active_members)

        return {"status": "error", "message": f"Unknown coordination mode: {coordination_mode}"}

    def _evaluate_task_complexity(self, task: str, context: dict[str, Any] | None = None) -> float:
        """
        Evaluate task complexity (0.0 to 1.0).

        Args:
            task: Task description
            context: Optional context

        Returns:
            Complexity score
        """
        # Simple heuristics for now
        # 1. Length of task description
        desc_score = min(len(task) / 500, 1.0)

        # 2. Number of requested artifacts (if in context)
        artifact_count = len(context.get("required_artifacts", [])) if context else 0
        artifact_score = min(artifact_count / 5, 1.0)

        # 3. Explicit complexity in context
        manual_score = context.get("complexity", 0.0) if context else 0.0

        return max(desc_score * 0.2 + artifact_score * 0.3 + manual_score * 0.5, 0.0)

    def coordinate_team_task_hierarchical(
        self,
        team_id: str,
        task: str,
        context: dict[str, Any] | None,
        active_members: list[AgentNode],
    ) -> dict[str, Any]:
        """Hierarchical coordination implementation."""
        team = self.hierarchy.get_team(team_id)
        if not team:
            return {"status": "error", "message": f"Team {team_id} not found"}

        lead = self.hierarchy.get_agent(team.lead_id)
        if not lead or lead.status != "active":
            return {"status": "error", "message": "Team lead not active"}

        assignments = []
        for member in active_members:
            if member.run_id != team.lead_id:
                rel = self.delegate_within_team(
                    from_agent_id=team.lead_id,
                    to_agent_id=member.run_id,
                    task=task,
                    context=context,
                )
                assignments.append(rel.relationship_id)

        return {
            "status": "success",
            "coordination_mode": "hierarchical",
            "assignments": assignments,
            "assigned_by": team.lead_id,
        }

    def coordinate_team_task_collaborative(
        self,
        team_id: str,
        task: str,
        context: dict[str, Any] | None,
        active_members: list[AgentNode],
    ) -> dict[str, Any]:
        """Collaborative (P2P) coordination implementation."""
        assignments = []
        for i, member1 in enumerate(active_members):
            for member2 in active_members[i + 1 :]:
                rel = self.delegate_within_team(
                    from_agent_id=member1.run_id,
                    to_agent_id=member2.run_id,
                    task=task,
                    context=context,
                )
                assignments.append(rel.relationship_id)

        return {
            "status": "success",
            "coordination_mode": "collaborative",
            "assignments": assignments,
            "participants": [m.run_id for m in active_members],
        }
