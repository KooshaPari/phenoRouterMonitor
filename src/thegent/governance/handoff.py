import re
from pathlib import Path
from typing import Any


class HandoffIntegrity:
    """WP-16005: Verifies that delegated prompts are complete and context-aware."""

    def __init__(self, workspace_root: Path) -> None:
        self.workspace_root = workspace_root

    def analyze_prompt(self, prompt: str) -> dict[str, Any]:
        """Analyze a prompt for potential missing context."""
        findings = []
        warnings = []

        # 1. Check prompt length
        if len(prompt.strip()) < 20:
            findings.append("Prompt is very short (< 20 characters), may lack context")

        # 2. Look for referenced files that exist in the workspace
        # Matches patterns like src/main.py, ./README.md, etc.
        file_patterns = re.findall(r"(?:[\./a-zA-Z0-9_\-]+\.[a-zA-Z0-9]+)", prompt)
        referenced_files = []
        missing_files = []
        for p in file_patterns:
            full_path = self.workspace_root / p
            if full_path.exists() and full_path.is_file():
                referenced_files.append(p)
            else:
                missing_files.append(p)

        if missing_files:
            warnings.append(f"Referenced files not found: {', '.join(missing_files[:3])}")

        # 3. Look for keywords that suggest missing context
        vague_keywords = ["implement this", "fix the bug", "as discussed", "you know what", "do it"]
        for kw in vague_keywords:
            if kw in prompt.lower():
                findings.append(f"Potential vague instruction: '{kw}'")

        # 4. Check for specific action verbs (good sign)
        action_verbs = ["create", "implement", "refactor", "update", "add", "remove", "fix", "test"]
        has_action = any(verb in prompt.lower() for verb in action_verbs)

        # 5. Check for context indicators (good sign)
        context_indicators = ["because", "since", "to", "for", "when", "if"]
        has_context = any(indicator in prompt.lower() for indicator in context_indicators)

        # 6. Check for code blocks or examples (good sign)
        has_code = "```" in prompt or "`" in prompt

        completeness_score = 0
        if has_action:
            completeness_score += 1
        if has_context:
            completeness_score += 1
        if has_code:
            completeness_score += 1
        if len(referenced_files) > 0:
            completeness_score += 1

        is_complete = len(findings) == 0 and completeness_score >= 2

        return {
            "referenced_files": referenced_files,
            "missing_files": missing_files,
            "findings": findings,
            "warnings": warnings,
            "is_complete": is_complete,
            "completeness_score": completeness_score,
            "has_action": has_action,
            "has_context": has_context,
            "has_code": has_code,
        }

    def suggest_improvements(self, prompt: str, analysis: dict[str, Any] | None = None) -> str:
        """
        Suggest ways to improve the handoff prompt.

        Args:
            prompt: Original prompt
            analysis: Optional analysis result from analyze_prompt()

        Returns:
            Improved prompt with suggestions
        """
        if analysis is None:
            analysis = self.analyze_prompt(prompt)

        if analysis["is_complete"]:
            return prompt

        suggestions = []

        # Add findings
        if analysis["findings"]:
            suggestions.extend([f"⚠️  {f}" for f in analysis["findings"]])

        # Add warnings
        if analysis["warnings"]:
            suggestions.extend([f"⚠️  {w}" for w in analysis["warnings"]])

        # Suggest improvements based on missing elements
        if not analysis["has_action"]:
            suggestions.append("💡 Add specific action verbs (create, implement, refactor, etc.)")

        if not analysis["has_context"]:
            suggestions.append("💡 Add context explaining why this task is needed")

        if not analysis["has_code"]:
            suggestions.append("💡 Consider adding code examples or file references")

        if len(analysis["referenced_files"]) == 0:
            suggestions.append("💡 Reference specific files that need to be modified")

        if suggestions:
            suggestions_text = "\n".join(suggestions)
            return f"{prompt}\n\n### Suggestions for improvement:\n{suggestions_text}"

        return prompt

    def validate_handoff(self, prompt: str, min_completeness_score: int = 2) -> tuple[bool, str]:
        """
        Validate that a handoff prompt meets minimum quality requirements.

        Args:
            prompt: Prompt to validate
            min_completeness_score: Minimum completeness score required (default: 2)

        Returns:
            Tuple of (is_valid, error_message)
        """
        analysis = self.analyze_prompt(prompt)

        if analysis["completeness_score"] < min_completeness_score:
            return False, f"Completeness score {analysis['completeness_score']} below minimum {min_completeness_score}"

        if analysis["findings"]:
            return False, f"Found issues: {', '.join(analysis['findings'][:2])}"

        return True, "Handoff prompt is valid"
