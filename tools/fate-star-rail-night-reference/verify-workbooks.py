#!/usr/bin/env python3

from __future__ import annotations

import argparse
import hashlib
import tempfile
from pathlib import Path

from workbook_authoring import author, verify


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, default=Path.cwd())
    parser.add_argument("--directory", type=Path, required=True)
    parser.add_argument("--templates", type=Path, required=True)
    args = parser.parse_args()
    root = args.root.resolve()
    directory = args.directory.resolve()
    templates = args.templates.resolve()
    counts = verify(root, directory, templates)
    with tempfile.TemporaryDirectory(prefix="g19-workbook-a-") as first_temp, tempfile.TemporaryDirectory(prefix="g19-workbook-b-") as second_temp:
        first = Path(first_temp)
        second = Path(second_temp)
        author(root, first, templates)
        author(root, second, templates)
        for workbook in sorted(directory.glob("*.xlsx")):
            if workbook.read_bytes() != (first / workbook.name).read_bytes() or workbook.read_bytes() != (second / workbook.name).read_bytes():
                raise ValueError(f"{workbook.name}: byte-stable generation drift")
    digests = {path.name: sha256(path) for path in sorted(directory.glob("*.xlsx"))}
    print(f"Verified {len(counts)} Fate sheets / {sum(counts.values())} rows and four byte-identical workbooks: {digests}")


if __name__ == "__main__":
    main()
