#!/usr/bin/env python3
"""Create complete new Goal 12 workbooks; never patch an existing target."""

from __future__ import annotations

import argparse
from pathlib import Path

from workbook_authoring import author, semantic_digest


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, default=Path.cwd())
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    root = args.root.resolve()
    output = args.output.resolve()
    counts = author(root, output)
    print(
        f"Authored {len(counts)} Currency Wars sheets with openpyxl; "
        f"semantic digest {semantic_digest(output)}."
    )


if __name__ == "__main__":
    main()
