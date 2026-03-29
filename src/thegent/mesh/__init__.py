"""heliosShield: High-performance agent mesh orchestration package."""

from thegent.mesh.audit import AuditManager
from thegent.mesh.cache import MeshCache, Singleflight
from thegent.mesh.consensus import CausalInfluenceTracker, ConsensusProtocol, EscalationWorkflow
from thegent.mesh.coordination import FileClaimsRegistry, HLCTimestamp, OptimisticConcurrencyControl
from thegent_gitops import GitParallelismManager, WorktreeContext, WorktreePool
from thegent.mesh.injection import ContextInjection, ShellInjection
from thegent.mesh.isolation import ResourceIsolation
from thegent.mesh.merge import SmartMerge
from thegent.mesh.mesh import MeshIPC, MeshManager
from thegent.mesh.observability import MeshLogger, MetricsAggregator
from thegent.mesh.resources import ResourceManager
from thegent.mesh.sandbox import AutonomyTier, Sandboxing
from thegent.mesh.smart_merge import (
    MergeResult,
    SmartMergeConfig,
    SmartMerger,
    configure_mergiraf_driver,
    is_mergiraf_available,
    make_smart_merger,
    merge_files,
)
from thegent.mesh.task_queue import MaildirQueue
from thegent.mesh.worktree import BranchCollisionError, WorktreeManager

__all__ = [
    "AuditManager",
    "AutonomyTier",
    "BranchCollisionError",
    "CausalInfluenceTracker",
    "ConsensusProtocol",
    "ContextInjection",
    "EscalationWorkflow",
    "FileClaimsRegistry",
    "GitParallelismManager",
    "HLCTimestamp",
    "MaildirQueue",
    "MaildirQueue",
    "MergeResult",
    "MeshCache",
    "MeshIPC",
    "MeshLogger",
    "MeshManager",
    "MetricsAggregator",
    "OptimisticConcurrencyControl",
    "ResourceIsolation",
    "ResourceManager",
    "Sandboxing",
    "ShellInjection",
    "Singleflight",
    "SmartMerge",
    "SmartMergeConfig",
    "SmartMerger",
    "WorktreeContext",
    "WorktreeManager",
    "WorktreePool",
    "configure_mergiraf_driver",
    "is_mergiraf_available",
    "make_smart_merger",
    "merge_files",
]
