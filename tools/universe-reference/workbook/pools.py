"""Generic candidate-pool workbook rows."""

from __future__ import annotations

from pathlib import Path

from workbook.data import load_json


def build_rows(root: Path) -> dict[str, list[dict]]:
    paths = load_json(root, "paths.json")
    blessings = load_json(root, "blessings.json")
    curios = load_json(root, "curios.json")
    occurrences = load_json(root, "occurrences.json")
    encounters = load_json(root, "encounter-groups.json")
    services = load_json(root, "services.json")
    definitions: list[tuple[str, str, list[str], str | None]] = [
        ("universe.pool.blessings.standard", "Blessing", [row["id"] for row in blessings], None),
        ("universe.pool.curios.standard", "Curio", [row["id"] for row in curios], None),
        ("universe.pool.occurrences.standard", "Occurrence", [row["id"] for row in occurrences], None),
        ("universe.pool.encounters.standard", "Encounter", [row["id"] for row in encounters], None),
        (
            "universe.pool.trailblaze-bonuses",
            "TrailblazeBonus",
            [row["id"] for row in services if row["kind"] == "TrailblazeBonus"],
            None,
        ),
    ]
    for path in paths:
        definitions.append((f"universe.pool.blessings.path.{path['id'].rsplit('.', 1)[1]}", "Blessing", path["blessing_ids"], None))
    for service in services:
        if service["kind"] == "BlessingShop":
            definitions.append((service["offer_pool_id"], "Shop", [row["id"] for row in blessings], "runtime.shop_eligibility"))
        elif service["kind"] == "CurioShop":
            definitions.append((service["offer_pool_id"], "Shop", [row["id"] for row in curios], "runtime.shop_eligibility"))
    definitions.sort(key=lambda item: item[0])
    if len({key for key, _, _, _ in definitions}) != len(definitions):
        raise ValueError("duplicate generic pool stable key")
    rows: dict[str, list[dict]] = {"UniverseContentPool": [], "UniverseContentPoolEntry": []}
    for pool_id, (stable_key, kind, entries, condition) in enumerate(definitions, start=1):
        rows["UniverseContentPool"].append({
            "id": pool_id,
            "stable_key": stable_key,
            "kind": kind,
            "ordering": "StableKeyAscending",
            "replacement": False,
        })
        for sequence, content_key in enumerate(sorted(entries), start=1):
            rows["UniverseContentPoolEntry"].append({
                "pool_id": pool_id,
                "sequence": sequence,
                "content_stable_key": content_key,
                "weight_decimal": "1",
                "condition": condition,
            })
    return rows
