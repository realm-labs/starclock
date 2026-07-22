"""Run services, currency rules and Ability Tree workbook rows."""

from __future__ import annotations

from pathlib import Path

from workbook.data import load_json, optional_joined, stable_ids


def build_rows(root: Path) -> dict[str, list[dict]]:
    services = load_json(root, "services.json")
    nodes = load_json(root, "ability-tree.json")
    service_ids = stable_ids(services)
    node_ids = stable_ids(nodes)
    rows: dict[str, list[dict]] = {
        "UniverseService": [],
        "UniverseServiceParameter": [],
        "UniverseAbilityTreeNode": [],
        "UniverseAbilityTreeEdge": [],
        "UniverseAbilityTreeCost": [],
        "UniverseAbilityTreeEffect": [],
        "UniverseAbilityTreeParameter": [],
    }
    for record in services:
        if len(record["rule_ids"]) != 1:
            raise ValueError(f"{record['id']} requires exactly one service rule")
        service_id = service_ids[record["id"]]
        rows["UniverseService"].append({
            "id": service_id,
            "stable_key": record["id"],
            "kind": record["kind"],
            "currency_stable_key": record["currency_id"] or None,
            "price_formula_stable_key": record["price_formula_id"] or None,
            "offer_pool_stable_key": record["offer_pool_id"] or None,
            "rule_stable_key": record["rule_ids"][0],
            "name_en": record["name_en"],
            "name_zh_cn": record["name_zh_cn"],
            "summary_en": record["summary_en"],
            "summary_zh_cn": record["summary_zh_cn"],
        })
        for sequence, parameter in enumerate(record["parameters"], start=1):
            rows["UniverseServiceParameter"].append({
                "service_id": service_id,
                "sequence": sequence,
                "key": parameter["key"],
                "value": parameter["value"],
            })
    for record in nodes:
        if len(record["rule_ids"]) != 1:
            raise ValueError(f"{record['id']} requires exactly one Ability Tree rule")
        node_id = node_ids[record["id"]]
        rows["UniverseAbilityTreeNode"].append({
            "id": node_id,
            "stable_key": record["id"],
            "important": record["important"],
            "effect_class": record["effect_class"],
            "effect_tag_en": record["effect_tag_en"],
            "effect_tag_zh_cn": record["effect_tag_zh_cn"],
            "external_unlock_ids": optional_joined(record["external_unlock_ids"]),
            "rule_stable_key": record["rule_ids"][0],
            "name_en": record["name_en"],
            "name_zh_cn": record["name_zh_cn"],
            "summary_en": record["summary_en"],
            "summary_zh_cn": record["summary_zh_cn"],
        })
        for sequence, prerequisite in enumerate(record["prerequisite_ids"], start=1):
            rows["UniverseAbilityTreeEdge"].append({
                "node_id": node_id,
                "sequence": sequence,
                "prerequisite_node_id": node_ids[prerequisite],
            })
        for sequence, cost in enumerate(record["cost"], start=1):
            rows["UniverseAbilityTreeCost"].append({
                "node_id": node_id,
                "sequence": sequence,
                "source_item_id": cost["source_item_id"],
                "amount_decimal": cost["amount"],
            })
        for sequence, effect in enumerate(record["effects"], start=1):
            rows["UniverseAbilityTreeEffect"].append({
                "node_id": node_id,
                "sequence": sequence,
                "kind": effect["kind"],
                "target": effect["target"],
                "value_decimal": effect["value"],
                "unit": effect["unit"],
                "condition": effect["condition"] or None,
            })
        for parameter in record["source_parameters"]:
            rows["UniverseAbilityTreeParameter"].append({
                "node_id": node_id,
                "sequence": parameter["index"],
                "value_decimal": parameter["value"],
            })
    return rows
