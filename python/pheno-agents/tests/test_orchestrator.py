"""Tests for pheno-agents orchestrator functionality."""

import pytest

from pheno_agents import Orchestrator, WorkflowStep, AgentState
from pheno_agents.base import SimpleAgent


class TestWorkflowStep:
    """Test WorkflowStep class."""

    def test_step_creation(self) -> None:
        """Test creating a workflow step."""
        step = WorkflowStep(
            name="process-data",
            action="process",
            parameters={"input": "data.txt"},
        )
        assert step.name == "process-data"
        assert step.action == "process"
        assert step.parameters == {"input": "data.txt"}

    def test_step_dependencies(self) -> None:
        """Test step dependencies."""
        step1 = WorkflowStep(name="step1")
        step2 = WorkflowStep(
            name="step2",
            dependencies=[step1.id],
        )

        assert not step1.has_dependencies()
        assert step2.has_dependencies()
        assert step1.id in step2.dependencies

    def test_step_retry_configuration(self) -> None:
        """Test step retry configuration."""
        step = WorkflowStep(
            name="test-step",
            max_retries=5,
            timeout=120,
        )
        assert step.max_retries == 5
        assert step.timeout == 120


class TestOrchestrator:
    """Test Orchestrator class."""

    def test_orchestrator_creation(self) -> None:
        """Test creating an orchestrator."""
        orchestrator = Orchestrator()
        assert orchestrator.list_workflows() == []
        assert orchestrator.list_agents() == []

    def test_orchestrator_register_agent(self) -> None:
        """Test registering agent with orchestrator."""
        orchestrator = Orchestrator()
        agent = SimpleAgent(name="test-agent")
        orchestrator.register_agent(agent)

        agents = orchestrator.list_agents()
        assert len(agents) == 1
        assert agent.id in agents

    def test_orchestrator_create_workflow(self) -> None:
        """Test creating a workflow."""
        orchestrator = Orchestrator()
        orchestrator.create_workflow("workflow-1")

        workflows = orchestrator.list_workflows()
        assert "workflow-1" in workflows

    def test_orchestrator_add_step(self) -> None:
        """Test adding step to workflow."""
        orchestrator = Orchestrator()
        orchestrator.create_workflow("workflow-1")
        step = WorkflowStep(name="step1")
        orchestrator.add_step("workflow-1", step)

        workflow = orchestrator.get_workflow("workflow-1")
        assert workflow is not None
        assert len(workflow) == 1
        assert workflow[0].name == "step1"

    def test_orchestrator_add_step_nonexistent_workflow(self) -> None:
        """Test adding step to nonexistent workflow raises error."""
        orchestrator = Orchestrator()
        step = WorkflowStep(name="step1")

        with pytest.raises(ValueError):
            orchestrator.add_step("nonexistent", step)

    def test_orchestrator_get_workflow_nonexistent(self) -> None:
        """Test getting nonexistent workflow returns None."""
        orchestrator = Orchestrator()
        result = orchestrator.get_workflow("nonexistent")
        assert result is None

    def test_orchestrator_validate_workflow_success(self) -> None:
        """Test validating valid workflow."""
        orchestrator = Orchestrator()
        agent = SimpleAgent(name="test-agent")
        orchestrator.register_agent(agent)

        orchestrator.create_workflow("workflow-1")
        step1 = WorkflowStep(name="step1", agent_id=agent.id)
        step2 = WorkflowStep(name="step2", agent_id=agent.id, dependencies=[step1.id])
        orchestrator.add_step("workflow-1", step1)
        orchestrator.add_step("workflow-1", step2)

        assert orchestrator.validate_workflow("workflow-1") is True

    def test_orchestrator_validate_workflow_missing_dependency(self) -> None:
        """Test validating workflow with missing dependency."""
        orchestrator = Orchestrator()
        orchestrator.create_workflow("workflow-1")
        step = WorkflowStep(
            name="step1",
            dependencies=["nonexistent-step"],
        )
        orchestrator.add_step("workflow-1", step)

        with pytest.raises(ValueError):
            orchestrator.validate_workflow("workflow-1")

    def test_orchestrator_validate_workflow_missing_agent(self) -> None:
        """Test validating workflow with missing agent."""
        orchestrator = Orchestrator()
        orchestrator.create_workflow("workflow-1")
        step = WorkflowStep(name="step1", agent_id="nonexistent-agent")
        orchestrator.add_step("workflow-1", step)

        with pytest.raises(ValueError):
            orchestrator.validate_workflow("workflow-1")

    def test_orchestrator_validate_nonexistent_workflow(self) -> None:
        """Test validating nonexistent workflow raises error."""
        orchestrator = Orchestrator()

        with pytest.raises(ValueError):
            orchestrator.validate_workflow("nonexistent")

    @pytest.mark.asyncio
    async def test_orchestrator_execute_simple_workflow(self) -> None:
        """Test executing a simple workflow."""
        orchestrator = Orchestrator()

        # Create a simple agent that always succeeds
        class SuccessAgent(SimpleAgent):
            async def execute(self, task):
                return {"status": "success"}

        agent = SuccessAgent(name="success-agent")
        orchestrator.register_agent(agent)

        # Create workflow with one step
        orchestrator.create_workflow("workflow-1")
        step = WorkflowStep(name="step1", agent_id=agent.id)
        orchestrator.add_step("workflow-1", step)

        # Execute
        result = await orchestrator.execute_workflow("workflow-1")

        assert result.workflow_id == "workflow-1"
        assert result.total_steps == 1
        assert result.steps_executed == 1

    @pytest.mark.asyncio
    async def test_orchestrator_get_execution_status(self) -> None:
        """Test getting execution status."""
        orchestrator = Orchestrator()

        class SuccessAgent(SimpleAgent):
            async def execute(self, task):
                return {"status": "success"}

        agent = SuccessAgent(name="success-agent")
        orchestrator.register_agent(agent)

        orchestrator.create_workflow("workflow-1")
        step = WorkflowStep(name="step1", agent_id=agent.id)
        orchestrator.add_step("workflow-1", step)

        await orchestrator.execute_workflow("workflow-1")

        status = orchestrator.get_execution_status("workflow-1")
        assert status is not None
        assert status["workflow_id"] == "workflow-1"
        assert status["success"] is True

    @pytest.mark.asyncio
    async def test_orchestrator_workflow_with_multiple_steps(self) -> None:
        """Test workflow with multiple sequential steps."""
        orchestrator = Orchestrator()

        class StepAgent(SimpleAgent):
            async def execute(self, task):
                return {"step": task.get("name", "unknown")}

        agent = StepAgent(name="step-agent")
        orchestrator.register_agent(agent)

        orchestrator.create_workflow("workflow-1")

        step1 = WorkflowStep(name="step1", agent_id=agent.id, parameters={"name": "step1"})
        step2 = WorkflowStep(
            name="step2",
            agent_id=agent.id,
            parameters={"name": "step2"},
            dependencies=[step1.id],
        )

        orchestrator.add_step("workflow-1", step1)
        orchestrator.add_step("workflow-1", step2)

        result = await orchestrator.execute_workflow("workflow-1")

        assert result.total_steps == 2

    def test_orchestrator_list_workflows(self) -> None:
        """Test listing workflows."""
        orchestrator = Orchestrator()

        for i in range(3):
            orchestrator.create_workflow(f"workflow-{i}")

        workflows = orchestrator.list_workflows()
        assert len(workflows) == 3

    def test_orchestrator_multiple_agents(self) -> None:
        """Test orchestrator with multiple agents."""
        orchestrator = Orchestrator()

        agents = [SimpleAgent(name=f"agent-{i}") for i in range(5)]
        for agent in agents:
            orchestrator.register_agent(agent)

        registered = orchestrator.list_agents()
        assert len(registered) == 5

    @pytest.mark.asyncio
    async def test_orchestrator_get_result(self) -> None:
        """Test getting workflow result."""
        orchestrator = Orchestrator()

        class SuccessAgent(SimpleAgent):
            async def execute(self, task):
                return {"status": "success"}

        agent = SuccessAgent(name="success-agent")
        orchestrator.register_agent(agent)

        orchestrator.create_workflow("workflow-1")
        step = WorkflowStep(name="step1", agent_id=agent.id)
        orchestrator.add_step("workflow-1", step)

        # Before execution, result is None
        assert orchestrator.get_result("workflow-1") is None

        # After execution
        await orchestrator.execute_workflow("workflow-1")
        result = orchestrator.get_result("workflow-1")

        assert result is not None
        assert result.workflow_id == "workflow-1"


class TestOrchestrationIntegration:
    """Integration tests for orchestration."""

    @pytest.mark.asyncio
    async def test_complex_workflow_simulation(self) -> None:
        """Simulate a complex multi-step workflow."""
        orchestrator = Orchestrator()

        # Create specialized agents
        class ProcessorAgent(SimpleAgent):
            async def execute(self, task):
                data = task.get("data", {})
                return {"processed": True, "output": data}

        class ValidatorAgent(SimpleAgent):
            async def execute(self, task):
                return {"validated": True}

        processor = ProcessorAgent(name="processor")
        validator = ValidatorAgent(name="validator")

        orchestrator.register_agent(processor)
        orchestrator.register_agent(validator)

        # Create workflow
        orchestrator.create_workflow("etl-workflow")

        step1 = WorkflowStep(
            name="extract",
            agent_id=processor.id,
            parameters={"data": {"source": "database"}},
        )
        step2 = WorkflowStep(
            name="validate",
            agent_id=validator.id,
            dependencies=[step1.id],
        )

        orchestrator.add_step("etl-workflow", step1)
        orchestrator.add_step("etl-workflow", step2)

        # Execute
        result = await orchestrator.execute_workflow("etl-workflow")

        assert result.success
        assert result.steps_executed == 2
