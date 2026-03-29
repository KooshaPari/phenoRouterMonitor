"""Infrastructure modules for thegent.

This package contains infrastructure utilities including:
- Runtime dispatcher for multi-runtime support
- Performance optimizations
- Error handling
- Progress indicators
- Configuration management
- Multi-runtime diagnostics
"""

from thegent.infra.enhanced_errors import (
    ConfigurationError,
    DependencyError,
    EnhancedError,
    NetworkError,
    create_config_error,
    create_dependency_error,
    create_network_error,
    create_runtime_error,
    error_report,
    format_error,
    format_error_with_context,
)
from thegent.infra.enhanced_errors import (
    InfraRuntimeError,
)
from thegent.infra.cache_v2 import MultiTierCache, get_cache
from thegent.infra.fast_file_ops import (
    copy_file,
    copy_tree,
)
from thegent.infra.fast_subprocess import run_subprocess_optimized
from thegent.infra.fast_yaml_parser import yaml_dump, yaml_load, yaml_loads
from thegent.infra.mojo_bridge import (
    MojoBridge,
    MojoNotAvailableError,
    MojoTask,
    check_mojo_status,
    get_bridge,
)
from thegent.infra.progress import (
    measure_time,
    print_section,
    print_status,
    print_step,
    progress_context,
    spinner_context,
)

__all__ = [
    "ConfigurationError",
    "DependencyError",
    # Enhanced errors
    "EnhancedError",
    "InfraRuntimeError",
    # Mojo bridge
    "MojoBridge",
    "MojoNotAvailableError",
    "MojoTask",
    # Cache
    "MultiTierCache",
    "NetworkError",
    "check_mojo_status",
    # File operations
    "copy_file",
    "copy_tree",
    "create_config_error",
    "create_dependency_error",
    "create_network_error",
    "create_runtime_error",
    "error_report",
    "format_error",
    "format_error_with_context",
    "get_bridge",
    "get_cache",
    "measure_time",
    "print_section",
    "print_status",
    "print_step",
    # Progress indicators
    "progress_context",
    # Subprocess
    "run_subprocess_optimized",
    "spinner_context",
    # YAML
    "yaml_dump",
    "yaml_load",
    "yaml_loads",
]
