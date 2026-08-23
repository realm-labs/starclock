"""Author the two Version 4.4 release-day character forms into production Excel.

This is an append-only C12 partition. Existing character and internal IDs stay
unchanged; the two newly released form identities use 1_160_001 and 1_160_002,
and their dependent definitions occupy the dedicated 140_000..149_999 block.
"""

from __future__ import annotations

import argparse
import hashlib
import json
from decimal import Decimal
from pathlib import Path
from typing import Any, Callable

import character_workbook_support as support


ROOT = Path(__file__).resolve().parents[2]
REFERENCE = ROOT / "content-reference" / "v4.4"
REFERENCE_DIGEST = "b237477309589c82a3866e553d8cdf4486f7027c25fb95ae7bf633f539424a7c"
BASE = 140_000
FORM_IDS = {
    "character.gilgamesh": 1_160_001,
    "character.rin-tohsaka": 1_160_002,
}

OWNED_TABLES = (
    "Ability",
    "AbilityLevelParameter",
    "AbilityPhase",
    "AbilityResourceDelta",
    "Character",
    "CharacterAbilityBinding",
    "CharacterStat",
    "Eidolon",
    "EidolonPatch",
    "ModifierDefinition",
    "ModifierFilter",
    "ModifierStackingGroup",
    "Selector",
    "TraceNode",
    "TracePatch",
    "ValueExpression",
)


def source_rows() -> tuple[list[dict[str, Any]], ...]:
    selected = set(FORM_IDS)
    result = []
    for name in ("characters", "character-abilities", "character-traces", "character-eidolons"):
        rows = support.read_json(REFERENCE / f"{name}.json")
        result.append([
            row
            for row in rows
            if row.get("id") in selected or row.get("character_id") in selected
        ])
    counts = tuple(len(rows) for rows in result)
    if counts != (2, 16, 36, 12):
        raise ValueError(f"Version 4.4 released character cardinality changed: {counts}")
    return tuple(result)


def internal_maps(
    abilities: list[dict[str, Any]],
    traces: list[dict[str, Any]],
    eidolons: list[dict[str, Any]],
) -> dict[str, dict[str, int]]:
    return {
        "ability": {
            row["id"]: BASE + 1 + index
            for index, row in enumerate(sorted(abilities, key=lambda row: row["id"]))
        },
        "trace": {
            row["id"]: BASE + 2_001 + index
            for index, row in enumerate(sorted(traces, key=lambda row: row["id"]))
        },
        "eidolon": {
            row["id"]: BASE + 3_001 + index
            for index, row in enumerate(sorted(eidolons, key=lambda row: row["id"]))
        },
    }


def trace_status(row: dict[str, Any]) -> list[tuple[str, Any]]:
    result = []
    for status in row["status_additions"]:
        property_type = status.get("PropertyType", status.get("type"))
        value = status.get("Value", status.get("value"))
        if property_type is None or value is None:
            raise ValueError(f"invalid released Trace status addition in {row['id']}")
        result.append((property_type, value))
    return result


def trace_kind(row: dict[str, Any]) -> str:
    if trace_status(row):
        return "MinorStat"
    if row["level_up_skill_source_ids"]:
        return "BasicLevel" if row["max_level"] <= 6 else "AbilityLevel"
    if row["mechanic_hints"]["operation_tags"]:
        return "MajorPassive"
    return "AbilityUnlock"


def modifier_spec(property_type: str) -> tuple[str, str, str, str | None]:
    direct = {
        "AttackAddedRatio": ("Atk", "PercentOfBase", "Stat", None),
        "CriticalChanceBase": ("CritRate", "BaseAdd", "Stat", None),
        "CriticalDamageBase": ("CritDamage", "BaseAdd", "Stat", None),
        "QuantumAddedRatio": ("Atk", "DamageBoost", "OrdinaryDamage", "Quantum"),
        "ThunderAddedRatio": ("Atk", "DamageBoost", "OrdinaryDamage", "Lightning"),
    }
    try:
        return direct[property_type]
    except KeyError as error:
        raise ValueError(f"unsupported released minor-Trace property {property_type}") from error


def generated_rows() -> tuple[dict[str, list[dict[str, Any]]], list[dict[str, Any]], list[dict[str, Any]]]:
    characters, abilities, traces, eidolons = source_rows()
    ids = internal_maps(abilities, traces, eidolons)
    rows = {name: [] for name in OWNED_TABLES}
    internals = []
    selector_owner, selector_subject, stacking_group = BASE + 6_001, BASE + 6_002, BASE + 6_003
    rows["ModifierStackingGroup"].append({
        "id": stacking_group,
        "stable_key": "c12.trace-minor.additive",
        "aggregation": "Sum",
    })
    for selector_id, origin in ((selector_owner, "Owner"), (selector_subject, "CurrentSubject")):
        internals.append(support.identity(
            selector_id,
            f"selector.c12.trace-minor.{origin.lower()}",
            "Selector",
            f"C12 Trace {origin} Selector",
            "C12 行迹选择器",
            "Generic single-subject selector for exact released minor-Trace modifiers.",
        ))
        rows["Selector"].append({
            "id": selector_id,
            "domain": "Battle",
            "origin": origin,
            "side_relationship": "SameSide",
            "life": "Alive",
            "presence": "Present",
            "reference_point": "CurrentState",
            "ordering": "StableId",
            "choice": "First",
            "minimum_count": 1,
            "maximum_count": 1,
            "allow_repeated_targets": False,
            "empty_pool_policy": "Fault",
        })

    for ability in sorted(abilities, key=lambda row: row["id"]):
        ability_id = ids["ability"][ability["id"]]
        kind = support.ability_kind(ability)
        internals.append(support.identity(
            ability_id,
            ability["id"],
            "Ability",
            ability["name_en"],
            ability["name_zh_cn"],
            "Released ability metadata and exact level parameters; mechanism execution remains bound by its evidence quality.",
        ))
        rows["Ability"].append({
            "id": ability_id,
            "kind": kind,
            "target_pattern": support.target_pattern(ability),
            "retarget_policy": "RecomputeEachHit" if support.target_pattern(ability) == "Bounce" else "CancelRemaining",
            "level_cap": max(1, int(ability["max_level"] or 1)),
            "cooldown_actions": max(0, int(ability.get("cooldown") or 0)),
            "semantic_tags_mask": support.semantic_mask(kind, ability),
        })
        rows["CharacterAbilityBinding"].append({
            "character_id": FORM_IDS[ability["character_id"]],
            "sequence": 0,
            "slot": support.ability_slot(kind),
            "ability_id": ability_id,
            "invested_level_cap": support.invested_level_cap(
                ability["kind"], max(1, int(ability["max_level"] or 1))
            ),
        })
        for level in ability["levels"]:
            for parameter_index, value in enumerate(level["parameters"], start=1):
                rows["AbilityLevelParameter"].append({
                    "ability_id": ability_id,
                    "effective_level": level["level"],
                    "parameter_key": f"parameter.{parameter_index:02d}",
                    "value_decimal": support.canonical_decimal(value),
                })
        delta_sequence = 1
        skill_points = ability.get("skill_point_cost")
        if kind == "Basic" or (skill_points is not None and Decimal(str(skill_points)) > 0):
            spends = skill_points is not None and Decimal(str(skill_points)) > 0
            amount = Decimal(str(skill_points)) if spends else Decimal(1)
            rows["AbilityResourceDelta"].append({
                "ability_id": ability_id,
                "sequence": delta_sequence,
                "resource_kind": "SkillPoints",
                "delta_kind": "Spend" if spends else "Gain",
                "timing": "ActionStarted" if spends else "AbilityResolved",
                "amount_decimal": support.canonical_decimal(abs(amount)),
            })
            delta_sequence += 1
        energy = ability.get("energy_gain")
        if energy not in (None, "0", 0):
            rows["AbilityResourceDelta"].append({
                "ability_id": ability_id,
                "sequence": delta_sequence,
                "resource_kind": "Energy",
                "delta_kind": "Gain",
                "timing": "AbilityResolved",
                "amount_decimal": support.canonical_decimal(energy),
            })
        rows["AbilityPhase"].append({"ability_id": ability_id, "sequence": 1, "kind": "Resolved"})

    by_character: dict[int, list[dict[str, Any]]] = {}
    for binding in rows["CharacterAbilityBinding"]:
        by_character.setdefault(int(binding["character_id"]), []).append(binding)
    for bindings in by_character.values():
        bindings.sort(key=lambda row: int(row["ability_id"]))
        for sequence, binding in enumerate(bindings, start=1):
            binding["sequence"] = sequence

    for character in sorted(characters, key=lambda row: row["id"]):
        character_id = FORM_IDS[character["id"]]
        rows["Character"].append({
            "id": character_id,
            "source_avatar_id": int(character["source_avatar_ids"][0]),
            "rarity": character["rarity"],
            "path": character["path"],
            "element": character["element"],
            "base_energy_decimal": character["max_energy"] or "0",
            "base_aggro_decimal": character["promotions"][0]["aggro"],
        })
        for promotion, stat in enumerate(character["promotions"]):
            first_level = 1 if promotion == 0 else promotion * 10 + 10
            max_level = int(stat.get("max_level", (promotion + 2) * 10))
            for level in range(first_level, max_level + 1):
                offset = Decimal(level - 1)
                rows["CharacterStat"].append({
                    "character_id": character_id,
                    "level": level,
                    "promotion": promotion,
                    "hp_decimal": support.canonical_decimal(Decimal(stat["hp_base"]) + Decimal(stat["hp_per_level"]) * offset),
                    "atk_decimal": support.canonical_decimal(Decimal(stat["atk_base"]) + Decimal(stat["atk_per_level"]) * offset),
                    "def_decimal": support.canonical_decimal(Decimal(stat["def_base"]) + Decimal(stat["def_per_level"]) * offset),
                    "spd_decimal": support.canonical_decimal(stat["spd"]),
                })

    point_to_trace = {source_id: row["id"] for row in traces for source_id in row["source_point_ids"]}
    minor_index = 0
    for trace in sorted(traces, key=lambda row: row["id"]):
        trace_id = ids["trace"][trace["id"]]
        internals.append(support.identity(
            trace_id,
            trace["id"],
            "Trace",
            trace["name_en"],
            trace["name_zh_cn"],
            "Released battle-relevant Trace identity and exact prepared payload.",
        ))
        prerequisites = sorted({
            ids["trace"][point_to_trace[source]]
            for source in trace["prerequisites"]
            if source in point_to_trace
        })
        rows["TraceNode"].append({
            "id": trace_id,
            "character_id": FORM_IDS[trace["character_id"]],
            "kind": trace_kind(trace),
            "promotion_requirement": 0,
            "prerequisite_trace_ids": "|".join(str(value) for value in prerequisites) or None,
        })
        patch_sequence = 1
        for property_type, value in sorted(set(trace_status(trace))):
            minor_index += 1
            modifier_id, expression_id = BASE + 4_001 + minor_index, BASE + 5_001 + minor_index
            stat, stage, purpose, element = modifier_spec(property_type)
            internals.append(support.identity(
                modifier_id,
                f"modifier.{trace['id']}.{property_type}",
                "Modifier",
                f"{trace['name_en']} {property_type}",
                f"{trace['name_zh_cn']} {property_type}",
                "Exact released minor-Trace stat addition compiled as a persistent modifier.",
            ))
            value_kind = "Ratio"
            rows["ValueExpression"].append({
                "id": expression_id,
                "stable_key": f"c12.trace-minor.value.{minor_index:03d}",
                "result_kind": value_kind,
                "node": json.dumps({
                    "type": f"{value_kind}Literal",
                    "value_decimal": support.canonical_decimal(value),
                }, separators=(",", ":")),
            })
            rows["ModifierDefinition"].append({
                "id": modifier_id,
                "owner_selector_id": selector_owner,
                "subject_selector_id": selector_subject,
                "stat": stat,
                "formula_stage": stage,
                "formula_purpose": purpose,
                "value_expression_id": expression_id,
                "value_domain": value_kind,
                "stacking_group_id": stacking_group,
                "priority": 0,
                "cap_formula_stage": stage,
                "snapshot_policy": "Dynamic",
                "duration_scope": "Battle",
            })
            if element is not None:
                rows["ModifierFilter"].append({
                    "modifier_id": modifier_id,
                    "sequence": 1,
                    "filter": json.dumps({"type": "Element", "element": element}, separators=(",", ":")),
                })
            rows["TracePatch"].append({
                "trace_id": trace_id,
                "sequence": patch_sequence,
                "patch": json.dumps({"type": "AddModifier", "modifier_identity_id": modifier_id}, separators=(",", ":")),
            })
            patch_sequence += 1
        for source_skill_id in trace["level_up_skill_source_ids"]:
            matching = next((row for row in abilities if source_skill_id in row["source_skill_ids"]), None)
            if matching is not None and matching["kind"] not in ("Maze", "MazeNormal"):
                rows["TracePatch"].append({
                    "trace_id": trace_id,
                    "sequence": patch_sequence,
                    "patch": json.dumps({
                        "type": "AdjustAbilityLevel",
                        "ability_id": ids["ability"][matching["id"]],
                        "bonus": 1,
                        "cap_delta": 1,
                    }, separators=(",", ":")),
                })
                patch_sequence += 1

    ability_by_source = {source_id: row for row in abilities for source_id in row["source_skill_ids"]}
    for eidolon in sorted(eidolons, key=lambda row: row["id"]):
        eidolon_id = ids["eidolon"][eidolon["id"]]
        internals.append(support.identity(
            eidolon_id,
            eidolon["id"],
            "Eidolon",
            eidolon["name_en"],
            eidolon["name_zh_cn"],
            "Released E1-E6 identity and exact prepared ability-level patches.",
        ))
        rows["Eidolon"].append({
            "id": eidolon_id,
            "character_id": FORM_IDS[eidolon["character_id"]],
            "rank": eidolon["rank"],
        })
        additions: dict[str, int] = {}
        for addition in eidolon["skill_level_additions"]:
            ability = ability_by_source.get(addition["source_skill_id"])
            if ability is not None:
                additions[ability["id"]] = max(additions.get(ability["id"], 0), int(addition["levels"]))
        for sequence, (ability_key, levels) in enumerate(sorted(additions.items()), start=1):
            rows["EidolonPatch"].append({
                "eidolon_id": eidolon_id,
                "sequence": sequence,
                "patch": json.dumps({
                    "type": "AdjustAbilityLevel",
                    "ability_id": ids["ability"][ability_key],
                    "bonus": levels,
                    "cap_delta": levels,
                }, separators=(",", ":")),
            })

    for table_rows in rows.values():
        table_rows.sort(key=lambda row: tuple(str(value) for value in row.values()))
    return rows, internals, abilities + traces + eidolons


def owned_predicate(name: str) -> Callable[[dict[str, Any]], bool]:
    field = {
        "Ability": "id",
        "AbilityLevelParameter": "ability_id",
        "AbilityPhase": "ability_id",
        "AbilityResourceDelta": "ability_id",
        "Character": "id",
        "CharacterAbilityBinding": "ability_id",
        "CharacterStat": "character_id",
        "Eidolon": "id",
        "EidolonPatch": "eidolon_id",
        "ModifierDefinition": "id",
        "ModifierFilter": "modifier_id",
        "ModifierStackingGroup": "id",
        "Selector": "id",
        "TraceNode": "id",
        "TracePatch": "trace_id",
        "ValueExpression": "id",
    }[name]
    if name in ("Character", "CharacterStat"):
        return lambda row: int(row[field]) in set(FORM_IDS.values())
    return lambda row: BASE <= int(row[field]) <= BASE + 9_999


def merged_table(name: str, authored: list[dict[str, Any]]) -> list[dict[str, Any]]:
    _, existing = support.workbook_rows(name)
    owns = owned_predicate(name)
    insertion = min((index for index, row in enumerate(existing) if owns(row)), default=len(existing))
    retained = [dict(row) for row in existing if not owns(row)]
    retained[insertion:insertion] = authored
    return retained


def update_metadata(
    internals: list[dict[str, Any]], source: list[dict[str, Any]]
) -> tuple[list[dict[str, Any]], list[dict[str, Any]]]:
    _, identities = support.workbook_rows("ContentIdentity")
    retained = [
        dict(row)
        for row in identities
        if str(row["stable_key"]) not in FORM_IDS
        and not (BASE <= int(row["id"]) <= BASE + 9_999)
    ]
    characters = {row["id"]: row for row in support.read_json(REFERENCE / "characters.json")}
    for stable_key, identity_id in FORM_IDS.items():
        character = characters[stable_key]
        identity = support.identity(
            identity_id,
            stable_key,
            "CharacterForm",
            character["name_en"],
            character["name_zh_cn"],
            "Complete released production statistics, abilities, Traces and E1-E6 rows are present.",
        )
        identity["source_record_ids"] = "2"
        retained.append(identity)
    for identity in internals:
        identity["source_record_ids"] = "2"
    retained.extend(internals)
    retained.sort(key=lambda row: int(row["id"]))

    source_by_id = {row["id"]: row for row in source}
    _, bindings = support.workbook_rows("ContentEvidenceBinding")
    owned_ids = set(FORM_IDS.values()) | {record["id"] for record in internals}
    kept = [dict(row) for row in bindings if int(row["content_id"]) not in owned_ids]
    for stable_key, identity_id in FORM_IDS.items():
        kept.append({
            "content_id": identity_id,
            "sequence": 1,
            "fact_key": f"c12.executable:{stable_key}",
            "source_record_id": 2,
            "evidence_record_id": 3,
            "quality": "ExactStructured",
            "mechanism_quality": "ApproximateFromReleasedText",
            "approximation_note": "Released public index supplies exact progression and level parameters; mechanics await ordinary post-release structured promotion.",
        })
    for record in internals:
        source_record = source_by_id.get(record["stable_key"])
        quality = source_record.get("quality", "ExactStructured") if source_record else "ExactStructured"
        mechanism = source_record.get("mechanism_quality", quality) if source_record else quality
        kept.append({
            "content_id": record["id"],
            "sequence": 1,
            "fact_key": f"c12.prepared:{record['stable_key']}",
            "source_record_id": 2,
            "evidence_record_id": 3,
            "quality": quality,
            "mechanism_quality": mechanism,
            "approximation_note": "Released public index row; no hidden or pre-release source used." if mechanism == "ApproximateFromReleasedText" else None,
        })
    kept.sort(key=lambda row: (int(row["content_id"]), int(row["sequence"])))
    return retained, kept


def update_provenance() -> tuple[list[dict[str, Any]], list[dict[str, Any]]]:
    manifest = support.read_json(REFERENCE / "manifest.json")
    repository = next(
        row for row in manifest["repositories"] if row["id"] == "mar-7th-star-rail-res"
    )
    _, sources = support.workbook_rows("SourceRecord")
    source = next(row for row in sources if int(row["id"]) == 2)
    source.update({
        "stable_key": f"source.{repository['id']}.{repository['revision']}",
        "publisher": repository["id"],
        "url": repository["remote"].removesuffix(".git"),
        "accessed_on": "2026-07-24",
        "applicable_game_version": "4.4",
        "category": "CommunityMaintained",
        "confidence": "PreparedExactStructured",
        "usage_note": f"{repository['usage']} Revision {repository['revision']}.",
        "evidence_sha256": hashlib.sha256(
            json.dumps(repository, ensure_ascii=False, separators=(",", ":")).encode("utf-8")
        ).hexdigest(),
    })
    _, evidence = support.workbook_rows("EvidenceRecord")
    pack = support.read_json(REFERENCE / "pack-index.json")
    reference = next(row for row in evidence if int(row["id"]) == 3)
    reference.update({
        "sha256": pack["pack_sha256"],
        "note": "Deterministically normalized current Version 4.4 reference pack used by production rows.",
    })
    return sources, evidence


def main() -> None:
    parser = argparse.ArgumentParser()
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--write", action="store_true")
    mode.add_argument("--check", action="store_true")
    arguments = parser.parse_args()
    if support.read_json(REFERENCE / "pack-index.json")["pack_sha256"] != REFERENCE_DIGEST:
        raise ValueError("prepared reference pack digest changed")
    rows, internals, source = generated_rows()
    expected = {name: merged_table(name, rows[name]) for name in OWNED_TABLES}
    identities, evidence = update_metadata(internals, source)
    source_records, evidence_records = update_provenance()
    if arguments.write:
        for name in OWNED_TABLES:
            support.write_rows(name, expected[name])
        support.write_rows("ContentIdentity", identities)
        support.write_rows("ContentEvidenceBinding", evidence)
        support.write_rows("SourceRecord", source_records)
        support.write_rows("EvidenceRecord", evidence_records)
        print("Authored two released Version 4.4 character forms into production workbooks.")
    else:
        for name in OWNED_TABLES:
            support.check_exact(name, expected[name])
        support.check_exact("ContentIdentity", identities)
        support.check_exact("ContentEvidenceBinding", evidence)
        support.check_exact("SourceRecord", source_records)
        support.check_exact("EvidenceRecord", evidence_records)
        print("Released Version 4.4 character workbooks match deterministic authoring output.")


if __name__ == "__main__":
    main()
