"""WP-16001+: Agent hierarchy management and team coordination.

Manages agent hierarchies, parent-child relationships, and team structures.
"""

import json
import uuid
from dataclasses import dataclass, field
from datetime import UTC, datetime
from enum import Enum
from pathlib import Path
from typing import Any
from thegent.integrations.base import SerializableMixin


class AgentRole(Enum):
    """Agent role levels."""

    EXECUTIVE = "executive"  # Level 1
    TEAM_LEAD = "team_lead"  # Level 2
    SPECIALIST = "specialist"  # Level 3


class RelationshipType(Enum):
    """Types of agent relationships."""

    DIRECT_PARENT_CHILD = "direct_parent_child"
    TEAM_MEMBERSHIP = "team_membership"
    CROSS_TEAM_COLLABORATION = "cross_team_collaboration"


class TeamType(Enum):
    """Types of teams."""

    FUNCTIONAL = "functional"
    PROJECT = "project"
    AD_HOC = "ad_hoc"


class CoordinationMode(Enum):
    """Team coordination modes."""

    HIERARCHICAL = "hierarchical"
    COLLABORATIVE = "collaborative"
    SWARM = "swarm"
    ADAPTIVE = "adaptive"


@dataclass
class AgentNode(SerializableMixin):
    """Represents an agent in the hierarchy."""

    agent_id: str
    run_id: str
    role: AgentRole
    team_id: str | None = None

    # Relationships
    parent_id: str | None = None
    children_ids: list[str] = field(default_factory=list)
    team_member_ids: list[str] = field(default_factory=list)

    # Metadata
    created_at: datetime = field(default_factory=lambda: datetime.now(UTC))
    status: str = "active"

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> "AgentNode":
        """Create from dictionary."""
        return cls(
            agent_id=data["agent_id"],
            run_id=data["run_id"],
            role=AgentRole(data["role"]),
            team_id=data.get("team_id"),
            parent_id=data.get("parent_id"),
            children_ids=data.get("children_ids", []),
            team_member_ids=data.get("team_member_ids", []),
            created_at=datetime.fromisoformat(data["created_at"]),
            status=data.get("status", "active"),
        )


@dataclass
class AgentRelationship:
    """Parent-child relationship between agents."""

    relationship_id: str
    parent_id: str
    child_id: str
    relationship_type: RelationshipType
    created_at: datetime = field(default_factory=lambda: datetime.now(UTC))
    status: str = "active"

    # Delegation context
    task_id: str | None = None
    delegation_prompt: str | None = None
    handoff_context: dict[str, Any] | None = None

    def to_dict(self) -> dict[str, Any]:
        """Serialize to dictionary."""
        return {
            "relationship_id": self.relationship_id,
            "parent_id": self.parent_id,
            "child_id": self.child_id,
            "relationship_type": self.relationship_type.value,
            "created_at": self.created_at.isoformat(),
            "status": self.status,
            "task_id": self.task_id,
            "delegation_prompt": self.delegation_prompt,
            "handoff_context": self.handoff_context,
        }

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> "AgentRelationship":
        """Create from dictionary."""
        return cls(
            relationship_id=data["relationship_id"],
            parent_id=data["parent_id"],
            child_id=data["child_id"],
            relationship_type=RelationshipType(data["relationship_type"]),
            created_at=datetime.fromisoformat(data["created_at"]),
            status=data.get("status", "active"),
            task_id=data.get("task_id"),
            delegation_prompt=data.get("delegation_prompt"),
            handoff_context=data.get("handoff_context"),
        )


@dataclass
class AgentTeam:
    """A team of agents working together."""

    team_id: str
    name: str
    description: str

    # Team composition
    lead_id: str
    # Team configuration
    team_type: TeamType
    coordination_mode: CoordinationMode
    # Fields with defaults must come after fields without defaults
    members: list[str] = field(default_factory=list)
    boundaries: dict[str, Any] = field(default_factory=dict)
    communication_channels: list[str] = field(default_factory=list)

    # Metadata
    created_at: datetime = field(default_factory=lambda: datetime.now(UTC))
    status: str = "active"

    def to_dict(self) -> dict[str, Any]:
        """Serialize to dictionary."""
        return {
            "team_id": self.team_id,
            "name": self.name,
            "description": self.description,
            "lead_id": self.lead_id,
            "members": self.members,
            "team_type": self.team_type.value,
            "coordination_mode": self.coordination_mode.value,
            "boundaries": self.boundaries,
            "communication_channels": self.communication_channels,
            "created_at": self.created_at.isoformat(),
            "status": self.status,
        }

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> "AgentTeam":
        """Create from dictionary."""
        return cls(
            team_id=data["team_id"],
            name=data["name"],
            description=data["description"],
            lead_id=data["lead_id"],
            members=data.get("members", []),
            team_type=TeamType(data["team_type"]),
            coordination_mode=CoordinationMode(data["coordination_mode"]),
            boundaries=data.get("boundaries", {}),
            communication_channels=data.get("communication_channels", []),
            created_at=datetime.fromisoformat(data["created_at"]),
            status=data.get("status", "active"),
        )


class AgentHierarchyManager:
    """Manages agent hierarchy, relationships, and teams."""

    def __init__(self, storage_path: Path) -> None:
        """
        Initialize hierarchy manager.

        Args:
            storage_path: Base path for storage (creates hierarchy.json, teams.json, relationships.json)
        """
        self.storage_path = storage_path
        self.storage_path.mkdir(parents=True, exist_ok=True)

        self._agents: dict[str, AgentNode] = {}
        self._relationships: dict[str, AgentRelationship] = {}
        self._teams: dict[str, AgentTeam] = {}

        self._load()

    def _load(self) -> None:
        """Load hierarchy data from storage."""
        # Load agents
        agents_file = self.storage_path / "hierarchy.json"
        if agents_file.exists():
            try:
                data = json.loads(agents_file.read_text())
                self._agents = {run_id: AgentNode.from_dict(node_data) for run_id, node_data in data.items()}
            except (json.JSONDecodeError, KeyError, ValueError):
                self._agents = {}

        # Load relationships
        relationships_file = self.storage_path / "relationships.json"
        if relationships_file.exists():
            try:
                data = json.loads(relationships_file.read_text())
                self._relationships = {
                    rel_id: AgentRelationship.from_dict(rel_data) for rel_id, rel_data in data.items()
                }
            except (json.JSONDecodeError, KeyError, ValueError):
                self._relationships = {}

        # Load teams
        teams_file = self.storage_path / "teams.json"
        if teams_file.exists():
            try:
                data = json.loads(teams_file.read_text())
                self._teams = {team_id: AgentTeam.from_dict(team_data) for team_id, team_data in data.items()}
            except (json.JSONDecodeError, KeyError, ValueError):
                self._teams = {}

    def _save(self) -> None:
        """Save hierarchy data to storage."""
        # Save agents
        agents_file = self.storage_path / "hierarchy.json"
        agents_data = {run_id: node.to_dict() for run_id, node in self._agents.items()}
        agents_file.write_text(json.dumps(agents_data, indent=2))

        # Save relationships
        relationships_file = self.storage_path / "relationships.json"
        relationships_data = {rel_id: rel.to_dict() for rel_id, rel in self._relationships.items()}
        relationships_file.write_text(json.dumps(relationships_data, indent=2))

        # Save teams
        teams_file = self.storage_path / "teams.json"
        teams_data = {team_id: team.to_dict() for team_id, team in self._teams.items()}
        teams_file.write_text(json.dumps(teams_data, indent=2))

    def register_agent(
        self,
        agent_id: str,
        run_id: str,
        role: AgentRole,
        parent_id: str | None = None,
        team_id: str | None = None,
        validate: bool = True,
    ) -> AgentNode:
        """
        Register a new agent in the hierarchy.

        Args:
            agent_id: Agent identifier (e.g., "coder", "researcher")
            run_id: Unique run identifier
            role: Agent role level
            parent_id: Optional parent agent run_id
            team_id: Optional team identifier
            validate: Whether to validate before registering (default: True)

        Returns:
            Created AgentNode

        Raises:
            ValueError: If validation fails
        """
        if validate:
            is_valid, error = self.validate_before_register(agent_id, run_id, parent_id, team_id)
            if not is_valid:
                raise ValueError(f"Validation failed: {error}")

        node = AgentNode(
            agent_id=agent_id,
            run_id=run_id,
            role=role,
            parent_id=parent_id,
            team_id=team_id,
        )
        self._agents[run_id] = node

        # Update parent's children list
        if parent_id:
            parent = self._agents.get(parent_id)
            if parent:
                if run_id not in parent.children_ids:
                    parent.children_ids.append(run_id)

        # Update team membership
        if team_id:
            team = self._teams.get(team_id)
            if team:
                if role == AgentRole.TEAM_LEAD:
                    team.lead_id = run_id
                elif run_id not in team.members:
                    team.members.append(run_id)

        self._save()
        return node

    def create_relationship(
        self,
        parent_id: str,
        child_id: str,
        relationship_type: RelationshipType,
        task_id: str | None = None,
        delegation_prompt: str | None = None,
        handoff_context: dict[str, Any] | None = None,
    ) -> AgentRelationship:
        """
        Create a relationship between agents.

        Args:
            parent_id: Parent agent run_id
            child_id: Child agent run_id
            relationship_type: Type of relationship
            task_id: Optional task identifier
            delegation_prompt: Optional delegation prompt
            handoff_context: Optional handoff context

        Returns:
            Created AgentRelationship
        """
        rel_id = f"REL-{uuid.uuid4().hex[:8]}"
        relationship = AgentRelationship(
            relationship_id=rel_id,
            parent_id=parent_id,
            child_id=child_id,
            relationship_type=relationship_type,
            task_id=task_id,
            delegation_prompt=delegation_prompt,
            handoff_context=handoff_context,
        )
        self._relationships[rel_id] = relationship
        self._save()
        return relationship

    def create_team(
        self,
        team_id: str,
        name: str,
        description: str,
        team_type: TeamType,
        coordination_mode: CoordinationMode,
        lead_id: str,
        boundaries: dict[str, Any] | None = None,
        communication_channels: list[str] | None = None,
    ) -> AgentTeam:
        """
        Create a new team.

        Args:
            team_id: Unique team identifier
            name: Team name
            description: Team description
            team_type: Type of team
            coordination_mode: Coordination mode
            lead_id: Team lead agent run_id
            boundaries: Optional team boundaries
            communication_channels: Optional communication channels

        Returns:
            Created AgentTeam
        """
        team = AgentTeam(
            team_id=team_id,
            name=name,
            description=description,
            team_type=team_type,
            coordination_mode=coordination_mode,
            lead_id=lead_id,
            boundaries=boundaries or {},
            communication_channels=communication_channels or [],
        )
        self._teams[team_id] = team
        self._save()
        return team

    def get_agent(self, run_id: str) -> AgentNode | None:
        """Get agent by run_id."""
        return self._agents.get(run_id)

    def get_team(self, team_id: str) -> AgentTeam | None:
        """Get team by team_id."""
        return self._teams.get(team_id)

    def get_team_members(self, team_id: str) -> list[AgentNode]:
        """Get all members of a team."""
        team = self._teams.get(team_id)
        if not team:
            return []

        members = []
        if team.lead_id:
            lead = self._agents.get(team.lead_id)
            if lead:
                members.append(lead)

        for member_id in team.members:
            member = self._agents.get(member_id)
            if member:
                members.append(member)

        return members

    def get_children(self, parent_id: str) -> list[AgentNode]:
        """Get all direct children of an agent."""
        parent = self._agents.get(parent_id)
        if not parent:
            return []

        children = []
        for child_id in parent.children_ids:
            child = self._agents.get(child_id)
            if child:
                children.append(child)

        return children

    def get_ancestors(self, agent_id: str) -> list[AgentNode]:
        """Get all ancestors of an agent (parent chain)."""
        ancestors = []
        current = self._agents.get(agent_id)

        while current and current.parent_id:
            parent = self._agents.get(current.parent_id)
            if parent:
                ancestors.append(parent)
                current = parent
            else:
                break

        return ancestors

    def get_descendants(self, agent_id: str) -> list[AgentNode]:
        """Get all descendants of an agent (all children recursively)."""
        descendants = []
        current = self._agents.get(agent_id)
        if not current:
            return descendants

        def collect_descendants(node: AgentNode) -> None:
            for child_id in node.children_ids:
                child = self._agents.get(child_id)
                if child:
                    descendants.append(child)
                    collect_descendants(child)

        collect_descendants(current)
        return descendants

    def can_delegate(
        self,
        from_agent_id: str,
        to_agent_id: str,
        task_context: dict[str, Any] | None = None,
    ) -> bool:
        """
        Check if agent can delegate to another agent.

        Args:
            from_agent_id: Source agent run_id
            to_agent_id: Target agent run_id
            task_context: Optional task context

        Returns:
            True if delegation is allowed
        """
        from_agent = self._agents.get(from_agent_id)
        to_agent = self._agents.get(to_agent_id)

        if not from_agent or not to_agent:
            return False

        # Executive can delegate to anyone
        if from_agent.role == AgentRole.EXECUTIVE:
            return True

        # Team leads can delegate to team members
        if from_agent.role == AgentRole.TEAM_LEAD:
            if to_agent.team_id == from_agent.team_id:
                return True

        # Cross-team delegation requires approval
        if from_agent.team_id != to_agent.team_id:
            # Check if cross-team collaboration is allowed
            if task_context and task_context.get("allow_cross_team", False):
                return True
            return False

        # Specialists can delegate to peers or lower-level agents
        if from_agent.role == AgentRole.SPECIALIST:
            if to_agent.role == AgentRole.SPECIALIST:
                return True

        return False

    def get_hierarchy_tree(self, root_id: str | None = None) -> dict[str, Any]:
        """
        Get hierarchy tree structure.

        Args:
            root_id: Optional root agent run_id (defaults to finding root)

        Returns:
            Tree structure as nested dictionary
        """
        if root_id:
            root = self._agents.get(root_id)
            if not root:
                return {}
        else:
            # Find root (agent with no parent)
            root = None
            for agent in self._agents.values():
                if agent.parent_id is None:
                    root = agent
                    break

            if not root:
                return {}

        def build_tree(node: AgentNode) -> dict[str, Any]:
            children = []
            for child_id in node.children_ids:
                child = self._agents.get(child_id)
                if child:
                    children.append(build_tree(child))

            return {
                "agent_id": node.agent_id,
                "run_id": node.run_id,
                "role": node.role.value,
                "team_id": node.team_id,
                "status": node.status,
                "children": children,
            }

        return build_tree(root)

    def list_all_agents(self) -> list[AgentNode]:
        """List all registered agents."""
        return list(self._agents.values())

    def list_all_teams(self) -> list[AgentTeam]:
        """List all teams."""
        return list(self._teams.values())

    def list_all_relationships(self) -> list[AgentRelationship]:
        """List all relationships."""
        return list(self._relationships.values())

    def update_agent_status(self, run_id: str, status: str) -> bool:
        """Update agent status."""
        agent = self._agents.get(run_id)
        if not agent:
            return False

        agent.status = status
        self._save()
        return True

    def update_team_status(self, team_id: str, status: str) -> bool:
        """Update team status."""
        team = self._teams.get(team_id)
        if not team:
            return False

        team.status = status
        self._save()
        return True

    def add_team_member(self, team_id: str, agent_run_id: str) -> bool:
        """Add member to team."""
        team = self._teams.get(team_id)
        agent = self._agents.get(agent_run_id)

        if not team or not agent:
            return False

        if agent_run_id not in team.members:
            team.members.append(agent_run_id)
            agent.team_id = team_id
            self._save()
            return True

        return False

    def remove_team_member(self, team_id: str, agent_run_id: str) -> bool:
        """Remove member from team."""
        team = self._teams.get(team_id)
        agent = self._agents.get(agent_run_id)

        if not team or not agent:
            return False

        if agent_run_id in team.members:
            team.members.remove(agent_run_id)
            if agent.team_id == team_id:
                agent.team_id = None
            self._save()
            return True

        return False

    def validate_agent_id(self, agent_id: str) -> tuple[bool, str | None]:
        """
        Validate that an agent ID exists and is valid.

        Args:
            agent_id: Agent run_id to validate

        Returns:
            Tuple of (is_valid, error_message)
        """
        if not agent_id or not isinstance(agent_id, str):
            return False, "Agent ID must be a non-empty string"

        if agent_id not in self._agents:
            return False, f"Agent ID '{agent_id}' not found in hierarchy"

        agent = self._agents[agent_id]
        if agent.status != "active":
            return False, f"Agent '{agent_id}' is not active (status: {agent.status})"

        return True, None

    def detect_circular_relationships(self, start_id: str) -> list[str]:
        """
        Detect circular relationships in the hierarchy starting from an agent.

        Args:
            start_id: Starting agent run_id

        Returns:
            List of agent IDs forming a cycle, empty if no cycle found
        """
        visited: set[str] = set()
        path: list[str] = []
        cycle: list[str] = []

        def dfs(node_id: str) -> bool:
            if node_id in path:
                # Found a cycle
                cycle_start = path.index(node_id)
                cycle.extend([*path[cycle_start:], node_id])
                return True

            if node_id in visited:
                return False

            visited.add(node_id)
            path.append(node_id)

            agent = self._agents.get(node_id)
            if agent and agent.parent_id:
                if dfs(agent.parent_id):
                    return True

            path.pop()
            return False

        dfs(start_id)
        return cycle

    def detect_orphaned_agents(self) -> list[tuple[str, str]]:
        """
        Detect agents with invalid parent references (orphaned agents).

        Returns:
            List of tuples (agent_id, error_message) for orphaned agents
        """
        orphaned = []

        for run_id, agent in self._agents.items():
            if agent.parent_id:
                parent = self._agents.get(agent.parent_id)
                if not parent:
                    orphaned.append((run_id, f"Parent '{agent.parent_id}' does not exist"))
                elif parent.status != "active":
                    orphaned.append((run_id, f"Parent '{agent.parent_id}' is not active"))

            # Check team membership
            if agent.team_id:
                team = self._teams.get(agent.team_id)
                if not team:
                    orphaned.append((run_id, f"Team '{agent.team_id}' does not exist"))
                elif run_id not in team.members and team.lead_id != run_id:
                    orphaned.append((run_id, f"Agent not in team '{agent.team_id}' members list"))

        return orphaned

    def check_team_consistency(self) -> list[tuple[str, str]]:
        """
        Check team consistency (members match team membership, leads exist, etc.).

        Returns:
            List of tuples (team_id, error_message) for inconsistent teams
        """
        inconsistencies = []

        for team_id, team in self._teams.items():
            # Check if lead exists and is active
            if team.lead_id:
                lead = self._agents.get(team.lead_id)
                if not lead:
                    inconsistencies.append((team_id, f"Team lead '{team.lead_id}' does not exist"))
                elif lead.status != "active":
                    inconsistencies.append((team_id, f"Team lead '{team.lead_id}' is not active"))
                elif lead.team_id != team_id:
                    inconsistencies.append((team_id, f"Team lead '{team.lead_id}' not assigned to this team"))

            # Check all members exist and are active
            for member_id in team.members:
                member = self._agents.get(member_id)
                if not member:
                    inconsistencies.append((team_id, f"Member '{member_id}' does not exist"))
                elif member.status != "active":
                    inconsistencies.append((team_id, f"Member '{member_id}' is not active"))
                elif member.team_id != team_id:
                    inconsistencies.append((team_id, f"Member '{member_id}' not assigned to this team"))

            # Check for duplicate members
            if len(team.members) != len(set(team.members)):
                duplicates = [m for m in team.members if team.members.count(m) > 1]
                inconsistencies.append((team_id, f"Duplicate members found: {set(duplicates)}"))

        return inconsistencies

    def validate_before_register(
        self, agent_id: str, run_id: str, parent_id: str | None = None, team_id: str | None = None
    ) -> tuple[bool, str | None]:
        """
        Validate before registering a new agent.

        Args:
            agent_id: Agent identifier
            run_id: Unique run identifier
            parent_id: Optional parent agent run_id
            team_id: Optional team identifier

        Returns:
            Tuple of (is_valid, error_message)
        """
        # Check if run_id already exists
        if run_id in self._agents:
            return False, f"Agent with run_id '{run_id}' already exists"

        # Validate parent if provided
        if parent_id:
            is_valid, error = self.validate_agent_id(parent_id)
            if not is_valid:
                return False, f"Invalid parent: {error}"

            # Check for circular relationship
            cycle = self.detect_circular_relationships(parent_id)
            if cycle and run_id in cycle:
                return False, f"Would create circular relationship: {' -> '.join(cycle)}"

        # Validate team if provided
        if team_id:
            team = self._teams.get(team_id)
            if not team:
                return False, f"Team '{team_id}' does not exist"
            if team.status != "active":
                return False, f"Team '{team_id}' is not active"

        return True, None
