#!/usr/bin/env python3
"""Merge contiguous private ``use crate::...`` imports in Rust modules.

The script deliberately handles only a conservative subset of Rust source:
plain private imports without attributes or comments. It groups imports by
their lexical brace scope and never moves an import across another item.
Use ``--write`` to update files; without it the script reports files that
would change.
"""

from __future__ import annotations

import argparse
import re
import sys
from collections import OrderedDict
from dataclasses import dataclass, field
from pathlib import Path


IDENT_RE = re.compile(r"[A-Za-z_][A-Za-z0-9_]*")


@dataclass(frozen=True)
class Token:
    value: str
    start: int
    end: int


@dataclass(frozen=True)
class UseSpan:
    start: int
    end: int
    token_index: int
    scope: tuple[int, ...]
    tokens: tuple[Token, ...]


@dataclass
class ImportNode:
    children: OrderedDict[str, "ImportNode"] = field(default_factory=OrderedDict)
    terminal_aliases: list[str | None] = field(default_factory=list)
    self_aliases: list[str | None] = field(default_factory=list)


class ParseError(ValueError):
    pass


class UseTreeParser:
    def __init__(self, tokens: list[str]) -> None:
        self.tokens = tokens
        self.index = 0

    def parse(self) -> list[tuple[list[str], str | None, bool]]:
        imports: list[tuple[list[str], str | None, bool]] = []
        self._parse_tree([], imports)
        if self.index != len(self.tokens):
            raise ParseError(f"unexpected token {self.tokens[self.index]!r}")
        return imports

    def _parse_tree(
        self,
        prefix: list[str],
        imports: list[tuple[list[str], str | None, bool]],
    ) -> None:
        if self.index >= len(self.tokens):
            raise ParseError("missing import tree")

        if self.tokens[self.index] == "self":
            self.index += 1
            alias = self._parse_alias()
            imports.append((prefix, alias, True))
            return

        segments: list[str] = []
        while self.index < len(self.tokens):
            token = self.tokens[self.index]
            if not self._is_segment(token):
                break
            segments.append(token)
            self.index += 1
            if self.index >= len(self.tokens) or self.tokens[self.index] != "::":
                break
            self.index += 1
            if self.index >= len(self.tokens):
                raise ParseError("path ends after ::")
            if self.tokens[self.index] == "{":
                self.index += 1
                self._parse_group(prefix + segments, imports)
                return
            if self.tokens[self.index] == "*":
                self.index += 1
                imports.append((prefix + segments + ["*"], None, False))
                return

        if not segments:
            raise ParseError(f"expected path segment, got {self.tokens[self.index]!r}")
        alias = self._parse_alias()
        imports.append((prefix + segments, alias, False))

    def _parse_group(
        self,
        prefix: list[str],
        imports: list[tuple[list[str], str | None, bool]],
    ) -> None:
        while True:
            if self.index >= len(self.tokens):
                raise ParseError("unterminated import group")
            if self.tokens[self.index] == "}":
                self.index += 1
                return
            self._parse_tree(prefix, imports)
            if self.index >= len(self.tokens):
                raise ParseError("unterminated import group")
            if self.tokens[self.index] == ",":
                self.index += 1
                continue
            if self.tokens[self.index] == "}":
                self.index += 1
                return
            raise ParseError(f"expected comma or }}, got {self.tokens[self.index]!r}")

    def _parse_alias(self) -> str | None:
        if self.index >= len(self.tokens) or self.tokens[self.index] != "as":
            return None
        self.index += 1
        if self.index >= len(self.tokens) or not self._is_segment(self.tokens[self.index]):
            raise ParseError("missing import alias")
        alias = self.tokens[self.index]
        self.index += 1
        return alias

    @staticmethod
    def _is_segment(token: str) -> bool:
        return token == "_" or IDENT_RE.fullmatch(token) is not None or token.startswith("r#")


def lex(source: str) -> list[Token]:
    tokens: list[Token] = []
    index = 0
    length = len(source)
    while index < length:
        char = source[index]
        if char.isspace():
            index += 1
            continue
        if source.startswith("//", index):
            newline = source.find("\n", index + 2)
            index = length if newline == -1 else newline + 1
            continue
        if source.startswith("/*", index):
            index = skip_block_comment(source, index)
            continue
        if char == '"':
            index = skip_quoted(source, index, char)
            continue
        if char == "'":
            lifetime = re.match(r"'([A-Za-z_][A-Za-z0-9_]*)", source[index:])
            if lifetime and not source.startswith(lifetime.group() + "'", index):
                end = index + len(lifetime.group())
                tokens.append(Token(lifetime.group(), index, end))
                index = end
            else:
                index = skip_quoted(source, index, char)
            continue
        if char == "r" and index + 1 < length and source[index + 1] == '"':
            index = skip_raw_string(source, index)
            continue
        if char == "r" and index + 2 < length and source[index + 1] == "#":
            quote = source.find('"', index + 2)
            if quote != -1:
                index = skip_raw_string(source, index, quote - index - 1)
                continue
        if source.startswith("::", index):
            tokens.append(Token("::", index, index + 2))
            index += 2
            continue
        raw_ident = source.startswith("r#", index)
        ident_start = index + 2 if raw_ident else index
        match = IDENT_RE.match(source, ident_start)
        if match:
            value = source[index : match.end()] if raw_ident else match.group()
            tokens.append(Token(value, index, match.end()))
            index = match.end()
            continue
        if char in "{};, *":
            if char != " ":
                tokens.append(Token(char, index, index + 1))
            index += 1
            continue
        tokens.append(Token(char, index, index + 1))
        index += 1
    return tokens


def skip_block_comment(source: str, index: int) -> int:
    depth = 1
    index += 2
    while index < len(source) and depth:
        if source.startswith("/*", index):
            depth += 1
            index += 2
        elif source.startswith("*/", index):
            depth -= 1
            index += 2
        else:
            index += 1
    return index


def skip_quoted(source: str, index: int, quote: str) -> int:
    index += 1
    while index < len(source):
        if source[index] == "\\":
            index += 2
        elif source[index] == quote:
            return index + 1
        else:
            index += 1
    return index


def skip_raw_string(source: str, index: int, hashes: int = 0) -> int:
    quote = source.find('"', index + 1)
    if quote == -1:
        return len(source)
    terminator = '"' + ("#" * hashes)
    end = source.find(terminator, quote + 1)
    return len(source) if end == -1 else end + len(terminator)


def find_use_spans(source: str) -> list[UseSpan]:
    tokens = lex(source)
    spans: list[UseSpan] = []
    scope_stack: list[int] = []
    for index, token in enumerate(tokens):
        if token.value == "{":
            scope_stack.append(token.start)
            continue
        if token.value == "}":
            if scope_stack:
                scope_stack.pop()
            continue
        if token.value != "use" or is_public_use(tokens, index):
            continue
        if index + 2 >= len(tokens) or tokens[index + 1].value != "crate":
            continue
        if tokens[index + 2].value != "::":
            continue
        end_index = index + 1
        while end_index < len(tokens) and tokens[end_index].value != ";":
            end_index += 1
        if end_index == len(tokens):
            continue
        start = token.start
        end = tokens[end_index].end
        if has_attribute_before(source, start) or "//" in source[start:end] or "/*" in source[start:end]:
            continue
        spans.append(
            UseSpan(
                start=start,
                end=end,
                token_index=index,
                scope=tuple(scope_stack),
                tokens=tuple(tokens[index + 1 : end_index]),
            )
        )
    return spans


def is_public_use(tokens: list[Token], index: int) -> bool:
    if index == 0:
        return False
    if tokens[index - 1].value == "pub":
        return True
    if tokens[index - 1].value != ")":
        return False
    depth = 0
    for token in reversed(tokens[max(0, index - 8) : index]):
        if token.value == ")":
            depth += 1
        elif token.value == "(":
            depth -= 1
            if depth == 0:
                return token is not None and any(
                    earlier.value == "pub" for earlier in tokens[max(0, index - 8) : index]
                )
    return False


def has_attribute_before(source: str, start: int) -> bool:
    index = start - 1
    while index >= 0 and source[index].isspace():
        index -= 1
    return index >= 0 and source[index] == "]"


def parse_imports(span: UseSpan) -> list[tuple[list[str], str | None, bool]]:
    values = [token.value for token in span.tokens]
    return UseTreeParser(values).parse()


def add_import(root: ImportNode, path: list[str], alias: str | None, is_self: bool) -> None:
    node = root
    for segment in path:
        node = node.children.setdefault(segment, ImportNode())
    if is_self:
        node.self_aliases.append(alias)
    else:
        node.terminal_aliases.append(alias)


def alias_text(name: str, alias: str | None) -> str:
    return name if alias is None else f"{name} as {alias}"


def render_node_entries(node: ImportNode) -> list[str]:
    entries: list[str] = []
    for alias in node.self_aliases:
        entries.append(alias_text("self", alias))
    for alias in node.terminal_aliases:
        entries.append(alias_text("self", alias))
    for name, child in node.children.items():
        entries.extend(render_child_entries(name, child))
    return entries


def render_child_entries(name: str, node: ImportNode) -> list[str]:
    if not node.children:
        aliases = node.self_aliases or node.terminal_aliases or [None]
        return [alias_text(name, alias) for alias in aliases]
    if not node.self_aliases and not node.terminal_aliases and len(node.children) == 1:
        child_name, child = next(iter(node.children.items()))
        return [f"{name}::{entry}" for entry in render_child_entries(child_name, child)]
    entries = render_node_entries(node)
    return [f"{name}::{{{', '.join(entries)}}}"]


def render_import(root: ImportNode, indent: str) -> str:
    crate = root.children.get("crate")
    if crate is None:
        raise ValueError("import tree has no crate root")
    entries = render_node_entries(crate)
    if not entries:
        raise ValueError("import tree is empty")
    rendered = ",\n".join(f"{indent}    {entry}" for entry in entries)
    return f"{indent}use crate::{{\n{rendered}\n{indent}}};"


def contiguous_runs(source: str, spans: list[UseSpan]) -> list[list[UseSpan]]:
    runs: list[list[UseSpan]] = []
    for span in spans:
        if not runs or span.scope != runs[-1][0].scope:
            runs.append([span])
            continue
        previous = runs[-1][-1]
        between = source[previous.end : span.start]
        if between.strip():
            runs.append([span])
        else:
            runs[-1].append(span)
    return [run for run in runs if len(run) >= 2]


def merge_source(source: str, path: Path) -> tuple[str, bool, list[str]]:
    spans = find_use_spans(source)
    replacements: list[tuple[int, int, str]] = []
    warnings: list[str] = []
    for run in contiguous_runs(source, spans):
        root = ImportNode()
        try:
            for span in run:
                for import_path, alias, is_self in parse_imports(span):
                    add_import(root, import_path, alias, is_self)
            line_start = source.rfind("\n", 0, run[0].start) + 1
            line_prefix = source[line_start : run[0].start]
            if line_prefix.strip():
                continue
            indent = re.match(r"[ \t]*", line_prefix).group()
            replacements.append((line_start, run[-1].end, render_import(root, indent)))
        except (ParseError, ValueError) as error:
            warnings.append(f"{path}: skipped import run at line {source.count(chr(10), 0, run[0].start) + 1}: {error}")

    updated = source
    for start, end, replacement in reversed(replacements):
        updated = updated[:start] + replacement + updated[end:]
    return updated, updated != source, warnings


def rust_files(paths: list[Path]) -> list[Path]:
    files: list[Path] = []
    for path in paths:
        if path.is_file() and path.suffix == ".rs":
            files.append(path)
        elif path.is_dir():
            files.extend(sorted(path.rglob("*.rs")))
    return sorted(set(files))


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("paths", nargs="*", type=Path, default=[Path("crates")])
    parser.add_argument("--write", action="store_true", help="write merged imports to disk")
    parser.add_argument(
        "--check",
        action="store_true",
        help="return a failure if any file would change",
    )
    args = parser.parse_args()
    if args.write and args.check:
        parser.error("--write and --check are mutually exclusive")

    changed = 0
    for path in rust_files(args.paths):
        source = path.read_text()
        updated, did_change, warnings = merge_source(source, path)
        for warning in warnings:
            print(warning, file=sys.stderr)
        if not did_change:
            continue
        changed += 1
        print(path)
        if args.write:
            path.write_text(updated)

    if not args.write and changed:
        print(f"{changed} file(s) would change; rerun with --write to apply", file=sys.stderr)
    return 1 if args.check and changed else 0


if __name__ == "__main__":
    raise SystemExit(main())
