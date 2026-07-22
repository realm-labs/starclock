"""Mechanic-rule contribution workbook rows."""

from __future__ import annotations

from pathlib import Path

from workbook.data import canonical_json, load_json, optional_joined, stable_ids


def build_rows(root: Path) -> dict[str, list[dict]]:
    rules = load_json(root, "mechanic-rules.json")
    rule_ids = stable_ids(rules)
    rows = {"UniverseMechanicRule": []}
    for record in rules:
        rows["UniverseMechanicRule"].append({
            "id": rule_ids[record["id"]],
            "stable_key": record["id"],
            "source_record_stable_key": record["source_record_id"],
            "source_file": record["source_file"],
            "rule_kind": record["rule_kind"],
            "native_handler_stable_key": record["native_handler_id"] or None,
            "source_binding_key": record["source_binding_key"] or None,
            "parameter_values_json": canonical_json(record["parameter_values"]),
            "mechanic_tags": optional_joined(record["mechanic_tags"]),
            "approximation_replacement_condition": record["approximation_replacement_condition"] or None,
            "name_en": record["name_en"],
            "name_zh_cn": record["name_zh_cn"],
            "summary_en": record["summary_en"],
            "summary_zh_cn": record["summary_zh_cn"],
        })
    return rows
