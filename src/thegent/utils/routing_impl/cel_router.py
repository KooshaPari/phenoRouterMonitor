"""GW-57: CEL-like expression routing for the thegent AI gateway.

Implements a lightweight CEL-inspired expression evaluator for routing rules.
Expressions operate on a request context dict and return a string (target model/provider)
or None (no match).

No external CEL library required — uses a custom recursive-descent parser.

Supported operators:
  ==, !=, <, <=, >, >=, in, contains, startsWith, endsWith
  &&, ||, !
  Attribute access: context.model, context.provider, context.metadata.user_tier
  Literals: strings (double-quoted), numbers (int/float), booleans (true/false), null
  Ternary: condition ? then_expr : else_expr

# @trace FR-AROUTE-057
"""

from __future__ import annotations

import logging
from dataclasses import dataclass
from typing import Any

_log = logging.getLogger(__name__)


# ---------------------------------------------------------------------------
# Public data structures
# ---------------------------------------------------------------------------


@dataclass
class CelRoute:
    """A CEL-expression-based routing rule."""

    expression: str  # CEL expression string, e.g. 'context.metadata.tier == "premium"'
    target: str  # model/provider to route to when expression is true
    name: str = ""  # optional human-readable name


@dataclass
class CelEvalResult:
    """Result of evaluating CEL routes against a context."""

    matched: bool
    target: str  # empty if no match
    route_name: str  # name of matched route or ""
    error: str  # non-empty if expression evaluation failed


# ---------------------------------------------------------------------------
# Tokenizer
# ---------------------------------------------------------------------------

_TK_IDENT = "IDENT"
_TK_STRING = "STRING"
_TK_NUMBER = "NUMBER"
_TK_OP = "OP"
_TK_LPAREN = "LPAREN"
_TK_RPAREN = "RPAREN"
_TK_LBRACKET = "LBRACKET"
_TK_RBRACKET = "RBRACKET"
_TK_COMMA = "COMMA"
_TK_EOF = "EOF"


@dataclass
class _Token:
    kind: str
    value: Any


def _tokenize(text: str) -> list[_Token]:
    """Tokenize a CEL-like expression string into a list of tokens."""
    tokens: list[_Token] = []
    i = 0
    n = len(text)

    while i < n:
        ch = text[i]

        # Skip whitespace
        if ch in " \t\n\r":
            i += 1
            continue

        # String literal (double-quoted)
        if ch == '"':
            j = i + 1
            buf: list[str] = []
            while j < n and text[j] != '"':
                if text[j] == "\\" and j + 1 < n:
                    esc = text[j + 1]
                    if esc == "n":
                        buf.append("\n")
                    elif esc == "t":
                        buf.append("\t")
                    elif esc == "\\":
                        buf.append("\\")
                    elif esc == '"':
                        buf.append('"')
                    else:
                        buf.append("\\")
                        buf.append(esc)
                    j += 2
                else:
                    buf.append(text[j])
                    j += 1
            tokens.append(_Token(_TK_STRING, "".join(buf)))
            i = j + 1
            continue

        # Numbers
        if ch.isdigit() or (ch == "-" and i + 1 < n and text[i + 1].isdigit()):
            j = i
            if text[j] == "-":
                j += 1
            while j < n and (text[j].isdigit() or text[j] == "."):
                j += 1
            raw = text[i:j]
            value: int | float = float(raw) if "." in raw else int(raw)
            tokens.append(_Token(_TK_NUMBER, value))
            i = j
            continue

        # Two-character operators
        if i + 1 < n:
            two = text[i : i + 2]
            if two in ("==", "!=", "<=", ">=", "&&", "||"):
                tokens.append(_Token(_TK_OP, two))
                i += 2
                continue

        # Single-character operators
        if ch in ("<", ">", "!", "?", ":", "."):
            tokens.append(_Token(_TK_OP, ch))
            i += 1
            continue

        if ch == "(":
            tokens.append(_Token(_TK_LPAREN, "("))
            i += 1
            continue

        if ch == ")":
            tokens.append(_Token(_TK_RPAREN, ")"))
            i += 1
            continue

        if ch == "[":
            tokens.append(_Token(_TK_LBRACKET, "["))
            i += 1
            continue

        if ch == "]":
            tokens.append(_Token(_TK_RBRACKET, "]"))
            i += 1
            continue

        if ch == ",":
            tokens.append(_Token(_TK_COMMA, ","))
            i += 1
            continue

        # Identifier (alphanumeric + underscore; dots handled in parser)
        if ch.isalpha() or ch == "_":
            j = i
            while j < n and (text[j].isalnum() or text[j] == "_" or text[j] == "."):
                j += 1
            ident = text[i:j]
            tokens.append(_Token(_TK_IDENT, ident))
            i = j
            continue

        raise ValueError(f"Unexpected character {ch!r} at position {i} in expression: {text!r}")

    tokens.append(_Token(_TK_EOF, None))
    return tokens


# ---------------------------------------------------------------------------
# Parser — recursive descent, produces AST dicts
# ---------------------------------------------------------------------------


class _Parser:
    """Recursive-descent parser for CEL-like expressions."""

    def __init__(self, tokens: list[_Token]) -> None:
        self._tokens = tokens
        self._pos = 0

    def _peek(self) -> _Token:
        return self._tokens[self._pos]

    def _consume(self) -> _Token:
        tok = self._tokens[self._pos]
        self._pos += 1
        return tok

    def _expect_op(self, op: str) -> _Token:
        tok = self._consume()
        if tok.kind != _TK_OP or tok.value != op:
            raise ValueError(f"Expected operator {op!r}, got {tok!r}")
        return tok

    def parse(self) -> dict:
        """Parse the full expression, expecting EOF at end."""
        node = self._parse_ternary()
        if self._peek().kind != _TK_EOF:
            raise ValueError(f"Unexpected token after expression: {self._peek()!r}")
        return node

    def _parse_ternary(self) -> dict:
        """Parse ternary: cond ? then : else."""
        cond = self._parse_or()
        if self._peek().kind == _TK_OP and self._peek().value == "?":
            self._consume()  # consume '?'
            then_expr = self._parse_or()
            self._expect_op(":")
            else_expr = self._parse_ternary()
            return {"op": "ternary", "cond": cond, "then": then_expr, "else": else_expr}
        return cond

    def _parse_or(self) -> dict:
        left = self._parse_and()
        while self._peek().kind == _TK_OP and self._peek().value == "||":
            self._consume()
            right = self._parse_and()
            left = {"op": "or", "left": left, "right": right}
        return left

    def _parse_and(self) -> dict:
        left = self._parse_not()
        while self._peek().kind == _TK_OP and self._peek().value == "&&":
            self._consume()
            right = self._parse_not()
            left = {"op": "and", "left": left, "right": right}
        return left

    def _parse_not(self) -> dict:
        if self._peek().kind == _TK_OP and self._peek().value == "!":
            self._consume()
            expr = self._parse_not()
            return {"op": "not", "expr": expr}
        return self._parse_comparison()

    def _parse_comparison(self) -> dict:
        left = self._parse_primary()
        tok = self._peek()
        if tok.kind == _TK_OP and tok.value in ("==", "!=", "<", "<=", ">", ">="):
            self._consume()
            right = self._parse_primary()
            op_map = {
                "==": "eq",
                "!=": "ne",
                "<": "lt",
                "<=": "lte",
                ">": "gt",
                ">=": "gte",
            }
            return {"op": op_map[tok.value], "left": left, "right": right}

        # `in` keyword: left in [...]
        if tok.kind == _TK_IDENT and tok.value == "in":
            self._consume()
            right = self._parse_primary()
            return {"op": "in", "left": left, "right": right}

        return left

    def _parse_primary(self) -> dict:
        tok = self._peek()

        # Parenthesised expression
        if tok.kind == _TK_LPAREN:
            self._consume()
            node = self._parse_ternary()
            if self._peek().kind != _TK_RPAREN:
                raise ValueError("Expected ')' after parenthesised expression")
            self._consume()
            return node

        # List literal: [ item, item, ... ]
        if tok.kind == _TK_LBRACKET:
            return self._parse_list_literal()

        # String literal
        if tok.kind == _TK_STRING:
            self._consume()
            return {"op": "literal", "value": tok.value}

        # Number literal
        if tok.kind == _TK_NUMBER:
            self._consume()
            return {"op": "literal", "value": tok.value}

        # Identifier: keyword or attribute path
        if tok.kind == _TK_IDENT:
            self._consume()
            ident = tok.value

            # Keywords
            if ident == "true":
                return {"op": "literal", "value": True}
            if ident == "false":
                return {"op": "literal", "value": False}
            if ident == "null":
                return {"op": "literal", "value": None}
            if ident == "in":
                raise ValueError("Unexpected 'in' keyword in primary position")

            # Method call: expr.method(arg)
            # The tokenizer absorbs dots into IDENT tokens, so we parse the
            # whole dotted path as a single IDENT. But we need to handle
            # .contains() / .startsWith() / .endsWith() method calls.
            # Strategy: peek for '(' to detect a method call on the final
            # segment of the path.
            # e.g. "context.model.contains" followed by "(" "arg" ")"
            base_path, _, method = ident.rpartition(".")
            if self._peek().kind == _TK_LPAREN and method in ("contains", "startsWith", "endsWith"):
                self._consume()  # '('
                arg = self._parse_ternary()
                if self._peek().kind != _TK_RPAREN:
                    raise ValueError(f"Expected ')' after {method}() argument")
                self._consume()  # ')'
                haystack: dict = {"op": "attr", "path": base_path} if base_path else {"op": "literal", "value": ident}
                return {"op": method, "haystack": haystack, "needle": arg}

            # Plain attribute path
            return {"op": "attr", "path": ident}

        raise ValueError(f"Unexpected token in primary: {tok!r}")

    def _parse_list_literal(self) -> dict:
        """Parse a list literal: [ expr, expr, ... ]."""
        self._consume()  # '['
        items: list[dict] = []
        while self._peek().kind != _TK_RBRACKET:
            if self._peek().kind == _TK_EOF:
                raise ValueError("Unterminated list literal")
            items.append(self._parse_ternary())
            if self._peek().kind == _TK_COMMA:
                self._consume()
        self._consume()  # ']'
        return {"op": "list", "items": items}


# ---------------------------------------------------------------------------
# AST evaluator
# ---------------------------------------------------------------------------


def _get_attr(path: str, context: dict) -> Any:
    """Traverse a dot-separated attribute path in a nested dict.

    Returns None for any missing key without raising.
    """
    parts = path.split(".")
    current: Any = context
    for part in parts:
        if not isinstance(current, dict):
            return None
        current = current.get(part)
    return current


def _eval_node(node: dict, context: dict) -> Any:
    """Recursively evaluate an AST node against context."""
    op = node["op"]

    if op == "literal":
        return node["value"]

    if op == "list":
        return [_eval_node(item, context) for item in node["items"]]

    if op == "attr":
        return _get_attr(node["path"], context)

    if op == "eq":
        return _eval_node(node["left"], context) == _eval_node(node["right"], context)

    if op == "ne":
        return _eval_node(node["left"], context) != _eval_node(node["right"], context)

    if op == "lt":
        left = _eval_node(node["left"], context)
        right = _eval_node(node["right"], context)
        if left is None or right is None:
            return False
        return left < right

    if op == "lte":
        left = _eval_node(node["left"], context)
        right = _eval_node(node["right"], context)
        if left is None or right is None:
            return False
        return left <= right

    if op == "gt":
        left = _eval_node(node["left"], context)
        right = _eval_node(node["right"], context)
        if left is None or right is None:
            return False
        return left > right

    if op == "gte":
        left = _eval_node(node["left"], context)
        right = _eval_node(node["right"], context)
        if left is None or right is None:
            return False
        return left >= right

    if op == "and":
        return bool(_eval_node(node["left"], context)) and bool(_eval_node(node["right"], context))

    if op == "or":
        return bool(_eval_node(node["left"], context)) or bool(_eval_node(node["right"], context))

    if op == "not":
        return not bool(_eval_node(node["expr"], context))

    if op == "in":
        left = _eval_node(node["left"], context)
        right = _eval_node(node["right"], context)
        if right is None:
            return False
        try:
            return left in right
        except TypeError:
            return False

    if op == "contains":
        haystack = _eval_node(node["haystack"], context)
        needle = _eval_node(node["needle"], context)
        if haystack is None or needle is None:
            return False
        return str(needle) in str(haystack)

    if op == "startsWith":
        haystack = _eval_node(node["haystack"], context)
        needle = _eval_node(node["needle"], context)
        if haystack is None or needle is None:
            return False
        return str(haystack).startswith(str(needle))

    if op == "endsWith":
        haystack = _eval_node(node["haystack"], context)
        needle = _eval_node(node["needle"], context)
        if haystack is None or needle is None:
            return False
        return str(haystack).endswith(str(needle))

    if op == "ternary":
        cond = _eval_node(node["cond"], context)
        if cond:
            return _eval_node(node["then"], context)
        return _eval_node(node["else"], context)

    raise ValueError(f"Unknown AST op: {op!r}")


# ---------------------------------------------------------------------------
# compile_expression / eval_expression
# ---------------------------------------------------------------------------


def compile_expression(expression: str) -> Any:
    """Pre-compile a CEL expression for repeated evaluation.

    Returns an opaque compiled expression object (an AST dict).
    Raises ValueError on syntax error.
    """
    tokens = _tokenize(expression)
    parser = _Parser(tokens)
    return parser.parse()


def eval_expression(compiled: Any, context: dict) -> bool:
    """Evaluate a pre-compiled expression against context. Returns bool."""
    result = _eval_node(compiled, context)
    return bool(result)


# ---------------------------------------------------------------------------
# CelEvaluator
# ---------------------------------------------------------------------------


class CelEvaluator:
    """Compiles and evaluates CEL-like routing expressions."""

    def __init__(self, routes: list[CelRoute]) -> None:
        self._routes = routes
        # Pre-compile all expressions; store None on compile error
        self._compiled: list[tuple[CelRoute, Any, str]] = []
        for route in routes:
            try:
                compiled = compile_expression(route.expression)
                self._compiled.append((route, compiled, ""))
            except Exception as exc:  # noqa: BLE001
                _log.warning("CEL compile error for route %r: %s", route.name, exc)
                self._compiled.append((route, None, str(exc)))

    def evaluate(self, context: dict) -> CelEvalResult:
        """Evaluate all routes in order, return first match."""
        for route, compiled, compile_error in self._compiled:
            if compiled is None:
                return CelEvalResult(
                    matched=False,
                    target="",
                    route_name=route.name,
                    error=compile_error,
                )
            try:
                result = _eval_node(compiled, context)
                if result is True or (not isinstance(result, bool) and result):
                    # For ternary expressions that return a string target
                    if isinstance(result, str):
                        _log.debug("CEL route matched: name=%r target=%r (ternary)", route.name, result)
                        return CelEvalResult(
                            matched=True,
                            target=result,
                            route_name=route.name,
                            error="",
                        )
                    _log.debug("CEL route matched: name=%r target=%r", route.name, route.target)
                    return CelEvalResult(
                        matched=True,
                        target=route.target,
                        route_name=route.name,
                        error="",
                    )
            except Exception as exc:  # noqa: BLE001
                _log.warning("CEL eval error for route %r: %s", route.name, exc)
                return CelEvalResult(
                    matched=False,
                    target="",
                    route_name=route.name,
                    error=str(exc),
                )

        return CelEvalResult(matched=False, target="", route_name="", error="")


# ---------------------------------------------------------------------------
# Convenience function
# ---------------------------------------------------------------------------


def evaluate_cel_routes(routes: list[CelRoute], context: dict) -> CelEvalResult:
    """Convenience function: evaluate a list of CelRoutes against context."""
    evaluator = CelEvaluator(routes)
    return evaluator.evaluate(context)
