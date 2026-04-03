using System.Collections.Generic;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;

namespace Phenotype.Packs;

/// <summary>
/// Detects conflicts between packs.
/// </summary>
public interface IConflictDetector
{
    Task<IReadOnlyList<PackConflict>> DetectConflictsAsync<TManifest>(
        TManifest newPack,
        IEnumerable<TManifest> existingPacks,
        CancellationToken cancellationToken = default)
        where TManifest : IPackManifest;
}

/// <summary>
/// Represents a conflict between packs.
/// </summary>
public readonly record struct PackConflict
{
    public required string PackId { get; init; }
    public required string OtherPackId { get; init; }
    public required string Reason { get; init; }
    public ConflictSeverity Severity { get; init; }
}

/// <summary>
/// Severity of a conflict.
/// </summary>
public enum ConflictSeverity
{
    Warning,
    Error
}

/// <summary>
/// Default conflict detector implementation.
/// </summary>
public class DefaultConflictDetector : IConflictDetector
{
    public Task<IReadOnlyList<PackConflict>> DetectConflictsAsync<TManifest>(
        TManifest newPack,
        IEnumerable<TManifest> existingPacks,
        CancellationToken cancellationToken = default)
        where TManifest : IPackManifest
    {
        var conflicts = new List<PackConflict>();
        var existingList = existingPacks.ToList();

        // Check for ID collision
        var idCollision = existingList.FirstOrDefault(p => p.Id == newPack.Id);
        if (idCollision != null)
        {
            conflicts.Add(new PackConflict
            {
                PackId = newPack.Id,
                OtherPackId = idCollision.Id,
                Reason = $"Pack with ID '{newPack.Id}' is already loaded",
                Severity = ConflictSeverity.Error
            });
        }

        // Check for name collision (warning only)
        var nameCollision = existingList.FirstOrDefault(p =>
            p.Name.Equals(newPack.Name, StringComparison.OrdinalIgnoreCase));
        if (nameCollision != null && idCollision == null)
        {
            conflicts.Add(new PackConflict
            {
                PackId = newPack.Id,
                OtherPackId = nameCollision.Id,
                Reason = $"Pack with similar name '{newPack.Name}' already exists",
                Severity = ConflictSeverity.Warning
            });
        }

        return Task.FromResult<IReadOnlyList<PackConflict>>(conflicts);
    }
}
