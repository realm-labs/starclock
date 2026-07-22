"""Generate complete new Universe workbooks; never patch existing files."""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

SCRIPT_ROOT = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_ROOT))

from workbook.common import author, semantic_digest  # noqa: E402
from workbook.blessings import build_rows as build_blessing_rows  # noqa: E402
from workbook.topology import build_rows as build_topology_rows  # noqa: E402


def build_rows(root: Path, empty: bool) -> dict[str, list[dict]]:
    if empty:
        return {}
    row_sets = (build_topology_rows(root), build_blessing_rows(root))
    combined: dict[str, list[dict]] = {}
    for rows in row_sets:
        overlap = sorted(set(combined) & set(rows))
        if overlap:
            raise ValueError(f"duplicate workbook row builders: {overlap}")
        combined.update(rows)
    return combined


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, default=Path.cwd())
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--empty", action="store_true")
    args = parser.parse_args()
    root = args.root.resolve()
    output = args.output.resolve()
    counts = author(root, output, build_rows(root, args.empty))
    print(f"Authored {len(counts)} Universe sheets with openpyxl; semantic digest {semantic_digest(output)}.")


if __name__ == "__main__":
    main()
