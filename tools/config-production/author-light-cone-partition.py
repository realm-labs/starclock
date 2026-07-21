"""Author one frozen Goal 01 Light Cone partition into production Excel.

The prepared Version 4.4 reference pack is the only content input. Each
partition owns a disjoint 10,000-ID block so completed imports compose without
rewriting prior Light Cone rows.

Run through the approved repository adapter:
  uv run --with openpyxl python tools/config-production/author-light-cone-partition.py L01 --write
  uv run --with openpyxl python tools/config-production/author-light-cone-partition.py L01 --check
"""

from __future__ import annotations

import argparse
import importlib.util
import json
import sys
from decimal import Decimal
from pathlib import Path
from typing import Any, Callable


ROOT = Path(__file__).resolve().parents[2]
REFERENCE = ROOT / "content-reference" / "v4.4"
PARTITIONS = ROOT / "content-manifests" / "core-combat-v1" / "partitions.json"
REFERENCE_DIGEST = "0dca8ae581b4fa1e9fe8ce0c9e67ac6eb72c251deacbd4831751ce685e45ef5a"


def load_sibling(filename: str, module_name: str) -> Any:
    path = Path(__file__).with_name(filename)
    spec = importlib.util.spec_from_file_location(module_name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError("unable to load production authoring helpers")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


V1B = load_sibling("author-character-v1b.py", "starclock_character_v1b_for_light_cones")

OWNED_TABLES = (
    "LightCone",
    "LightConeStat",
    "LightConeSuperimposition",
    "ModifierDefinition",
    "ModifierStackingGroup",
    "RuleDefinition",
    "Selector",
    "ValueExpression",
)

ALL_DAMAGE_PURPOSES = (
    "OrdinaryDamage",
    "Dot",
    "AdditionalDamage",
    "JointDamage",
    "ElationDamage",
)


def partition(code: str) -> tuple[int, list[str]]:
    if not code.startswith("L") or not code[1:].isdigit():
        raise ValueError("partition must be L01 through L11")
    index = int(code[1:])
    records = V1B.read_json(PARTITIONS)["light_cone_partitions"]
    if not 1 <= index <= len(records):
        raise ValueError("partition must be L01 through L11")
    row = records[index - 1]
    if row["batch_id"] != f"G01-P7-{code}":
        raise ValueError(f"frozen {code} partition changed")
    return 200_000 + index * 10_000, row["ids"]


def sources(selected_ids: list[str]) -> list[dict[str, Any]]:
    selected = set(selected_ids)
    rows = [row for row in V1B.read_json(REFERENCE / "light-cones.json") if row["id"] in selected]
    if len(rows) != len(selected):
        raise ValueError("prepared Light Cone partition cardinality changed")
    return rows


def modifier_specs(property_type: str) -> list[tuple[str, str, str, str]]:
    if property_type == "AllDamageTypeAddedRatio":
        return [("Atk", "DamageBoost", purpose, "Ratio") for purpose in ALL_DAMAGE_PURPOSES]
    direct = {
        "AttackAddedRatio": ("Atk", "PercentOfBase", "Stat", "Ratio"),
        "BaseSpeed": ("Spd", "Flat", "Stat", "Scalar"),
        "BreakDamageAddedRatioBase": ("BreakEffect", "BaseAdd", "Stat", "Ratio"),
        "CriticalChanceBase": ("CritRate", "BaseAdd", "Stat", "Ratio"),
        "CriticalDamageBase": ("CritDamage", "BaseAdd", "Stat", "Ratio"),
        "DefenceAddedRatio": ("Def", "PercentOfBase", "Stat", "Ratio"),
        "ElationDamageAddedRatioBase": ("Atk", "DamageBoost", "ElationDamage", "Ratio"),
        "HealRatioBase": ("OutgoingHealing", "BaseAdd", "Healing", "Ratio"),
        "HealTakenRatio": ("IncomingHealing", "BaseAdd", "Healing", "Ratio"),
        "HPAddedRatio": ("Hp", "PercentOfBase", "Stat", "Ratio"),
        "SpeedAddedRatio": ("Spd", "PercentOfBase", "Stat", "Ratio"),
        "SPRatioBase": ("EnergyRegenerationRate", "BaseAdd", "Stat", "Ratio"),
        "StatusProbabilityBase": ("EffectHitRate", "BaseAdd", "Stat", "Ratio"),
        "StatusResistanceBase": ("EffectResistance", "BaseAdd", "Stat", "Ratio"),
    }
    try:
        return [direct[property_type]]
    except KeyError as error:
        raise ValueError(f"unsupported prepared Light Cone property {property_type}") from error


def validate_superimpositions(cone: dict[str, Any]) -> list[list[str]]:
    ranks = cone["passive"]["superimpositions"]
    if [int(row["rank"]) for row in ranks] != [1, 2, 3, 4, 5]:
        raise ValueError(f"{cone['id']} lacks exact S1-S5 rows")
    vectors = [[V1B.canonical_decimal(value) for value in row["parameters"]] for row in ranks]
    if not vectors[0] or any(len(vector) != len(vectors[0]) for vector in vectors):
        raise ValueError(f"{cone['id']} has an incomplete S1-S5 parameter vector")
    return vectors


def generated_rows(
    code: str,
) -> tuple[dict[str, list[dict[str, Any]]], list[dict[str, Any]], list[dict[str, Any]], list[str], int]:
    base, selected = partition(code)
    cones = sources(selected)
    frozen = V1B.identity_map()
    rows = {name: [] for name in OWNED_TABLES}
    internals: list[dict[str, Any]] = []
    owner_selector, subject_selector, stacking_group = base + 6_001, base + 6_002, base + 6_003
    rows["ModifierStackingGroup"].append(
        {
            "id": stacking_group,
            "stable_key": f"{code.lower()}.light-cone-passive.additive",
            "aggregation": "Sum",
        }
    )
    for selector_id, origin in ((owner_selector, "Owner"), (subject_selector, "CurrentSubject")):
        internals.append(
            V1B.identity(
                selector_id,
                f"selector.{code.lower()}.light-cone-passive.{origin.lower()}",
                "Selector",
                f"{code} Light Cone {origin} Selector",
                f"{code}光锥选择器",
                "Generic single-subject selector for equipped Light Cone modifiers.",
            )
        )
        rows["Selector"].append(
            {
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
            }
        )

    modifier_index = 0
    for cone_index, cone in enumerate(sorted(cones, key=lambda row: row["id"]), start=1):
        cone_id = frozen[cone["id"]]
        passive = cone["passive"]
        rule_id = base + cone_index
        internals.append(
            V1B.identity(
                rule_id,
                f"rule.{cone['id']}.passive",
                "Rule",
                passive["name_en"],
                passive["name_zh_cn"],
                "Source-bound equipped Light Cone passive rule with exact S1-S5 parameter selection.",
            )
        )
        rows["RuleDefinition"].append(
            {
                "id": rule_id,
                "domain": "Battle",
                "source_definition_identity_id": cone_id,
                "source_class": "Equipment",
                "source_digest_sha256": passive["source_text"]["sha256"],
            }
        )
        rows["LightCone"].append(
            {
                "id": cone_id,
                "rarity": cone["rarity"],
                "path": {"The Hunt": "Hunt", "Warrior": "Destruction"}.get(
                    cone["path"], cone["path"]
                ),
                "applicability": "MatchingPath",
                "passive_rule_identity_id": rule_id,
            }
        )
        for promotion, stat in enumerate(cone["promotions"]):
            first_level = 1 if promotion == 0 else promotion * 10 + 10
            for level in range(first_level, int(stat["max_level"]) + 1):
                offset = Decimal(level - 1)
                rows["LightConeStat"].append(
                    {
                        "light_cone_id": cone_id,
                        "level": level,
                        "promotion": promotion,
                        "hp_decimal": V1B.canonical_decimal(
                            Decimal(stat["hp_base"]) + Decimal(stat["hp_per_level"]) * offset
                        ),
                        "atk_decimal": V1B.canonical_decimal(
                            Decimal(stat["atk_base"]) + Decimal(stat["atk_per_level"]) * offset
                        ),
                        "def_decimal": V1B.canonical_decimal(
                            Decimal(stat["def_base"]) + Decimal(stat["def_per_level"]) * offset
                        ),
                    }
                )
        vectors = validate_superimpositions(cone)
        for rank_index, rank in enumerate(passive["superimpositions"]):
            modifier_ids = []
            for property_index, prop in enumerate(rank["properties"], start=1):
                property_type = prop["PropertyType"]
                value = V1B.canonical_decimal(prop["Value"])
                for spec_index, (stat, stage, purpose, value_domain) in enumerate(
                    modifier_specs(property_type), start=1
                ):
                    modifier_index += 1
                    modifier_id = base + 1_000 + modifier_index
                    expression_id = base + 3_000 + modifier_index
                    modifier_ids.append(modifier_id)
                    suffix = f"{property_index:02d}.{spec_index:02d}.{purpose.lower()}"
                    internals.append(
                        V1B.identity(
                            modifier_id,
                            f"modifier.{cone['id']}.s{rank['rank']}.{suffix}",
                            "Modifier",
                            f"{passive['name_en']} S{rank['rank']} {property_type} {purpose}",
                            f"{passive['name_zh_cn']} 叠影{rank['rank']} {property_type}",
                            "Exact prepared S-rank property addition compiled as an equipped modifier.",
                        )
                    )
                    literal = f"{value_domain}Literal"
                    rows["ValueExpression"].append(
                        {
                            "id": expression_id,
                            "stable_key": f"{code.lower()}.light-cone.value.{modifier_index:04d}",
                            "result_kind": value_domain,
                            "node": json.dumps(
                                {"type": literal, "value_decimal": value}, separators=(",", ":")
                            ),
                        }
                    )
                    rows["ModifierDefinition"].append(
                        {
                            "id": modifier_id,
                            "source_rule_id": rule_id,
                            "owner_selector_id": owner_selector,
                            "subject_selector_id": subject_selector,
                            "stat": stat,
                            "formula_stage": stage,
                            "formula_purpose": purpose,
                            "value_expression_id": expression_id,
                            "value_domain": value_domain,
                            "stacking_group_id": stacking_group,
                            "priority": 0,
                            "cap_formula_stage": stage,
                            "snapshot_policy": "Dynamic",
                            "duration_scope": "Battle",
                        }
                    )
            for parameter_index, value in enumerate(vectors[rank_index], start=1):
                values_at_position = [vector[parameter_index - 1] for vector in vectors]
                rows["LightConeSuperimposition"].append(
                    {
                        "light_cone_id": cone_id,
                        "parameter_key": f"parameter.{parameter_index:02d}",
                        "rank": rank["rank"],
                        "value_decimal": value,
                        "constant_across_ranks": len(set(values_at_position)) == 1,
                        "modifier_identity_ids": "|".join(str(value) for value in modifier_ids)
                        if parameter_index == 1 and modifier_ids
                        else None,
                    }
                )

    for table_rows in rows.values():
        table_rows.sort(key=lambda row: tuple(str(value) for value in row.values()))
    return rows, internals, cones, selected, base


def owned_predicate(name: str, base: int, selected: list[str]) -> Callable[[dict[str, Any]], bool]:
    field = {
        "LightCone": "id",
        "LightConeStat": "light_cone_id",
        "LightConeSuperimposition": "light_cone_id",
        "ModifierDefinition": "id",
        "ModifierStackingGroup": "id",
        "RuleDefinition": "id",
        "Selector": "id",
        "ValueExpression": "id",
    }[name]
    frozen_ids = {V1B.identity_map()[key] for key in selected}
    if name in ("LightCone", "LightConeStat", "LightConeSuperimposition"):
        return lambda row: int(row[field]) in frozen_ids
    return lambda row: base <= int(row[field]) <= base + 9_999


def merged_table(
    name: str, authored: list[dict[str, Any]], base: int, selected: list[str]
) -> list[dict[str, Any]]:
    _, existing = V1B.workbook_rows(name)
    owns = owned_predicate(name, base, selected)
    positions = [index for index, row in enumerate(existing) if owns(row)]
    insertion = min(positions) if positions else len(existing)
    retained = [dict(row) for row in existing if not owns(row)]
    retained[insertion:insertion] = authored
    return retained


def update_metadata(
    code: str,
    internals: list[dict[str, Any]],
    selected: list[str],
    base: int,
) -> tuple[list[dict[str, Any]], list[dict[str, Any]]]:
    _, identities = V1B.workbook_rows("ContentIdentity")
    selected_set = set(selected)
    retained = [dict(row) for row in identities if not (base <= int(row["id"]) <= base + 9_999)]
    for row in retained:
        if str(row["stable_key"]) in selected_set:
            row["enabled"] = True
            row["coverage_state"] = "GoldenVerified"
            row["summary_en"] = str(row["summary_en"]).replace(
                " Catalog identity only; executable rows remain pending.",
                " Complete production stat curves, S1-S5 parameters and source-bound passive behavior are present.",
            )
            row["summary_zh_cn"] = str(row["summary_zh_cn"]).replace(
                " 当前仅为目录身份；可执行数据尚待转录。",
                " 已具备完整生产属性曲线、叠影一至五参数与来源绑定被动行为。",
            )
    retained.extend(internals)
    retained.sort(key=lambda row: int(row["id"]))
    _, bindings = V1B.workbook_rows("ContentEvidenceBinding")
    selected_ids = {V1B.identity_map()[key] for key in selected}
    kept = [
        dict(row)
        for row in bindings
        if not (base <= int(row["content_id"]) <= base + 9_999)
        and not (int(row["content_id"]) in selected_ids and int(row["sequence"]) >= 2)
    ]
    for record in internals:
        kept.append(
            {
                "content_id": record["id"],
                "sequence": 1,
                "fact_key": f"{code.lower()}.prepared:{record['stable_key']}",
                "source_record_id": 1,
                "evidence_record_id": 3,
                "quality": "ExactStructured",
                "mechanism_quality": "ExactStructured",
            }
        )
    for stable_key in sorted(selected):
        kept.append(
            {
                "content_id": V1B.identity_map()[stable_key],
                "sequence": 2,
                "fact_key": f"{code.lower()}.executable:{stable_key}",
                "source_record_id": 1,
                "evidence_record_id": 3,
                "quality": "ExactStructured",
                "mechanism_quality": "ExactStructured",
            }
        )
    kept.sort(key=lambda row: (int(row["content_id"]), int(row["sequence"])))
    return retained, kept


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("partition")
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--write", action="store_true")
    mode.add_argument("--check", action="store_true")
    args = parser.parse_args()
    code = args.partition.upper()
    if V1B.read_json(REFERENCE / "pack-index.json")["pack_sha256"] != REFERENCE_DIGEST:
        raise ValueError("prepared reference pack digest changed")
    rows, internals, _cones, selected, base = generated_rows(code)
    expected = {name: merged_table(name, rows[name], base, selected) for name in OWNED_TABLES}
    identities, evidence = update_metadata(code, internals, selected, base)
    _, manifest_rows = V1B.workbook_rows("ConfigManifest")
    if len(manifest_rows) != 1:
        raise ValueError("production ConfigManifest must remain a singleton")
    if args.write:
        manifest_rows[0]["data_revision"] = f"core-combat-v1-phase7-{code.lower()}"
        for name in OWNED_TABLES:
            V1B.write_rows(name, expected[name])
        V1B.write_rows("ContentIdentity", identities)
        V1B.write_rows("ContentEvidenceBinding", evidence)
        V1B.write_rows("ConfigManifest", manifest_rows)
        print(f"Authored frozen {code} Light Cone partition into production workbooks.")
    else:
        for name in OWNED_TABLES:
            V1B.check_exact(name, expected[name])
        V1B.check_exact("ContentIdentity", identities)
        V1B.check_exact("ContentEvidenceBinding", evidence)
        V1B.check_exact("ConfigManifest", manifest_rows)
        print(f"Frozen {code} Light Cone workbooks match deterministic authoring output.")


if __name__ == "__main__":
    main()
