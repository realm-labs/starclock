#!/usr/bin/env python3
"""Verify complete Goal 16 workbook structure and semantics."""

from __future__ import annotations

import argparse
from pathlib import Path

from workbook_authoring import byte_digests, semantic_digest, verify


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, default=Path.cwd())
    parser.add_argument("--directory", type=Path, required=True)
    args = parser.parse_args()
    directory = args.directory.resolve()
    counts = verify(args.root.resolve(), directory)
    print(
        f"Verified {len(counts)} Galactic Baseballer sheets and "
        f"{sum(counts.values())} rows; semantic digest "
        f"{semantic_digest(directory)}; byte digests "
        f"{byte_digests(directory)}."
    )


if __name__ == "__main__":
    main()
