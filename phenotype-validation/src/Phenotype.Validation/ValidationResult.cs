using System.Collections.Generic;

namespace Phenotype.Validation;

/// <summary>
/// Immutable result of a schema validation operation.
/// Use <see cref="Success()"/> or <see cref="Failure(IReadOnlyList{ValidationError})"/> to construct.
/// </summary>
public sealed class ValidationResult
{
    /// <summary>Whether the validated content satisfied all rules.</summary>
    public bool IsValid { get; }

    /// <summary>
    /// Errors produced during validation. Empty when <see cref="IsValid"/> is <c>true</c>.
    /// </summary>
    public IReadOnlyList<ValidationError> Errors { get; }

    private ValidationResult(bool isValid, IReadOnlyList<ValidationError> errors)
    {
        IsValid = isValid;
        Errors = errors;
    }

    /// <summary>Creates a successful <see cref="ValidationResult"/> with no errors.</summary>
    public static ValidationResult Success()
        => new ValidationResult(true, new List<ValidationError>().AsReadOnly());

    /// <summary>
    /// Creates a failed <see cref="ValidationResult"/> carrying the supplied errors.
    /// </summary>
    /// <param name="errors">One or more errors that caused validation to fail.</param>
    public static ValidationResult Failure(IReadOnlyList<ValidationError> errors)
        => new ValidationResult(false, errors);

    /// <summary>
    /// Creates a failed <see cref="ValidationResult"/> with a single error.
    /// </summary>
    /// <param name="path">The path to the error.</param>
    /// <param name="message">The error message.</param>
    /// <param name="rule">The rule that was violated.</param>
    public static ValidationResult Failure(string path, string message, string rule)
        => Failure(new List<ValidationError> { new(path, message, rule) }.AsReadOnly());
}

/// <summary>
/// Describes a single validation rule violation.
/// </summary>
public sealed class ValidationError
{
    /// <summary>
    /// Dot-separated path to the offending field within the document (e.g. "loads.units[0]").
    /// </summary>
    public string Path { get; }

    /// <summary>Human-readable description of why the value is invalid.</summary>
    public string Message { get; }

    /// <summary>
    /// The validation rule that was violated (e.g. "required", "type", "pattern").
    /// </summary>
    public string Rule { get; }

    /// <summary>
    /// Creates a new <see cref="ValidationError"/>.
    /// </summary>
    /// <param name="path">The path to the error.</param>
    /// <param name="message">The error message.</param>
    /// <param name="rule">The rule that was violated.</param>
    public ValidationError(string path, string message, string rule)
    {
        Path = path ?? "";
        Message = message ?? "";
        Rule = rule ?? "unknown";
    }
}
