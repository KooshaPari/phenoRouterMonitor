"""Task routing and categorization for thegent.

Implements Pareto frontier routing based on Terminal Bench 2.0 benchmarks.
Routes tasks to optimal models based on complexity, cost constraints, and quality requirements.

Key components:
- TaskRouter: Main routing engine with constraint validation
- TaskClassifier: Categorizes tasks (FAST/NORMAL/COMPLEX/HIGH_COMPLEX)
- ConstraintValidator: Validates hard constraints (quality, cost, speed)
- Pareto router: Hard constraints → Pareto frontier → lexicographic selection
- Auto router: Gemini Flash classifier + Pareto routing for agent/model="auto"
"""

# Import only ParetoRouter and RouteCandidate for now - avoid cliproxy_client
# which has config dependencies
from thegent.utils.routing_impl.pareto_router import ParetoRouter, RouteCandidate

__all__ = [
    "ParetoRouter",
    "RouteCandidate",
]
