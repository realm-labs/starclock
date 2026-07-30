#!/usr/bin/env python3
"""Canonicalize Sora-created template archives for byte-stable evidence."""

from __future__ import annotations

import argparse
from pathlib import Path

from workbook_authoring import normalize_archive


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("directory", type=Path)
    args = parser.parse_args()
    directory = args.directory.resolve()
    files = sorted(directory.glob("*.xlsx"))
    if not files:
        raise FileNotFoundError(f"no Sora templates in {directory}")
    for file in files:
        normalize_archive(file)
    print(f"Canonicalized {len(files)} Sora Excel templates.")


if __name__ == "__main__":
    main()
