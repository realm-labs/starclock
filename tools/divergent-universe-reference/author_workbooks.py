#!/usr/bin/env python3
"""Author complete Divergent Universe workbooks without overwriting targets."""

from __future__ import annotations

import argparse
import os
import subprocess
import sys
from pathlib import Path

from workbook_authoring import (
    WORKBOOKS,
    author,
    normalize_archive,
    semantic_digest,
    verify,
)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("root", nargs="?", default=".")
    parser.add_argument(
        "--output",
        default="config/divergent-universe/data",
    )
    parser.add_argument(
        "--raw-child",
        action="store_true",
        help=argparse.SUPPRESS,
    )
    args = parser.parse_args()
    root = Path(args.root).resolve()
    output = (root / args.output).resolve()
    if args.raw_child:
        author(root, output)
        return
    command = [
        sys.executable,
        str(Path(__file__).resolve()),
        str(root),
        "--output",
        str(output),
        "--raw-child",
    ]
    environment = {**os.environ, "PYTHONDONTWRITEBYTECODE": "1"}
    subprocess.run(command, check=True, env=environment)
    for workbook in WORKBOOKS:
        normalize_archive(output / workbook)
    counts = verify(root, output)
    print(
        "Authored Divergent Universe workbooks: "
        f"{len(counts)} tables, {sum(counts.values())} rows, "
        f"semantic_sha256={semantic_digest(output)}"
    )


if __name__ == "__main__":
    main()
