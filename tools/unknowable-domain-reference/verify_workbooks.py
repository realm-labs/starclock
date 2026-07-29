"""Verify complete Goal 10 workbooks against normalized rows and Sora schema."""

from __future__ import annotations

import argparse
from pathlib import Path

from workbook_authoring import author, semantic_digest, verify


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, default=Path.cwd())
    parser.add_argument(
        "--data-root",
        type=Path,
        default=Path("config/unknowable-domain/data"),
    )
    args = parser.parse_args()
    root = args.root.resolve()
    data_root = (root / args.data_root).resolve()
    counts = verify(root, data_root)
    try:
        author(root, data_root)
    except FileExistsError:
        pass
    else:
        raise ValueError("authoring did not refuse to overwrite existing workbooks")
    print(
        f"Verified {sum(counts.values())} authored rows across "
        f"{len(counts)} tables; semantic digest {semantic_digest(data_root)}."
    )


if __name__ == "__main__":
    main()
