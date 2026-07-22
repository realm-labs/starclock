"""Curio definition and lifecycle-state workbook rows."""

from __future__ import annotations

from pathlib import Path

from workbook.data import load_json, optional_joined, stable_ids


def build_rows(root: Path) -> dict[str, list[dict]]:
    curios = load_json(root, "curios.json")
    states = load_json(root, "curio-states.json")
    curio_ids = stable_ids(curios)
    state_ids = stable_ids(states)
    rows: dict[str, list[dict]] = {
        "UniverseCurio": [],
        "UniverseCurioState": [],
        "UniverseCurioParameter": [],
    }
    for record in curios:
        if len(record["rule_ids"]) != 1:
            raise ValueError(f"{record['id']} requires exactly one definition rule")
        if record["initial_state_id"] not in state_ids:
            raise ValueError(f"{record['id']} has a missing initial state")
        rows["UniverseCurio"].append({
            "id": curio_ids[record["id"]],
            "stable_key": record["id"],
            "initial_state_stable_key": record["initial_state_id"],
            "handbook_order": record["handbook_order"],
            "name_en": record["name_en"],
            "name_zh_cn": record["name_zh_cn"],
            "summary_en": record["summary_en"],
            "summary_zh_cn": record["summary_zh_cn"],
            "tags": optional_joined(record["tags"]),
            "pool_tags": optional_joined(record["pool_tags"]),
            "rule_stable_key": record["rule_ids"][0],
        })
    for record in states:
        if len(record["rule_ids"]) != 1:
            raise ValueError(f"{record['id']} requires exactly one state rule")
        curio_id = curio_ids[record["curio_id"]]
        state_id = state_ids[record["id"]]
        replacement = record["replacement_curio_id"]
        rows["UniverseCurioState"].append({
            "id": state_id,
            "stable_key": record["id"],
            "curio_id": curio_id,
            "state_kind": record["state_kind"],
            "charges_decimal": record["charges"] or None,
            "charge_parameter_index": record["charge_parameter_index"],
            "next_state_id": state_ids[record["next_state_id"]] if record["next_state_id"] else None,
            "repair_state_id": state_ids[record["repair_state_id"]] if record["repair_state_id"] else None,
            "replacement_curio_id": curio_ids[replacement] if replacement else None,
            "source_effect_id": record["source_effect_id"],
            "rule_stable_key": record["rule_ids"][0],
            "name_en": record["name_en"],
            "name_zh_cn": record["name_zh_cn"],
            "summary_en": record["summary_en"],
            "summary_zh_cn": record["summary_zh_cn"],
        })
        for parameter in record["parameter_values"]:
            rows["UniverseCurioParameter"].append({
                "curio_state_id": state_id,
                "sequence": parameter["index"],
                "value_decimal": parameter["value"],
            })
    return rows
