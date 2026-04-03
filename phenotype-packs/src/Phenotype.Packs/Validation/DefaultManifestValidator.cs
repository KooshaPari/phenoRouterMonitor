using System.Collections.Generic;
using System.Linq;

namespace Phenotype.Packs.Validation
{
    /// <summary>
    /// Service for validating pack manifests.
    /// </summary>
    public interface IManifestValidator<TManifest> where TManifest : IPackManifest
    {
        ValidationResult Validate(TManifest manifest);
    }

    /// <summary>
    /// Default manifest validator with common validation rules.
    /// </summary>
    public class DefaultManifestValidator<TManifest> : IManifestValidator<TManifest> where TManifest : IPackManifest
    {
        public ValidationResult Validate(TManifest manifest)
        {
            var errors = new List<ValidationError>();
            var warnings = new List<ValidationWarning>();

            // Validate ID
            if (string.IsNullOrWhiteSpace(manifest.Id))
            {
                errors.Add(new ValidationError
                {
                    Property = nameof(manifest.Id),
                    Message = "Pack ID is required",
                    Severity = ErrorSeverity.Critical
                });
            }
            else if (!IsValidIdentifier(manifest.Id))
            {
                errors.Add(new ValidationError
                {
                    Property = nameof(manifest.Id),
                    Message = "Pack ID must contain only lowercase letters, numbers, and hyphens",
                    Severity = ErrorSeverity.Error
                });
            }

            // Validate Name
            if (string.IsNullOrWhiteSpace(manifest.Name))
            {
                warnings.Add(new ValidationWarning
                {
                    Property = nameof(manifest.Name),
                    Message = "Pack name is recommended"
                });
            }

            // Validate Version
            if (string.IsNullOrWhiteSpace(manifest.Version))
            {
                errors.Add(new ValidationError
                {
                    Property = nameof(manifest.Version),
                    Message = "Version is required",
                    Severity = ErrorSeverity.Critical
                });
            }
            else if (!IsValidSemver(manifest.Version))
            {
                warnings.Add(new ValidationWarning
                {
                    Property = nameof(manifest.Version),
                    Message = "Version should follow semantic versioning (e.g., 1.0.0)"
                });
            }

            // Validate Framework Version
            if (string.IsNullOrWhiteSpace(manifest.FrameworkVersion))
            {
                warnings.Add(new ValidationWarning
                {
                    Property = nameof(manifest.FrameworkVersion),
                    Message = "Framework version constraint is recommended"
                });
            }

            // Validate Author
            if (string.IsNullOrWhiteSpace(manifest.Author))
            {
                warnings.Add(new ValidationWarning
                {
                    Property = nameof(manifest.Author),
                    Message = "Author information is recommended"
                });
            }

            // Validate Type
            if (string.IsNullOrWhiteSpace(manifest.Type))
            {
                warnings.Add(new ValidationWarning
                {
                    Property = nameof(manifest.Type),
                    Message = "Pack type is recommended"
                });
            }

            // Validate Dependencies
            foreach (var dep in manifest.Dependencies)
            {
                if (string.IsNullOrWhiteSpace(dep.PackId))
                {
                    errors.Add(new ValidationError
                    {
                        Property = "Dependencies",
                        Message = "Dependency pack ID is required",
                        Severity = ErrorSeverity.Error
                    });
                }

                if (string.IsNullOrWhiteSpace(dep.VersionConstraint))
                {
                    errors.Add(new ValidationError
                    {
                        Property = "Dependencies",
                        Message = $"Version constraint is required for dependency '{dep.PackId}'",
                        Severity = ErrorSeverity.Error
                    });
                }
            }

            return new ValidationResult
            {
                IsValid = !errors.Any(e => e.Severity == ErrorSeverity.Critical || e.Severity == ErrorSeverity.Error),
                Errors = errors.AsReadOnly(),
                Warnings = warnings.AsReadOnly()
            };
        }

        private static bool IsValidIdentifier(string id)
        {
            if (string.IsNullOrEmpty(id))
                return false;

            return id.All(c => char.IsLower(c) || char.IsDigit(c) || c == '-' || c == '_');
        }

        private static bool IsValidSemver(string version)
        {
            // Basic semver check (major.minor.patch)
            var parts = version.Split('.');
            if (parts.Length < 2)
                return false;

            return parts.All(p => int.TryParse(p.Split('-')[0], out _));
        }
    }
}
