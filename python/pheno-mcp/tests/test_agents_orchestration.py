"""Tests for CrewAI agent orchestration abstraction.

Traces to: FR-MCP-003 - CrewAI Agent Orchestration Adapter
"""

import pytest
from typing import Optional
from pheno_mcp.agents import (
    AgentOrchestrator,
    Agent,
    AgentRole,
    TaskDefinition,
)


class TestAgent:
    """Test individual Agent configuration."""

    def test_agent_initialization(self):
        """Test that Agent initializes with required parameters."""
        agent = Agent(
            role=AgentRole.MANAGER,
            name="test-agent",
            description="Test agent"
        )
        assert agent.role == AgentRole.MANAGER
        assert agent.name == "test-agent"
        assert agent.description == "Test agent"

    def test_agent_with_tools(self):
        """Test that Agent can be configured with tools."""
        agent = Agent(
            role=AgentRole.WORKER,
            name="worker-agent",
            tools=["tool1", "tool2"]
        )
        assert len(agent.tools) == 2
        assert "tool1" in agent.tools

    def test_agent_no_atoms_specifics(self):
        """Test that Agent has no atoms-specific naming or structure."""
        agent = Agent(
            role=AgentRole.ANALYST,
            name="generic-analyst"
        )
        # Should not have atoms-specific attributes
        assert not hasattr(agent, "atoms_role")
        assert agent.name == "generic-analyst"

    def test_agent_role_enum(self):
        """Test that AgentRole enum includes standard roles."""
        assert AgentRole.MANAGER is not None
        assert AgentRole.WORKER is not None
        assert AgentRole.ANALYST is not None


class TestTaskDefinition:
    """Test task definition for agents."""

    def test_task_initialization(self):
        """Test TaskDefinition initialization."""
        task = TaskDefinition(
            name="process-data",
            description="Process input data",
            expected_output="Processed results"
        )
        assert task.name == "process-data"
        assert task.description == "Process input data"
        assert task.expected_output == "Processed results"

    def test_task_with_agent_assignment(self):
        """Test assigning task to an agent."""
        agent = Agent(role=AgentRole.WORKER, name="worker")
        task = TaskDefinition(
            name="task1",
            description="Do something",
            assigned_to=agent
        )
        assert task.assigned_to == agent

    def test_task_with_dependencies(self):
        """Test task dependencies."""
        task1 = TaskDefinition(name="task1", description="First")
        task2 = TaskDefinition(
            name="task2",
            description="Second",
            depends_on=[task1]
        )
        assert len(task2.depends_on) == 1
        assert task2.depends_on[0] == task1


class TestAgentOrchestrator:
    """Test the AgentOrchestrator for CrewAI integration."""

    def test_orchestrator_initialization(self):
        """Test that AgentOrchestrator initializes."""
        orchestrator = AgentOrchestrator()
        assert orchestrator is not None
        assert orchestrator.agents() == []

    def test_orchestrator_add_agent(self):
        """Test adding agents to orchestrator."""
        orchestrator = AgentOrchestrator()
        agent = Agent(role=AgentRole.MANAGER, name="manager")
        orchestrator.add_agent(agent)

        agents = orchestrator.agents()
        assert len(agents) == 1
        assert agents[0] == agent

    def test_orchestrator_add_multiple_agents(self):
        """Test adding multiple agents."""
        orchestrator = AgentOrchestrator()
        agent1 = Agent(role=AgentRole.MANAGER, name="manager")
        agent2 = Agent(role=AgentRole.WORKER, name="worker")
        agent3 = Agent(role=AgentRole.ANALYST, name="analyst")

        orchestrator.add_agent(agent1)
        orchestrator.add_agent(agent2)
        orchestrator.add_agent(agent3)

        agents = orchestrator.agents()
        assert len(agents) == 3

    def test_orchestrator_add_task(self):
        """Test adding tasks to orchestrator."""
        orchestrator = AgentOrchestrator()
        task = TaskDefinition(name="task1", description="Do something")
        orchestrator.add_task(task)

        tasks = orchestrator.tasks()
        assert len(tasks) == 1
        assert tasks[0] == task

    def test_orchestrator_execute_simple_workflow(self):
        """Test executing a simple workflow."""
        orchestrator = AgentOrchestrator()

        # Add agent
        agent = Agent(role=AgentRole.WORKER, name="worker")
        orchestrator.add_agent(agent)

        # Add task
        task = TaskDefinition(
            name="simple_task",
            description="A simple task",
            assigned_to=agent
        )
        orchestrator.add_task(task)

        # Execute
        result = orchestrator.execute()
        assert result is not None

    def test_orchestrator_with_tool_binding(self):
        """Test binding tools to agents in orchestrator."""
        orchestrator = AgentOrchestrator()

        agent = Agent(
            role=AgentRole.WORKER,
            name="worker",
            tools=["mcp_tool_1", "mcp_tool_2"]
        )
        orchestrator.add_agent(agent)

        agents = orchestrator.agents()
        assert len(agents[0].tools) == 2

    def test_orchestrator_workflow_with_dependencies(self):
        """Test workflow with task dependencies."""
        orchestrator = AgentOrchestrator()

        agent1 = Agent(role=AgentRole.ANALYST, name="analyst")
        agent2 = Agent(role=AgentRole.WORKER, name="worker")

        orchestrator.add_agent(agent1)
        orchestrator.add_agent(agent2)

        task1 = TaskDefinition(
            name="analyze",
            description="Analyze data",
            assigned_to=agent1
        )
        task2 = TaskDefinition(
            name="process",
            description="Process results",
            assigned_to=agent2,
            depends_on=[task1]
        )

        orchestrator.add_task(task1)
        orchestrator.add_task(task2)

        tasks = orchestrator.tasks()
        assert len(tasks) == 2
        assert tasks[1].depends_on[0] == task1

    def test_orchestrator_no_atoms_specifics(self):
        """Test that orchestrator has no atoms-specific implementation."""
        orchestrator = AgentOrchestrator()
        # Should not have atoms-specific attributes
        assert not hasattr(orchestrator, "atoms_config")
        assert not hasattr(orchestrator, "atoms_specific_agents")


class TestAgentOrchestrationIntegration:
    """Integration tests for agent orchestration workflows."""

    def test_complete_workflow_with_multiple_agents(self):
        """Test a complete workflow with multiple agents."""
        orchestrator = AgentOrchestrator()

        # Create team
        manager = Agent(role=AgentRole.MANAGER, name="project_manager")
        analyst = Agent(role=AgentRole.ANALYST, name="data_analyst")
        worker = Agent(role=AgentRole.WORKER, name="executor")

        orchestrator.add_agent(manager)
        orchestrator.add_agent(analyst)
        orchestrator.add_agent(worker)

        # Create tasks
        task1 = TaskDefinition(
            name="plan",
            description="Plan the work",
            assigned_to=manager
        )
        task2 = TaskDefinition(
            name="analyze",
            description="Analyze requirements",
            assigned_to=analyst,
            depends_on=[task1]
        )
        task3 = TaskDefinition(
            name="execute",
            description="Execute the plan",
            assigned_to=worker,
            depends_on=[task2]
        )

        orchestrator.add_task(task1)
        orchestrator.add_task(task2)
        orchestrator.add_task(task3)

        # Verify setup
        assert len(orchestrator.agents()) == 3
        assert len(orchestrator.tasks()) == 3

    def test_orchestrator_with_mcp_tools_integration(self):
        """Test orchestrator with MCP tools bound to agents."""
        from pheno_mcp.tools import tool_registry

        # Create tool registry with some tools
        registry = tool_registry.ToolRegistry()
        registry.register("analyze", lambda x: f"analyzed: {x}")
        registry.register("process", lambda x: f"processed: {x}")

        # Create orchestrator with tool-aware agents
        orchestrator = AgentOrchestrator()

        agent = Agent(
            role=AgentRole.WORKER,
            name="tool_user",
            tools=["analyze", "process"]
        )
        orchestrator.add_agent(agent)

        task = TaskDefinition(
            name="use_tools",
            description="Use MCP tools",
            assigned_to=agent
        )
        orchestrator.add_task(task)

        assert len(orchestrator.agents()) == 1
        assert len(orchestrator.agents()[0].tools) == 2
