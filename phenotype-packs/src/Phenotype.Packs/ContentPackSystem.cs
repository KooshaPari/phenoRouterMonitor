using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using Phenotype.Packs.Discovery;
using Phenotype.Packs.Loading;
using Phenotype.Packs.Resolution;
using Phenotype.Packs.Validation;

namespace Phenotype.Packs
{
    /// <summary>
    /// Default implementation of IContentPackSystem.
    /// Extracted and generalized from DINOForge SDK.
    /// </summary>
    public class ContentPackSystem<TManifest, TDefinition> : IContentPackSystem<TManifest, TDefinition>
        where TManifest : class, IPackManifest, new()
        where TDefinition : class, IContentDefinition
    {
        private readonly IPackDiscoveryService _discoveryService;
        private readonly IPackLoader<TManifest, TDefinition> _packLoader;
        private readonly IDependencyResolver _dependencyResolver;
        private readonly IConflictDetector<TDefinition> _conflictDetector;
        private readonly IManifestValidator<TManifest> _manifestValidator;
        private readonly Dictionary<string, LoadedPack<TManifest, TDefinition>> _loadedPacks;

        public event EventHandler<PackLoadedEventArgs<TManifest, TDefinition>>? PackLoaded;
        public event EventHandler<PackLoadFailedEventArgs<TManifest>>? PackLoadFailed;

        public ContentPackSystem(
            IPackDiscoveryService? discoveryService = null,
            IPackLoader<TManifest, TDefinition>? packLoader = null,
            IDependencyResolver? dependencyResolver = null,
            IConflictDetector<TDefinition>? conflictDetector = null,
            IManifestValidator<TManifest>? manifestValidator = null)
        {
            _discoveryService = discoveryService ?? new FileSystemPackDiscovery();
            _packLoader = packLoader ?? new YamlPackLoader<TManifest, TDefinition>();
            _dependencyResolver = dependencyResolver ?? new SemverDependencyResolver();
            _conflictDetector = conflictDetector ?? new DefaultConflictDetector<TDefinition>();
            _manifestValidator = manifestValidator ?? new DefaultManifestValidator<TManifest>();
            _loadedPacks = new Dictionary<string, LoadedPack<TManifest, TDefinition>>();
        }

        public PackLoadResult<TManifest, TDefinition> LoadPack(string path)
        {
            var stopwatch = System.Diagnostics.Stopwatch.StartNew();
            var errors = new List<string>();
            var warnings = new List<string>();

            try
            {
                // Validate path exists
                if (!Directory.Exists(path) && !File.Exists(path))
                {
                    errors.Add($"Pack path not found: {path}");
                    return CreateFailedResult(errors, stopwatch);
                }

                // Discover and load manifest
                var manifest = _discoveryService.DiscoverManifest(path);
                if (manifest == null)
                {
                    errors.Add($"No manifest found in: {path}");
                    return CreateFailedResult(errors, stopwatch);
                }

                // Validate manifest
                var validation = _manifestValidator.Validate(manifest);
                if (!validation.IsValid)
                {
                    errors.AddRange(validation.Errors.Select(e => $"[Manifest] {e.Property}: {e.Message}"));
                    if (validation.Errors.Any(e => e.Severity == ErrorSeverity.Critical))
                    {
                        return CreateFailedResult(errors, stopwatch);
                    }
                }
                warnings.AddRange(validation.Warnings.Select(w => $"[Manifest] {w.Property}: {w.Message}"));

                // Check for duplicate pack ID
                if (_loadedPacks.ContainsKey(manifest.Id))
                {
                    errors.Add($"Pack with ID '{manifest.Id}' is already loaded");
                    return CreateFailedResult(errors, stopwatch);
                }

                // Load definitions
                var definitions = _packLoader.LoadDefinitions(path, manifest).ToList();

                // Validate definitions
                foreach (var def in definitions)
                {
                    var defValidation = ValidateDefinition(def);
                    if (!defValidation.IsValid)
                    {
                        errors.AddRange(defValidation.Errors.Select(e => $"[Definition {def.Id}] {e.Property}: {e.Message}"));
                    }
                }

                // Create loaded pack
                var loadedPack = new LoadedPack<TManifest, TDefinition>
                {
                    Manifest = manifest,
                    SourcePath = path,
                    Definitions = definitions,
                    LoadedAt = DateTime.UtcNow
                };

                _loadedPacks[manifest.Id] = loadedPack;

                stopwatch.Stop();

                var result = new PackLoadResult<TManifest, TDefinition>
                {
                    Success = true,
                    Manifest = manifest,
                    Definitions = definitions,
                    Errors = errors.AsReadOnly(),
                    Warnings = warnings.AsReadOnly(),
                    LoadDuration = stopwatch.Elapsed
                };

                PackLoaded?.Invoke(this, new PackLoadedEventArgs<TManifest, TDefinition> { Pack = loadedPack });
                return result;
            }
            catch (Exception ex)
            {
                errors.Add($"Exception during pack load: {ex.Message}");
                return CreateFailedResult(errors, stopwatch);
            }
        }

        public BatchLoadResult<TManifest, TDefinition> LoadPacks(IEnumerable<string> paths)
        {
            var stopwatch = System.Diagnostics.Stopwatch.StartNew();
            var loadedPacks = new List<LoadedPack<TManifest, TDefinition>>();
            var failedPacks = new List<PackLoadResult<TManifest, TDefinition>>();

            // Load all packs individually first
            var manifests = new List<TManifest>();
            foreach (var path in paths)
            {
                var result = LoadPack(path);
                if (result.Success && result.Manifest != null)
                {
                    loadedPacks.Add(_loadedPacks[result.Manifest.Id]);
                    manifests.Add(result.Manifest);
                }
                else
                {
                    failedPacks.Add(result);
                }
            }

            // Resolve dependencies
            var dependencyGraph = _dependencyResolver.Resolve(manifests);

            // Detect conflicts
            var allDefinitions = loadedPacks.SelectMany(p => p.Definitions).ToList();
            var conflictReport = _conflictDetector.DetectConflicts(allDefinitions);

            stopwatch.Stop();

            return new BatchLoadResult<TManifest, TDefinition>
            {
                LoadedPacks = loadedPacks.AsReadOnly(),
                FailedPacks = failedPacks.AsReadOnly(),
                DependencyGraph = dependencyGraph,
                ConflictReport = conflictReport,
                TotalDuration = stopwatch.Elapsed
            };
        }

        public DependencyGraph<TManifest> ResolveDependencies(IEnumerable<TManifest> packs)
        {
            return _dependencyResolver.Resolve(packs);
        }

        public ConflictReport<TDefinition> DetectConflicts(IEnumerable<TDefinition> definitions)
        {
            return _conflictDetector.DetectConflicts(definitions);
        }

        public ValidationResult ValidateManifest(TManifest manifest)
        {
            return _manifestValidator.Validate(manifest);
        }

        public bool UnloadPack(string packId)
        {
            return _loadedPacks.Remove(packId);
        }

        public IReadOnlyList<LoadedPack<TManifest, TDefinition>> GetLoadedPacks()
        {
            return _loadedPacks.Values.ToList().AsReadOnly();
        }

        private PackLoadResult<TManifest, TDefinition> CreateFailedResult(List<string> errors, System.Diagnostics.Stopwatch stopwatch)
        {
            stopwatch.Stop();
            var result = new PackLoadResult<TManifest, TDefinition>
            {
                Success = false,
                Manifest = null,
                Definitions = Array.Empty<TDefinition>(),
                Errors = errors.AsReadOnly(),
                Warnings = Array.Empty<string>().AsReadOnly(),
                LoadDuration = stopwatch.Elapsed
            };

            PackLoadFailed?.Invoke(this, new PackLoadFailedEventArgs<TManifest>
            {
                Errors = errors.AsReadOnly()
            });

            return result;
        }

        private ValidationResult ValidateDefinition(TDefinition definition)
        {
            // Base implementation - subclasses can override
            return new ValidationResult { IsValid = true };
        }
    }
}
