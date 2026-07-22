"""Path, Resonance and Blessing workbook rows."""

from __future__ import annotations

from pathlib import Path

from workbook.data import load_json, optional_joined, stable_ids


def build_rows(root: Path) -> dict[str, list[dict]]:
    paths = load_json(root, "paths.json")
    resonances = load_json(root, "resonances.json")
    blessings = load_json(root, "blessings.json")
    levels = load_json(root, "blessing-levels.json")
    path_ids = stable_ids(paths)
    resonance_ids = stable_ids(resonances)
    blessing_ids = stable_ids(blessings)
    level_ids = stable_ids(levels)
    rows: dict[str, list[dict]] = {
        "UniversePath": [],
        "UniversePathBlessing": [],
        "UniverseResonance": [],
        "UniverseResonanceParameter": [],
        "UniverseBlessing": [],
        "UniverseBlessingPrerequisite": [],
        "UniverseBlessingLevel": [],
        "UniverseBlessingParameter": [],
    }
    for record in paths:
        path_id = path_ids[record["id"]]
        rows["UniversePath"].append({
            "id": path_id,
            "stable_key": record["id"],
            "buff_type": str(record["buff_type"]),
            "name_en": record["name_en"],
            "name_zh_cn": record["name_zh_cn"],
            "summary_en": record["summary_en"],
            "summary_zh_cn": record["summary_zh_cn"],
            "unlock_policy_stable_key": record["unlock_policy_id"],
        })
        for sequence, blessing_key in enumerate(record["blessing_ids"], start=1):
            if blessing_key not in blessing_ids:
                raise ValueError(f"{record['id']} references missing Blessing {blessing_key}")
            rows["UniversePathBlessing"].append({
                "path_id": path_id,
                "sequence": sequence,
                "blessing_stable_key": blessing_key,
            })
    for record in resonances:
        resonance_id = resonance_ids[record["id"]]
        if len(record["rule_ids"]) != 1:
            raise ValueError(f"{record['id']} requires exactly one rule")
        rows["UniverseResonance"].append({
            "id": resonance_id,
            "stable_key": record["id"],
            "path_id": path_ids[record["path_id"]],
            "kind": record["kind"],
            "threshold": record["threshold"],
            "energy_max_decimal": record["energy_max"],
            "initial_energy_decimal": record["initial_energy"],
            "name_en": record["name_en"],
            "name_zh_cn": record["name_zh_cn"],
            "summary_en": record["summary_en"],
            "summary_zh_cn": record["summary_zh_cn"],
            "mechanic_tags": optional_joined(record["mechanic_tags"]),
            "source_binding_key": record["source_binding_key"],
            "rule_stable_key": record["rule_ids"][0],
        })
        for parameter in record["parameter_values"]:
            rows["UniverseResonanceParameter"].append({
                "resonance_id": resonance_id,
                "sequence": parameter["index"],
                "value_decimal": parameter["value"],
            })
    for record in blessings:
        blessing_id = blessing_ids[record["id"]]
        if len(record["rule_ids"]) != 1:
            raise ValueError(f"{record['id']} requires exactly one definition rule")
        rows["UniverseBlessing"].append({
            "id": blessing_id,
            "stable_key": record["id"],
            "path_id": path_ids[record["path_id"]],
            "rarity": record["rarity"],
            "name_en": record["name_en"],
            "name_zh_cn": record["name_zh_cn"],
            "summary_en": record["summary_en"],
            "summary_zh_cn": record["summary_zh_cn"],
            "pool_tags": optional_joined(record["pool_tags"]),
            "mechanic_tags": optional_joined(record["mechanic_tags"]),
            "rule_stable_key": record["rule_ids"][0],
            "source_description_sha256_en": record["source_description_sha256_en"],
            "source_description_sha256_zh_cn": record["source_description_sha256_zh_cn"],
        })
        for sequence, prerequisite in enumerate(record["prerequisite_ids"], start=1):
            rows["UniverseBlessingPrerequisite"].append({
                "blessing_id": blessing_id,
                "sequence": sequence,
                "prerequisite_stable_key": prerequisite,
            })
    for record in levels:
        level_id = level_ids[record["id"]]
        if len(record["rule_ids"]) != 1:
            raise ValueError(f"{record['id']} requires exactly one level rule")
        rows["UniverseBlessingLevel"].append({
            "id": level_id,
            "stable_key": record["id"],
            "blessing_id": blessing_ids[record["blessing_id"]],
            "level": record["level"],
            "source_binding_key": record["source_binding_key"],
            "rule_stable_key": record["rule_ids"][0],
            "name_en": record["name_en"],
            "name_zh_cn": record["name_zh_cn"],
            "summary_en": record["summary_en"],
            "summary_zh_cn": record["summary_zh_cn"],
        })
        for parameter in record["parameter_values"]:
            rows["UniverseBlessingParameter"].append({
                "blessing_level_id": level_id,
                "sequence": parameter["index"],
                "value_decimal": parameter["value"],
            })
    return rows
