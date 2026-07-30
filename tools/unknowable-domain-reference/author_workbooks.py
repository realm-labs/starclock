"""Generate complete new Goal 10 workbooks; never patch existing files."""

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
        f"Authored {len(counts)} Unknowable Domain tables with openpyxl; "
        f"{sum(counts.values())} rows; semantic digest {semantic_digest(output)}."
    )


if __name__ == "__main__":
    main()
