using System;
using System.Runtime.InteropServices;
using System.Collections.Generic;
using System.Threading.Tasks;

namespace Phenotype.Skills;

/// <summary>
/// C# bindings for the Phenotype Skills Rust library
/// </summary>
public class SkillRegistry : IDisposable
{
    private IntPtr _handle;
    private bool _disposed;

    // Native function imports from Rust library
    [DllImport("phenotype_skills", CallingConvention = CallingConvention.Cdecl)]
    private static extern IntPtr skill_registry_new();

    [DllImport("phenotype_skills", CallingConvention = CallingConvention.Cdecl)]
    private static extern void skill_registry_free(IntPtr handle);

    [DllImport("phenotype_skills", CallingConvention = CallingConvention.Cdecl)]
    private static extern int skill_registry_register(IntPtr handle, string manifest_path);

    [DllImport("phenotype_skills", CallingConvention = CallingConvention.Cdecl)]
    private static extern int skill_registry_unregister(IntPtr handle, string skill_id);

    [DllImport("phenotype_skills", CallingConvention = CallingConvention.Cdecl)]
    private static extern int skill_registry_list(IntPtr handle, IntPtr[] skill_ids, int capacity);

    public SkillRegistry()
    {
        _handle = skill_registry_new();
        if (_handle == IntPtr.Zero)
        {
            throw new InvalidOperationException("Failed to create skill registry");
        }
    }

    /// <summary>
    /// Register a skill from a manifest file
    /// </summary>
    public void Register(string manifestPath)
    {
        ThrowIfDisposed();
        int result = skill_registry_register(_handle, manifestPath);
        if (result != 0)
        {
            throw new InvalidOperationException($"Failed to register skill: {result}");
        }
    }

    /// <summary>
    /// Unregister a skill by ID
    /// </summary>
    public void Unregister(string skillId)
    {
        ThrowIfDisposed();
        int result = skill_registry_unregister(_handle, skillId);
        if (result != 0)
        {
            throw new InvalidOperationException($"Failed to unregister skill: {result}");
        }
    }

    /// <summary>
    /// List all registered skills
    /// </summary>
    public IReadOnlyList<string> List()
    {
        ThrowIfDisposed();
        var skillIds = new IntPtr[100];
        int count = skill_registry_list(_handle, skillIds, skillIds.Length);
        
        var result = new List<string>();
        for (int i = 0; i < count; i++)
        {
            result.Add(Marshal.PtrToStringAnsi(skillIds[i]) ?? string.Empty);
        }
        
        return result.AsReadOnly();
    }

    private void ThrowIfDisposed()
    {
        if (_disposed)
        {
            throw new ObjectDisposedException(nameof(SkillRegistry));
        }
    }

    public void Dispose()
    {
        if (!_disposed)
        {
            if (_handle != IntPtr.Zero)
            {
                skill_registry_free(_handle);
                _handle = IntPtr.Zero;
            }
            _disposed = true;
        }
        GC.SuppressFinalize(this);
    }

    ~SkillRegistry()
    {
        Dispose();
    }
}

/// <summary>
/// Skill manifest definition
/// </summary>
public class SkillManifest
{
    public string Name { get; set; } = string.Empty;
    public string Version { get; set; } = "0.1.0";
    public string Description { get; set; } = string.Empty;
    public string Author { get; set; } = string.Empty;
    public string License { get; set; } = "MIT";
    public string EntryPoint { get; set; } = string.Empty;
    public List<SkillDependency> Dependencies { get; set; } = new();
    public Dictionary<string, string> Metadata { get; set; } = new();
}

/// <summary>
/// Skill dependency definition
/// </summary>
public class SkillDependency
{
    public string Name { get; set; } = string.Empty;
    public string VersionRequirement { get; set; } = string.Empty;
    public bool Optional { get; set; } = false;
}
