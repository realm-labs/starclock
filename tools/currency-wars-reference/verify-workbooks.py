#!/usr/bin/env python3
"""Verify complete Goal 12 workbook structure and semantics."""

from __future__ import annotations

import argparse
from pathlib import Path

from workbook_authoring import semantic_digest, verify


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, default=Path.cwd())
    parser.add_argument("--directory", type=Path, required=True)
    args = parser.parse_args()
    counts = verify(args.root.resolve(), args.directory.resolve())
    total = sum(counts.values())
    print(
        f"Verified {len(counts)} Currency Wars sheets and {total} authored rows; "
        f"semantic digest {semantic_digest(args.directory.resolve())}."
    )


if __name__ == "__main__":
    main()
