#!/usr/bin/env python3
"""Create complete new Goal 17 workbooks; never patch an existing target."""

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
    counts = author(args.root.resolve(), args.output.resolve(), args.templates.resolve())
    print(
        f"Authored {len(counts)} Memory of Chaos sheets with openpyxl==3.1.5; "
        f"{sum(counts.values())} rows; semantic digest {semantic_digest(args.output.resolve())}; "
        f"byte digests {byte_digests(args.output.resolve())}."
    )


if __name__ == "__main__":
    main()
