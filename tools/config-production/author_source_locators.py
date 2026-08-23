"""Author stable upstream locators required by cross-mode build joins."""

from __future__ import annotations

import argparse
from pathlib import Path

import character_workbook_support as support


ROOT = Path(__file__).resolve().parents[2]
REFERENCE = ROOT / "content-reference" / "v4.4"


def identity_ids() -> dict[str, int]:
    _, rows = support.workbook_rows("ContentIdentity")
    return {str(row["stable_key"]): int(row["id"]) for row in rows}


def expected_rows(
    table: str,
    reference_file: str,
    source_field: str,
    output_field: str,
) -> list[dict[str, object]]:
    identities = identity_ids()
    source_by_id = {}
    for row in support.read_json(REFERENCE / reference_file):
        identity = identities.get(row["id"])
        if identity is None:
            continue
        values = row[source_field]
        source = int(values[0] if isinstance(values, list) else values)
        source_by_id[identity] = source
    _, existing = support.workbook_rows(table)
    result = []
    for row in existing:
        authored = dict(row)
        identity = int(row["id"])
        try:
            authored[output_field] = source_by_id[identity]
        except KeyError as error:
            raise ValueError(f"{table} {identity} has no released source locator") from error
        result.append(authored)
    return result


def main() -> None:
    parser = argparse.ArgumentParser()
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--write", action="store_true")
    mode.add_argument("--check", action="store_true")
    arguments = parser.parse_args()
    tables = {
        "Character": expected_rows(
            "Character", "characters.json", "source_avatar_ids", "source_avatar_id"
        ),
        "LightCone": expected_rows(
            "LightCone", "light-cones.json", "source_equipment_id", "source_equipment_id"
        ),
    }
    if arguments.write:
        for name, rows in tables.items():
            support.write_rows(name, rows)
    for name, rows in tables.items():
        support.check_exact(name, rows)
    print("Production character and Light Cone source locators are deterministic.")


if __name__ == "__main__":
    main()
