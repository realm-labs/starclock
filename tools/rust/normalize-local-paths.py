#!/usr/bin/env python3
"""Move inline ``super::``/``self::`` paths to local imports.

This is intentionally a conservative source rewrite. It uses the lexer from
``merge-crate-imports.py`` so comments, strings and attributes are not
mistaken for Rust paths. Parent imports are inserted at the containing module
scope; function-local paths therefore do not create function-local imports.
"""

from __future__ import annotations

import argparse
import importlib.util
import re
import sys
from dataclasses import dataclass
from pathlib import Path


MERGE_SCRIPT = Path(__file__).with_name("merge-crate-imports.py")
SPEC = importlib.util.spec_from_file_location("merge_crate_imports", MERGE_SCRIPT)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError(f"cannot load {MERGE_SCRIPT}")
MERGE = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = MERGE
SPEC.loader.exec_module(MERGE)


@dataclass(frozen=True)
class PathUse:
    start: int
    end: int
    kind: str
    segment: str | None
    scope: tuple[int, ...]


def module_opening(tokens: list[MERGE.Token], index: int) -> bool:
    for token in reversed(tokens[max(0, index - 8) : index]):
        if token.value in {";", "{", "}"}:
            return False
        if token.value == "mod":
            return True
    return False


def scan(
    source: str,
) -> tuple[
    list[PathUse],
    dict[tuple[int, ...], list[int]],
    dict[tuple[int, ...], list[tuple[int, int]]],
]:
    tokens = MERGE.lex(source)
    paths: list[PathUse] = []
    module_openings: dict[tuple[int, ...], list[int]] = {}
    use_spans: dict[tuple[int, ...], list[tuple[int, int]]] = {}
    all_braces: list[tuple[int, bool]] = []
    modules: list[int] = []

    for index, token in enumerate(tokens):
        if token.value == "{":
            is_module = module_opening(tokens, index)
            all_braces.append((token.start, is_module))
            if is_module:
                modules.append(token.start)
                module_openings.setdefault(tuple(modules), []).append(token.start)
            continue
        if token.value == "}":
            if all_braces:
                _, is_module = all_braces.pop()
                if is_module:
                    modules.pop()
            continue
        if token.value == "use":
            end_index = index + 1
            while end_index < len(tokens) and tokens[end_index].value != ";":
                end_index += 1
            if end_index < len(tokens):
                use_spans.setdefault(tuple(modules), []).append(
                    (token.start, tokens[end_index].end)
                )
            continue
        if token.value not in {"super", "self"}:
            continue
        if index and tokens[index - 1].value in {"use", "::"}:
            continue
        if index + 1 >= len(tokens) or tokens[index + 1].value != "::":
            continue
        if token.value == "self":
            paths.append(PathUse(token.start, tokens[index + 1].end, "self", None, tuple(modules)))
            continue
        if index + 2 >= len(tokens):
            continue
        segment = tokens[index + 2].value
        if segment == "super":
            raise ValueError(f"unsupported super::super path at byte {token.start}")
        paths.append(
            PathUse(
                token.start,
                tokens[index + 2].end,
                "super",
                segment,
                tuple(modules),
            )
        )
    return paths, module_openings, use_spans


def direct_binding_exists(source: str, name: str) -> bool:
    declaration = re.compile(
        rf"\b(?:mod|struct|enum|trait|type|const|static|fn)\s+{re.escape(name)}\b"
    )
    if declaration.search(source):
        return True
    direct_use = re.compile(
        rf"(?:^|[{{,]|::)\s*{re.escape(name)}\s*(?:as\s+\w+\s*)?(?:[,}};]|$)",
        re.MULTILINE,
    )
    return any(direct_use.search(line) for line in source.splitlines() if line.lstrip().startswith("use "))


def alias_for(source: str, segment: str, used: set[str]) -> str:
    natural = segment
    if not direct_binding_exists(source, natural) and natural not in used:
        used.add(natural)
        return natural
    candidate = f"Parent{natural}" if natural[:1].isupper() else f"parent_{natural}"
    suffix = 2
    while candidate in used:
        candidate = f"{candidate}_{suffix}"
        suffix += 1
    used.add(candidate)
    return candidate


def indentation(source: str, position: int, extra: str = "") -> str:
    line_start = source.rfind("\n", 0, position) + 1
    prefix = source[line_start:position]
    return re.match(r"[ \t]*", prefix).group() + extra


def render_imports(imports: list[tuple[str, str]]) -> str:
    entries = [name if name == alias else f"{name} as {alias}" for name, alias in imports]
    return "use super::{" + ", ".join(entries) + "};"


def normalize_source(source: str, path: Path) -> tuple[str, bool, list[str]]:
    paths, module_openings, use_spans = scan(source)
    if not paths:
        return source, False, []

    used: set[str] = set()
    aliases: dict[tuple[tuple[int, ...], str], str] = {}
    replacements: list[tuple[int, int, str]] = []
    imports: dict[tuple[int, ...], list[tuple[str, str]]] = {}
    for path_use in paths:
        if path_use.kind == "self":
            replacements.append((path_use.start, path_use.end, ""))
            continue
        assert path_use.segment is not None
        key = (path_use.scope, path_use.segment)
        alias = aliases.get(key)
        if alias is None:
            alias = alias_for(source, path_use.segment, used)
            aliases[key] = alias
            imports.setdefault(path_use.scope, []).append((path_use.segment, alias))
        replacements.append((path_use.start, path_use.end, alias))

    for scope, scope_imports in imports.items():
        if scope:
            opening = scope[-1]
            insert_at = opening + 1
            indent = indentation(source, opening, "    ")
            insertion = f"\n{indent}{render_imports(scope_imports)}"
        else:
            scope_uses = use_spans.get(scope, [])
            if scope_uses:
                last_use_start, last_use_end = scope_uses[-1]
                insert_at = last_use_end
                indent = indentation(source, last_use_start)
                insertion = f"\n{indent}{render_imports(scope_imports)}"
            else:
                insert_at = 0
                indent = ""
                insertion = f"{render_imports(scope_imports)}\n"
        replacements.append((insert_at, insert_at, insertion))

    updated = source
    for start, end, replacement in sorted(replacements, reverse=True):
        updated = updated[:start] + replacement + updated[end:]
    return updated, updated != source, []


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
    parser.add_argument("--write", action="store_true")
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    if args.write and args.check:
        parser.error("--write and --check are mutually exclusive")

    changed = 0
    for path in rust_files(args.paths):
        source = path.read_text()
        updated, did_change, warnings = normalize_source(source, path)
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
