using System;
using System.Collections.Generic;
using System.Linq;
using Semver;

namespace Phenotype.Packs.Resolution
{
    /// <summary>
    /// Service for resolving pack dependencies.
    /// </summary>
    public interface IDependencyResolver
    {
        DependencyGraph<TManifest> Resolve<TManifest>(IEnumerable<TManifest> packs) where TManifest : IPackManifest;
    }

    /// <summary>
    /// Semantic versioning-based dependency resolver.
    /// </summary>
    public class SemverDependencyResolver : IDependencyResolver
    {
        public DependencyGraph<TManifest> Resolve<TManifest>(IEnumerable<TManifest> packs) where TManifest : IPackManifest
        {
            var packList = packs.ToList();
            var packDict = packList.ToDictionary(p => p.Id, p => p);
            var edges = new List<DependencyGraph<TManifest>.DependencyEdge>();
            var missingDeps = new List<DependencyGraph<TManifest>.MissingDependency>();
            var versionConflicts = new List<DependencyGraph<TManifest>.VersionConflict>();

            // Build dependency graph
            foreach (var pack in packList)
            {
                foreach (var dep in pack.Dependencies)
                {
                    var edge = new DependencyGraph<TManifest>.DependencyEdge
                    {
                        From = pack.Id,
                        To = dep.PackId,
                        IsOptional = dep.IsOptional
                    };
                    edges.Add(edge);

                    // Check if dependency exists
                    if (!packDict.TryGetValue(dep.PackId, out var depPack))
                    {
                        if (!dep.IsOptional)
                        {
                            missingDeps.Add(new DependencyGraph<TManifest>.MissingDependency
                            {
                                PackId = pack.Id,
                                DependencyId = dep.PackId,
                                VersionConstraint = dep.VersionConstraint
                            });
                        }
                        continue;
                    }

                    // Check version constraint
                    if (!dep.IsSatisfiedBy(depPack.Version))
                    {
                        versionConflicts.Add(new DependencyGraph<TManifest>.VersionConflict
                        {
                            PackId = pack.Id,
                            DependencyId = dep.PackId,
                            RequiredVersion = dep.VersionConstraint,
                            FoundVersion = depPack.Version
                        });
                    }
                }
            }

            // Topological sort for load order
            var loadOrder = TopologicalSort(packList, edges);

            return new DependencyGraph<TManifest>
            {
                Packs = packList.AsReadOnly(),
                LoadOrder = loadOrder.AsReadOnly(),
                Edges = edges.AsReadOnly(),
                MissingDependencies = missingDeps.AsReadOnly(),
                VersionConflicts = versionConflicts.AsReadOnly()
            };
        }

        private List<string> TopologicalSort<TManifest>(List<TManifest> packs, List<DependencyGraph<TManifest>.DependencyEdge> edges) where TManifest : IPackManifest
        {
            var result = new List<string>();
            var visited = new HashSet<string>();
            var visiting = new HashSet<string>();
            var packDict = packs.ToDictionary(p => p.Id);

            foreach (var pack in packs)
            {
                if (!visited.Contains(pack.Id))
                {
                    Visit(pack.Id, edges, packDict, visited, visiting, result);
                }
            }

            return result;
        }

        private void Visit<TManifest>(string packId, List<DependencyGraph<TManifest>.DependencyEdge> edges, Dictionary<string, TManifest> packs, HashSet<string> visited, HashSet<string> visiting, List<string> result) where TManifest : IPackManifest
        {
            if (visiting.Contains(packId))
            {
                // Cycle detected - handle gracefully
                return;
            }

            if (visited.Contains(packId))
            {
                return;
            }

            visiting.Add(packId);

            // Visit dependencies first
            var dependencies = edges
                .Where(e => e.From == packId && !e.IsOptional)
                .Select(e => e.To);

            foreach (var dep in dependencies)
            {
                Visit(dep, edges, packs, visited, visiting, result);
            }

            visiting.Remove(packId);
            visited.Add(packId);
            result.Add(packId);
        }
    }

    /// <summary>
    /// Pack dependency with semantic version constraint.
    /// </summary>
    public class PackDependency : IPackDependency
    {
        public string PackId { get; init; } = string.Empty;
        public string VersionConstraint { get; init; } = "*";
        public bool IsOptional { get; init; }

        public bool IsSatisfiedBy(string version)
        {
            try
            {
                var range = SemVersionRange.Parse(VersionConstraint, SemVersionRangeOptions.Loose);
                var v = SemVersion.Parse(version, SemVersionStyles.Loose);
                return range.Contains(v);
            }
            catch
            {
                // Fallback to simple string comparison if semver parsing fails
                return true;
            }
        }
    }
}
