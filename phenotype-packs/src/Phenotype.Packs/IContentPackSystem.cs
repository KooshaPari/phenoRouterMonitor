using System;
using System.Collections.Generic;

namespace Phenotype.Packs
{
    /// <summary>
    /// Core interface for content pack systems. Generic over manifest and definition types
    /// to support any domain (game mods, configuration packs, asset bundles, etc.)
    /// </summary>
    /// <typeparam name="TManifest">The pack manifest type containing metadata and dependencies</typeparam>
    /// <typeparam name="TDefinition">The content definition type within packs</typeparam>
    public interface IContentPackSystem<TManifest, TDefinition>
        where TManifest : IPackManifest
        where TDefinition : IContentDefinition
    {
        /// <summary>
        /// Loads a pack from the specified path, resolving dependencies and validating content.
        /// </summary>
        PackLoadResult<TManifest, TDefinition> LoadPack(string path);

        /// <summary>
        /// Loads multiple packs with dependency resolution and conflict detection.
        /// </summary>
        BatchLoadResult<TManifest, TDefinition> LoadPacks(IEnumerable<string> paths);

        /// <summary>
        /// Resolves dependencies for a set of packs, producing a load order.
        /// </summary>
        DependencyGraph<TManifest> ResolveDependencies(IEnumerable<TManifest> packs);

        /// <summary>
        /// Detects conflicts between loaded definitions (duplicate IDs, version mismatches, etc.)
        /// </summary>
        ConflictReport<TDefinition> DetectConflicts(IEnumerable<TDefinition> definitions);

        /// <summary>
        /// Validates a pack manifest against the system's schema requirements.
        /// </summary>
        ValidationResult ValidateManifest(TManifest manifest);

        /// <summary>
        /// Unloads a pack and all its loaded definitions.
        /// </summary>
        bool UnloadPack(string packId);

        /// <summary>
        /// Gets all currently loaded packs.
        /// </summary>
        IReadOnlyList<LoadedPack<TManifest, TDefinition>> GetLoadedPacks();

        /// <summary>
        /// Event raised when a pack is successfully loaded.
        /// </summary>
        event EventHandler<PackLoadedEventArgs<TManifest, TDefinition>>? PackLoaded;

        /// <summary>
        /// Event raised when a pack fails to load.
        /// </summary>
        event EventHandler<PackLoadFailedEventArgs<TManifest>>? PackLoadFailed;
    }

    /// <summary>
    /// Represents a pack manifest containing metadata and dependencies.
    /// </summary>
    public interface IPackManifest
    {
        /// <summary>
        /// Unique identifier for the pack.
        /// </summary>
        string Id { get; }

        /// <summary>
        /// Human-readable name.
        /// </summary>
        string Name { get; }

        /// <summary>
        /// Semantic version string.
        /// </summary>
        string Version { get; }

        /// <summary>
        /// Framework version constraint (e.g., ">=1.0.0").
        /// </summary>
        string FrameworkVersion { get; }

        /// <summary>
        /// Pack author or organization.
        /// </summary>
        string Author { get; }

        /// <summary>
        /// Pack type/classification.
        /// </summary>
        string Type { get; }

        /// <summary>
        /// Optional description.
        /// </summary>
        string? Description { get; }

        /// <summary>
        /// Dependencies required by this pack.
        /// </summary>
        IReadOnlyList<IPackDependency> Dependencies { get; }

        /// <summary>
        /// Conflicting packs that cannot be loaded simultaneously.
        /// </summary>
        IReadOnlyList<string> Conflicts { get; }
    }

    /// <summary>
    /// Represents a content definition within a pack.
    /// </summary>
    public interface IContentDefinition
    {
        /// <summary>
        /// Unique identifier within the pack namespace.
        /// </summary>
        string Id { get; }

        /// <summary>
        /// Definition type/category.
        /// </summary>
        string DefinitionType { get; }

        /// <summary>
        /// Source pack identifier.
        /// </summary>
        string SourcePackId { get; }

        /// <summary>
        /// Whether this definition overrides a base definition.
        /// </summary>
        bool IsOverride { get; }
    }

    /// <summary>
    /// Represents a dependency on another pack.
    /// </summary>
    public interface IPackDependency
    {
        /// <summary>
        /// Pack ID being depended upon.
        /// </summary>
        string PackId { get; }

        /// <summary>
        /// Version constraint (semver range).
        /// </summary>
        string VersionConstraint { get; }

        /// <summary>
        /// Whether this is an optional dependency.
        /// </summary>
        bool IsOptional { get; }

        /// <summary>
        /// Checks if a version satisfies this dependency's constraint.
        /// </summary>
        bool IsSatisfiedBy(string version);
    }

    /// <summary>
    /// Result of loading a single pack.
    /// </summary>
    public class PackLoadResult<TManifest, TDefinition>
        where TManifest : IPackManifest
        where TDefinition : IContentDefinition
    {
        public bool Success { get; init; }
        public TManifest? Manifest { get; init; }
        public IReadOnlyList<TDefinition> Definitions { get; init; } = Array.Empty<TDefinition>();
        public IReadOnlyList<string> Errors { get; init; } = Array.Empty<string>();
        public IReadOnlyList<string> Warnings { get; init; } = Array.Empty<string>();
        public TimeSpan LoadDuration { get; init; }
    }

    /// <summary>
    /// Result of loading multiple packs in a batch.
    /// </summary>
    public class BatchLoadResult<TManifest, TDefinition>
        where TManifest : IPackManifest
        where TDefinition : IContentDefinition
    {
        public IReadOnlyList<LoadedPack<TManifest, TDefinition>> LoadedPacks { get; init; } = Array.Empty<LoadedPack<TManifest, TDefinition>>();
        public IReadOnlyList<PackLoadResult<TManifest, TDefinition>> FailedPacks { get; init; } = Array.Empty<PackLoadResult<TManifest, TDefinition>>();
        public DependencyGraph<TManifest>? DependencyGraph { get; init; }
        public ConflictReport<TDefinition>? ConflictReport { get; init; }
        public TimeSpan TotalDuration { get; init; }
    }

    /// <summary>
    /// Represents a successfully loaded pack.
    /// </summary>
    public class LoadedPack<TManifest, TDefinition>
        where TManifest : IPackManifest
        where TDefinition : IContentDefinition
    {
        public string PackId => Manifest.Id;
        public TManifest Manifest { get; init; } = default!;
        public string SourcePath { get; init; } = string.Empty;
        public IReadOnlyList<TDefinition> Definitions { get; init; } = Array.Empty<TDefinition>();
        public DateTime LoadedAt { get; init; }
    }

    /// <summary>
    /// Dependency graph with topological sort for load ordering.
    /// </summary>
    public class DependencyGraph<TManifest> where TManifest : IPackManifest
    {
        public IReadOnlyList<TManifest> Packs { get; init; } = Array.Empty<TManifest>();
        public IReadOnlyList<string> LoadOrder { get; init; } = Array.Empty<string>();
        public IReadOnlyList<DependencyEdge> Edges { get; init; } = Array.Empty<DependencyEdge>();
        public IReadOnlyList<MissingDependency> MissingDependencies { get; init; } = Array.Empty<MissingDependency>();
        public IReadOnlyList<VersionConflict> VersionConflicts { get; init; } = Array.Empty<VersionConflict>();

        public class DependencyEdge
        {
            public string From { get; init; } = string.Empty;
            public string To { get; init; } = string.Empty;
            public bool IsOptional { get; init; }
        }

        public class MissingDependency
        {
            public string PackId { get; init; } = string.Empty;
            public string DependencyId { get; init; } = string.Empty;
            public string VersionConstraint { get; init; } = string.Empty;
        }

        public class VersionConflict
        {
            public string PackId { get; init; } = string.Empty;
            public string DependencyId { get; init; } = string.Empty;
            public string RequiredVersion { get; init; } = string.Empty;
            public string FoundVersion { get; init; } = string.Empty;
        }
    }

    /// <summary>
    /// Report of conflicts between loaded definitions.
    /// </summary>
    public class ConflictReport<TDefinition> where TDefinition : IContentDefinition
    {
        public IReadOnlyList<DuplicateIdConflict<TDefinition>> DuplicateIds { get; init; } = Array.Empty<DuplicateIdConflict<TDefinition>>();
        public IReadOnlyList<OverrideConflict<TDefinition>> OverrideConflicts { get; init; } = Array.Empty<OverrideConflict<TDefinition>>();
        public bool HasConflicts => DuplicateIds.Count > 0 || OverrideConflicts.Count > 0;
    }

    public class DuplicateIdConflict<TDefinition> where TDefinition : IContentDefinition
    {
        public string ConflictingId { get; init; } = string.Empty;
        public IReadOnlyList<TDefinition> Definitions { get; init; } = Array.Empty<TDefinition>();
    }

    public class OverrideConflict<TDefinition> where TDefinition : IContentDefinition
    {
        public string BaseId { get; init; } = string.Empty;
        public TDefinition BaseDefinition { get; init; } = default!;
        public IReadOnlyList<TDefinition> OverrideDefinitions { get; init; } = Array.Empty<TDefinition>();
    }

    /// <summary>
    /// Validation result for manifest or definition validation.
    /// </summary>
    public class ValidationResult
    {
        public bool IsValid { get; init; }
        public IReadOnlyList<ValidationError> Errors { get; init; } = Array.Empty<ValidationError>();
        public IReadOnlyList<ValidationWarning> Warnings { get; init; } = Array.Empty<ValidationWarning>();
    }

    public class ValidationError
    {
        public string Property { get; init; } = string.Empty;
        public string Message { get; init; } = string.Empty;
        public ErrorSeverity Severity { get; init; }
    }

    public class ValidationWarning
    {
        public string Property { get; init; } = string.Empty;
        public string Message { get; init; } = string.Empty;
    }

    public enum ErrorSeverity { Warning, Error, Critical }

    /// <summary>
    /// Event args for pack loaded event.
    /// </summary>
    public class PackLoadedEventArgs<TManifest, TDefinition> : EventArgs
        where TManifest : IPackManifest
        where TDefinition : IContentDefinition
    {
        public LoadedPack<TManifest, TDefinition> Pack { get; init; } = default!;
    }

    /// <summary>
    /// Event args for pack load failed event.
    /// </summary>
    public class PackLoadFailedEventArgs<TManifest> : EventArgs where TManifest : IPackManifest
    {
        public string PackPath { get; init; } = string.Empty;
        public TManifest? Manifest { get; init; }
        public IReadOnlyList<string> Errors { get; init; } = Array.Empty<string>();
    }
}
