namespace Phenotype.Skills;

/// <summary>
/// Execution modes for skill sandboxing
/// </summary>
public enum ExecutionMode
{
    /// <summary>
    /// In-process execution (no sandbox)
    /// </summary>
    InProcess,
    
    /// <summary>
    /// WASM sandbox (Tier 1) - ~1ms startup, ~1MB memory
    /// </summary>
    WASM,
    
    /// <summary>
    /// gVisor container (Tier 2) - ~90ms startup, ~20MB memory
    /// </summary>
    GVisor,
    
    /// <summary>
    /// Firecracker microVM (Tier 3) - ~125ms startup, &lt;5MB memory
    /// </summary>
    Firecracker
}

/// <summary>
/// Skill event types
/// </summary>
public enum SkillEventType
{
    Registered,
    Unregistered,
    Activated,
    Deactivated,
    Updated,
    DependencyResolved,
    ExecutionStarted,
    ExecutionCompleted
}

/// <summary>
/// Skill event
/// </summary>
public class SkillEvent
{
    public SkillEventType Type { get; set; }
    public string SkillId { get; set; } = string.Empty;
    public DateTime Timestamp { get; set; }
    public Dictionary<string, object> Data { get; set; } = new();
}
