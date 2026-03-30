"""Tests for pheno-agents base agent functionality."""

import pytest

from pheno_agents import Agent, AgentState, AgentRole
from pheno_agents.base import SimpleAgent, AgentPool


class TestAgentState:
    """Test AgentState enumeration."""

    def test_agent_state_values(self) -> None:
        """Test that all agent states have correct values."""
        assert AgentState.IDLE.value == "idle"
        assert AgentState.RUNNING.value == "running"
        assert AgentState.PAUSED.value == "paused"
        assert AgentState.FAILED.value == "failed"
        assert AgentState.COMPLETED.value == "completed"
        assert AgentState.TERMINATED.value == "terminated"


class TestAgentRole:
    """Test AgentRole enumeration."""

    def test_agent_role_values(self) -> None:
        """Test that all agent roles have correct values."""
        assert AgentRole.ORCHESTRATOR.value == "orchestrator"
        assert AgentRole.WORKER.value == "worker"
        assert AgentRole.MONITOR.value == "monitor"
        assert AgentRole.LOGGER.value == "logger"
        assert AgentRole.MANAGER.value == "manager"


class TestSimpleAgent:
    """Test SimpleAgent implementation."""

    @pytest.mark.asyncio
    async def test_agent_initialization(self) -> None:
        """Test agent initialization."""
        agent = SimpleAgent(name="test-agent")
        await agent.initialize()
        assert agent.state == AgentState.IDLE

    @pytest.mark.asyncio
    async def test_agent_shutdown(self) -> None:
        """Test agent shutdown."""
        agent = SimpleAgent(name="test-agent")
        await agent.initialize()
        await agent.shutdown()
        assert agent.state == AgentState.TERMINATED

    def test_agent_capabilities(self) -> None:
        """Test agent capability management."""
        agent = SimpleAgent(name="test-agent")
        assert not agent.has_capability("task1")

        agent.add_capability("task1")
        assert agent.has_capability("task1")

        agent.add_capability("task2")
        assert len(agent.capabilities) == 2

        agent.remove_capability("task1")
        assert not agent.has_capability("task1")
        assert agent.has_capability("task2")

    def test_agent_duplicate_capability(self) -> None:
        """Test that duplicate capabilities are not added."""
        agent = SimpleAgent(name="test-agent")
        agent.add_capability("task1")
        agent.add_capability("task1")
        assert agent.capabilities.count("task1") == 1

    def test_agent_get_status(self) -> None:
        """Test getting agent status."""
        agent = SimpleAgent(
            name="test-agent",
            role=AgentRole.WORKER,
        )
        agent.add_capability("task1")

        status = agent.get_status()
        assert status["name"] == "test-agent"
        assert status["role"] == "worker"
        assert status["state"] == "idle"
        assert "task1" in status["capabilities"]

    def test_agent_metadata(self) -> None:
        """Test agent with metadata."""
        metadata = {"version": "1.0", "author": "test"}
        agent = SimpleAgent(
            name="test-agent",
            metadata=metadata,
        )
        assert agent.metadata == metadata

    def test_agent_max_retries(self) -> None:
        """Test agent max retries setting."""
        agent = SimpleAgent(name="test-agent", max_retries=5)
        assert agent.max_retries == 5

    def test_agent_timeout(self) -> None:
        """Test agent timeout setting."""
        agent = SimpleAgent(name="test-agent", timeout_seconds=120)
        assert agent.timeout_seconds == 120

    @pytest.mark.asyncio
    async def test_agent_execute_not_implemented(self) -> None:
        """Test that execute raises NotImplementedError in base class."""
        agent = SimpleAgent(name="test-agent")
        with pytest.raises(NotImplementedError):
            await agent.execute({})


class TestAgentPool:
    """Test AgentPool class."""

    def test_pool_add_and_get_agent(self) -> None:
        """Test adding and retrieving agents."""
        pool = AgentPool()
        agent = SimpleAgent(name="agent-1")
        pool.add_agent(agent)

        retrieved = pool.get_agent(agent.id)
        assert retrieved is not None
        assert retrieved.name == "agent-1"

    def test_pool_get_nonexistent_agent(self) -> None:
        """Test getting nonexistent agent returns None."""
        pool = AgentPool()
        result = pool.get_agent("nonexistent-id")
        assert result is None

    def test_pool_get_by_role(self) -> None:
        """Test filtering agents by role."""
        pool = AgentPool()
        worker1 = SimpleAgent(name="worker-1", role=AgentRole.WORKER)
        worker2 = SimpleAgent(name="worker-2", role=AgentRole.WORKER)
        monitor = SimpleAgent(name="monitor-1", role=AgentRole.MONITOR)

        pool.add_agent(worker1)
        pool.add_agent(worker2)
        pool.add_agent(monitor)

        workers = pool.get_by_role(AgentRole.WORKER)
        monitors = pool.get_by_role(AgentRole.MONITOR)

        assert len(workers) == 2
        assert len(monitors) == 1

    def test_pool_get_by_capability(self) -> None:
        """Test filtering agents by capability."""
        pool = AgentPool()
        agent1 = SimpleAgent(name="agent-1")
        agent2 = SimpleAgent(name="agent-2")
        agent3 = SimpleAgent(name="agent-3")

        agent1.add_capability("task1")
        agent1.add_capability("task2")
        agent2.add_capability("task1")
        agent3.add_capability("task3")

        task1_agents = pool.get_by_capability("task1")
        task3_agents = pool.get_by_capability("task3")

        # Before adding to pool, should have no agents
        assert len(task1_agents) == 0

        # Add agents
        pool.add_agent(agent1)
        pool.add_agent(agent2)
        pool.add_agent(agent3)

        # Now filter should work
        task1_agents = pool.get_by_capability("task1")
        task3_agents = pool.get_by_capability("task3")

        assert len(task1_agents) == 2
        assert len(task3_agents) == 1

    def test_pool_list_all(self) -> None:
        """Test listing all agents."""
        pool = AgentPool()
        agents = [SimpleAgent(name=f"agent-{i}") for i in range(5)]
        for agent in agents:
            pool.add_agent(agent)

        all_agents = pool.list_all()
        assert len(all_agents) == 5

    def test_pool_remove_agent(self) -> None:
        """Test removing agent from pool."""
        pool = AgentPool()
        agent = SimpleAgent(name="test-agent")
        pool.add_agent(agent)

        removed = pool.remove_agent(agent.id)
        assert removed is True
        assert pool.get_agent(agent.id) is None

    def test_pool_remove_nonexistent_agent(self) -> None:
        """Test removing nonexistent agent returns False."""
        pool = AgentPool()
        removed = pool.remove_agent("nonexistent-id")
        assert removed is False

    @pytest.mark.asyncio
    async def test_pool_get_idle_agents(self) -> None:
        """Test getting idle agents."""
        pool = AgentPool()
        agent1 = SimpleAgent(name="agent-1")
        agent2 = SimpleAgent(name="agent-2")
        agent3 = SimpleAgent(name="agent-3")

        await agent1.initialize()
        await agent2.initialize()
        await agent3.initialize()

        agent2.state = AgentState.RUNNING

        pool.add_agent(agent1)
        pool.add_agent(agent2)
        pool.add_agent(agent3)

        idle = pool.get_idle_agents()
        assert len(idle) == 2

    @pytest.mark.asyncio
    async def test_pool_get_running_agents(self) -> None:
        """Test getting running agents."""
        pool = AgentPool()
        agent1 = SimpleAgent(name="agent-1")
        agent2 = SimpleAgent(name="agent-2")

        await agent1.initialize()
        await agent2.initialize()

        agent1.state = AgentState.RUNNING

        pool.add_agent(agent1)
        pool.add_agent(agent2)

        running = pool.get_running_agents()
        assert len(running) == 1
        assert running[0].name == "agent-1"


class TestAgentIntegration:
    """Integration tests for agents."""

    def test_multiple_agents_with_different_roles(self) -> None:
        """Test creating multiple agents with different roles."""
        pool = AgentPool()

        roles = [AgentRole.ORCHESTRATOR, AgentRole.WORKER, AgentRole.MONITOR]
        agents = [SimpleAgent(name=f"agent-{i}", role=role) for i, role in enumerate(roles)]

        for agent in agents:
            pool.add_agent(agent)

        for role in roles:
            filtered = pool.get_by_role(role)
            assert len(filtered) == 1
            assert filtered[0].role == role

    def test_agent_pool_workflow_simulation(self) -> None:
        """Simulate a workflow with agent pool."""
        pool = AgentPool()

        # Create agents with specific capabilities
        preprocessor = SimpleAgent(name="preprocessor", role=AgentRole.WORKER)
        preprocessor.add_capability("preprocess")

        processor = SimpleAgent(name="processor", role=AgentRole.WORKER)
        processor.add_capability("process")

        monitor = SimpleAgent(name="monitor", role=AgentRole.MONITOR)
        monitor.add_capability("monitoring")

        pool.add_agent(preprocessor)
        pool.add_agent(processor)
        pool.add_agent(monitor)

        # Verify structure
        workers = pool.get_by_role(AgentRole.WORKER)
        assert len(workers) == 2

        preprocessors = pool.get_by_capability("preprocess")
        assert len(preprocessors) == 1
