//! Fuzzy matching and string similarity utilities.

/// Calculate the Levenshtein distance between two strings.
///
/// The Levenshtein distance is the minimum number of single-character edits
/// (insertions, deletions, or substitutions) required to change one string
/// into another.
///
/// # Examples
/// ```
/// use phenotype_string::levenshtein_distance;
///
/// assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
/// assert_eq!(levenshtein_distance("hello", "hello"), 0);
/// assert_eq!(levenshtein_distance("", "abc"), 3);
/// assert_eq!(levenshtein_distance("abc", ""), 3);
/// ```
pub fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let len1 = s1.len();
    let len2 = s2.len();

    // Handle empty string cases
    if len1 == 0 {
        return len2;
    }
    if len2 == 0 {
        return len1;
    }

    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();

    // Create a matrix to store distances
    let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

    // Initialize first row and column
    for i in 0..=len1 {
        matrix[i][0] = i;
    }
    for j in 0..=len2 {
        matrix[0][j] = j;
    }

    // Fill the matrix
    for (i, &c1) in s1_chars.iter().enumerate() {
        for (j, &c2) in s2_chars.iter().enumerate() {
            let cost = if c1 == c2 { 0 } else { 1 };
            matrix[i + 1][j + 1] = std::cmp::min(
                std::cmp::min(
                    matrix[i][j + 1] + 1,     // deletion
                    matrix[i + 1][j] + 1,     // insertion
                ),
                matrix[i][j] + cost,          // substitution
            );
        }
    }

    matrix[len1][len2]
}

/// Calculate similarity score between two strings (0.0 to 1.0).
///
/// Uses normalized Levenshtein distance:
/// `score = 1.0 - (distance / max_length)`
///
/// A score of 1.0 means identical strings, 0.0 means completely different.
///
/// # Examples
/// ```
/// use phenotype_string::similarity_score;
///
/// assert_eq!(similarity_score("hello", "hello"), 1.0);
/// assert_eq!(similarity_score("", ""), 1.0);
/// let score = similarity_score("kitten", "sitting");
/// assert!(score > 0.5 && score < 0.8);
/// ```
pub fn similarity_score(s1: &str, s2: &str) -> f64 {
    let distance = levenshtein_distance(s1, s2) as f64;
    let max_len = std::cmp::max(s1.len(), s2.len()) as f64;

    if max_len == 0.0 {
        return 1.0; // Both strings are empty
    }

    1.0 - (distance / max_len)
}

/// Perform fuzzy matching between a pattern and a text.
///
/// Returns `true` if the pattern matches the text fuzzily, allowing for
/// character omissions. Characters must appear in the same order.
///
/// # Examples
/// ```
/// use phenotype_string::fuzzy_match;
///
/// assert!(fuzzy_match("hlo", "hello"));
/// assert!(fuzzy_match("hello", "hello"));
/// assert!(!fuzzy_match("hello", "hlo"));
/// assert!(fuzzy_match("abc", "aXbXc"));
/// ```
pub fn fuzzy_match(pattern: &str, text: &str) -> bool {
    let pattern_chars: Vec<char> = pattern.chars().collect();
    let text_chars: Vec<char> = text.chars().collect();

    let mut pattern_idx = 0;
    let mut text_idx = 0;

    while pattern_idx < pattern_chars.len() && text_idx < text_chars.len() {
        if pattern_chars[pattern_idx].eq_ignore_ascii_case(&text_chars[text_idx]) {
            pattern_idx += 1;
        }
        text_idx += 1;
    }

    pattern_idx == pattern_chars.len()
}

/// Perform case-insensitive fuzzy matching.
///
/// Returns `true` if all characters of the pattern appear in the text
/// in the same order, ignoring case.
///
/// # Examples
/// ```
/// use phenotype_string::fuzzy_match;
///
/// assert!(fuzzy_match("HeLLo", "hello"));
/// assert!(fuzzy_match("ABC", "aXbXc"));
/// ```
pub fn fuzzy_match_case_sensitive(pattern: &str, text: &str) -> bool {
    let pattern_chars: Vec<char> = pattern.chars().collect();
    let text_chars: Vec<char> = text.chars().collect();

    let mut pattern_idx = 0;
    let mut text_idx = 0;

    while pattern_idx < pattern_chars.len() && text_idx < text_chars.len() {
        if pattern_chars[pattern_idx] == text_chars[text_idx] {
            pattern_idx += 1;
        }
        text_idx += 1;
    }

    pattern_idx == pattern_chars.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Levenshtein distance tests
    #[test]
    fn test_levenshtein_identical() {
        assert_eq!(levenshtein_distance("hello", "hello"), 0);
        assert_eq!(levenshtein_distance("", ""), 0);
        assert_eq!(levenshtein_distance("a", "a"), 0);
    }

    #[test]
    fn test_levenshtein_empty_string() {
        assert_eq!(levenshtein_distance("abc", ""), 3);
        assert_eq!(levenshtein_distance("", "xyz"), 3);
        assert_eq!(levenshtein_distance("", ""), 0);
    }

    #[test]
    fn test_levenshtein_single_char() {
        assert_eq!(levenshtein_distance("a", "b"), 1);
        assert_eq!(levenshtein_distance("a", "ab"), 1);
        assert_eq!(levenshtein_distance("ab", "a"), 1);
    }

    #[test]
    fn test_levenshtein_insertion() {
        assert_eq!(levenshtein_distance("cat", "cats"), 1);
        assert_eq!(levenshtein_distance("at", "cat"), 1);
    }

    #[test]
    fn test_levenshtein_deletion() {
        assert_eq!(levenshtein_distance("cats", "cat"), 1);
        assert_eq!(levenshtein_distance("hello", "helo"), 1);
    }

    #[test]
    fn test_levenshtein_substitution() {
        assert_eq!(levenshtein_distance("cat", "car"), 1);
        assert_eq!(levenshtein_distance("hello", "hallo"), 1);
    }

    #[test]
    fn test_levenshtein_multiple_edits() {
        assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
        assert_eq!(levenshtein_distance("saturday", "sunday"), 3);
    }

    #[test]
    fn test_levenshtein_completely_different() {
        assert_eq!(levenshtein_distance("abc", "xyz"), 3);
        assert_eq!(levenshtein_distance("a", "bcde"), 4);
    }

    #[test]
    fn test_levenshtein_unicode() {
        // The é and ï are counted as single characters
        assert_eq!(levenshtein_distance("café", "cafe"), 1);
        assert_eq!(levenshtein_distance("naïve", "naive"), 1);
        // These equivalents work because we're comparing raw bytes
        assert_eq!(levenshtein_distance("résumé", "resume"), 2);
    }

    #[test]
    fn test_levenshtein_case_sensitive() {
        assert_eq!(levenshtein_distance("Hello", "hello"), 1);
        assert_eq!(levenshtein_distance("ABC", "abc"), 3);
    }

    // Similarity score tests
    #[test]
    fn test_similarity_identical() {
        assert_eq!(similarity_score("hello", "hello"), 1.0);
        assert_eq!(similarity_score("", ""), 1.0);
    }

    #[test]
    fn test_similarity_completely_different() {
        assert_eq!(similarity_score("abc", "xyz"), 0.0);
    }

    #[test]
    fn test_similarity_single_char_diff() {
        let score = similarity_score("cat", "car");
        assert!(score > 0.6 && score < 0.7);
    }

    #[test]
    fn test_similarity_empty_vs_text() {
        assert_eq!(similarity_score("", "abc"), 0.0);
        assert_eq!(similarity_score("abc", ""), 0.0);
    }

    #[test]
    fn test_similarity_kitten_sitting() {
        let score = similarity_score("kitten", "sitting");
        assert!(score > 0.4 && score < 0.6);
    }

    #[test]
    fn test_similarity_score_bounds() {
        // Similarity should always be between 0.0 and 1.0
        let pairs = vec![
            ("hello", "world"),
            ("a", "bcd"),
            ("test", "test"),
            ("abc", "xyz"),
        ];

        for (s1, s2) in pairs {
            let score = similarity_score(s1, s2);
            assert!(score >= 0.0 && score <= 1.0);
        }
    }

    // Fuzzy match tests
    #[test]
    fn test_fuzzy_match_exact() {
        assert!(fuzzy_match("hello", "hello"));
        assert!(fuzzy_match("test", "test"));
    }

    #[test]
    fn test_fuzzy_match_subset() {
        assert!(fuzzy_match("hlo", "hello"));
        assert!(fuzzy_match("abc", "aXbXc"));
        assert!(fuzzy_match("hlo", "h e l l o"));
    }

    #[test]
    fn test_fuzzy_match_empty_pattern() {
        assert!(fuzzy_match("", "hello"));
        assert!(fuzzy_match("", ""));
    }

    #[test]
    fn test_fuzzy_match_empty_text() {
        assert!(!fuzzy_match("hello", ""));
        assert!(fuzzy_match("", ""));
    }

    #[test]
    fn test_fuzzy_match_pattern_longer() {
        assert!(!fuzzy_match("hello", "hi"));
        assert!(!fuzzy_match("abcd", "ad"));
    }

    #[test]
    fn test_fuzzy_match_out_of_order() {
        assert!(!fuzzy_match("helo", "hllo"));
        assert!(!fuzzy_match("abc", "cba"));
    }

    #[test]
    fn test_fuzzy_match_case_insensitive() {
        assert!(fuzzy_match("HELLO", "hello"));
        assert!(fuzzy_match("HeLLo", "hello"));
        assert!(fuzzy_match("abc", "ABC"));
    }

    #[test]
    fn test_fuzzy_match_single_char() {
        assert!(fuzzy_match("a", "abc"));
        assert!(fuzzy_match("z", "xyz"));
        assert!(!fuzzy_match("d", "abc"));
    }

    #[test]
    fn test_fuzzy_match_with_numbers() {
        assert!(fuzzy_match("123", "1a2b3c"));
        assert!(fuzzy_match("42", "4x2y"));
    }

    // Case-sensitive fuzzy match tests
    #[test]
    fn test_fuzzy_match_case_sensitive_exact() {
        assert!(fuzzy_match_case_sensitive("Hello", "Hello"));
        assert!(!fuzzy_match_case_sensitive("Hello", "hello"));
    }

    #[test]
    fn test_fuzzy_match_case_sensitive_subset() {
        assert!(fuzzy_match_case_sensitive("Hlo", "Hello"));
        assert!(!fuzzy_match_case_sensitive("hlo", "Hello"));
    }

    #[test]
    fn test_fuzzy_match_case_sensitive_all_caps() {
        assert!(fuzzy_match_case_sensitive("ABC", "AxBxC"));
        assert!(!fuzzy_match_case_sensitive("abc", "AxBxC"));
    }

    // Combined tests
    #[test]
    fn test_fuzzy_vs_levenshtein() {
        // "hlo" fuzzy matches "hello"
        assert!(fuzzy_match("hlo", "hello"));
        // But levenshtein distance is 2 (two deletions)
        assert_eq!(levenshtein_distance("hlo", "hello"), 2);
    }

    #[test]
    fn test_pattern_matching_workflow() {
        let text = "The quick brown fox";

        // Fuzzy match various patterns
        assert!(fuzzy_match("qbf", text));
        assert!(fuzzy_match("quick", text));
        assert!(fuzzy_match("fox", text));
        assert!(!fuzzy_match("slow", text));
    }

    #[test]
    fn test_similarity_vs_fuzzy_match() {
        // Both work for different use cases
        let text = "programming";

        // Fuzzy match is good for finding substrings
        assert!(fuzzy_match("prgm", text));

        // Similarity score is good for finding close matches
        let score = similarity_score("programing", text);
        assert!(score > 0.8);
    }
}
