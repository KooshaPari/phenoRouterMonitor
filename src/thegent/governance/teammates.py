"""WP-16001/16002: Thegent Teammates orchestration and delegation protocol."""

import json
import re
import uuid
from dataclasses import asdict, dataclass, field
from datetime import UTC, datetime
from pathlib import Path
from typing import Any

from .agent_hierarchy import (
    AgentHierarchyManager,
    AgentRole,
    CoordinationMode,
    RelationshipType,
    TeamType,
)


@dataclass
class TeammatePersona:
    """Specialized agent persona for the teammate swarm."""

    id: str
    role: str
    description: str
    capabilities: list[str]
    default_model: str = "gemini-3-flash"
    priority: int = 1  # 1 (high) to 5 (background)
    odd: str | None = None  # Operational Design Domain (WP-16001)


@dataclass
class DelegationRequest:
    """Record of a delegated task to a teammate."""

    id: str
    teammate_id: str
    parent_run_id: str
    prompt: str
    status: str = "pending"  # pending, running, completed, failed, deferred, paused
    created_at: str = field(default_factory=lambda: datetime.now(UTC).isoformat())
    completed_at: str | None = None
    result_summary: str | None = None
    artifact_path: str | None = None


class TeammateManager:
    """Manages discovery and delegation for the teammate swarm."""

    def __init__(self, storage_path: Path, hierarchy_manager: AgentHierarchyManager | None = None) -> None:
        """
        Initialize teammate manager.

        Args:
            storage_path: Path for storing delegations (JSON file)
            hierarchy_manager: Optional AgentHierarchyManager for hierarchy support
        """
        self.storage_path = storage_path
        # Use a directory sibling to the storage file for hierarchy (WP-16001)
        hierarchy_dir = storage_path.parent / "teammate_hierarchy"
        self.hierarchy = hierarchy_manager or AgentHierarchyManager(hierarchy_dir)
        self._delegations: dict[str, DelegationRequest] = {}
        self._load()

    def _load(self) -> None:
        if self.storage_path.exists():
            try:
                data = json.loads(self.storage_path.read_text())
                for did, ddata in data.items():
                    self._delegations[did] = DelegationRequest(**ddata)
            except (json.JSONDecodeError, KeyError):
                pass

    def _save(self) -> None:
        self.storage_path.parent.mkdir(parents=True, exist_ok=True)
        data = {did: asdict(d) for did, d in self._delegations.items()}
        self.storage_path.write_text(json.dumps(data, indent=2))

    def list_personas(self) -> list[TeammatePersona]:
        """WP-16001: Discover teammates from agent markdown files (recursive)."""
        personas = []
        # Look for agents/ in several possible locations, precedence to current working dir
        possible_dirs = [
            Path.cwd() / "agents",
            Path("agents"),
            Path(__file__).parent.parent.parent.parent / "agents",
            Path(__file__).parent.parent.parent.parent.parent / "thegent" / "agents",
        ]

        agents_dir = None
        for d in possible_dirs:
            if d.exists() and d.is_dir():
                agents_dir = d
                # We stop at the first one that exists to avoid mixing definitions
                break

        if not agents_dir:
            return []

        from thegent.infra.fast_yaml_parser import yaml_loads

        # Recursive search for all .md files
        for md_file in agents_dir.rglob("*.md"):
            try:
                content = md_file.read_text(encoding="utf-8")

                # Default persona if no metadata found
                meta = {}

                # 1. Try YAML frontmatter extraction
                if content.strip().startswith("---"):
                    parts = content.split("---", 2)
                    if len(parts) >= 3:
                        try:
                            meta = yaml_loads(parts[1])
                        except Exception:
                            meta = {}

                # Check if it's explicitly a teammate
                is_teammate = meta.get("teammate") is True

                # 2. Heuristic extraction if no frontmatter or not explicitly teammate

                # Look for title as name
                title_match = re.search(r"^#\s+(.+)$", content, re.MULTILINE)
                name = meta.get("name") or (
                    title_match.group(1).split()[0].lower() if title_match else md_file.stem.lower()
                )

                # Look for "Role: " or "Role :" at start of line
                role_match = re.search(r"^\s*Role:\s*(.+)$", content, re.MULTILINE | re.IGNORECASE)
                role = meta.get("role") or (role_match.group(1).strip() if role_match else "specialist")

                # Heuristic for description
                desc_match = re.search(r"^\s*Description:\s*(.+)$", content, re.MULTILINE | re.IGNORECASE)
                description = meta.get("description") or (
                    desc_match.group(1).strip() if desc_match else (content[:200].replace("\n", " ").strip() + "...")
                )

                # Heuristic teammate check if not already confirmed
                if not is_teammate:
                    if (
                        "teammate" in content.lower()
                        or "specialized agent" in content.lower()
                        or "persona" in content.lower()
                    ):
                        is_teammate = True

                # Only include if it looks like a teammate
                if not is_teammate:
                    continue

                # Ensure tools is a list
                tools = meta.get("tools", [])
                if isinstance(tools, str):
                    tools = [tools]

                # WP-16001: Support model-specific persona variants
                # If frontmatter has 'model', create an ID suffix
                model_suffix = meta.get("model", "")
                persona_id = f"{name}-{model_suffix}" if model_suffix else name

                personas.append(
                    TeammatePersona(
                        id=persona_id,
                        role=role,
                        description=description,
                        capabilities=tools,
                        default_model=meta.get("model", "gemini-3-flash"),
                        priority=meta.get("priority", 3),
                        odd=meta.get("odd"),
                    )
                )
            except Exception:
                continue

        return sorted(personas, key=lambda x: x.id)

    def delegate(
        self,
        teammate_id: str,
        parent_run_id: str,
        prompt: str,
        team_id: str | None = None,
        relationship_type: RelationshipType = RelationshipType.DIRECT_PARENT_CHILD,
    ) -> DelegationRequest:
        """
        WP-16002: Delegate a task to a teammate with hierarchy support.

        Args:
            teammate_id: Teammate agent identifier
            parent_run_id: Parent agent run_id
            prompt: Task prompt
            team_id: Optional team identifier
            relationship_type: Type of relationship

        Returns:
            DelegationRequest
        """
        req_id = f"DEL-{uuid.uuid4().hex[:8]}"

        # Ensure parent exists in hierarchy (WP-16001 auto-registration)
        if not self.hierarchy.get_agent(parent_run_id):
            parent_role = AgentRole.EXECUTIVE if parent_run_id == "CLI-USER" else AgentRole.TEAM_LEAD
            self.hierarchy.register_agent(
                agent_id="human" if parent_run_id == "CLI-USER" else "parent-agent",
                run_id=parent_run_id,
                role=parent_role,
                validate=False,
            )

        # Infer role from teammate_id or use SPECIALIST as default
        role = self._infer_role(teammate_id)

        # Register child agent in hierarchy
        self.hierarchy.register_agent(
            agent_id=teammate_id,
            run_id=req_id,
            role=role,
            parent_id=parent_run_id,
            team_id=team_id,
        )

        # Create relationship
        self.hierarchy.create_relationship(
            parent_id=parent_run_id,
            child_id=req_id,
            relationship_type=relationship_type,
            delegation_prompt=prompt,
            task_id=req_id,
        )

        # Create delegation request
        request = DelegationRequest(
            id=req_id, teammate_id=teammate_id, parent_run_id=parent_run_id, prompt=prompt, status="pending"
        )
        self._delegations[req_id] = request
        self._save()

        # WP-16002: Trigger background execution
        try:
            # Resolve effective agent ID (canonical name)
            from thegent.agents.registry import resolve_agent
            from thegent.cli.commands.impl import bg_impl
            from thegent.config_provider import get_config_provider

            effective_agent = resolve_agent(teammate_id) or teammate_id

            bg_impl(
                agent=effective_agent,
                prompt=prompt,
                cd=None,  # Use current directory
                mode="write",
                timeout=600,
                full=False,
                model=None,
                provider=None,
                owner=f"teammate:{teammate_id}",
                run_id=req_id,
                lane="standard",
                task_id=req_id,
                config_provider=get_config_provider(),
            )
            # Update status to running once backgrounded
            self.update_status(req_id, "running")
        except Exception as e:
            # If execution fails to start, mark as failed
            self.update_status(req_id, "failed", summary=f"Failed to start: {e}")

        # WP-16003: heliosShield Integration
        try:
            from thegent.governance.heliosShield_bridge import heliosShieldBridge

            bridge = heliosShieldBridge()
            if bridge.is_available():
                bridge.create_shared_task(
                    task_id=req_id, description=f"Delegated from {parent_run_id}: {prompt[:50]}..."
                )
                bridge.broadcast_intent(agent_id=f"thegent:{parent_run_id}", intent_type="delegate", target=teammate_id)
        except ImportError:
            # heliosShield bridge not available, continue without it
            pass

        # In a real implementation, this would trigger 'thegent bg'
        return request

    def _infer_role(self, teammate_id: str) -> AgentRole:
        """Infer agent role from teammate_id."""
        teammate_id_lower = teammate_id.lower()

        # Check for team lead indicators
        if "lead" in teammate_id_lower or "manager" in teammate_id_lower:
            return AgentRole.TEAM_LEAD

        # Check for executive indicators
        if "executive" in teammate_id_lower or "orchestrator" in teammate_id_lower or "sitback" in teammate_id_lower:
            return AgentRole.EXECUTIVE

        # Default to specialist
        return AgentRole.SPECIALIST

    def create_team(
        self,
        team_id: str,
        name: str,
        description: str,
        team_type: TeamType,
        coordination_mode: CoordinationMode,
        lead_id: str,
    ) -> Any:
        """
        Create a new team.

        Args:
            team_id: Team identifier
            name: Team name
            description: Team description
            team_type: Type of team
            coordination_mode: Coordination mode
            lead_id: Team lead agent run_id

        Returns:
            Created AgentTeam
        """
        return self.hierarchy.create_team(
            team_id=team_id,
            name=name,
            description=description,
            team_type=team_type,
            coordination_mode=coordination_mode,
            lead_id=lead_id,
        )

    def update_status(self, req_id: str, status: str, summary: str | None = None) -> bool:
        """Update the status of a delegation."""
        if req_id not in self._delegations:
            return False

        req = self._delegations[req_id]
        req.status = status
        if status in ("completed", "failed", "deferred", "paused"):
            if status not in ("deferred", "paused"):
                req.completed_at = datetime.now(UTC).isoformat()
            req.result_summary = summary

        self._save()
        return True

    def get_delegations(self, parent_run_id: str | None = None) -> list[DelegationRequest]:
        """List all delegations, optionally filtered by parent run."""
        if parent_run_id:
            return [d for d in self._delegations.values() if d.parent_run_id == parent_run_id]
        return list(self._delegations.values())
