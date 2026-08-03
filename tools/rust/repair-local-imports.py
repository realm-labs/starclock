#!/usr/bin/env python3
"""Move local ``use super::...`` declarations out of nested Rust scopes.

The import normalizer puts stable local imports at module scope.  This repair
pass handles imports left behind by an earlier rewrite that inserted them in a
function or another non-module block.
"""

from __future__ import annotations

import argparse
import importlib.util
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
class ImportSpan:
    start: int
    end: int
    scope: tuple[int, ...]
    nested: bool


def module_opening(tokens: list[MERGE.Token], index: int) -> bool:
    for token in reversed(tokens[max(0, index - 8) : index]):
        if token.value in {";", "{", "}"}:
            return False
        if token.value == "mod":
            return True
    return False


def scan(source: str) -> tuple[list[ImportSpan], dict[tuple[int, ...], list[tuple[int, int]]]]:
    tokens = MERGE.lex(source)
    imports: list[ImportSpan] = []
    module_uses: dict[tuple[int, ...], list[tuple[int, int]]] = {}
    braces: list[tuple[int, bool]] = []
    modules: list[int] = []

    for index, token in enumerate(tokens):
        if token.value == "{":
            is_module = module_opening(tokens, index)
            braces.append((token.start, is_module))
            if is_module:
                modules.append(token.start)
            continue
        if token.value == "}":
            if braces:
                _, is_module = braces.pop()
                if is_module:
                    modules.pop()
            continue
        if token.value != "use":
            continue
        end_index = index + 1
        while end_index < len(tokens) and tokens[end_index].value != ";":
            end_index += 1
        if end_index >= len(tokens):
            continue
        scope = tuple(modules)
        if len(braces) == len(modules):
            module_uses.setdefault(scope, []).append(
                (token.start, tokens[end_index].end)
            )
        if index + 1 >= len(tokens) or tokens[index + 1].value != "super":
            continue
        imports.append(
            ImportSpan(
                token.start,
                tokens[end_index].end,
                scope,
                len(braces) > len(modules),
            )
        )
    return imports, module_uses


def line_indent(source: str, position: int) -> str:
    line_start = source.rfind("\n", 0, position) + 1
    prefix = source[line_start:position]
    return prefix[: len(prefix) - len(prefix.lstrip(" \t"))]


def repair_source(source: str) -> tuple[str, bool]:
    imports, module_uses = scan(source)
    nested = [item for item in imports if item.nested]
    if not nested:
        return source, False

    module_openings = {scope: scope[-1] for scope in {item.scope for item in nested} if scope}
    insertion: dict[tuple[int, ...], int] = {}
    for item in nested:
        if item.scope in insertion:
            continue
        uses = module_uses.get(item.scope)
        if uses:
            insertion[item.scope] = uses[-1][1]
        elif item.scope:
            insertion[item.scope] = module_openings[item.scope] + 1
        else:
            insertion[item.scope] = 0

    removals: list[tuple[int, int, str]] = []
    additions: dict[int, list[str]] = {}
    for item in nested:
        statement = source[item.start : item.end].strip()
        removals.append((item.start, item.end, ""))
        position = insertion[item.scope]
        indent = line_indent(source, position)
        if item.scope:
            indent += "    "
        additions.setdefault(position, []).append(f"{indent}{statement}")

    edits = removals[:]
    for position, statements in additions.items():
        prefix = "\n" if position else ""
        edits.append((position, position, prefix + "\n".join(statements) + "\n"))

    updated = source
    for start, end, replacement in sorted(edits, reverse=True):
        updated = updated[:start] + replacement + updated[end:]
    return updated, updated != source


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
        updated, did_change = repair_source(source)
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
