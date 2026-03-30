use regex::Regex;
use uuid::Uuid;

/// Validates an email address using a regex pattern.
///
/// Checks for:
/// - Alphanumeric characters, dots, underscores, percent signs, plus signs, hyphens before @
/// - Valid domain with alphanumeric characters, dots, hyphens
/// - At least 2-character TLD
///
/// # Examples
///
/// ```
/// use phenotype_validation::validators::is_valid_email;
///
/// assert!(is_valid_email("user@example.com"));
/// assert!(!is_valid_email("invalid.email"));
/// ```
pub fn is_valid_email(email: &str) -> bool {
    let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    email_regex.is_match(email)
}

/// Validates a URL by checking for http:// or https:// protocol.
///
/// This is a practical validator that ensures URLs have:
/// - Valid scheme (http or https)
/// - Non-empty host after the scheme
///
/// # Examples
///
/// ```
/// use phenotype_validation::validators::is_valid_url;
///
/// assert!(is_valid_url("https://example.com"));
/// assert!(is_valid_url("http://localhost:8080"));
/// assert!(!is_valid_url("ftp://example.com"));
/// ```
pub fn is_valid_url(url: &str) -> bool {
    let url_lower = url.to_lowercase();

    // Check if URL starts with http:// or https://
    if !url_lower.starts_with("http://") && !url_lower.starts_with("https://") {
        return false;
    }

    // Extract the part after the scheme
    let after_scheme = if url_lower.starts_with("https://") {
        &url[8..]
    } else {
        &url[7..]
    };

    // Ensure there's a non-empty host
    !after_scheme.trim().is_empty()
}

/// Validates a UUID string by attempting to parse it.
///
/// Accepts UUIDs in standard formats:
/// - Hyphenated: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
/// - Simple: xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
/// - URN: urn:uuid:xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
///
/// # Examples
///
/// ```
/// use phenotype_validation::validators::is_valid_uuid;
///
/// assert!(is_valid_uuid("550e8400-e29b-41d4-a716-446655440000"));
/// assert!(is_valid_uuid("550e8400e29b41d4a716446655440000"));
/// assert!(!is_valid_uuid("not-a-uuid"));
/// ```
pub fn is_valid_uuid(id: &str) -> bool {
    Uuid::parse_str(id).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Email validation tests
    #[test]
    fn test_is_valid_email_standard() {
        assert!(is_valid_email("user@example.com"));
    }

    #[test]
    fn test_is_valid_email_with_plus() {
        assert!(is_valid_email("user+tag@example.com"));
    }

    #[test]
    fn test_is_valid_email_with_dot() {
        assert!(is_valid_email("first.last@example.com"));
    }

    #[test]
    fn test_is_valid_email_with_underscore() {
        assert!(is_valid_email("user_name@example.com"));
    }

    #[test]
    fn test_is_valid_email_subdomain() {
        assert!(is_valid_email("user@mail.example.com"));
    }

    #[test]
    fn test_is_valid_email_numeric_domain() {
        assert!(is_valid_email("user@123.456.com"));
    }

    #[test]
    fn test_is_invalid_email_no_at() {
        assert!(!is_valid_email("userexample.com"));
    }

    #[test]
    fn test_is_invalid_email_no_domain() {
        assert!(!is_valid_email("user@"));
    }

    #[test]
    fn test_is_invalid_email_no_tld() {
        assert!(!is_valid_email("user@example"));
    }

    #[test]
    fn test_is_invalid_email_single_char_tld() {
        assert!(!is_valid_email("user@example.c"));
    }

    #[test]
    fn test_is_invalid_email_spaces() {
        assert!(!is_valid_email("user @example.com"));
    }

    #[test]
    fn test_is_invalid_email_empty() {
        assert!(!is_valid_email(""));
    }

    // URL validation tests
    #[test]
    fn test_is_valid_url_https() {
        assert!(is_valid_url("https://example.com"));
    }

    #[test]
    fn test_is_valid_url_http() {
        assert!(is_valid_url("http://example.com"));
    }

    #[test]
    fn test_is_valid_url_with_port() {
        assert!(is_valid_url("https://localhost:8080"));
    }

    #[test]
    fn test_is_valid_url_with_path() {
        assert!(is_valid_url("https://example.com/path/to/resource"));
    }

    #[test]
    fn test_is_valid_url_with_query() {
        assert!(is_valid_url("https://example.com/search?q=test"));
    }

    #[test]
    fn test_is_valid_url_with_fragment() {
        assert!(is_valid_url("https://example.com#section"));
    }

    #[test]
    fn test_is_valid_url_case_insensitive() {
        assert!(is_valid_url("HTTPS://example.com"));
        assert!(is_valid_url("Http://example.com"));
    }

    #[test]
    fn test_is_invalid_url_no_scheme() {
        assert!(!is_valid_url("example.com"));
    }

    #[test]
    fn test_is_invalid_url_ftp() {
        assert!(!is_valid_url("ftp://example.com"));
    }

    #[test]
    fn test_is_invalid_url_only_scheme() {
        assert!(!is_valid_url("https://"));
    }

    #[test]
    fn test_is_invalid_url_empty() {
        assert!(!is_valid_url(""));
    }

    // UUID validation tests
    #[test]
    fn test_is_valid_uuid_hyphenated() {
        assert!(is_valid_uuid("550e8400-e29b-41d4-a716-446655440000"));
    }

    #[test]
    fn test_is_valid_uuid_simple() {
        assert!(is_valid_uuid("550e8400e29b41d4a716446655440000"));
    }

    #[test]
    fn test_is_valid_uuid_uppercase() {
        assert!(is_valid_uuid("550E8400-E29B-41D4-A716-446655440000"));
    }

    #[test]
    fn test_is_valid_uuid_urn() {
        assert!(is_valid_uuid(
            "urn:uuid:550e8400-e29b-41d4-a716-446655440000"
        ));
    }

    #[test]
    fn test_is_valid_uuid_v4() {
        assert!(is_valid_uuid("6ba7b810-9dad-11d1-80b4-00c04fd430c8"));
    }

    #[test]
    fn test_is_invalid_uuid_not_uuid() {
        assert!(!is_valid_uuid("not-a-uuid"));
    }

    #[test]
    fn test_is_invalid_uuid_partial() {
        assert!(!is_valid_uuid("550e8400-e29b-41d4-a716"));
    }

    #[test]
    fn test_is_invalid_uuid_malformed() {
        assert!(!is_valid_uuid("550e8400-e29b-41d4-a716-44665544000g"));
    }

    #[test]
    fn test_is_invalid_uuid_empty() {
        assert!(!is_valid_uuid(""));
    }

    #[test]
    fn test_is_invalid_uuid_wrong_hyphens() {
        assert!(!is_valid_uuid("550e8400e29b-41d4-a716-446655440000"));
    }
}
