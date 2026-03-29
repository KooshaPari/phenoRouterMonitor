"""Fast string operations with optimized backends.

This module provides optimized string operations:
- rapidfuzz for fuzzy matching (already installed!)
- regex for advanced regex patterns (already installed!)
- Optimized string operations

Performance improvements:
- rapidfuzz: 10-100x faster fuzzy matching
- regex: Faster complex regex patterns
- Optimized string operations
"""

try:
    from rapidfuzz import fuzz, process

    RAPIDFUZZ_AVAILABLE = True
except ImportError:
    RAPIDFUZZ_AVAILABLE = False

try:
    import regex

    REGEX_AVAILABLE = True
except ImportError:
    REGEX_AVAILABLE = False

import re


class FastStringOps:
    """High-performance string operations with optimized backends."""

    @staticmethod
    def fuzzy_match(
        query: str, choices: list[str], limit: int = 5, score_cutoff: int = 60
    ) -> list[tuple[str, float, int]]:
        """Fuzzy string matching using rapidfuzz (10-100x faster).

        Args:
            query: Query string
            choices: List of strings to match against
            limit: Maximum number of results
            score_cutoff: Minimum similarity score (0-100)

        Returns:
            List of (match, score, index) tuples

        Performance:
            - rapidfuzz: 10-100x faster than fuzzywuzzy
            - Uses optimized C++ implementation
        """
        if RAPIDFUZZ_AVAILABLE:
            results = process.extract(query, choices, limit=limit, score_cutoff=score_cutoff)
            return [(match, score, idx) for idx, (match, score, _) in enumerate(results)]
        # Fallback to simple substring matching
        matches = []
        query_lower = query.lower()
        for idx, choice in enumerate(choices):
            if query_lower in choice.lower():
                matches.append((choice, 100.0, idx))
        return matches[:limit]

    @staticmethod
    def fuzzy_ratio(str1: str, str2: str) -> float:
        """Calculate fuzzy similarity ratio (0-100).

        Args:
            str1: First string
            str2: Second string

        Returns:
            Similarity ratio (0-100)
        """
        if RAPIDFUZZ_AVAILABLE:
            return fuzz.ratio(str1, str2)
        # Simple fallback
        if str1 == str2:
            return 100.0
        if str1.lower() == str2.lower():
            return 95.0
        return 0.0

    @staticmethod
    def regex_search(pattern: str, text: str, **kwargs) -> "regex.Match[str] | re.Match[str] | None":
        """Search using regex library (faster for complex patterns).

        Args:
            pattern: Regex pattern
            text: Text to search
            **kwargs: Additional regex options

        Returns:
            Match object or None

        Performance:
            - regex library: Faster for complex patterns
            - Better Unicode support
            - More features than standard re
        """
        if REGEX_AVAILABLE:
            return regex.search(pattern, text, **kwargs)
        return re.search(pattern, text, **kwargs)

    @staticmethod
    def regex_findall(pattern: str, text: str, **kwargs) -> list[str]:
        """Find all matches using regex library.

        Args:
            pattern: Regex pattern
            text: Text to search
            **kwargs: Additional regex options

        Returns:
            List of matches
        """
        if REGEX_AVAILABLE:
            return regex.findall(pattern, text, **kwargs)
        return re.findall(pattern, text, **kwargs)


# Convenience functions
def fuzzy_match(query: str, choices: list[str], limit: int = 5, score_cutoff: int = 60) -> list[tuple[str, float, int]]:
    """Fuzzy string matching."""
    return FastStringOps.fuzzy_match(query, choices, limit, score_cutoff)


def fuzzy_ratio(str1: str, str2: str) -> float:
    """Calculate fuzzy similarity ratio."""
    return FastStringOps.fuzzy_ratio(str1, str2)


def regex_search(pattern: str, text: str, **kwargs):
    """Search using optimized regex."""
    return FastStringOps.regex_search(pattern, text, **kwargs)


def regex_findall(pattern: str, text: str, **kwargs) -> list[str]:
    """Find all matches using optimized regex."""
    return FastStringOps.regex_findall(pattern, text, **kwargs)
