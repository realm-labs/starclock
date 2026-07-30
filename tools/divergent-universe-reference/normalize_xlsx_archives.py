#!/usr/bin/env python3
"""Canonicalize Sora template ZIP metadata for deterministic commits."""

from __future__ import annotations

import re
import sys
import zipfile
from pathlib import Path


def normalize(path: Path) -> None:
    temporary = path.with_suffix(f"{path.suffix}.canonical")
    with zipfile.ZipFile(path, "r") as source:
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
                    rb"(<dcterms:modified[^>]*>)[^<]*(</dcterms:modified>)",
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
    temporary.replace(path)


def main() -> None:
    if len(sys.argv) < 2:
        raise SystemExit("usage: normalize_xlsx_archives.py PATH [PATH ...]")
    for argument in sys.argv[1:]:
        normalize(Path(argument))


if __name__ == "__main__":
    main()
