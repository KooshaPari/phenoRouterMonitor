"""
Type definitions for Phenotype Skills Python bindings.
"""

from enum import Enum
from dataclasses import dataclass
from typing import Optional, List, Dict, Any
from datetime import datetime


class ExecutionMode(Enum):
    """Execution modes for skill sandboxing."""
    IN_PROCESS = "in_process"
    WASM = "wasm"
    GVISOR = "gvisor"
    FIRECRACKER = "firecracker"


@dataclass
class SkillDependency:
    """A dependency on another skill."""
    name: str
    version_req: str
    optional: bool = False


@dataclass
class SkillMetadata:
    """Additional metadata for skills."""
    tags: List[str]
    categories: List[str]
    homepage: Optional[str] = None
    repository: Optional[str] = None
    created_at: Optional[datetime] = None
    updated_at: Optional[datetime] = None
    attributes: Dict[str, str]


@dataclass
class SkillEvent:
    """Events that can occur in the skill system."""
    event_type: str
    skill_id: str
    timestamp: datetime
    data: Dict[str, Any]
