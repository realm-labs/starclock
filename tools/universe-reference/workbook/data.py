"""Normalized-pack loading and stable workbook identity helpers."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path

def load_json(root: Path, name: str):
    path = root / "content-reference" / "standard-universe-v1" / name
    return json.loads(path.read_text(encoding="utf-8"))


def canonical_json(value: object) -> str:
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":"))


def joined(values: list[object]) -> str:
    return "|".join(str(value) for value in values)


def optional_joined(values: list[object]) -> str | None:
    return joined(values) or None


def stable_ids(records: list[dict]) -> dict[str, int]:
    keys = sorted(record["id"] for record in records)
    if len(keys) != len(set(keys)):
        raise ValueError("duplicate stable keys cannot receive workbook IDs")
    return {key: offset for offset, key in enumerate(keys, start=1)}


def sha256_file(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()
