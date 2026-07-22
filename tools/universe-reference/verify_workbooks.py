"""Verify authored Universe workbooks against normalized builders and Sora output."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

SCRIPT_ROOT = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_ROOT))

from author_workbooks import build_rows  # noqa: E402
from workbook.common import semantic_digest, verify  # noqa: E402
from workbook.data import load_json  # noqa: E402


def debug_count(directory: Path, table: str) -> int:
    payload = json.loads((directory / f"{table}.json").read_text(encoding="utf-8"))
    return len(payload["table"]["rows"])


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, default=Path.cwd())
    parser.add_argument("--data-root", type=Path, required=True)
    parser.add_argument("--debug-root", type=Path)
    args = parser.parse_args()
    root = args.root.resolve()
    data_root = args.data_root.resolve()
    expected_rows = build_rows(root, False)
    actual = verify(root, data_root)
    expected = {table: len(expected_rows.get(table, [])) for table in actual}
    if actual != expected:
        differences = {table: [expected[table], actual[table]] for table in actual if actual[table] != expected[table]}
        raise ValueError(f"workbook row-count drift: {differences}")
    if args.debug_root:
        debug_root = args.debug_root.resolve()
        differences = {
            table: [count, debug_count(debug_root, table)]
            for table, count in expected.items()
            if debug_count(debug_root, table) != count
        }
        if differences:
            raise ValueError(f"Sora debug row-count drift: {differences}")
    enemy_keys = {record["id"] for record in load_json(root, "../v4.4/enemy-variants.json")}
    used_enemy_keys = {
        row["enemy_variant_stable_key"]
        for table in ("UniverseDifficultyEnemy", "UniverseEncounterWaveEnemy")
        for row in expected_rows[table]
    }
    missing = sorted(used_enemy_keys - enemy_keys)
    if missing:
        raise ValueError(f"workbooks reference unknown Goal 01 enemy variants: {missing}")
    content_keys = {row["content_stable_key"] for row in expected_rows["UniverseContentAudit"]}
    source_ids = {row["id"] for row in expected_rows["UniverseSourceRecord"]}
    pool_keys = {row["stable_key"] for row in expected_rows["UniverseContentPool"]}
    for row in expected_rows["UniverseContentAudit"]:
        provenance = {int(value) for value in row["provenance_ids"].split("|")}
        if not provenance or not provenance <= source_ids:
            raise ValueError(f"{row['content_stable_key']} has invalid workbook provenance")
    for row in expected_rows["UniverseContentPoolEntry"]:
        if row["content_stable_key"] not in content_keys:
            raise ValueError(f"pool entry references unaudited content {row['content_stable_key']}")
    for row in expected_rows["UniverseService"]:
        offer_pool = row["offer_pool_stable_key"]
        if offer_pool and offer_pool not in pool_keys:
            raise ValueError(f"service references missing pool {offer_pool}")
    if sum(row["required"] for row in expected_rows["UniverseCoverage"]) != len(content_keys):
        raise ValueError("coverage denominator differs from audited content")
    if {row["source_record_stable_key"] for row in expected_rows["UniverseMechanicRule"]} - content_keys:
        raise ValueError("mechanic rule references unaudited content")
    print(
        f"Verified {sum(expected.values())} authored rows across {len(expected)} Universe sheets; "
        f"{len(used_enemy_keys)} enemy stable keys close; semantic digest {semantic_digest(data_root)}."
    )


if __name__ == "__main__":
    main()
