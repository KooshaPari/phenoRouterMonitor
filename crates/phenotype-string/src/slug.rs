//! URL slugification and HTML entity encoding/decoding.

use once_cell::sync::Lazy;
use regex::Regex;

static SLUG_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"[^a-z0-9]+").expect("Invalid regex"));

static WHITESPACE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+").expect("Invalid regex"));

static ACCENTS: Lazy<Regex> = Lazy::new(|| {
    // Matches common accented characters
    Regex::new(r"[àáâãäåèéêëìíîïòóôõöùúûüñç]").expect("Invalid regex")
});

/// HTML entity mapping for encoding.
const HTML_ENTITIES: &[(&str, &str)] = &[
    ("&", "&amp;"),
    ("<", "&lt;"),
    (">", "&gt;"),
    ("\"", "&quot;"),
    ("'", "&#39;"),
];

/// HTML entity mapping for decoding.
const HTML_ENTITIES_DECODE: &[(&str, &str)] = &[
    ("&lt;", "<"),
    ("&gt;", ">"),
    ("&quot;", "\""),
    ("&#39;", "'"),
    ("&amp;", "&"), // Must be last to avoid double-decoding
];

/// Convert text to a URL-safe slug.
///
/// - Converts to lowercase
/// - Removes accents and special characters
/// - Replaces whitespace and separators with hyphens
/// - Removes leading/trailing hyphens
/// - Collapses consecutive hyphens
///
/// # Examples
/// ```
/// use phenotype_string::to_slug;
///
/// assert_eq!(to_slug("Hello World!"), "hello-world");
/// assert_eq!(to_slug("Hello  World"), "hello-world");
/// assert_eq!(to_slug("hello_world_example"), "hello-world-example");
/// assert_eq!(to_slug("Café résumé"), "cafe-resume");
/// ```
pub fn to_slug(input: &str) -> String {
    if input.is_empty() {
        return String::new();
    }

    // Step 1: Convert to lowercase
    let lowercased = input.to_lowercase();

    // Step 2: Remove/replace accents
    let normalized = remove_accents(&lowercased);

    // Step 3: Replace whitespace with hyphens
    let with_hyphens = WHITESPACE.replace_all(&normalized, "-");

    // Step 4: Remove non-alphanumeric characters (except hyphens)
    let sanitized = SLUG_PATTERN.replace_all(&with_hyphens, "-");

    // Step 5: Collapse consecutive hyphens
    let collapsed = collapse_hyphens(&sanitized);

    // Step 6: Trim hyphens from edges
    collapsed.trim_matches('-').to_string()
}

/// Encode HTML special characters.
///
/// Converts the following characters to HTML entities:
/// - `&` -> `&amp;`
/// - `<` -> `&lt;`
/// - `>` -> `&gt;`
/// - `"` -> `&quot;`
/// - `'` -> `&#39;`
///
/// # Examples
/// ```
/// use phenotype_string::encode_html;
///
/// assert_eq!(encode_html("Hello & <World>"), "Hello &amp; &lt;World&gt;");
/// assert_eq!(encode_html("Say \"hello\""), "Say &quot;hello&quot;");
/// assert_eq!(encode_html("Tom's cat"), "Tom&#39;s cat");
/// ```
pub fn encode_html(input: &str) -> String {
    let mut result = input.to_string();

    // Encode in order (& must be first to avoid double-encoding)
    for (char_str, entity) in HTML_ENTITIES {
        result = result.replace(char_str, entity);
    }

    result
}

/// Decode HTML entities to their character equivalents.
///
/// Converts the following HTML entities to characters:
/// - `&lt;` -> `<`
/// - `&gt;` -> `>`
/// - `&quot;` -> `"`
/// - `&#39;` -> `'`
/// - `&amp;` -> `&`
///
/// # Examples
/// ```
/// use phenotype_string::decode_html;
///
/// assert_eq!(decode_html("Hello &amp; &lt;World&gt;"), "Hello & <World>");
/// assert_eq!(decode_html("Say &quot;hello&quot;"), "Say \"hello\"");
/// assert_eq!(decode_html("Tom&#39;s cat"), "Tom's cat");
/// ```
pub fn decode_html(input: &str) -> String {
    let mut result = input.to_string();

    // Decode in order (&amp; must be last to avoid double-decoding)
    for (entity, char_str) in HTML_ENTITIES_DECODE {
        result = result.replace(entity, char_str);
    }

    result
}

/// Remove or replace common accented characters.
fn remove_accents(input: &str) -> String {
    let mut result = String::new();

    for ch in input.chars() {
        match ch {
            'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' => result.push('a'),
            'è' | 'é' | 'ê' | 'ë' => result.push('e'),
            'ì' | 'í' | 'î' | 'ï' => result.push('i'),
            'ò' | 'ó' | 'ô' | 'õ' | 'ö' => result.push('o'),
            'ù' | 'ú' | 'û' | 'ü' => result.push('u'),
            'ñ' => result.push('n'),
            'ç' => result.push('c'),
            other => result.push(other),
        }
    }

    result
}

/// Collapse consecutive hyphens into a single hyphen.
fn collapse_hyphens(input: &str) -> String {
    let mut result = String::new();
    let mut last_was_hyphen = false;

    for ch in input.chars() {
        if ch == '-' {
            if !last_was_hyphen {
                result.push(ch);
            }
            last_was_hyphen = true;
        } else {
            result.push(ch);
            last_was_hyphen = false;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // Slugification tests
    #[test]
    fn test_to_slug_basic() {
        assert_eq!(to_slug("Hello World"), "hello-world");
        assert_eq!(to_slug("hello world"), "hello-world");
    }

    #[test]
    fn test_to_slug_special_chars() {
        assert_eq!(to_slug("Hello! World?"), "hello-world");
        assert_eq!(to_slug("test@example.com"), "test-example-com");
        assert_eq!(to_slug("price: $99.99"), "price-9999");
    }

    #[test]
    fn test_to_slug_underscores_and_hyphens() {
        assert_eq!(to_slug("hello_world"), "hello-world");
        assert_eq!(to_slug("hello-world"), "hello-world");
        assert_eq!(to_slug("hello_-_world"), "hello-world");
    }

    #[test]
    fn test_to_slug_multiple_spaces() {
        assert_eq!(to_slug("hello  world"), "hello-world");
        assert_eq!(to_slug("hello   world"), "hello-world");
        assert_eq!(to_slug("hello  multiple  spaces"), "hello-multiple-spaces");
    }

    #[test]
    fn test_to_slug_accents() {
        assert_eq!(to_slug("Café"), "cafe");
        assert_eq!(to_slug("résumé"), "resume");
        assert_eq!(to_slug("Crème brûlée"), "creme-brulee");
        assert_eq!(to_slug("naïve"), "naive");
    }

    #[test]
    fn test_to_slug_leading_trailing() {
        assert_eq!(to_slug("-hello-world-"), "hello-world");
        assert_eq!(to_slug("--hello-world--"), "hello-world");
        assert_eq!(to_slug("hello-world-"), "hello-world");
    }

    #[test]
    fn test_to_slug_numbers() {
        assert_eq!(to_slug("Section 123"), "section-123");
        assert_eq!(to_slug("Part2Part3"), "part2part3");
    }

    #[test]
    fn test_to_slug_empty() {
        assert_eq!(to_slug(""), "");
    }

    #[test]
    fn test_to_slug_only_special_chars() {
        assert_eq!(to_slug("!!!"), "");
        assert_eq!(to_slug("---"), "");
    }

    #[test]
    fn test_to_slug_mixed() {
        assert_eq!(to_slug("My Blog Post - 2024!"), "my-blog-post-2024");
        assert_eq!(to_slug("Python_vs_Rust (2024)"), "python-vs-rust-2024");
    }

    // HTML encoding tests
    #[test]
    fn test_encode_html_ampersand() {
        assert_eq!(encode_html("Hello & World"), "Hello &amp; World");
        assert_eq!(encode_html("A & B & C"), "A &amp; B &amp; C");
    }

    #[test]
    fn test_encode_html_angle_brackets() {
        assert_eq!(encode_html("<div>"), "&lt;div&gt;");
        assert_eq!(encode_html("a < b > c"), "a &lt; b &gt; c");
    }

    #[test]
    fn test_encode_html_quotes() {
        assert_eq!(encode_html("Say \"hello\""), "Say &quot;hello&quot;");
        assert_eq!(encode_html("It's here"), "It&#39;s here");
    }

    #[test]
    fn test_encode_html_combined() {
        assert_eq!(
            encode_html("Hello & <World>"),
            "Hello &amp; &lt;World&gt;"
        );
        assert_eq!(
            encode_html("<p>\"Hello & Goodbye\"</p>"),
            "&lt;p&gt;&quot;Hello &amp; Goodbye&quot;&lt;/p&gt;"
        );
    }

    #[test]
    fn test_encode_html_no_change() {
        assert_eq!(encode_html("Plain text"), "Plain text");
        assert_eq!(encode_html("hello123"), "hello123");
    }

    #[test]
    fn test_encode_html_empty() {
        assert_eq!(encode_html(""), "");
    }

    // HTML decoding tests
    #[test]
    fn test_decode_html_ampersand() {
        assert_eq!(decode_html("Hello &amp; World"), "Hello & World");
        assert_eq!(decode_html("A &amp; B &amp; C"), "A & B & C");
    }

    #[test]
    fn test_decode_html_angle_brackets() {
        assert_eq!(decode_html("&lt;div&gt;"), "<div>");
        assert_eq!(decode_html("a &lt; b &gt; c"), "a < b > c");
    }

    #[test]
    fn test_decode_html_quotes() {
        assert_eq!(decode_html("Say &quot;hello&quot;"), "Say \"hello\"");
        assert_eq!(decode_html("It&#39;s here"), "It's here");
    }

    #[test]
    fn test_decode_html_combined() {
        assert_eq!(
            decode_html("Hello &amp; &lt;World&gt;"),
            "Hello & <World>"
        );
        assert_eq!(
            decode_html("&lt;p&gt;&quot;Hello &amp; Goodbye&quot;&lt;/p&gt;"),
            "<p>\"Hello & Goodbye\"</p>"
        );
    }

    #[test]
    fn test_decode_html_no_change() {
        assert_eq!(decode_html("Plain text"), "Plain text");
        assert_eq!(decode_html("hello123"), "hello123");
    }

    #[test]
    fn test_decode_html_empty() {
        assert_eq!(decode_html(""), "");
    }

    // Round-trip tests
    #[test]
    fn test_encode_decode_roundtrip() {
        let original = "Hello & <World> \"Test\"";
        let encoded = encode_html(original);
        let decoded = decode_html(&encoded);
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_encode_decode_complex() {
        let original = "<div>Tom's \"cat\" & dog</div>";
        let encoded = encode_html(original);
        let decoded = decode_html(&encoded);
        assert_eq!(decoded, original);
    }
}
