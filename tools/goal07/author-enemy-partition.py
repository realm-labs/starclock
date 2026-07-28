"""Author and verify Goal 07 enemy partitions in the core production workbooks.

S01 owns the Abundant Ebon Deer (Complete) definition, its three finite AI
graphs, level-specific Universe stats, branch summons and executable abilities.
"""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
from typing import Any, Callable

from openpyxl import load_workbook
from openpyxl.cell.cell import TYPE_ERROR, TYPE_FORMULA


ROOT = Path(__file__).resolve().parents[2]
DATA = ROOT / "config" / "data"
DEBUG = ROOT / "config" / "generated" / "debug-json"
PARTITIONS = (
    ROOT
    / "content-manifests"
    / "standard-universe-mechanics-complete-v1"
    / "content-partitions.json"
)
ANCHOR = (
    ROOT
    / "evidence"
    / "standard-universe-mechanics-complete-v1"
    / "sources"
    / "G07-P5-M15-S01-numeric-anchors.json"
)
PARTITION = "G07-P5-M15-S01"
VARIANT_KEY = "enemy.abundant-ebon-deer-complete.littleboss.variant.01"
BASE = 980_000


def json_cell(type_: str, **fields: Any) -> str:
    return json.dumps({"type": type_, **fields}, separators=(",", ":"))


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def sha256_text(value: str) -> str:
    return sha256_bytes(value.encode("utf-8"))


def fields(sheet: Any) -> dict[str, int]:
    return {
        str(cell.value): cell.column
        for cell in sheet[3]
        if cell.value not in (None, "#field")
    }


def sheet_rows(sheet: Any) -> list[dict[str, Any]]:
    columns = fields(sheet)
    result: list[dict[str, Any]] = []
    for cells in sheet.iter_rows(min_row=8):
        row = {name: cells[column - 1].value for name, column in columns.items()}
        if all(value is None for value in row.values()):
            continue
        for column in columns.values():
            cell = cells[column - 1]
            if cell.data_type in (TYPE_ERROR, TYPE_FORMULA):
                raise ValueError(
                    f"{sheet.title}/{cell.coordinate}: formulas and errors are forbidden"
                )
        result.append(row)
    return result


def workbook_rows(table: str) -> list[dict[str, Any]]:
    workbook = load_workbook(DATA / f"{table}.xlsx", read_only=True, data_only=False)
    return sheet_rows(workbook.active)


def write_rows(table: str, rows: list[dict[str, Any]]) -> None:
    path = DATA / f"{table}.xlsx"
    workbook = load_workbook(path)
    sheet = workbook.active
    columns = fields(sheet)
    for row in rows:
        unknown = set(row) - set(columns)
        if unknown:
            raise ValueError(f"{table}: unknown fields {sorted(unknown)}")
    if sheet.max_row >= 8:
        sheet.delete_rows(8, sheet.max_row - 7)
    for row_index, row in enumerate(rows, start=8):
        for name, column in columns.items():
            value = row.get(name)
            if value is not None:
                sheet.cell(row=row_index, column=column, value=value)
    workbook.save(path)


def normalize(value: Any) -> Any:
    if value is None or isinstance(value, bool):
        return value
    if isinstance(value, str):
        stripped = value.strip()
        if stripped.startswith(("{", "[")):
            try:
                return normalize(json.loads(stripped))
            except json.JSONDecodeError:
                pass
        return value
    if isinstance(value, list):
        return [normalize(item) for item in value]
    if isinstance(value, dict):
        if set(value) == {"Object"}:
            return normalize(value["Object"])
        return {key: normalize(inner) for key, inner in sorted(value.items())}
    return str(value)


def normalize_field(field: str, value: Any) -> Any:
    if field in {"ability_ids", "source_record_ids"}:
        if isinstance(value, str):
            value = [part for part in value.split("|") if part]
        if isinstance(value, list):
            return [normalize(item) for item in value]
    return normalize(value)


def semantic_digest(rows: list[dict[str, Any]]) -> str:
    projected = [
        {
            key: normalize_field(key, value)
            for key, value in sorted(row.items())
            if value is not None
        }
        for row in rows
    ]
    projected.sort(
        key=lambda row: json.dumps(
            row, ensure_ascii=False, sort_keys=True, separators=(",", ":")
        )
    )
    return sha256_text(
        json.dumps(
            projected, ensure_ascii=False, sort_keys=True, separators=(",", ":")
        )
    )


def unwrap(value: Any) -> Any:
    if value == "Null":
        return None
    if isinstance(value, list):
        return [unwrap(item) for item in value]
    if isinstance(value, dict):
        if len(value) == 1:
            tag, inner = next(iter(value.items()))
            if tag in {"String", "Integer", "Bool", "Decimal"}:
                return inner
            if tag == "List":
                return [unwrap(item) for item in inner]
        return {key: unwrap(inner) for key, inner in value.items()}
    return value


def sora_rows(table: str) -> list[dict[str, Any]]:
    payload = json.loads((DEBUG / f"{table}.json").read_text(encoding="utf-8"))
    return [
        {key: unwrap(value) for key, value in row["values"].items()}
        for row in payload["table"]["rows"]
    ]


def identity(
    id_: int,
    stable_key: str,
    kind: str,
    name_en: str,
    name_zh_cn: str,
    summary: str,
    sources: str = "1",
) -> dict[str, Any]:
    return {
        "id": id_,
        "stable_key": stable_key,
        "content_kind": kind,
        "name_en": name_en,
        "name_zh_cn": name_zh_cn,
        "summary_en": summary,
        "summary_zh_cn": "Goal 07 S01 来源绑定的丰饶玄鹿可执行定义。",
        "game_version_introduced": "1.2",
        "game_version_snapshot": "4.4",
        "release_state": "Released",
        "enabled": True,
        "coverage_state": "GoldenVerified",
        "source_record_ids": sources,
    }


def selector(
    id_: int,
    origin: str,
    side: str,
    life: str = "Alive",
    presence: str = "Present",
    minimum: int = 1,
    maximum: int = 1,
    empty: str = "Fault",
    choice: str = "First",
) -> dict[str, Any]:
    return {
        "id": id_,
        "domain": "Battle",
        "origin": origin,
        "side_relationship": side,
        "life": life,
        "presence": presence,
        "reference_point": "CurrentState",
        "ordering": "Formation",
        "minimum_count": minimum,
        "maximum_count": maximum,
        "empty_pool_policy": empty,
        "choice": choice,
        "allow_repeated_targets": False,
    }


def operation(
    id_: int,
    key: str,
    payload: str,
    target: int | None = None,
    empty: str = "Fault",
) -> dict[str, Any]:
    return {
        "id": id_,
        "stable_key": f"goal07.enemy.s01.operation.{key}",
        "domain": "Battle",
        "target_selector_id": target,
        "empty_target_policy": empty,
        "snapshot_boundary": "Dynamic",
        "fault_policy": "Rollback",
        "payload": payload,
    }


def owned_rows() -> dict[str, list[dict[str, Any]]]:
    anchor = json.loads(ANCHOR.read_text(encoding="utf-8"))
    manifest = json.loads(PARTITIONS.read_text(encoding="utf-8"))
    assigned = next(item for item in manifest["partitions"] if item["id"] == PARTITION)
    if assigned["enemy_variant_ids"] != [VARIANT_KEY]:
        raise ValueError("S01 frozen enemy assignment changed")

    variant = BASE + 1
    template = BASE + 2
    graphs = [BASE + 10, BASE + 11, BASE + 12]
    boss_abilities = {skill: BASE + 100 + skill for skill in range(1, 15)}
    branch_abilities = {
        "wintry": BASE + 121,
        "maple": BASE + 122,
        "glorious": BASE + 123,
        "lavish": BASE + 124,
    }
    linked = {
        "wintry-1": BASE + 201,
        "wintry-2": BASE + 202,
        "maple-1": BASE + 203,
        "maple-2": BASE + 204,
        "glorious-1": BASE + 205,
        "lavish-1": BASE + 206,
        "lavish-2": BASE + 207,
    }
    selectors = {
        "actor": BASE + 401,
        "owner": BASE + 402,
        "opposing-single": BASE + 403,
        "opposing-all": BASE + 404,
        "same-all": BASE + 405,
        "branches": BASE + 406,
        "lavish-branches": BASE + 407,
        "current-subject": BASE + 408,
        "maple-branches": BASE + 409,
        "primary-target": BASE + 410,
    }
    effects = {
        "wind-shear": BASE + 501,
        "shock": BASE + 502,
        "synwood-renewal": BASE + 503,
        "hardy-leaf": BASE + 504,
        "engender": BASE + 505,
        "vigor-overflow": BASE + 506,
        "maple-counter": BASE + 507,
        "outrage": BASE + 508,
    }
    modifiers = {
        "hardy-leaf-defense": BASE + 521,
        "engender-attack": BASE + 523,
        "vigor-damage": BASE + 524,
    }
    modifier_groups = {
        "hardy-leaf-defense": BASE + 522,
        "engender-attack": BASE + 525,
        "vigor-damage": BASE + 526,
    }
    programs: dict[str, int] = {}
    next_program = BASE + 301
    next_operation = BASE + 1_001
    next_expression = BASE + 1_101
    rows: dict[str, list[dict[str, Any]]] = {}

    def add(table: str, row: dict[str, Any]) -> None:
        rows.setdefault(table, []).append(row)

    identities = [
        identity(
            variant,
            VARIANT_KEY,
            "EnemyVariant",
            "Abundant Ebon Deer (Complete)",
            "丰饶玄鹿（完整）",
            "Goal 07 exact three-phase boss variant with reviewed Universe level rows.",
            "1|3",
        ),
        identity(
            template,
            "enemy.abundant-ebon-deer-complete.littleboss",
            "Enemy",
            "Abundant Ebon Deer (Complete) Template",
            "丰饶玄鹿（完整）模板",
            "Version 4.4 boss template retained from source monster 2024011.",
        ),
    ]
    for sequence, graph in enumerate(graphs, start=1):
        identities.append(
            identity(
                graph,
                f"ai.goal07.abundant-ebon-deer-complete.phase-{sequence}",
                "AiGraph",
                f"Abundant Ebon Deer Phase {sequence} AI",
                f"丰饶玄鹿第{sequence}阶段AI",
                "Finite source-ordered phase action graph.",
            )
        )
    for skill, ability in boss_abilities.items():
        identities.append(
            identity(
                ability,
                f"enemy.abundant-ebon-deer-complete.littleboss.ability.skill{skill:02d}",
                "Ability",
                f"Abundant Ebon Deer Skill {skill:02d}",
                f"丰饶玄鹿技能{skill:02d}",
                f"Executable transcription of source skill 2024011{skill:02d}.",
            )
        )
    for name, ability in branch_abilities.items():
        identities.append(
            identity(
                ability,
                f"enemy.abundant-ebon-deer-complete.branch.ability.{name}",
                "Ability",
                f"Abundant Ebon Deer {name.title()} Branch Action",
                f"丰饶玄鹿{name}枝条行动",
                "Executable linked-branch action used by the S01 boss.",
            )
        )
    for name, unit in linked.items():
        identities.append(
            identity(
                unit,
                f"unit.goal07.abundant-ebon-deer-complete.branch.{name}",
                "CharacterForm",
                f"Abundant Ebon Deer Branch {name}",
                f"丰饶玄鹿枝条{name}",
                "Internal linked-unit slot preserving one authored branch summon.",
            )
        )
    for name, selector_id in selectors.items():
        identities.append(
            identity(
                selector_id,
                f"selector.goal07.abundant-ebon-deer-complete.{name}",
                "Selector",
                f"Abundant Ebon Deer {name} Selector",
                f"丰饶玄鹿{name}选择器",
                "S01 battle selector.",
            )
        )
    for name, effect in effects.items():
        identities.append(
            identity(
                effect,
                f"effect.goal07.abundant-ebon-deer-complete.{name}",
                "Effect",
                f"Abundant Ebon Deer {name} Effect",
                f"丰饶玄鹿{name}效果",
                "S01 executable enemy effect.",
            )
        )
    for name, modifier in modifiers.items():
        display = {
            "hardy-leaf-defense": ("Hardy Leaf Defense", "蕉覆防御", "Exact +200% base DEF."),
            "engender-attack": ("Engender Attack", "繁生攻击", "Exact +30% base ATK for two turns."),
            "vigor-damage": (
                "Vigor Overflow Damage",
                "生机充盈伤害",
                "Exact +15% damage per enemy-granted buff occurrence.",
            ),
        }[name]
        identities.append(
            identity(
                modifier,
                f"modifier.goal07.abundant-ebon-deer-complete.{name}",
                "Modifier",
                display[0],
                display[1],
                display[2],
            )
        )

    add(
        "Selector",
        selector(selectors["actor"], "Actor", "SameSide"),
    )
    add(
        "Selector",
        selector(selectors["owner"], "Owner", "SameSide"),
    )
    add(
        "Selector",
        selector(selectors["current-subject"], "CurrentSubject", "SameSide"),
    )
    add(
        "Selector",
        selector(selectors["opposing-single"], "Actor", "OpposingSide"),
    )
    add(
        "Selector",
        selector(selectors["primary-target"], "PrimaryTarget", "OpposingSide"),
    )
    add(
        "Selector",
        selector(
            selectors["opposing-all"],
            "Actor",
            "OpposingSide",
            minimum=1,
            maximum=4,
            choice="All",
        ),
    )
    add(
        "Selector",
        selector(
            selectors["same-all"],
            "Actor",
            "SameSide",
            minimum=1,
            maximum=16,
            choice="All",
        ),
    )
    add(
        "Selector",
        selector(
            selectors["branches"],
            "Actor",
            "SameSide",
            presence="Linked",
            minimum=0,
            maximum=16,
            empty="NoOp",
            choice="All",
        ),
    )
    add(
        "SelectorPredicate",
        {
            "selector_id": selectors["branches"],
            "sequence": 1,
            "predicate": json_cell(
                "OwnedBy", owner_selector_id=selectors["actor"]
            ),
        },
    )
    add(
        "Selector",
        selector(
            selectors["maple-branches"],
            "Actor",
            "SameSide",
            presence="Linked",
            minimum=0,
            maximum=2,
            empty="NoOp",
            choice="All",
        ),
    )
    add(
        "SelectorPredicate",
        {
            "selector_id": selectors["maple-branches"],
            "sequence": 1,
            "predicate": json_cell(
                "OwnedBy", owner_selector_id=selectors["actor"]
            ),
        },
    )
    add(
        "SelectorPredicate",
        {
            "selector_id": selectors["maple-branches"],
            "sequence": 2,
            "predicate": json_cell(
                "FormationRange", minimum_index=10, maximum_index=11
            ),
        },
    )
    add(
        "Selector",
        selector(
            selectors["lavish-branches"],
            "Actor",
            "SameSide",
            presence="Linked",
            minimum=0,
            maximum=2,
            empty="NoOp",
            choice="All",
        ),
    )
    add(
        "SelectorPredicate",
        {
            "selector_id": selectors["lavish-branches"],
            "sequence": 1,
            "predicate": json_cell(
                "OwnedBy", owner_selector_id=selectors["actor"]
            ),
        },
    )
    add(
        "SelectorPredicate",
        {
            "selector_id": selectors["lavish-branches"],
            "sequence": 2,
            "predicate": json_cell(
                "FormationRange", minimum_index=13, maximum_index=14
            ),
        },
    )

    expression_ids: dict[str, int] = {}

    def expr(name: str, kind: str, node: str) -> int:
        nonlocal next_expression
        id_ = next_expression
        next_expression += 1
        expression_ids[name] = id_
        add(
            "ValueExpression",
            {
                "id": id_,
                "stable_key": f"goal07.enemy.s01.expression.{name}",
                "result_kind": kind,
                "node": node,
            },
        )
        return id_

    actor_atk = expr(
        "actor-atk",
        "Scalar",
        json_cell(
            "QueryStat",
            subject_selector_id=selectors["actor"],
            stat="Atk",
            formula_purpose="OrdinaryDamage",
        ),
    )
    owner_atk = expr(
        "owner-atk",
        "Scalar",
        json_cell(
            "QueryStat",
            subject_selector_id=selectors["owner"],
            stat="Atk",
            formula_purpose="OrdinaryDamage",
        ),
    )
    actor_hp_heal = expr(
        "actor-hp-heal",
        "Scalar",
        json_cell(
            "QueryStat",
            subject_selector_id=selectors["actor"],
            stat="Hp",
            formula_purpose="Healing",
        ),
    )
    owner_hp_heal = expr(
        "owner-hp-heal",
        "Scalar",
        json_cell(
            "QueryStat",
            subject_selector_id=selectors["owner"],
            stat="Hp",
            formula_purpose="Healing",
        ),
    )
    ratios: dict[str, int] = {}
    for name, value in {
        "ratio-3-5": "3.5",
        "ratio-2-8": "2.8",
        "ratio-2-7": "2.7",
        "ratio-1-5": "1.5",
        "ratio-1-2": "1.2",
        "ratio-0-5": "0.5",
        "ratio-0-25": "0.25",
        "ratio-0-3": "0.3",
        "ratio-0-15": "0.15",
        "ratio-0-1": "0.1",
        "ratio-0-04": "0.04",
        "ratio-1": "1",
        "ratio-2": "2",
    }.items():
        ratios[name] = expr(
            name, "Scalar", json_cell("ScalarLiteral", value_decimal=value)
        )
    duration_two = expr(
        "duration-two", "Integer", json_cell("IntegerLiteral", value=2)
    )
    duration_one = expr(
        "duration-one", "Integer", json_cell("IntegerLiteral", value=1)
    )
    branch_count = expr(
        "branch-count",
        "Integer",
        json_cell("SelectorCount", selector_id=selectors["branches"]),
    )
    branch_count_scalar = expr(
        "branch-count-scalar",
        "Scalar",
        json_cell(
            "Convert",
            operand_expression_id=branch_count,
            target_kind="Scalar",
            rounding="NearestTiesAway",
        ),
    )
    vigor_stacks = expr(
        "vigor-stacks",
        "Integer",
        json_cell(
            "QueryEffectStacks",
            subject_selector_id=selectors["current-subject"],
            effect_id=effects["vigor-overflow"],
        ),
    )
    vigor_stacks_scalar = expr(
        "vigor-stacks-scalar",
        "Scalar",
        json_cell(
            "Convert",
            operand_expression_id=vigor_stacks,
            target_kind="Scalar",
            rounding="NearestTiesAway",
        ),
    )

    def multiply(name: str, left: int, right: int) -> int:
        return expr(
            name,
            "Scalar",
            json_cell(
                "CheckedBinary",
                operator="CheckedMultiply",
                left_expression_id=left,
                right_expression_id=right,
                rounding="NearestTiesAway",
            ),
        )

    def add_expr(name: str, left: int, right: int) -> int:
        return expr(
            name,
            "Scalar",
            json_cell(
                "CheckedBinary",
                operator="CheckedAdd",
                left_expression_id=left,
                right_expression_id=right,
                rounding="NearestTiesAway",
            ),
        )

    damage_amounts = {
        "wavering": multiply("wavering-damage", actor_atk, ratios["ratio-3-5"]),
        "caress": multiply("caress-damage", actor_atk, ratios["ratio-2-8"]),
        "wintry": multiply("wintry-damage", actor_atk, ratios["ratio-1-2"]),
        "maple": multiply("maple-damage", actor_atk, ratios["ratio-1-5"]),
        "maple-counter": multiply(
            "maple-counter-damage", owner_atk, ratios["ratio-1-5"]
        ),
    }
    vigor_damage_boost = multiply(
        "vigor-damage-boost",
        vigor_stacks_scalar,
        ratios["ratio-0-15"],
    )
    gore_bonus = multiply(
        "gore-branch-bonus",
        branch_count_scalar,
        ratios["ratio-0-5"],
    )
    gore_ratio = add_expr("gore-ratio", ratios["ratio-2-7"], gore_bonus)
    damage_amounts["gore"] = multiply("gore-damage", actor_atk, gore_ratio)
    heal_amounts = {
        "everlife": multiply("everlife-heal", actor_hp_heal, ratios["ratio-0-1"]),
        "renewal": multiply("renewal-heal", owner_hp_heal, ratios["ratio-0-04"]),
        "last-spring": multiply(
            "last-spring-heal", actor_hp_heal, ratios["ratio-0-25"]
        ),
    }
    dot_amounts = {
        "wind-shear": multiply(
            "wind-shear-damage", actor_atk, ratios["ratio-0-5"]
        ),
        "shock": multiply("shock-damage", actor_atk, ratios["ratio-0-5"]),
    }

    def program(name: str, operations: list[dict[str, Any]]) -> int:
        nonlocal next_program
        id_ = next_program
        next_program += 1
        programs[name] = id_
        identities.append(
            identity(
                id_,
                f"program.goal07.abundant-ebon-deer-complete.{name}",
                "Program",
                f"Abundant Ebon Deer {name} Program",
                f"丰饶玄鹿{name}程序",
                "Ordered Rule IR program for the S01 enemy.",
            )
        )
        add("Program", {"id": id_, "domain": "Battle"})
        for sequence, row in enumerate(operations, start=1):
            add(
                "ProgramStep",
                {
                    "program_id": id_,
                    "sequence": sequence,
                    "step": json_cell("Operation", operation_id=row["id"]),
                },
            )
            add("Operation", row)
        return id_

    def damage_op(
        name: str,
        amount: int,
        element: str,
        all_targets: bool,
        target_selector: int | None = None,
    ) -> dict[str, Any]:
        nonlocal next_operation
        id_ = next_operation
        next_operation += 1
        return operation(
            id_,
            name,
            json_cell(
                "Damage",
                amount_expression_id=amount,
                damage_class="Ordinary",
                element=element,
                can_crit=True,
            ),
            target_selector
            if target_selector is not None
            else selectors["opposing-all" if all_targets else "opposing-single"],
        )

    def heal_op(name: str, amount: int, target: int) -> dict[str, Any]:
        nonlocal next_operation
        id_ = next_operation
        next_operation += 1
        return operation(
            id_,
            name,
            json_cell("Heal", amount_expression_id=amount),
            target,
        )

    def apply_effect_op(
        name: str,
        effect: int,
        target: int,
        resistible: bool,
        stacks_expression_id: int | None = None,
    ) -> dict[str, Any]:
        nonlocal next_operation
        id_ = next_operation
        next_operation += 1
        return operation(
            id_,
            name,
            json_cell(
                "ApplyEffect",
                effect_id=effect,
                stacks_expression_id=stacks_expression_id,
                chance_policy="Resistible" if resistible else "Guaranteed",
                base_chance_expression_id=ratios["ratio-1"] if resistible else None,
                rng_purpose_key="effect-application" if resistible else None,
            ),
            target,
        )

    def summon_op(name: str, unit: int) -> dict[str, Any]:
        nonlocal next_operation
        id_ = next_operation
        next_operation += 1
        return operation(
            id_,
            name,
            json_cell(
                "Summon",
                unit_definition_identity_id=unit,
                owner_selector_id=selectors["actor"],
            ),
            None,
        )

    def grant_extra_turn_op(name: str) -> dict[str, Any]:
        nonlocal next_operation
        id_ = next_operation
        next_operation += 1
        return operation(
            id_,
            name,
            json_cell(
                "GrantExtraTurn",
                actor_selector_id=selectors["current-subject"],
            ),
            None,
        )

    def despawn_lavish_branches_op(name: str) -> dict[str, Any]:
        nonlocal next_operation
        id_ = next_operation
        next_operation += 1
        return operation(
            id_,
            name,
            json_cell("Despawn"),
            selectors["lavish-branches"],
            "NoOp",
        )

    ability_programs: dict[int, int] = {}
    ability_programs[boss_abilities[5]] = program(
        "wavering-bleat",
        [
            damage_op(
                "wavering-bleat-damage",
                damage_amounts["wavering"],
                "Lightning",
                False,
                selectors["primary-target"],
            )
        ],
    )
    ability_programs[boss_abilities[6]] = program(
        "caress-of-wind",
        [damage_op("caress-of-wind-damage", damage_amounts["caress"], "Wind", True)],
    )
    ability_programs[boss_abilities[8]] = program(
        "flamboyant-gore",
        [
            damage_op("flamboyant-gore-damage", damage_amounts["gore"], "Lightning", True),
            despawn_lavish_branches_op("gore-despawn-lavish-branches"),
        ],
    )
    ability_programs[boss_abilities[9]] = program(
        "everlife",
        [
            heal_op("everlife-heal", heal_amounts["everlife"], selectors["actor"]),
            apply_effect_op(
                "everlife-synwood-renewal",
                effects["synwood-renewal"],
                selectors["actor"],
                False,
            ),
            apply_effect_op(
                "everlife-vigor-owner",
                effects["vigor-overflow"],
                selectors["actor"],
                False,
            ),
            apply_effect_op(
                "everlife-vigor-branches",
                effects["vigor-overflow"],
                selectors["branches"],
                False,
            ),
        ],
    )
    ability_programs[boss_abilities[7]] = program(
        "hardy-leaf-sheath",
        [
            apply_effect_op(
                "hardy-leaf-sheath",
                effects["hardy-leaf"],
                selectors["actor"],
                False,
            ),
            apply_effect_op(
                "hardy-leaf-vigor-owner",
                effects["vigor-overflow"],
                selectors["actor"],
                False,
            ),
            apply_effect_op(
                "hardy-leaf-vigor-branches",
                effects["vigor-overflow"],
                selectors["branches"],
                False,
            ),
        ],
    )
    for skill, names in {
        10: ["wintry-1", "wintry-2"],
        11: ["maple-1", "maple-2"],
        12: ["maple-1", "maple-2", "wintry-1", "glorious-1"],
        13: ["lavish-1", "lavish-2", "maple-1", "wintry-1"],
    }.items():
        summon_operations = [
            summon_op(f"summon-skill-{skill:02d}-{name}", linked[name])
            for name in names
        ]
        if any(name.startswith("maple-") for name in names):
            summon_operations.append(
                apply_effect_op(
                    f"summon-skill-{skill:02d}-maple-counter",
                    effects["maple-counter"],
                    selectors["maple-branches"],
                    False,
                )
            )
        if skill == 13:
            summon_operations.extend(
                [
                    damage_op(
                        "entwined-vines-immediate-gore",
                        damage_amounts["gore"],
                        "Lightning",
                        True,
                    ),
                    despawn_lavish_branches_op(
                        "entwined-vines-despawn-lavish-branches"
                    ),
                ]
            )
        ability_programs[boss_abilities[skill]] = program(
            f"summon-skill-{skill:02d}",
            summon_operations,
        )
    ability_programs[branch_abilities["wintry"]] = program(
        "wintry-branch-action",
        [
            damage_op(
                "wintry-branch-damage",
                damage_amounts["wintry"],
                "Wind",
                False,
                selectors["primary-target"],
            ),
            apply_effect_op(
                "wintry-branch-wind-shear",
                effects["wind-shear"],
                selectors["primary-target"],
                True,
            ),
            apply_effect_op(
                "wintry-branch-outrage",
                effects["outrage"],
                selectors["primary-target"],
                True,
            ),
        ],
    )
    ability_programs[branch_abilities["maple"]] = program(
        "maple-branch-action",
        [
            apply_effect_op(
                "maple-engender",
                effects["engender"],
                selectors["owner"],
                False,
            ),
            apply_effect_op(
                "maple-vigor-owner",
                effects["vigor-overflow"],
                selectors["owner"],
                False,
            ),
            apply_effect_op(
                "maple-vigor-branches",
                effects["vigor-overflow"],
                selectors["branches"],
                False,
            ),
        ],
    )
    ability_programs[branch_abilities["glorious"]] = program(
        "glorious-branch-action",
        [heal_op("glorious-last-spring", heal_amounts["last-spring"], selectors["same-all"])],
    )

    renewal_program = program(
        "synwood-renewal-turn-heal",
        [heal_op("synwood-renewal-turn-heal", heal_amounts["renewal"], selectors["owner"])],
    )
    maple_counter_program = program(
        "maple-counter",
        [
            damage_op(
                "maple-counter-damage",
                damage_amounts["maple-counter"],
                "Lightning",
                False,
                selectors["actor"],
            ),
            apply_effect_op(
                "maple-counter-shock",
                effects["shock"],
                selectors["actor"],
                True,
            ),
        ],
    )
    phase_three_entry_program = program(
        "phase-three-extra-actions",
        [
            grant_extra_turn_op("phase-three-extra-action-one"),
            grant_extra_turn_op("phase-three-extra-action-two"),
        ],
    )

    target_patterns = {
        5: "SingleTarget",
        6: "Aoe",
        8: "Aoe",
        7: "Enhance",
        9: "Enhance",
        10: "Enhance",
        11: "Enhance",
        12: "Enhance",
        13: "Enhance",
    }
    for skill, ability in boss_abilities.items():
        kind = "Passive" if skill == 14 else "Skill"
        pattern = target_patterns.get(skill, "None")
        add(
            "Ability",
            {
                "id": ability,
                "kind": kind,
                "target_pattern": pattern,
                "retarget_policy": "CancelRemaining",
                "level_cap": 1,
                "cooldown_actions": 1,
                "semantic_tags_mask": 5 if skill in {5, 6, 8, 14} else 4,
            },
        )
        add(
            "AbilityPhase",
            {
                "ability_id": ability,
                "sequence": 1,
                "kind": "Resolved",
                "program_identity_id": ability_programs.get(ability),
            },
        )
        add(
            "EnemyAbility",
            {
                "id": ability,
                "telegraph": "Charge" if skill == 8 else "None",
                "cooldown_actions": 1,
                "initial_cooldown_actions": 0,
                "charge_actions": 0,
                "ai_tag": f"skill{skill:02d}",
            },
        )
    for name, ability in branch_abilities.items():
        add(
            "Ability",
            {
                "id": ability,
                "kind": "Summon",
                "target_pattern": (
                    "Support" if name == "glorious" else
                    "None" if name == "lavish" else
                    "SingleTarget"
                ),
                "retarget_policy": "CancelRemaining",
                "level_cap": 1,
                "cooldown_actions": 0,
                "semantic_tags_mask": 5 if name in {"wintry", "maple"} else 4,
            },
        )
        add(
            "AbilityPhase",
            {
                "ability_id": ability,
                "sequence": 1,
                "kind": "Resolved",
                "program_identity_id": ability_programs.get(ability),
            },
        )
        add(
            "EnemyAbility",
            {
                "id": ability,
                "telegraph": "None",
                "cooldown_actions": 0,
                "initial_cooldown_actions": 0,
                "charge_actions": 0,
                "ai_tag": f"branch-{name}",
            },
        )

    for name, effect in effects.items():
        is_dot = name in {"wind-shear", "shock"}
        permanent = name in {"vigor-overflow", "maple-counter"}
        add(
            "Effect",
            {
                "id": effect,
                "category": (
                    "Dot" if is_dot else
                    "Control" if name == "outrage" else
                    "NeutralState" if name == "maple-counter" else
                    "Buff"
                ),
                "dispel_category": (
                    "DispellableDebuff" if is_dot else
                    "CleanseableControl" if name == "outrage" else
                    "NonDispellable" if permanent else
                    "DispellableBuff"
                ),
                "stack_limit": (
                    5 if name == "wind-shear" else
                    100 if name == "vigor-overflow" else
                    1
                ),
                "duration_expression_id": (
                    None if permanent else
                    duration_two if name in {"wind-shear", "shock", "engender", "outrage"} else
                    duration_one
                ),
                "duration_clock": "Permanent" if permanent else "TargetTurnEnd",
                "tick_phase": "TurnStart" if is_dot else "None",
                "stack_policy": (
                    "RefreshAndAddStacks"
                    if name in {"wind-shear", "vigor-overflow"}
                    else "Refresh"
                ),
                "magnitude_comparator_expression_id": dot_amounts.get(name),
                "dot_element": (
                    "Wind" if name == "wind-shear" else
                    "Lightning" if name == "shock" else None
                ),
                "snapshot_policy": "OnApplication",
                "teardown_policy": "RemoveWithOwner",
                "application_priority": 0,
            },
        )

    add(
        "EffectTag",
        {
            "effect_id": effects["hardy-leaf"],
            "sequence": 1,
            "tag": "prevents-toughness-reduction",
        },
    )
    add(
        "EffectTag",
        {
            "effect_id": effects["outrage"],
            "sequence": 1,
            "tag": "forced-basic-attack-random-ally",
        },
    )

    for name, group in modifier_groups.items():
        add(
            "ModifierStackingGroup",
            {
                "id": group,
                "stable_key": f"goal07.enemy.s01.{name}",
                "aggregation": "Sum",
            },
        )
    modifier_rows = {
        "hardy-leaf-defense": {
            "effect": "hardy-leaf",
            "stat": "Def",
            "stage": "PercentOfBase",
            "value": ratios["ratio-2"],
            "snapshot": "OnApplication",
        },
        "engender-attack": {
            "effect": "engender",
            "stat": "Atk",
            "stage": "PercentOfBase",
            "value": ratios["ratio-0-3"],
            "snapshot": "OnApplication",
        },
        "vigor-damage": {
            "effect": "vigor-overflow",
            "stat": "Atk",
            "stage": "DamageBoost",
            "value": vigor_damage_boost,
            "snapshot": "Dynamic",
        },
    }
    for name, modifier in modifiers.items():
        definition = modifier_rows[name]
        add(
            "ModifierDefinition",
            {
                "id": modifier,
                "source_effect_id": effects[definition["effect"]],
                "owner_selector_id": selectors["owner"],
                "subject_selector_id": selectors["current-subject"],
                "stat": definition["stat"],
                "formula_stage": definition["stage"],
                "formula_purpose": "Stat",
                "value_expression_id": definition["value"],
                "value_domain": "Ratio",
                "stacking_group_id": modifier_groups[name],
                "priority": 0,
                "cap_formula_stage": definition["stage"],
                "snapshot_policy": definition["snapshot"],
                "duration_scope": "Turn",
            },
        )
        add(
            "EffectModifierBinding",
            {
                "effect_id": effects[definition["effect"]],
                "sequence": 1,
                "modifier_id": modifier,
            },
        )

    regen_rule = BASE + 541
    regen_filter = BASE + 542
    regen_trigger = BASE + 543
    counter_rule = BASE + 544
    counter_filter = BASE + 545
    counter_trigger = BASE + 546
    identities.append(
        identity(
            regen_rule,
            "rule.goal07.abundant-ebon-deer-complete.synwood-renewal",
            "Rule",
            "Synwood Renewal Turn Heal",
            "古木逢春回合治疗",
            "Effect-owned turn-start heal rule.",
        )
    )
    add(
        "RuleDefinition",
        {
            "id": regen_rule,
            "domain": "Battle",
            "source_definition_identity_id": effects["synwood-renewal"],
            "source_class": "Effect",
            "source_digest_sha256": (
                "316d3542e27f1a899ceed57eab31bdbcbd60ec6dd4168f638f00d75b80d0590a"
            ),
        },
    )
    add(
        "EventFilter",
        {
            "id": regen_filter,
            "stable_key": "goal07.enemy.s01.filter.synwood-renewal-owner-turn",
            "owner_selector_id": selectors["owner"],
            "cause_ancestry": "Any",
        },
    )
    add(
        "RuleTrigger",
        {
            "id": regen_trigger,
            "stable_key": "goal07.enemy.s01.trigger.synwood-renewal-owner-turn",
            "rule_id": regen_rule,
            "sequence": 1,
            "event": json_cell("Turn", point="Started"),
            "phase": "AfterEvent",
            "filter_id": regen_filter,
            "condition_id": BASE + 551,
            "once_scope": "Event",
            "priority": 0,
            "program_id": renewal_program,
        },
    )
    add(
        "EffectRuleBinding",
        {
            "effect_id": effects["synwood-renewal"],
            "sequence": 1,
            "rule_id": regen_rule,
        },
    )
    identities.append(
        identity(
            counter_rule,
            "rule.goal07.abundant-ebon-deer-complete.maple-retaliation",
            "Rule",
            "Maple Retaliation",
            "缃叶反击",
            "Damage-targeted immediate Leaf Hinging counter rule.",
        )
    )
    add(
        "RuleDefinition",
        {
            "id": counter_rule,
            "domain": "Battle",
            "source_definition_identity_id": effects["maple-counter"],
            "source_class": "Effect",
            "source_digest_sha256": (
                "a43312b59cc4ea5ef3c70e0c65594b5cbd80beb15c78d39cc73a33b66568fe70"
            ),
        },
    )
    add(
        "EventFilter",
        {
            "id": counter_filter,
            "stable_key": "goal07.enemy.s01.filter.maple-retaliation",
            "target_selector_id": selectors["owner"],
            "damage_class": "Ordinary",
            "cause_ancestry": "Any",
        },
    )
    add(
        "RuleTrigger",
        {
            "id": counter_trigger,
            "stable_key": "goal07.enemy.s01.trigger.maple-retaliation",
            "rule_id": counter_rule,
            "sequence": 1,
            "event": json_cell("Damage", point="Applied"),
            "phase": "AfterEvent",
            "filter_id": counter_filter,
            "condition_id": BASE + 551,
            "once_scope": "Event",
            "priority": 0,
            "program_id": maple_counter_program,
        },
    )
    add(
        "EffectRuleBinding",
        {
            "effect_id": effects["maple-counter"],
            "sequence": 1,
            "rule_id": counter_rule,
        },
    )

    add(
        "ConditionExpression",
        {
            "id": BASE + 551,
            "stable_key": "goal07.enemy.s01.condition.always",
            "node": json_cell("Constant", value=True),
        },
    )
    add(
        "ConditionExpression",
        {
            "id": BASE + 552,
            "stable_key": "goal07.enemy.s01.condition.no-branches",
            "node": json_cell(
                "SelectorCardinality",
                selector_id=selectors["branches"],
                minimum_count=0,
                maximum_count=0,
            ),
        },
    )
    phase_sequences = [
        [10, 11, 5, 5, 6],
        [12, 9, 5, 5, 6],
        [13, 7, 5, 5, 6],
    ]
    next_state = BASE + 701
    next_candidate = BASE + 801
    next_transition = BASE + 901
    phase_initial_states: list[int] = []
    for phase_index, skills in enumerate(phase_sequences):
        state_ids = list(range(next_state, next_state + len(skills)))
        next_state += len(skills)
        phase_initial_states.append(state_ids[0])
        add(
            "AiGraph",
            {
                "id": graphs[phase_index],
                "initial_state_id": state_ids[0],
                "automatic_transition_budget": 8,
            },
        )
        for offset, (state_id, skill) in enumerate(zip(state_ids, skills)):
            main_ability = boss_abilities[skill]
            add(
                "AiState",
                {
                    "id": state_id,
                    "stable_key": (
                        f"goal07.enemy.s01.ai.phase-{phase_index + 1}.state-{offset + 1}"
                    ),
                    "graph_id": graphs[phase_index],
                    "mandatory_fallback_ability_id": boss_abilities[5],
                    "turn_counter_reset": offset == 0,
                },
            )
            sequence = 1
            condition = (
                BASE + 552 if skill in {10, 11, 12, 13} else BASE + 551
            )
            target = (
                selectors["opposing-single"] if skill == 5 else
                selectors["opposing-all"] if skill in {6, 8} else
                selectors["actor"]
            )
            add(
                "AiCandidate",
                {
                    "id": next_candidate,
                    "stable_key": (
                        f"goal07.enemy.s01.ai.phase-{phase_index + 1}."
                        f"state-{offset + 1}.main"
                    ),
                    "state_id": state_id,
                    "sequence": sequence,
                    "ability_id": main_ability,
                    "condition_id": condition,
                    "target_selector_id": target,
                    "priority": 10,
                    "selection": "FirstLegal",
                    "no_target_fallback": "UseFallbackAbility",
                    "fallback_ability_id": boss_abilities[5],
                },
            )
            next_candidate += 1
            add(
                "AiTransition",
                {
                    "id": next_transition,
                    "stable_key": (
                        f"goal07.enemy.s01.ai.phase-{phase_index + 1}."
                        f"transition-{offset + 1}"
                    ),
                    "state_id": state_id,
                    "sequence": 1,
                    "target_state_id": state_ids[(offset + 1) % len(state_ids)],
                    "condition_id": BASE + 551,
                    "priority": 0,
                    "timing": "AfterAction",
                },
            )
            next_transition += 1

    add(
        "EnemyTemplate",
        {
            "id": template,
            "rank": "Boss",
            "base_aggro_decimal": "100",
            "default_ai_graph_id": graphs[0],
        },
    )
    add(
        "EnemyVariant",
        {
            "id": variant,
            "template_id": template,
            "ai_graph_id": graphs[0],
            "mechanically_distinct_key": VARIANT_KEY,
        },
    )
    for level in anchor["levels"]:
        add(
            "EnemyStat",
            {
                "variant_id": variant,
                "level": level["authored_level"],
                "difficulty_key": "standard-universe-v1",
                "hp_decimal": level["base_hp"],
                "atk_decimal": level["base_atk"],
                "def_decimal": level["base_def"],
                "spd_decimal": level["base_spd"],
                "effect_hit_rate_decimal": level["effect_hit_rate"],
                "effect_resistance_decimal": level["effect_resistance"],
                "crit_damage_decimal": "0.2",
            },
        )
    for sequence, weakness in enumerate(["Fire", "Ice", "Quantum"], start=1):
        add(
            "EnemyWeakness",
            {"variant_id": variant, "sequence": sequence, "element": weakness},
        )
    for element in ["Physical", "Lightning", "Wind", "Imaginary"]:
        add(
            "EnemyResistance",
            {"variant_id": variant, "element": element, "value_decimal": "0.2"},
        )
    for category, value in [("Frozen", "0.5"), ("Imprisonment", "0.5")]:
        add(
            "EnemyDebuffResistance",
            {
                "variant_id": variant,
                "category_key": category,
                "value_decimal": value,
            },
        )
    add(
        "EnemyToughnessLayer",
        {
            "variant_id": variant,
            "sequence": 1,
            "layer_key": "ordinary",
            "kind": "Ordinary",
            "maximum_decimal": "420",
            "recovery_ratio_decimal": "1",
            "active_at_start": True,
        },
    )
    for sequence, skill in enumerate(range(1, 15), start=1):
        add(
            "EnemyVariantAbility",
            {
                "variant_id": variant,
                "sequence": sequence,
                "ability_id": boss_abilities[skill],
            },
        )
    for sequence, graph in enumerate(graphs, start=1):
        add(
            "EnemyPhase",
            {
                "id": BASE + 600 + sequence,
                "stable_key": f"goal07.enemy.s01.phase-{sequence}",
                "variant_id": variant,
                "sequence": sequence,
                "entry_condition_id": BASE + 551,
                "exit_condition_id": BASE + 551,
                "replacement_priority": sequence,
                "ai_graph_id": graph,
                "targetable": True,
                "transition_model": "TransformSameUnit",
                "entry_program_id": (
                    phase_three_entry_program if sequence == 3 else None
                ),
                "hp_carry": "Reset",
                "action_gauge_carry": "Reset",
                "effect_carry": "Clear",
                "toughness_carry": "Reset",
                "summon_carry": "Clear",
            },
        )

    branch_type = {
        "wintry": (branch_abilities["wintry"], "0.173333", "120"),
        "maple": (branch_abilities["maple"], "0.266667", "144"),
        "glorious": (branch_abilities["glorious"], "0.266667", "120"),
        "lavish": (branch_abilities["lavish"], "0.4", "83"),
    }
    for formation, (name, unit) in enumerate(linked.items(), start=8):
        kind = name.split("-")[0]
        action, hp_ratio, speed = branch_type[kind]
        add(
            "LinkedUnitDefinition",
            {
                "id": unit,
                "source_definition_identity_id": unit,
                "kind": "Summon",
                "presence": "Linked",
                "ability_ids": str(action),
                "action_ability_id": None if kind == "lavish" else action,
                "formation_index": formation,
                "initial_gauge_decimal": "10000",
                "hp_owner_ratio_decimal": hp_ratio,
                "hp_flat_decimal": "0",
                "atk_owner_ratio_decimal": "1",
                "atk_flat_decimal": "0",
                "def_owner_ratio_decimal": "1",
                "def_flat_decimal": "0",
                "spd_owner_ratio_decimal": "0",
                "spd_flat_decimal": speed,
                "owner_defeat_policy": "Depart",
                "owner_departure_policy": "Depart",
                "wave_policy": "Depart",
                "combatant_digest_sha256": sha256_text(
                    f"goal07-s01-linked-{name}-v1"
                ),
            },
        )

    anchor_digest = sha256_bytes(ANCHOR.read_bytes())
    add(
        "SourceRecord",
        {
            "id": 3,
            "stable_key": "source.hsr-wiki.abundant-ebon-deer-complete.2026-07-28",
            "category": "CommunityMaintained",
            "publisher": "Honkai: Star Rail Wiki",
            "url": anchor["source"]["url"],
            "accessed_on": anchor["source"]["accessed_on"],
            "applicable_game_version": anchor["source"]["game_version"],
            "confidence": "SecondaryVersionSensitiveCrossCheck",
            "evidence_sha256": anchor_digest,
            "usage_note": (
                "Exact public per-level HP, ATK, DEF, SPD, EHR and Effect RES "
                "transcribed into committed Goal 07 evidence."
            ),
        },
    )
    add(
        "EvidenceRecord",
        {
            "id": 4,
            "stable_key": "evidence.goal07.enemy.s01.numeric-anchors",
            "kind": "SourcePayload",
            "source_record_id": 3,
            "sha256": anchor_digest,
            "note": "Committed exact public per-level numeric anchors for Goal 07 S01.",
        },
    )
    for item in identities:
        add("ContentIdentity", item)
        add(
            "ContentEvidenceBinding",
            {
                "content_id": item["id"],
                "sequence": 1,
                "fact_key": f"goal07.s01.executable:{item['stable_key']}",
                "source_record_id": 1,
                "evidence_record_id": 3,
                "quality": "ExactStructured",
                "mechanism_quality": "ExactStructured",
            },
        )
    add(
        "ContentEvidenceBinding",
        {
            "content_id": variant,
            "sequence": 2,
            "fact_key": "goal07.s01.public-level-stats",
            "source_record_id": 3,
            "evidence_record_id": 4,
            "quality": "ExactStructured",
            "mechanism_quality": "ExactStructured",
        },
    )

    for table_rows in rows.values():
        table_rows.sort(
            key=lambda row: json.dumps(
                row, ensure_ascii=False, sort_keys=True, default=str
            )
        )
    return rows


OWNERSHIP: dict[str, Callable[[dict[str, Any]], bool]] = {
    "Ability": lambda row: int(row["id"]) >= BASE,
    "AbilityPhase": lambda row: int(row["ability_id"]) >= BASE,
    "AiCandidate": lambda row: int(row["id"]) >= BASE,
    "AiGraph": lambda row: int(row["id"]) >= BASE,
    "AiState": lambda row: int(row["id"]) >= BASE,
    "AiTransition": lambda row: int(row["id"]) >= BASE,
    "ConditionExpression": lambda row: int(row["id"]) >= BASE,
    "ContentEvidenceBinding": lambda row: int(row["content_id"]) >= BASE,
    "ContentIdentity": lambda row: int(row["id"]) >= BASE,
    "Effect": lambda row: int(row["id"]) >= BASE,
    "EffectModifierBinding": lambda row: int(row["effect_id"]) >= BASE,
    "EffectRuleBinding": lambda row: int(row["effect_id"]) >= BASE,
    "EffectTag": lambda row: int(row["effect_id"]) >= BASE,
    "EnemyAbility": lambda row: int(row["id"]) >= BASE,
    "EnemyDebuffResistance": lambda row: int(row["variant_id"]) >= BASE,
    "EnemyPhase": lambda row: int(row["id"]) >= BASE,
    "EnemyResistance": lambda row: int(row["variant_id"]) >= BASE,
    "EnemyStat": lambda row: int(row["variant_id"]) >= BASE,
    "EnemyTemplate": lambda row: int(row["id"]) >= BASE,
    "EnemyToughnessLayer": lambda row: int(row["variant_id"]) >= BASE,
    "EnemyVariant": lambda row: int(row["id"]) >= BASE,
    "EnemyVariantAbility": lambda row: int(row["variant_id"]) >= BASE,
    "EnemyWeakness": lambda row: int(row["variant_id"]) >= BASE,
    "EventFilter": lambda row: int(row["id"]) >= BASE,
    "LinkedUnitDefinition": lambda row: int(row["id"]) >= BASE,
    "ModifierDefinition": lambda row: int(row["id"]) >= BASE,
    "ModifierStackingGroup": lambda row: int(row["id"]) >= BASE,
    "Operation": lambda row: int(row["id"]) >= BASE,
    "Program": lambda row: int(row["id"]) >= BASE,
    "ProgramStep": lambda row: int(row["program_id"]) >= BASE,
    "RuleDefinition": lambda row: int(row["id"]) >= BASE,
    "RuleTrigger": lambda row: int(row["id"]) >= BASE,
    "Selector": lambda row: int(row["id"]) >= BASE,
    "SelectorPredicate": lambda row: int(row["selector_id"]) >= BASE,
    "ValueExpression": lambda row: int(row["id"]) >= BASE,
    "SourceRecord": lambda row: int(row["id"]) == 3,
    "EvidenceRecord": lambda row: int(row["id"]) == 4,
}


def selected(table: str, rows: list[dict[str, Any]]) -> list[dict[str, Any]]:
    owns = OWNERSHIP[table]
    return [row for row in rows if owns(row)]


def merged(table: str, expected: list[dict[str, Any]]) -> list[dict[str, Any]]:
    owns = OWNERSHIP[table]
    current = [row for row in workbook_rows(table) if not owns(row)]
    current.extend(expected)
    return current


def build_golden(expected: dict[str, list[dict[str, Any]]]) -> dict[str, Any]:
    tables: dict[str, Any] = {}
    for table, rows in sorted(expected.items()):
        excel = selected(table, workbook_rows(table))
        if semantic_digest(excel) != semantic_digest(rows):
            raise ValueError(f"{table}: S01 Excel rows differ")
        exported = selected(table, sora_rows(table))
        if semantic_digest(exported) != semantic_digest(rows):
            raise ValueError(f"{table}: S01 Sora rows differ")
        tables[table] = {
            "rows": len(rows),
            "semantic_sha256": semantic_digest(rows),
        }
    return {
        "schema_revision": "starclock.goal07-enemy-partition-golden.v1",
        "partition_id": PARTITION,
        "enemy_variant_ids": [VARIANT_KEY],
        "tables": tables,
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--partition", required=True)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--write", action="store_true")
    mode.add_argument("--write-golden", action="store_true")
    mode.add_argument("--check", action="store_true")
    args = parser.parse_args()
    if args.partition != PARTITION:
        raise ValueError(f"{args.partition}: enemy partition authoring is not implemented")
    expected = owned_rows()
    golden_path = (
        ROOT
        / "evidence"
        / "standard-universe-mechanics-complete-v1"
        / "goldens"
        / f"{PARTITION}.json"
    )
    if args.write:
        for table, rows in expected.items():
            write_rows(table, merged(table, rows))
        print(f"Authored Goal 07 enemy partition {PARTITION}.")
        return
    golden = build_golden(expected)
    encoded = json.dumps(golden, ensure_ascii=False, indent=2) + "\n"
    if args.write_golden:
        golden_path.parent.mkdir(parents=True, exist_ok=True)
        golden_path.write_text(encoded, encoding="utf-8")
        print(f"Wrote {golden_path.relative_to(ROOT)}.")
        return
    if not golden_path.is_file() or golden_path.read_text(encoding="utf-8") != encoded:
        raise ValueError(f"{PARTITION}: enemy partition golden drifted")
    print(f"Goal 07 enemy partition {PARTITION} matches Excel and Sora.")


if __name__ == "__main__":
    main()
