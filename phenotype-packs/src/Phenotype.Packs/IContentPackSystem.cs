using System;
using System.Collections.Generic;
using System.Threading;
using System.Threading.Tasks;

namespace Phenotype.Packs;

/// <summary>
/// Unique identifier for a content pack.
/// </summary>
public readonly record struct PackIdentifier(string Id, string Version)
{
    public override string ToString() => $"{Id}@{Version}";
}

/// <summary>
/// Represents a pack manifest with metadata and dependencies.
/// </summary>
public interface IPackManifest
{
    string Id { get; }
    string Name { get; }
    string Version { get; }
    string Author { get; }
    IReadOnlyList<IPackDependency> Dependencies { get; }
}

/// <summary>
/// Represents a dependency on another pack.
/// </summary>
public interface IPackDependency
{
    string PackId { get; }
    string VersionConstraint { get; }
    bool IsRequired { get; }
}

/// <summary>
/// Represents a pack definition (content, configuration, etc.).
/// </summary>
public interface IPackDefinition
{
    string DefinitionType { get; }
    string SourcePath { get; }
}

/// <summary>
/// Context for pack loading operations.
/// </summary>
public class PackContext<TManifest, TDefinition>
    where TManifest : IPackManifest, new()
    where TDefinition : IPackDefinition
{
    public required string PackPath { get; init; }
    public required TManifest Manifest { get; init; }
    public IReadOnlyList<TDefinition> Definitions { get; init; } = new List<TDefinition>();
    public Dictionary<string, object> Metadata { get; } = new();
}

/// <summary>
/// Result of a pack load operation.
/// </summary>
public readonly record struct PackLoadResult
{
    public required bool Success { get; init; }
    public required PackIdentifier PackId { get; init; }
    public IReadOnlyList<string> Errors { get; init; }
    public IReadOnlyList<string> Warnings { get; init; }
    public TimeSpan LoadDuration { get; init; }

    public PackLoadResult()
    {
        Errors = new List<string>();
        Warnings = new List<string>();
    }

    public static PackLoadResult SuccessResult(PackIdentifier id, TimeSpan duration) =>
        new() { Success = true, PackId = id, LoadDuration = duration };

    public static PackLoadResult FailureResult(PackIdentifier id, IEnumerable<string> errors) =>
        new() { Success = false, PackId = id, Errors = errors.ToList() };
}

/// <summary>
/// Result of dependency resolution.
/// </summary>
public readonly record struct ResolutionResult
{
    public required bool Success { get; init; }
    public required IReadOnlyList<PackIdentifier> ResolvedOrder { get; init; }
    public IReadOnlyList<DependencyConflict> Conflicts { get; init; }
    public IReadOnlyList<string> MissingDependencies { get; init; }
    public IReadOnlyList<IPackManifest> CircularDependencies { get; init; }

    public ResolutionResult()
    {
        Conflicts = new List<DependencyConflict>();
        MissingDependencies = new List<string>();
        CircularDependencies = new List<IPackManifest>();
    }
}

/// <summary>
/// Represents a dependency conflict between packs.
/// </summary>
public readonly record struct DependencyConflict
{
    public required string PackId { get; init; }
    public required string ConflictingPackId { get; init; }
    public required string Reason { get; init; }
}

/// <summary>
/// Core interface for content pack operations.
/// </summary>
/// <typeparam name="TManifest">The manifest type.</typeparam>
/// <typeparam name="TDefinition">The definition type.</typeparam>
public interface IContentPackSystem<TManifest, TDefinition>
    where TManifest : IPackManifest, new()
    where TDefinition : IPackDefinition
{
    /// <summary>
    /// Load a pack from the specified path.
    /// </summary>
    Task<PackLoadResult> LoadPackAsync(
        PackIdentifier id,
        PackContext<TManifest, TDefinition> context,
        CancellationToken cancellationToken = default);

    /// <summary>
    /// Unload a previously loaded pack.
    /// </summary>
    Task<bool> UnloadPackAsync(PackIdentifier id, CancellationToken cancellationToken = default);

    /// <summary>
    /// Resolve dependencies for a set of packs.
    /// </summary>
    Task<ResolutionResult> ResolveDependenciesAsync(
        IEnumerable<TManifest> manifests,
        CancellationToken cancellationToken = default);

    /// <summary>
    /// Get all currently loaded packs.
    /// </summary>
    IReadOnlyList<PackIdentifier> GetLoadedPacks();

    /// <summary>
    /// Check if a pack is loaded.
    /// </summary>
    bool IsPackLoaded(PackIdentifier id);

    /// <summary>
    /// Event raised when a pack is loaded.
    /// </summary>
    event EventHandler<PackLoadEventArgs<TManifest, TDefinition>>? PackLoaded;

    /// <summary>
    /// Event raised when a pack is unloaded.
    /// </summary>
    event EventHandler<PackUnloadEventArgs>? PackUnloaded;
}

/// <summary>
/// Event arguments for pack loading.
/// </summary>
public class PackLoadEventArgs<TManifest, TDefinition> : EventArgs
    where TManifest : IPackManifest
    where TDefinition : IPackDefinition
{
    public required PackIdentifier PackId { get; init; }
    public required TManifest Manifest { get; init; }
    public required IReadOnlyList<TDefinition> Definitions { get; init; }
}

/// <summary>
/// Event arguments for pack unloading.
/// </summary>
public class PackUnloadEventArgs : EventArgs
{
    public required PackIdentifier PackId { get; init; }
}
