using System;
using System.Collections.Concurrent;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;
using Phenotype.Packs.Discovery;
using Phenotype.Packs.Loading;
using Phenotype.Packs.Resolution;
using Phenotype.Packs.Validation;

namespace Phenotype.Packs;

/// <summary>
/// Production implementation of the content pack system.
/// </summary>
public class ContentPackSystem<TManifest, TDefinition> : IContentPackSystem<TManifest, TDefinition>
    where TManifest : IPackManifest, new()
    where TDefinition : IPackDefinition
{
    private readonly ConcurrentDictionary<PackIdentifier, LoadedPack<TManifest, TDefinition>> _loadedPacks = new();
    private readonly IPackDiscovery _discovery;
    private readonly IPackLoader<TManifest, TDefinition> _loader;
    private readonly IDependencyResolver _dependencyResolver;
    private readonly IManifestValidator<TManifest> _validator;
    private readonly IConflictDetector _conflictDetector;

    public ContentPackSystem(
        IPackDiscovery? discovery = null,
        IPackLoader<TManifest, TDefinition>? loader = null,
        IDependencyResolver? dependencyResolver = null,
        IManifestValidator<TManifest>? validator = null,
        IConflictDetector? conflictDetector = null)
    {
        _discovery = discovery ?? new FileSystemPackDiscovery();
        _loader = loader ?? new YamlPackLoader<TManifest, TDefinition>();
        _dependencyResolver = dependencyResolver ?? new SemverDependencyResolver();
        _validator = validator ?? new DefaultManifestValidator<TManifest>();
        _conflictDetector = conflictDetector ?? new DefaultConflictDetector();
    }

    /// <inheritdoc />
    public event EventHandler<PackLoadEventArgs<TManifest, TDefinition>>? PackLoaded;

    /// <inheritdoc />
    public event EventHandler<PackUnloadEventArgs>? PackUnloaded;

    /// <inheritdoc />
    public async Task<PackLoadResult> LoadPackAsync(
        PackIdentifier id,
        PackContext<TManifest, TDefinition> context,
        CancellationToken cancellationToken = default)
    {
        var stopwatch = Stopwatch.StartNew();
        var errors = new List<string>();
        var warnings = new List<string>();

        try
        {
            // Validate manifest
            var validationResult = await _validator.ValidateAsync(context.Manifest, cancellationToken);
            if (!validationResult.IsValid)
            {
                errors.AddRange(validationResult.Errors);
                return PackLoadResult.FailureResult(id, errors);
            }

            // Check for conflicts
            var conflicts = await _conflictDetector.DetectConflictsAsync(
                context.Manifest,
                _loadedPacks.Values.Select(p => p.Manifest),
                cancellationToken);

            if (conflicts.Any())
            {
                errors.AddRange(conflicts.Select(c => $"Conflict: {c.Reason}"));
                return PackLoadResult.FailureResult(id, errors);
            }

            // Load definitions if not already provided
            var definitions = context.Definitions.Any()
                ? context.Definitions
                : await _loader.LoadDefinitionsAsync(context, cancellationToken);

            // Store loaded pack
            var loadedPack = new LoadedPack<TManifest, TDefinition>
            {
                Id = id,
                Manifest = context.Manifest,
                Definitions = definitions.ToList(),
                LoadedAt = DateTime.UtcNow,
                SourcePath = context.PackPath
            };

            _loadedPacks[id] = loadedPack;

            stopwatch.Stop();

            // Raise event
            PackLoaded?.Invoke(this, new PackLoadEventArgs<TManifest, TDefinition>
            {
                PackId = id,
                Manifest = context.Manifest,
                Definitions = definitions
            });

            return new PackLoadResult
            {
                Success = true,
                PackId = id,
                Warnings = warnings,
                LoadDuration = stopwatch.Elapsed
            };
        }
        catch (Exception ex)
        {
            stopwatch.Stop();
            errors.Add($"Exception during load: {ex.Message}");
            return PackLoadResult.FailureResult(id, errors);
        }
    }

    /// <inheritdoc />
    public Task<bool> UnloadPackAsync(PackIdentifier id, CancellationToken cancellationToken = default)
    {
        if (_loadedPacks.TryRemove(id, out _))
        {
            PackUnloaded?.Invoke(this, new PackUnloadEventArgs { PackId = id });
            return Task.FromResult(true);
        }

        return Task.FromResult(false);
    }

    /// <inheritdoc />
    public Task<ResolutionResult> ResolveDependenciesAsync(
        IEnumerable<TManifest> manifests,
        CancellationToken cancellationToken = default)
    {
        return _dependencyResolver.ResolveAsync(manifests, cancellationToken);
    }

    /// <inheritdoc />
    public IReadOnlyList<PackIdentifier> GetLoadedPacks() => _loadedPacks.Keys.ToList();

    /// <inheritdoc />
    public bool IsPackLoaded(PackIdentifier id) => _loadedPacks.ContainsKey(id);

    /// <summary>
    /// Get a loaded pack by ID.
    /// </summary>
    public LoadedPack<TManifest, TDefinition>? GetPack(PackIdentifier id) =>
        _loadedPacks.TryGetValue(id, out var pack) ? pack : null;
}

/// <summary>
/// Represents a loaded pack in memory.
/// </summary>
public class LoadedPack<TManifest, TDefinition>
    where TManifest : IPackManifest
    where TDefinition : IPackDefinition
{
    public required PackIdentifier Id { get; init; }
    public required TManifest Manifest { get; init; }
    public required List<TDefinition> Definitions { get; init; }
    public required DateTime LoadedAt { get; init; }
    public required string SourcePath { get; init; }
}
