#!/usr/bin/env python3
"""Verify complete Goal 11 workbook structure and semantic content."""

from __future__ import annotations

import argparse
from pathlib import Path

from workbook_authoring import semantic_digest, verify


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("root", nargs="?", default=".")
    parser.add_argument(
        "--directory",
        default="config/divergent-universe/data",
    )
    args = parser.parse_args()
    root = Path(args.root).resolve()
    directory = (root / args.directory).resolve()
    counts = verify(root, directory)
    print(
        "Verified Divergent Universe workbooks: "
        f"{len(counts)} tables, {sum(counts.values())} rows, "
        f"semantic_sha256={semantic_digest(directory)}"
    )


if __name__ == "__main__":
    main()
