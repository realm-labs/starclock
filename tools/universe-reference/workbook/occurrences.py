"""Occurrence graph, choice, cost and outcome workbook rows."""

from __future__ import annotations

from pathlib import Path

from workbook.data import canonical_json, joined, load_json, optional_joined, stable_ids


def build_rows(root: Path) -> dict[str, list[dict]]:
    occurrences = load_json(root, "occurrences.json")
    variants = load_json(root, "occurrence-variants.json")
    choices = load_json(root, "occurrence-choices.json")
    occurrence_ids = stable_ids(occurrences)
    variant_ids = stable_ids(variants)
    choice_ids = stable_ids(choices)
    rows: dict[str, list[dict]] = {
        "UniverseOccurrence": [],
        "UniverseOccurrenceVariant": [],
        "UniverseOccurrenceChoice": [],
        "UniverseOccurrenceCost": [],
        "UniverseOccurrenceOutcome": [],
    }
    for record in occurrences:
        rows["UniverseOccurrence"].append({
            "id": occurrence_ids[record["id"]],
            "stable_key": record["id"],
            "choice_graph_stable_key": record["choice_graph_id"],
            "pool_tags": optional_joined(record["pool_tags"]),
            "index_only": record["index_only"],
            "name_en": record["name_en"],
            "name_zh_cn": record["name_zh_cn"],
            "summary_en": record["summary_en"],
            "summary_zh_cn": record["summary_zh_cn"],
        })
    for record in variants:
        rows["UniverseOccurrenceVariant"].append({
            "id": variant_ids[record["id"]],
            "stable_key": record["id"],
            "occurrence_id": occurrence_ids[record["occurrence_id"]],
            "entry_node_id": record["entry_node_id"],
            "condition_ids": optional_joined(record["condition_ids"]),
            "source_dialogue_type": record["source_dialogue_type"],
            "name_en": record["name_en"],
            "name_zh_cn": record["name_zh_cn"],
            "summary_en": record["summary_en"],
            "summary_zh_cn": record["summary_zh_cn"],
        })
    for record in choices:
        choice_id = choice_ids[record["id"]]
        rows["UniverseOccurrenceChoice"].append({
            "id": choice_id,
            "stable_key": record["id"],
            "variant_id": variant_ids[record["variant_id"]],
            "condition_ids": optional_joined(record["condition_ids"]),
            "next_node_id": record["next_node_id"] or None,
            "parameter_vectors_json": canonical_json(record["parameter_vectors"]),
            "choice_label_sha256_en": record["choice_label_sha256_en"],
            "choice_label_sha256_zh_cn": record["choice_label_sha256_zh_cn"],
            "result_sha256_en": record["result_sha256_en"],
            "result_sha256_zh_cn": record["result_sha256_zh_cn"],
            "name_en": record["name_en"],
            "name_zh_cn": record["name_zh_cn"],
            "summary_en": record["summary_en"],
            "summary_zh_cn": record["summary_zh_cn"],
        })
        for sequence, cost in enumerate(record["costs"], start=1):
            rows["UniverseOccurrenceCost"].append({
                "choice_id": choice_id,
                "sequence": sequence,
                "kind": cost["kind"],
                "targets": optional_joined(cost["targets"]),
            })
        for sequence, outcome in enumerate(record["outcomes"], start=1):
            rows["UniverseOccurrenceOutcome"].append({
                "choice_id": choice_id,
                "sequence": sequence,
                "kinds": joined(outcome["kinds"]),
                "targets": optional_joined(outcome["targets"]),
                "numeric_literals": optional_joined(outcome["numeric_literals"]),
                "parameter_refs": optional_joined(outcome["parameter_refs"]),
                "chance_percentages": optional_joined(outcome["chance_percentages"]),
                "unspecified_random_policy": outcome["unspecified_random_policy"] or None,
            })
    return rows
