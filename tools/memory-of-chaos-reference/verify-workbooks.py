#!/usr/bin/env python3
"""Verify complete Goal 17 workbook structure and semantics."""

from __future__ import annotations

import argparse
from pathlib import Path

from workbook_authoring import byte_digests, semantic_digest, verify


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, default=Path.cwd())
    parser.add_argument("--directory", type=Path, required=True)
    parser.add_argument("--templates", type=Path, required=True)
    args = parser.parse_args()
    counts = verify(args.root.resolve(), args.directory.resolve(), args.templates.resolve())
    print(
        f"Verified {len(counts)} Memory of Chaos sheets and {sum(counts.values())} rows; "
        f"semantic digest {semantic_digest(args.directory.resolve())}; "
        f"byte digests {byte_digests(args.directory.resolve())}."
    )


if __name__ == "__main__":
    main()
