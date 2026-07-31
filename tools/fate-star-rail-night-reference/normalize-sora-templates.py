#!/usr/bin/env python3
"""Canonicalize Sora-created template archives for byte-stable evidence."""

from __future__ import annotations

import argparse
import re
import zipfile
from pathlib import Path


def normalize_archive(file: Path) -> None:
    temporary = file.with_suffix(f"{file.suffix}.canonical")
    with zipfile.ZipFile(file, "r") as source:
        members = [(name, source.read(name)) for name in sorted(source.namelist())]
    with zipfile.ZipFile(
        temporary,
        "w",
        compression=zipfile.ZIP_DEFLATED,
        compresslevel=9,
    ) as target:
        for name, payload in members:
            if name == "docProps/core.xml":
                payload = re.sub(
                    rb"(<dcterms:(?:created|modified)[^>]*>)[^<]*(</dcterms:(?:created|modified)>)",
                    rb"\g<1>2000-01-01T00:00:00Z\g<2>",
                    payload,
                )
            info = zipfile.ZipInfo(name, date_time=(2000, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            info.create_system = 0
            info.external_attr = 0
            target.writestr(
                info,
                payload,
                compress_type=zipfile.ZIP_DEFLATED,
                compresslevel=9,
            )
    temporary.replace(file)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("directory", type=Path)
    directory = parser.parse_args().directory.resolve()
    files = sorted(directory.glob("*.xlsx"))
    if not files:
        raise FileNotFoundError(f"no Sora templates in {directory}")
    for file in files:
        normalize_archive(file)
    print(f"Canonicalized {len(files)} Sora Excel templates.")


if __name__ == "__main__":
    main()
