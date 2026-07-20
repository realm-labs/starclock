"""Deterministically remap the six V1a mechanic probes into production IDs."""

from __future__ import annotations

import csv
import json
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[2]
PROBES = ROOT / "config" / "probes" / "v1a"

PROBE_BASES = {
    "asta-modifier": 24_000,
    "kafka-dot": 24_100,
    "clara-counter": 24_200,
    "firefly-damage": 24_300,
    "firefly-transform": 24_400,
    "aglaea-memosprite": 24_500,
}

SOURCE_IDENTITIES = {
    "asta-modifier": {
        1: "character.asta.ability.astrometry.skillp01",
        5: "character.asta.ability.astral-blessing.ultra",
    },
    "kafka-dot": {
        1: "character.kafka.ability.caressing-moonlight.bpskill",
        2: "character.kafka.ability.twilight-trill.ultra",
        3: "character.kafka.ability.gentle-but-cruel.skillp01",
    },
    "clara-counter": {
        2: "character.clara.ability.promise-not-command.ultra",
        3: "character.clara.ability.svarog-watches-over-you.bpskill",
    },
    "firefly-damage": {
        1: "character.firefly.ability.order-aerial-bombardment.bpskill",
        2: "character.firefly.ability.fyrefly-type-iv-complete-combustion.ultra",
        6: "character.firefly.ability.fyrefly-type-iv-deathstar-overload.bpskill",
        7: "character.firefly.trace.08.module-β-autoreactive-armor",
    },
    "firefly-transform": {
        1: "character.firefly.ability.fyrefly-type-iv-complete-combustion.ultra",
        3: "character.firefly.ability.order-aerial-bombardment.bpskill",
        4: "character.firefly.ability.fyrefly-type-iv-deathstar-overload.bpskill",
    },
    "aglaea-memosprite": {
        1: "character.aglaea.ability.rise-exalted-renown.bpskill",
        2: "character.aglaea.ability.rosy-fingered.skillp01",
        3: "character.aglaea.ability.slash-by-a-thousandfold-kiss.normal",
        4: "character.aglaea.ability.dance-destined-weaveress.ultra",
    },
}

TABLES = (
    "Ability",
    "AbilityPhase",
    "ConditionExpression",
    "Effect",
    "EventFilter",
    "ModifierDefinition",
    "ModifierStackingGroup",
    "Operation",
    "Program",
    "ProgramStep",
    "RuleDefinition",
    "RuleTrigger",
    "Selector",
    "StateSlot",
    "StateSlotReset",
    "ValueExpression",
)


def tsv_rows(probe: str, table: str) -> list[dict[str, Any]]:
    path = PROBES / probe / "rows" / f"{table}.tsv"
    if not path.exists():
        return []
    with path.open("r", encoding="utf-8", newline="") as handle:
        return [
            {key: (value if value != "" else None) for key, value in row.items()}
            for row in csv.DictReader(handle, delimiter="\t")
        ]


def content_rows(probe: str) -> dict[int, dict[str, Any]]:
    return {int(row["id"]): row for row in tsv_rows(probe, "ContentIdentity")}


class Remap:
    def __init__(self, probe: str, stable_ids: dict[str, int]):
        self.probe = probe
        self.base = PROBE_BASES[probe]
        self.content = content_rows(probe)
        self.identity = {
            local: stable_ids[stable]
            for local, stable in SOURCE_IDENTITIES[probe].items()
        }
        for local in self.content:
            self.identity.setdefault(local, self.base + local)

    def selector(self, value: int) -> int:
        return self.base + 50 + value

    def expression(self, value: int) -> int:
        return self.base + value

    def operation(self, value: int) -> int:
        return self.base + value

    def condition(self, value: int) -> int:
        return self.base + value

    def filter(self, value: int) -> int:
        return self.base + value

    def stacking(self, value: int) -> int:
        return self.base + value

    def program(self, value: int) -> int:
        if self.probe == "firefly-damage" and value == 6:
            return self.identity[8]
        return self.identity[value]


IDENTITY_KEYS = {
    "ability_id",
    "effect_id",
    "eidolon_id",
    "modifier_identity_id",
    "new_ability_id",
    "old_ability_id",
    "replacement_definition_identity_id",
    "replacement_program_identity_id",
    "rule_id",
    "rule_identity_id",
    "source_effect_id",
    "source_rule_id",
    "slot_identity_id",
    "source_definition_identity_id",
    "state_slot_id",
    "trace_id",
    "unit_definition_identity_id",
}
SELECTOR_KEYS = {
    "actor_selector_id",
    "applier_selector_id",
    "owner_selector_id",
    "subject_selector_id",
    "target_selector_id",
    "selector_id",
}
EXPRESSION_KEYS = {
    "amount_expression_id",
    "base_chance_expression_id",
    "cap_expression_id",
    "comparator_expression_id",
    "default_expression_id",
    "duration_expression_id",
    "floor_expression_id",
    "initial_expression_id",
    "magnitude_comparator_expression_id",
    "maximum_expression_id",
    "minimum_expression_id",
    "value_expression_id",
}


def remap_value(key: str, value: Any, maps: Remap) -> Any:
    if value is None:
        return None
    if isinstance(value, dict):
        return {nested: remap_value(nested, nested_value, maps) for nested, nested_value in value.items()}
    if isinstance(value, list):
        return [remap_value(key, item, maps) for item in value]
    if key in IDENTITY_KEYS:
        return maps.identity[int(value)]
    if key in SELECTOR_KEYS or key.endswith("_selector_id"):
        return maps.selector(int(value))
    if key in EXPRESSION_KEYS or key.endswith("_expression_id"):
        return maps.expression(int(value))
    if key == "operation_id":
        return maps.operation(int(value))
    if key == "condition_id":
        return maps.condition(int(value))
    if key == "filter_id":
        return maps.filter(int(value))
    if key in ("program_id", "then_program_id", "else_program_id", "body_program_id"):
        return maps.program(int(value))
    if key == "stacking_group_id":
        return maps.stacking(int(value))
    return value


def remap_json(value: str | None, maps: Remap) -> str | None:
    if value is None:
        return None
    payload = json.loads(value)
    return json.dumps(remap_value("", payload, maps), separators=(",", ":"))


def remap_row(table: str, row: dict[str, Any], maps: Remap) -> dict[str, Any]:
    result = dict(row)
    id_mapper = {
        "Ability": lambda value: maps.identity[value],
        "AbilityPhase": None,
        "ConditionExpression": maps.condition,
        "Effect": lambda value: maps.identity[value],
        "EventFilter": maps.filter,
        "ModifierDefinition": lambda value: maps.identity[value],
        "ModifierStackingGroup": maps.stacking,
        "Operation": maps.operation,
        "Program": maps.program,
        "ProgramStep": None,
        "RuleDefinition": lambda value: maps.identity[value],
        "RuleTrigger": maps.operation,
        "Selector": maps.selector,
        "StateSlot": lambda value: maps.identity[value],
        "StateSlotReset": None,
        "ValueExpression": maps.expression,
    }[table]
    if id_mapper is not None and result.get("id") is not None:
        result["id"] = id_mapper(int(result["id"]))
    for key, value in list(result.items()):
        if key == "id" or value is None:
            continue
        if key in ("node", "payload", "step"):
            result[key] = remap_json(value, maps)
        elif key in IDENTITY_KEYS:
            result[key] = maps.identity[int(value)]
        elif key in SELECTOR_KEYS or key.endswith("_selector_id"):
            result[key] = maps.selector(int(value))
        elif key in EXPRESSION_KEYS or key.endswith("_expression_id"):
            result[key] = maps.expression(int(value))
        elif key == "operation_id":
            result[key] = maps.operation(int(value))
        elif key == "condition_id":
            result[key] = maps.condition(int(value))
        elif key == "filter_id":
            result[key] = maps.filter(int(value))
        elif key in ("program_id", "then_program_id", "else_program_id", "body_program_id"):
            result[key] = maps.program(int(value))
        elif key == "stacking_group_id":
            result[key] = maps.stacking(int(value))
    return result


def promoted_identity(local: dict[str, Any], global_id: int) -> dict[str, Any]:
    return {
        "id": global_id,
        "stable_key": local["stable_key"].replace("probe.", "program.v1b."),
        "content_kind": local["content_kind"],
        "name_en": local["name_en"].replace(" Probe", ""),
        "name_zh_cn": local["name_zh_cn"].replace("探针", ""),
        "summary_en": local["summary_en"].replace("Non-production ", "Production "),
        "summary_zh_cn": local["summary_zh_cn"].replace("非生产", "生产"),
        "game_version_introduced": local["game_version_introduced"],
        "game_version_snapshot": "4.4",
        "release_state": "Released",
        "enabled": True,
        "coverage_state": "DataReady",
        "source_record_ids": "1",
    }


def selector_identity(probe: str, local_id: int, global_id: int) -> dict[str, Any]:
    label = probe.replace("-", " ").title()
    return {
        "id": global_id,
        "stable_key": f"selector.v1b.{probe}.{local_id}",
        "content_kind": "Selector",
        "name_en": f"{label} Selector {local_id}",
        "name_zh_cn": f"{label}选择器{local_id}",
        "summary_en": "Typed selector promoted with the representative production mechanic.",
        "summary_zh_cn": "随代表性生产机制一并提升的类型化选择器。",
        "game_version_introduced": "unresolved",
        "game_version_snapshot": "4.4",
        "release_state": "Released",
        "enabled": True,
        "coverage_state": "DataReady",
        "source_record_ids": "1",
    }


def generate(stable_ids: dict[str, int]) -> tuple[dict[str, list[dict[str, Any]]], list[dict[str, Any]], dict[str, dict[str, int]]]:
    output = {table: [] for table in TABLES}
    identities: list[dict[str, Any]] = []
    resolved: dict[str, dict[str, int]] = {}
    for probe in PROBE_BASES:
        maps = Remap(probe, stable_ids)
        resolved[probe] = {str(local): global_id for local, global_id in maps.identity.items()}
        for local, row in maps.content.items():
            if local not in SOURCE_IDENTITIES[probe]:
                identities.append(promoted_identity(row, maps.identity[local]))
        selectors = tsv_rows(probe, "Selector")
        for row in selectors:
            local_id = int(row["id"])
            identities.append(selector_identity(probe, local_id, maps.selector(local_id)))
        for table in TABLES:
            for row in tsv_rows(probe, table):
                remapped = remap_row(table, row, maps)
                if table in ("Ability", "AbilityPhase"):
                    local_ability = int(row["ability_id"] if table == "AbilityPhase" else row["id"])
                    if local_ability in SOURCE_IDENTITIES[probe]:
                        continue
                output[table].append(remapped)

    # Firefly's Ultimate composes the four-operation mode entry with the
    # three-operation form/ability replacement program in one authored phase.
    combined_id = 24_601
    identities.append({
        "id": combined_id,
        "stable_key": "program.v1b.firefly.complete-combustion",
        "content_kind": "Program",
        "name_en": "Complete Combustion Composite Program",
        "name_zh_cn": "完全燃烧组合程序",
        "summary_en": "Ordered Complete Combustion state, timeline, form and ability replacement operations.",
        "summary_zh_cn": "按序执行完全燃烧状态、时间线、形态与技能替换操作。",
        "game_version_introduced": "2.3",
        "game_version_snapshot": "4.4",
        "release_state": "Released",
        "enabled": True,
        "coverage_state": "DataReady",
        "source_record_ids": "1",
    })
    output["Program"].append({"id": combined_id, "domain": "Battle"})
    firefly_damage = Remap("firefly-damage", stable_ids)
    firefly_transform = Remap("firefly-transform", stable_ids)
    combined_operations = [
        firefly_transform.operation(value) for value in (1, 2, 3)
    ] + [firefly_damage.operation(value) for value in (4, 5, 6, 7)]
    for sequence, operation_id in enumerate(combined_operations, start=1):
        output["ProgramStep"].append({
            "program_id": combined_id,
            "sequence": sequence,
            "step": json.dumps({"type": "Operation", "operation_id": operation_id}, separators=(",", ":")),
        })

    for rows in output.values():
        rows.sort(key=lambda row: tuple(str(value) for value in row.values()))
    identities.sort(key=lambda row: int(row["id"]))
    return output, identities, resolved


def phase_program_overrides(stable_ids: dict[str, int], resolved: dict[str, dict[str, int]]) -> dict[int, int]:
    return {
        stable_ids["character.asta.ability.astral-blessing.ultra"]: resolved["asta-modifier"]["7"],
        stable_ids["character.kafka.ability.caressing-moonlight.bpskill"]: resolved["kafka-dot"]["5"],
        stable_ids["character.kafka.ability.twilight-trill.ultra"]: resolved["kafka-dot"]["6"],
        stable_ids["character.firefly.ability.order-aerial-bombardment.bpskill"]: resolved["firefly-damage"]["3"],
        stable_ids["character.firefly.ability.fyrefly-type-iv-complete-combustion.ultra"]: 24_601,
        stable_ids["character.firefly.ability.fyrefly-type-iv-deathstar-overload.bpskill"]: resolved["firefly-damage"]["8"],
        stable_ids["character.aglaea.ability.rise-exalted-renown.bpskill"]: resolved["aglaea-memosprite"]["6"],
    }


def entry_rule_overrides(stable_ids: dict[str, int], resolved: dict[str, dict[str, int]]) -> dict[int, int]:
    return {
        stable_ids["character.asta.ability.astrometry.skillp01"]: resolved["asta-modifier"]["2"],
        stable_ids["character.kafka.ability.gentle-but-cruel.skillp01"]: resolved["kafka-dot"]["8"],
        stable_ids["character.clara.ability.because-were-family.skillp01"]: resolved["clara-counter"]["5"],
    }
