use phenotype_validation::validators::{is_valid_email, is_valid_url, is_valid_uuid};

// ============================================================================
// Email Validation Tests
// ============================================================================

#[test]
fn test_email_valid_standard_format() {
    assert!(is_valid_email("user@example.com"));
}

#[test]
fn test_email_valid_with_numbers() {
    assert!(is_valid_email("user123@example456.com"));
}

#[test]
fn test_email_valid_with_plus_addressing() {
    assert!(is_valid_email("user+tag@example.com"));
}

#[test]
fn test_email_valid_with_multiple_dots() {
    assert!(is_valid_email("first.last.name@example.co.uk"));
}

#[test]
fn test_email_valid_with_hyphen() {
    assert!(is_valid_email("user-name@example-domain.com"));
}

#[test]
fn test_email_valid_with_underscore() {
    assert!(is_valid_email("user_name@example.com"));
}

#[test]
fn test_email_valid_with_percent() {
    assert!(is_valid_email("user%test@example.com"));
}

#[test]
fn test_email_valid_short_tld() {
    assert!(is_valid_email("user@example.co"));
}

#[test]
fn test_email_valid_long_tld() {
    assert!(is_valid_email("user@example.museum"));
}

#[test]
fn test_email_valid_numeric_domain() {
    assert!(is_valid_email("user@123.com"));
}

#[test]
fn test_email_invalid_no_at_sign() {
    assert!(!is_valid_email("userexample.com"));
}

#[test]
fn test_email_invalid_no_local_part() {
    assert!(!is_valid_email("@example.com"));
}

#[test]
fn test_email_invalid_no_domain() {
    assert!(!is_valid_email("user@"));
}

#[test]
fn test_email_invalid_no_tld() {
    assert!(!is_valid_email("user@example"));
}

#[test]
fn test_email_invalid_single_char_tld() {
    assert!(!is_valid_email("user@example.c"));
}

#[test]
fn test_email_invalid_spaces() {
    assert!(!is_valid_email("user @example.com"));
    assert!(!is_valid_email("user@ example.com"));
}

#[test]
fn test_email_invalid_multiple_at_signs() {
    assert!(!is_valid_email("user@@example.com"));
    assert!(!is_valid_email("user@exam@ple.com"));
}

#[test]
fn test_email_invalid_empty_string() {
    assert!(!is_valid_email(""));
}

#[test]
fn test_email_invalid_only_at_sign() {
    assert!(!is_valid_email("@"));
}

// ============================================================================
// URL Validation Tests
// ============================================================================

#[test]
fn test_url_valid_https_standard() {
    assert!(is_valid_url("https://example.com"));
}

#[test]
fn test_url_valid_http_standard() {
    assert!(is_valid_url("http://example.com"));
}

#[test]
fn test_url_valid_https_with_port() {
    assert!(is_valid_url("https://localhost:8080"));
}

#[test]
fn test_url_valid_http_with_port() {
    assert!(is_valid_url("http://example.com:3000"));
}

#[test]
fn test_url_valid_with_path() {
    assert!(is_valid_url("https://example.com/path/to/resource"));
}

#[test]
fn test_url_valid_with_query_parameters() {
    assert!(is_valid_url("https://example.com/search?q=test&limit=10"));
}

#[test]
fn test_url_valid_with_fragment() {
    assert!(is_valid_url("https://example.com#section"));
}

#[test]
fn test_url_valid_with_path_and_query() {
    assert!(is_valid_url("https://example.com/api/v1/users?id=123"));
}

#[test]
fn test_url_valid_with_authentication() {
    assert!(is_valid_url("https://user:pass@example.com"));
}

#[test]
fn test_url_valid_https_uppercase() {
    assert!(is_valid_url("HTTPS://example.com"));
}

#[test]
fn test_url_valid_http_uppercase() {
    assert!(is_valid_url("HTTP://example.com"));
}

#[test]
fn test_url_valid_mixed_case() {
    assert!(is_valid_url("HtTpS://example.com"));
}

#[test]
fn test_url_valid_localhost() {
    assert!(is_valid_url("http://localhost"));
}

#[test]
fn test_url_valid_subdomain() {
    assert!(is_valid_url("https://api.example.com"));
}

#[test]
fn test_url_invalid_no_scheme() {
    assert!(!is_valid_url("example.com"));
}

#[test]
fn test_url_invalid_wrong_scheme_ftp() {
    assert!(!is_valid_url("ftp://example.com"));
}

#[test]
fn test_url_invalid_only_scheme() {
    assert!(!is_valid_url("https://"));
}

#[test]
fn test_url_invalid_http_no_host() {
    assert!(!is_valid_url("http://"));
}

#[test]
fn test_url_invalid_empty_string() {
    assert!(!is_valid_url(""));
}

// ============================================================================
// UUID Validation Tests
// ============================================================================

#[test]
fn test_uuid_valid_hyphenated_standard() {
    assert!(is_valid_uuid("550e8400-e29b-41d4-a716-446655440000"));
}

#[test]
fn test_uuid_valid_v1() {
    assert!(is_valid_uuid("6ba7b810-9dad-11d1-80b4-00c04fd430c8"));
}

#[test]
fn test_uuid_valid_v4() {
    assert!(is_valid_uuid("f47ac10b-58cc-4372-a567-0e02b2c3d479"));
}

#[test]
fn test_uuid_valid_simple_format() {
    assert!(is_valid_uuid("550e8400e29b41d4a716446655440000"));
}

#[test]
fn test_uuid_valid_uppercase() {
    assert!(is_valid_uuid("550E8400-E29B-41D4-A716-446655440000"));
}

#[test]
fn test_uuid_valid_lowercase() {
    assert!(is_valid_uuid("550e8400-e29b-41d4-a716-446655440000"));
}

#[test]
fn test_uuid_valid_mixed_case() {
    assert!(is_valid_uuid("550E8400-e29b-41d4-A716-446655440000"));
}

#[test]
fn test_uuid_valid_simple_uppercase() {
    assert!(is_valid_uuid("550E8400E29B41D4A716446655440000"));
}

#[test]
fn test_uuid_valid_nil_uuid() {
    assert!(is_valid_uuid("00000000-0000-0000-0000-000000000000"));
}

#[test]
fn test_uuid_valid_max_uuid() {
    assert!(is_valid_uuid("ffffffff-ffff-ffff-ffff-ffffffffffff"));
}

#[test]
fn test_uuid_valid_urn_format() {
    assert!(is_valid_uuid(
        "urn:uuid:550e8400-e29b-41d4-a716-446655440000"
    ));
}

#[test]
fn test_uuid_valid_braced_format() {
    assert!(is_valid_uuid("{550e8400-e29b-41d4-a716-446655440000}"));
}

#[test]
fn test_uuid_invalid_not_uuid_string() {
    assert!(!is_valid_uuid("not-a-uuid"));
}

#[test]
fn test_uuid_invalid_partial_hyphenated() {
    assert!(!is_valid_uuid("550e8400-e29b-41d4-a716"));
}

#[test]
fn test_uuid_invalid_extra_characters() {
    assert!(!is_valid_uuid("550e8400-e29b-41d4-a716-446655440000x"));
}

#[test]
fn test_uuid_invalid_malformed_hex() {
    assert!(!is_valid_uuid("550e8400-e29b-41d4-a716-44665544000g"));
}

#[test]
fn test_uuid_invalid_wrong_format() {
    assert!(!is_valid_uuid("550e8400_e29b_41d4_a716_446655440000"));
}

#[test]
fn test_uuid_invalid_empty_string() {
    assert!(!is_valid_uuid(""));
}

#[test]
fn test_uuid_invalid_only_hyphens() {
    assert!(!is_valid_uuid("----"));
}

#[test]
fn test_uuid_invalid_spaces() {
    assert!(!is_valid_uuid("550e8400-e29b-41d4-a716-446655440000 "));
    assert!(!is_valid_uuid(" 550e8400-e29b-41d4-a716-446655440000"));
}

// ============================================================================
// Cross-Validator Tests (Integration)
// ============================================================================

#[test]
fn test_validators_dont_cross_contaminate() {
    // Email should not validate as UUID
    assert!(!is_valid_uuid("user@example.com"));
    // UUID should not validate as email
    assert!(!is_valid_email("550e8400-e29b-41d4-a716-446655440000"));
    // URL should not validate as UUID
    assert!(!is_valid_uuid("https://example.com"));
}

#[test]
fn test_multiple_valid_emails() {
    let valid_emails = vec![
        "test@example.com",
        "user+tag@domain.co.uk",
        "firstname.lastname@company.net",
    ];
    for email in valid_emails {
        assert!(is_valid_email(email), "Email {} should be valid", email);
    }
}

#[test]
fn test_multiple_valid_urls() {
    let valid_urls = vec![
        "https://example.com",
        "http://localhost:8080",
        "https://api.example.com/v1/users?id=123",
    ];
    for url in valid_urls {
        assert!(is_valid_url(url), "URL {} should be valid", url);
    }
}

#[test]
fn test_multiple_valid_uuids() {
    let valid_uuids = vec![
        "550e8400-e29b-41d4-a716-446655440000",
        "550e8400e29b41d4a716446655440000",
        "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
        "00000000-0000-0000-0000-000000000000",
    ];
    for uuid in valid_uuids {
        assert!(is_valid_uuid(uuid), "UUID {} should be valid", uuid);
    }
}
