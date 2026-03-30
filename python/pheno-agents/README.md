# pheno-agents

Agent orchestration for the Phenotype SDK.

## Overview

`pheno-agents` provides abstractions for managing, coordinating, and orchestrating multiple agents in complex workflows. It includes agent base classes, agent pools, and workflow orchestration with step dependencies.

## Features

- **Agent Base**: Abstract agent class with standard lifecycle
- **Agent States**: Full state machine (idle, running, paused, failed, completed, terminated)
- **Agent Roles**: Support for multiple agent roles (orchestrator, worker, monitor, logger, manager)
- **Agent Pools**: Efficient management and filtering of multiple agents
- **Workflow Orchestration**: Sophisticated workflow execution with step dependencies
- **Async Support**: Full async/await support for concurrent execution

## Installation

```bash
pip install pheno-agents
```

## Quick Start

```python
from pheno_agents import Agent, Orchestrator, WorkflowStep, AgentState, AgentRole
import asyncio

# Create an orchestrator
orchestrator = Orchestrator()

# Define a custom agent
class MyAgent(Agent):
    async def execute(self, task):
        return {"result": "completed"}

    async def initialize(self):
        self.state = AgentState.IDLE

    async def shutdown(self):
        self.state = AgentState.TERMINATED

# Register agent
agent = MyAgent(name="worker-1", role=AgentRole.WORKER)
orchestrator.register_agent(agent)

# Create and execute workflow
async def main():
    orchestrator.create_workflow("my-workflow")
    step1 = WorkflowStep(name="step1", agent_id=agent.id)
    orchestrator.add_step("my-workflow", step1)

    result = await orchestrator.execute_workflow("my-workflow")
    print(f"Success: {result.success}")
    print(f"Executed: {result.steps_executed}/{result.total_steps}")

asyncio.run(main())
```

## Testing

```bash
pip install -e ".[dev]"
pytest tests/ -v
```

## License

MIT
