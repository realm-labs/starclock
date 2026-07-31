#!/usr/bin/env python3
"""Create complete new Goal 16 workbooks; never patch an existing target."""

from __future__ import annotations

import argparse
from pathlib import Path

from workbook_authoring import author, byte_digests, semantic_digest


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, default=Path.cwd())
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--templates", type=Path, required=True)
    args = parser.parse_args()
    root = args.root.resolve()
    output = args.output.resolve()
    templates = args.templates.resolve()
    counts = author(root, output, templates)
    print(
        f"Authored {len(counts)} Galactic Baseballer sheets with "
        f"openpyxl==3.1.5; {sum(counts.values())} rows; "
        f"semantic digest {semantic_digest(output)}; "
        f"byte digests {byte_digests(output)}."
    )


if __name__ == "__main__":
    main()
