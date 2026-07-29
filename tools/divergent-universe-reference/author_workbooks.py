#!/usr/bin/env python3
"""Author the complete Goal 11 workbooks without overwriting existing files."""

from __future__ import annotations

import argparse
from pathlib import Path

from workbook_authoring import author, semantic_digest


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("root", nargs="?", default=".")
    parser.add_argument(
        "--output",
        default="config/divergent-universe/data",
    )
    args = parser.parse_args()
    root = Path(args.root).resolve()
    output = (root / args.output).resolve()
    counts = author(root, output)
    print(
        "Authored Divergent Universe workbooks: "
        f"{len(counts)} tables, {sum(counts.values())} rows, "
        f"semantic_sha256={semantic_digest(output)}"
    )


if __name__ == "__main__":
    main()
