#!/usr/bin/env python3

from __future__ import annotations

import argparse
from pathlib import Path

from workbook_authoring import author


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, default=Path.cwd())
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--templates", type=Path, required=True)
    args = parser.parse_args()
    counts = author(args.root.resolve(), args.output.resolve(), args.templates.resolve())
    print(f"Authored {len(counts)} Fate sheets / {sum(counts.values())} rows with openpyxl 3.1.5.")


if __name__ == "__main__":
    main()
