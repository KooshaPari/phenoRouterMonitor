using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;
using Semver;

namespace Phenotype.Packs.Resolution;

/// <summary>
/// Resolves dependencies between packs.
/// </summary>
public interface IDependencyResolver
{
    Task<ResolutionResult> ResolveAsync<TManifest>(
        IEnumerable<TManifest> manifests,
        CancellationToken cancellationToken = default)
        where TManifest : IPackManifest;
}

/// <summary>
/// Semantic versioning dependency resolver.
/// </summary>
public class SemverDependencyResolver : IDependencyResolver
{
    public Task<ResolutionResult> ResolveAsync<TManifest>(
        IEnumerable<TManifest> manifests,
        CancellationToken cancellationToken = default)
        where TManifest : IPackManifest
    {
        var manifestList = manifests.ToList();
        var resolved = new List<PackIdentifier>();
        var conflicts = new List<DependencyConflict>();
        var missing = new List<string>();
        var resolvedIds = new HashSet<string>();

        // Build dependency graph
        var graph = new Dictionary<string, List<string>>();
        var versions = new Dictionary<string, string>();

        foreach (var manifest in manifestList)
        {
            cancellationToken.ThrowIfCancellationRequested();

            graph[manifest.Id] = manifest.Dependencies.Select(d => d.PackId).ToList();
            versions[manifest.Id] = manifest.Version;
        }

        // Topological sort with dependency resolution
        var visited = new HashSet<string>();
        var temp = new HashSet<string>();

        bool Visit(string packId)
        {
            if (temp.Contains(packId))
            {
                conflicts.Add(new DependencyConflict
                {
                    PackId = packId,
                    ConflictingPackId = packId,
                    Reason = "Circular dependency detected"
                });
                return false;
            }

            if (visited.Contains(packId))
            {
                return true;
            }

            temp.Add(packId);

            // Check dependencies exist
            if (graph.TryGetValue(packId, out var deps))
            {
                foreach (var dep in deps)
                {
                    if (!graph.ContainsKey(dep))
                    {
                        // Check if dependency is optional
                        var manifest = manifestList.First(m => m.Id == packId);
                        var depInfo = manifest.Dependencies.First(d => d.PackId == dep);

                        if (depInfo.IsRequired)
                        {
                            missing.Add(dep);
                            return false;
                        }
                    }
                    else if (!Visit(dep))
                    {
                        return false;
                    }
                }
            }

            temp.Remove(packId);
            visited.Add(packId);

            if (resolvedIds.Add(packId))
            {
                resolved.Add(new PackIdentifier(packId, versions[packId]));
            }

            return true;
        }

        foreach (var manifest in manifestList)
        {
            cancellationToken.ThrowIfCancellationRequested();

            if (!Visit(manifest.Id))
            {
                // Failed to resolve
            }
        }

        // Check version constraints
        foreach (var manifest in manifestList)
        {
            foreach (var dep in manifest.Dependencies)
            {
                if (!versions.TryGetValue(dep.PackId, out var actualVersion))
                    continue;

                if (!SatisfiesConstraint(actualVersion, dep.VersionConstraint))
                {
                    conflicts.Add(new DependencyConflict
                    {
                        PackId = manifest.Id,
                        ConflictingPackId = dep.PackId,
                        Reason = $"Version constraint not satisfied: {dep.VersionConstraint} required, found {actualVersion}"
                    });
                }
            }
        }

        return Task.FromResult(new ResolutionResult
        {
            Success = conflicts.Count == 0 && missing.Count == 0,
            ResolvedOrder = resolved,
            Conflicts = conflicts,
            MissingDependencies = missing
        });
    }

    private static bool SatisfiesConstraint(string version, string constraint)
    {
        try
        {
            var v = SemVersion.Parse(version, SemVersionStyles.Any);
            var range = SemVersionRange.Parse(constraint);
            return range.Contains(v);
        }
        catch
        {
            return true; // Be lenient on parse errors
        }
    }
}
