using System.Collections.Generic;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;

namespace Phenotype.Packs.Validation;

/// <summary>
/// Validates pack manifests.
/// </summary>
public interface IManifestValidator<TManifest>
    where TManifest : IPackManifest
{
    Task<ValidationResult> ValidateAsync(TManifest manifest, CancellationToken cancellationToken = default);
}

/// <summary>
/// Result of validation.
/// </summary>
public readonly record struct ValidationResult
{
    public required bool IsValid { get; init; }
    public IReadOnlyList<string> Errors { get; init; }
    public IReadOnlyList<string> Warnings { get; init; }

    public ValidationResult()
    {
        Errors = new List<string>();
        Warnings = new List<string>();
    }

    public static ValidationResult Success() => new() { IsValid = true };
    public static ValidationResult Failure(IEnumerable<string> errors) =>
        new() { IsValid = false, Errors = errors.ToList() };
}

/// <summary>
/// Default manifest validator with basic rules.
/// </summary>
public class DefaultManifestValidator<TManifest> : IManifestValidator<TManifest>
    where TManifest : IPackManifest
{
    public Task<ValidationResult> ValidateAsync(TManifest manifest, CancellationToken cancellationToken = default)
    {
        var errors = new List<string>();
        var warnings = new List<string>();

        // Required fields
        if (string.IsNullOrWhiteSpace(manifest.Id))
        {
            errors.Add("Pack ID is required");
        }
        else if (!IsValidId(manifest.Id))
        {
            errors.Add("Pack ID must be lowercase alphanumeric with hyphens/underscores only");
        }

        if (string.IsNullOrWhiteSpace(manifest.Name))
        {
            errors.Add("Pack name is required");
        }

        if (string.IsNullOrWhiteSpace(manifest.Version))
        {
            errors.Add("Pack version is required");
        }
        else if (!IsValidVersion(manifest.Version))
        {
            warnings.Add("Version should follow semantic versioning (e.g., 1.0.0)");
        }

        if (string.IsNullOrWhiteSpace(manifest.Author))
        {
            warnings.Add("Pack author is recommended");
        }

        // Check for duplicate dependencies
        var duplicateDeps = manifest.Dependencies
            .GroupBy(d => d.PackId)
            .Where(g => g.Count() > 1)
            .Select(g => g.Key)
            .ToList();

        foreach (var dup in duplicateDeps)
        {
            errors.Add($"Duplicate dependency on pack: {dup}");
        }

        return Task.FromResult(errors.Any()
            ? ValidationResult.Failure(errors)
            : new ValidationResult { IsValid = true, Warnings = warnings });
    }

    private static bool IsValidId(string id)
    {
        return id.All(c => char.IsLower(c) || char.IsDigit(c) || c == '-' || c == '_');
    }

    private static bool IsValidVersion(string version)
    {
        // Basic semver check
        var parts = version.Split('.');
        return parts.Length >= 2 && parts.All(p => int.TryParse(p.Split('-')[0], out _));
    }
}
