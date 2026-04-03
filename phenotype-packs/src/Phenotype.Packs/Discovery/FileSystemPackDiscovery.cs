using System.Collections.Generic;
using System.Threading;
using System.Threading.Tasks;

namespace Phenotype.Packs.Discovery;

/// <summary>
/// Discovers packs from a source (filesystem, HTTP, etc.).
/// </summary>
public interface IPackDiscovery
{
    Task<IReadOnlyList<PackDiscoveryResult>> DiscoverPacksAsync(
        string searchPath,
        DiscoveryOptions? options = null,
        CancellationToken cancellationToken = default);
}

/// <summary>
/// Options for pack discovery.
/// </summary>
public class DiscoveryOptions
{
    public bool Recursive { get; set; } = true;
    public string ManifestFileName { get; set; } = "pack.yaml";
    public List<string> ExcludePatterns { get; set; } = new();
}

/// <summary>
/// Result of discovering a pack.
/// </summary>
public readonly record struct PackDiscoveryResult
{
    public required string ManifestPath { get; init; }
    public required string PackDirectory { get; init; }
    public required string PackId { get; init; }
    public string? Version { get; init; }
}

/// <summary>
/// File system implementation of pack discovery.
/// </summary>
public class FileSystemPackDiscovery : IPackDiscovery
{
    public Task<IReadOnlyList<PackDiscoveryResult>> DiscoverPacksAsync(
        string searchPath,
        DiscoveryOptions? options = null,
        CancellationToken cancellationToken = default)
    {
        options ??= new DiscoveryOptions();
        var results = new List<PackDiscoveryResult>();

        if (!Directory.Exists(searchPath))
        {
            return Task.FromResult<IReadOnlyList<PackDiscoveryResult>>(results);
        }

        var searchOption = options.Recursive ? SearchOption.AllDirectories : SearchOption.TopDirectoryOnly;

        foreach (var manifestPath in Directory.GetFiles(searchPath, options.ManifestFileName, searchOption))
        {
            cancellationToken.ThrowIfCancellationRequested();

            var packDir = Path.GetDirectoryName(manifestPath) ?? "";
            var packId = Path.GetFileName(packDir);

            // Skip excluded patterns
            if (options.ExcludePatterns.Any(pattern =>
                packDir.Contains(pattern, StringComparison.OrdinalIgnoreCase)))
            {
                continue;
            }

            results.Add(new PackDiscoveryResult
            {
                ManifestPath = manifestPath,
                PackDirectory = packDir,
                PackId = packId
            });
        }

        return Task.FromResult<IReadOnlyList<PackDiscoveryResult>>(results);
    }
}
