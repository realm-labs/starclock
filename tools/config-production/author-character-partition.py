"""Author one frozen Goal 01 character partition into production Excel.

The prepared Version 4.4 reference pack is the only content input. Each
partition owns a disjoint 10,000-ID block so completed imports compose without
rewriting prior character rows.

Run through the approved repository adapter:
  uv run --with openpyxl python tools/config-production/author-character-partition.py C01 --write
  uv run --with openpyxl python tools/config-production/author-character-partition.py C01 --check
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


def load_v1b() -> Any:
    path = Path(__file__).with_name("author-character-v1b.py")
    spec = importlib.util.spec_from_file_location("starclock_character_v1b", path)
    if spec is None or spec.loader is None:
        raise RuntimeError("unable to load V1B authoring helpers")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


V1B = load_v1b()

OWNED_TABLES = (
    "Ability",
    "AbilityHitPlanBinding",
    "AbilityLevelParameter",
    "AbilityPhase",
    "AbilityResourceDelta",
    "Character",
    "CharacterAbilityBinding",
    "CharacterResource",
    "CharacterStat",
    "Eidolon",
    "EidolonPatch",
    "HitPlan",
    "HitPlanHit",
    "ModifierDefinition",
    "ModifierFilter",
    "ModifierStackingGroup",
    "Selector",
    "TraceNode",
    "TracePatch",
    "ValueExpression",
)

# (target group, level-parameter position, repeated hits). The prepared source
# supplies the exact coefficients; this map supplies their semantic placement.
C01_DAMAGE = {
    "character.acheron.ability.octobolt-flash.bpskill": [("Primary", 1, 1), ("Adjacent", 2, 1)],
    "character.acheron.ability.quadrivalent-ascendance.maze": [("All", 1, 1)],
    "character.acheron.ability.rainblade.ultra": [("Primary", 1, 1), ("All", 2, 1)],
    "character.acheron.ability.slashed-dream-cries-in-red.ultra": [("Primary", 6, 1), ("All", 7, 1)],
    "character.acheron.ability.stygian-resurge.ultra": [("All", 1, 1)],
    "character.acheron.ability.trilateral-wiltcross.normal": [("Primary", 1, 1)],
    "character.anaxa.ability.fractal-exiles-fallacy.bpskill": [("Primary", 1, 1), ("BounceDraw", 3, 4)],
    "character.anaxa.ability.pain-brews-truth.normal": [("Primary", 1, 1)],
    "character.anaxa.ability.sprouting-life-sculpts-earth.ultra": [("All", 1, 1)],
    "character.anaxa.ability.tetrad-wisdom-reigns-thrice.skillp01": [("BounceDraw", 1, 5)],
    "character.archer.ability.caladbolg-ii-fake-spiral-sword.bpskill": [("Primary", 1, 1), ("Adjacent", 2, 1)],
    "character.archer.ability.clairvoyance.maze": [("All", 1, 1)],
    "character.archer.ability.kanshou-and-bakuya.normal": [("Primary", 1, 1)],
    "character.archer.ability.minds-eye-true.talent": [("BounceDraw", 1, 1)],
    "character.archer.ability.unlimited-blade-works.ultra": [("Primary", 1, 1)],
    "character.argenti.ability.fleeting-fragrance.normal": [("Primary", 1, 1)],
    "character.argenti.ability.for-in-this-garden-supreme-beauty-bestows.ultra": [("All", 1, 1)],
    "character.argenti.ability.justice-hereby-blooms.bpskill": [("All", 1, 1)],
    "character.argenti.ability.manifesto-of-purest-virtue.maze": [("All", 2, 1)],
    "character.argenti.ability.merit-bestowed-in-my-garden.ultra": [("All", 1, 1), ("BounceDraw", 3, 6)],
    "character.arlan.ability.frenzied-punishment.ultra": [("Primary", 1, 1), ("Adjacent", 2, 1)],
    "character.arlan.ability.lightning-rush.normal": [("Primary", 1, 1)],
    "character.arlan.ability.shackle-breaker.bpskill": [("Primary", 2, 1)],
    "character.arlan.ability.swift-harvest.maze": [("All", 1, 1)],
    "character.ashveil.ability.banquet-insatiable-appetite.ultra": [("All", 1, 1)],
    "character.ashveil.ability.devour-o-loathsome-hand.maze": [("All", 2, 1)],
    "character.ashveil.ability.flog-smite-evil.bpskill": [("All", 1, 1)],
    "character.ashveil.ability.rancor-enmity-reprisal.skillp01": [("Primary", 3, 1)],
    "character.ashveil.ability.talons-inculcate-decorum.normal": [("Primary", 1, 1)],
    "character.aventurine.ability.roulette-shark.ultra": [("Primary", 2, 1)],
    "character.aventurine.ability.shot-loaded-right.skillp01": [("Primary", 3, 1), ("BounceDraw", 3, 6)],
    "character.aventurine.ability.straight-bet.normal": [("Primary", 1, 1)],
    "character.bailu.ability.diagnostic-kick.normal": [("Primary", 1, 1)],
}

C02_DAMAGE = {
    "character.black-swan.ability.bliss-of-otherworlds-embrace.ultra": [("All", 1, 1)],
    "character.black-swan.ability.decadence-false-twilight.bpskill": [("Primary", 1, 1), ("Adjacent", 1, 1)],
    "character.black-swan.ability.percipience-silent-dawn.normal": [("Primary", 1, 1)],
    "character.blade.ability.death-sentence.ultra": [("Primary", 2, 1), ("Adjacent", 4, 1)],
    "character.blade.ability.forest-of-swords.normal": [("Primary", 4, 1), ("Adjacent", 5, 1)],
    "character.blade.ability.karma-wind.maze": [("All", 1, 1)],
    "character.blade.ability.shard-sword.normal": [("Primary", 1, 1)],
    "character.blade.ability.shuhus-gift.skillp01": [("All", 4, 1)],
    "character.boothill.ability.dust-devils-sunset-rodeo.ultra": [("Primary", 1, 1)],
    "character.boothill.ability.fanning-the-hammer.normal": [("Primary", 1, 1)],
    "character.boothill.ability.skullcrush-spurs.normal": [("Primary", 1, 1)],
    "character.bronya.ability.windrider-bullet.normal": [("Primary", 1, 1)],
    "character.castorice.ability.boneclaw-doomdrakes-embrace.bpskill": [("All", 1, 1)],
    "character.castorice.ability.lament-netherseas-ripple.normal": [("Primary", 1, 1)],
    "character.castorice.ability.silence-wraithflys-caress.bpskill": [("Primary", 1, 1), ("Adjacent", 3, 1)],
    "character.cerydra.ability.kings-castling.normal": [("Primary", 1, 1)],
    "character.cerydra.ability.scholars-mate.ultra": [("All", 1, 1)],
    "character.cipher.ability.hey-jackpot-for-the-taking.bpskill": [("Primary", 1, 1), ("Adjacent", 2, 1)],
    "character.cipher.ability.oops-a-missed-catch.normal": [("Primary", 1, 1)],
    "character.cipher.ability.the-hospitable-dolosian.skillp01": [("Primary", 1, 1)],
    "character.cipher.ability.yours-truly-kitty-phantom-thief.ultra": [("Primary", 1, 1), ("All", 2, 1)],
    "character.cyrene.ability.lo-hope-takes-flight.normal": [("Primary", 1, 1)],
    "character.cyrene.ability.to-love-and-tomorrow.normal": [("Primary", 1, 1), ("All", 3, 1)],
}

C03_DAMAGE = {
    "character.dan-heng.ability.cloudlancer-art-north-wind.normal": [("Primary", 1, 1)],
    "character.dan-heng.ability.cloudlancer-art-torrent.bpskill": [("Primary", 1, 1)],
    "character.dan-heng.ability.ethereal-dream.ultra": [("Primary", 1, 1)],
    "character.dan-heng-imbibitor-lunae.ability.azures-aqua-ablutes-all.ultra": [("Primary", 1, 1), ("Adjacent", 2, 1)],
    "character.dan-heng-imbibitor-lunae.ability.beneficent-lotus.normal": [("Primary", 1, 1)],
    "character.dan-heng-imbibitor-lunae.ability.divine-spear.normal": [("Primary", 1, 1), ("Adjacent", 2, 1)],
    "character.dan-heng-imbibitor-lunae.ability.fulgurant-leap.normal": [("Primary", 1, 1), ("Adjacent", 2, 1)],
    "character.dan-heng-imbibitor-lunae.ability.heaven-quelling-prismadrakon.maze": [("All", 3, 1)],
    "character.dan-heng-imbibitor-lunae.ability.transcendence.normal": [("Primary", 1, 1)],
    "character.dan-heng-permansor-terrae.ability.a-dragons-zenith-knows-no-rue.ultra": [("All", 1, 1)],
    "character.dan-heng-permansor-terrae.ability.aegis-vitae.normal": [("Primary", 1, 1)],
    "character.dr-ratio.ability.cogito-ergo-sum.skillp01": [("Primary", 1, 1)],
    "character.dr-ratio.ability.intellectual-midwifery.bpskill": [("Primary", 1, 1)],
    "character.dr-ratio.ability.mind-is-might.normal": [("Primary", 1, 1)],
    "character.dr-ratio.ability.syllogistic-paradox.ultra": [("Primary", 1, 1)],
    "character.evanescia.ability.discipline-final-verdict.bpskill": [("Primary", 2, 1), ("Adjacent", 3, 1)],
    "character.evanescia.ability.scarlet-elation-or-execution.elationdamage": [("All", 2, 1)],
    "character.evanescia.ability.swordsong-absolution-denied.ultra": [("All", 1, 1)],
    "character.evanescia.ability.syllabus-pop-quiz.normal": [("Primary", 1, 1)],
    "character.evanescia.ability.youth-halcyon-evermore.skillp01": [("All", 1, 1)],
    "character.evernight.ability.o-wakeful-world-goodnight.ultra": [("All", 1, 1)],
    "character.evernight.ability.time-thence-blurs.normal": [("Primary", 1, 1)],
    "character.feixiao.ability.boltsunder-blitz.ultra": [("Primary", 1, 1)],
    "character.feixiao.ability.boltsunder.normal": [("Primary", 1, 1)],
    "character.feixiao.ability.stormborn.maze": [("All", 2, 1)],
    "character.feixiao.ability.terrasplit.ultra": [("Primary", 4, 1)],
    "character.feixiao.ability.thunderhunt.skillp01": [("Primary", 1, 1)],
    "character.feixiao.ability.waraxe-skyward.ultra": [("Primary", 1, 1)],
    "character.feixiao.ability.waraxe.bpskill": [("Primary", 1, 1)],
    "character.fu-xuan.ability.novaburst.normal": [("Primary", 1, 1)],
    "character.fu-xuan.ability.woes-of-many-morphed-to-one.ultra": [("All", 1, 1)],
}

DAMAGE_BY_PARTITION = {"C01": C01_DAMAGE, "C02": C02_DAMAGE, "C03": C03_DAMAGE}

C01_TARGET_OVERRIDES = {
    "character.acheron.ability.rainblade.ultra": "Aoe",
    "character.acheron.ability.slashed-dream-cries-in-red.ultra": "Aoe",
    "character.acheron.ability.stygian-resurge.ultra": "Aoe",
    "character.anaxa.ability.tetrad-wisdom-reigns-thrice.skillp01": "Bounce",
    "character.archer.ability.caladbolg-ii-fake-spiral-sword.bpskill": "Blast",
    "character.archer.ability.minds-eye-true.talent": "Bounce",
    "character.argenti.ability.merit-bestowed-in-my-garden.ultra": "Bounce",
    "character.aventurine.ability.shot-loaded-right.skillp01": "Bounce",
}

C02_TARGET_OVERRIDES = {
    "character.blade.ability.forest-of-swords.normal": "Blast",
    "character.blade.ability.shuhus-gift.skillp01": "Aoe",
    "character.cipher.ability.yours-truly-kitty-phantom-thief.ultra": "Blast",
    "character.cyrene.ability.to-love-and-tomorrow.normal": "Blast",
}

C03_TARGET_OVERRIDES = {
    "character.feixiao.ability.boltsunder-blitz.ultra": "SingleTarget",
    "character.feixiao.ability.waraxe-skyward.ultra": "SingleTarget",
}

TARGET_OVERRIDES = {**C01_TARGET_OVERRIDES, **C02_TARGET_OVERRIDES, **C03_TARGET_OVERRIDES}

C01_SCALING_STATS = {
    "character.aventurine.ability.roulette-shark.ultra": "Def",
    "character.aventurine.ability.shot-loaded-right.skillp01": "Def",
}

C02_SCALING_STATS = {
    "character.blade.ability.death-sentence.ultra": "Hp",
    "character.blade.ability.forest-of-swords.normal": "Hp",
    "character.blade.ability.shuhus-gift.skillp01": "Hp",
    "character.castorice.ability.boneclaw-doomdrakes-embrace.bpskill": "Hp",
    "character.castorice.ability.silence-wraithflys-caress.bpskill": "Hp",
}

C03_SCALING_STATS = {
    "character.evernight.ability.o-wakeful-world-goodnight.ultra": "Hp",
    "character.evernight.ability.time-thence-blurs.normal": "Hp",
    "character.fu-xuan.ability.novaburst.normal": "Hp",
    "character.fu-xuan.ability.woes-of-many-morphed-to-one.ultra": "Hp",
}

SCALING_STATS = {**C01_SCALING_STATS, **C02_SCALING_STATS, **C03_SCALING_STATS}

ABILITY_KIND_OVERRIDES = {
    "character.blade.ability.forest-of-swords.normal": "EnhancedBasic",
    "character.boothill.ability.fanning-the-hammer.normal": "EnhancedBasic",
    "character.cyrene.ability.to-love-and-tomorrow.normal": "EnhancedBasic",
    "character.dan-heng-imbibitor-lunae.ability.divine-spear.normal": "EnhancedBasic",
    "character.dan-heng-imbibitor-lunae.ability.fulgurant-leap.normal": "EnhancedBasic",
    "character.dan-heng-imbibitor-lunae.ability.transcendence.normal": "EnhancedBasic",
    "character.feixiao.ability.boltsunder-blitz.ultra": "Passive",
    "character.feixiao.ability.waraxe-skyward.ultra": "Passive",
}

SKILL_POINT_COST_OVERRIDES = {
    "character.dan-heng-imbibitor-lunae.ability.transcendence.normal": "1",
    "character.dan-heng-imbibitor-lunae.ability.divine-spear.normal": "2",
    "character.dan-heng-imbibitor-lunae.ability.fulgurant-leap.normal": "3",
}

CHARACTER_RESOURCES = {
    "character.acheron": [("slashed-dream", "9", "0")],
    "character.archer": [("charge", "4", "0")],
    "character.argenti": [("apotheosis", "10", "0")],
    "character.ashveil": [("rancor", "12", "0")],
    "character.aventurine": [("blind-bet", "10", "0")],
    "character.bailu": [("revive", "1", "1")],
    "character.blade": [("charge", "5", "0")],
    "character.boothill": [("pocket-trickshot", "3", "0")],
    "character.castorice": [("newbud", "100", "0")],
    "character.cerydra": [("charge", "6", "0")],
    "character.cyrene": [("recollection", "24", "0")],
    "character.dan-heng-imbibitor-lunae": [("squama-sacrosancta", "3", "0")],
    "character.dan-heng-permansor-terrae": [("empowered-souldragon-actions", "2", "0")],
    "character.evanescia": [("certified-banger", "480", "0")],
    "character.evernight": [("memoria", "16", "0")],
    "character.feixiao": [("flying-aureus", "12", "0")],
    "character.fu-xuan": [("self-heal-charge", "2", "1")],
}

CHARACTER_RESOURCE_COSTS = {
    "character.castorice.ability.doomshriek-dawns-chime.ultra": [("newbud", "100")],
    "character.feixiao.ability.terrasplit.ultra": [("flying-aureus", "6")],
}

BASE_ENERGY_OVERRIDES = {
    "character.feixiao": "0",
}


def partition(code: str) -> tuple[int, list[str]]:
    if not code.startswith("C") or not code[1:].isdigit():
        raise ValueError("partition must be C01 through C11")
    index = int(code[1:])
    records = V1B.read_json(PARTITIONS)["character_partitions"]
    if not 1 <= index <= len(records):
        raise ValueError("partition must be C01 through C11")
    row = records[index - 1]
    if row["batch_id"] != f"G01-P7-{code}":
        raise ValueError(f"frozen {code} partition changed")
    return 20_000 + index * 10_000, row["ids"]


def sources(selected_ids: list[str]) -> tuple[list[dict[str, Any]], ...]:
    selected = set(selected_ids)
    names = ("characters", "character-abilities", "character-traces", "character-eidolons")
    result = []
    for name in names:
        rows = V1B.read_json(REFERENCE / f"{name}.json")
        result.append([
            row for row in rows
            if row.get("id") in selected or row.get("character_id") in selected
        ])
    if len(result[0]) != len(selected):
        raise ValueError("prepared character partition cardinality changed")
    return tuple(result)


def internal_maps(base: int, abilities: list[dict[str, Any]], traces: list[dict[str, Any]], eidolons: list[dict[str, Any]], damage: dict[str, list[tuple[str, int, int]]]) -> dict[str, dict[str, int]]:
    return {
        "ability": {row["id"]: base + 1 + index for index, row in enumerate(sorted(abilities, key=lambda row: row["id"]))},
        "hit_plan": {row["id"]: base + 1_001 + index for index, row in enumerate(sorted((row for row in abilities if row["id"] in damage), key=lambda row: row["id"]))},
        "trace": {row["id"]: base + 2_001 + index for index, row in enumerate(sorted(traces, key=lambda row: row["id"]))},
        "eidolon": {row["id"]: base + 3_001 + index for index, row in enumerate(sorted(eidolons, key=lambda row: row["id"]))},
    }


def target_pattern(row: dict[str, Any]) -> str:
    if row["id"] in TARGET_OVERRIDES:
        return TARGET_OVERRIDES[row["id"]]
    return {
        "SingleEnemy": "SingleTarget", "Blast": "Blast", "AllEnemies": "Aoe",
        "RandomEnemy": "Bounce", "AllAllies": "Support", "SingleAlly": "Support",
        "Self": "Enhance", "Battlefield": "ContentDefined", "": "None",
    }.get(row["mechanic_hints"]["target_hint"], "ContentDefined")


def trace_status(row: dict[str, Any]) -> list[tuple[str, Any]]:
    result = []
    for status in row["status_additions"]:
        property_type = status.get("PropertyType", status.get("type"))
        value = status.get("Value", status.get("value"))
        if property_type is None or value is None:
            raise ValueError(f"invalid prepared Trace status addition in {row['id']}")
        result.append((property_type, value))
    return result


def trace_kind(row: dict[str, Any]) -> str:
    if trace_status(row):
        return "MinorStat"
    if row["point_type"] == 2:
        return "BasicLevel" if row["max_level"] <= 6 else "AbilityLevel"
    if row["point_type"] == 3 or row["mechanic_hints"]["operation_tags"]:
        return "MajorPassive"
    return "AbilityUnlock"


def modifier_spec(property_type: str) -> tuple[str, str, str, str | None]:
    direct = {
        "AttackAddedRatio": ("Atk", "PercentOfBase", "Stat", None),
        "HPAddedRatio": ("Hp", "PercentOfBase", "Stat", None),
        "DefenceAddedRatio": ("Def", "PercentOfBase", "Stat", None),
        "SpeedDelta": ("Spd", "Flat", "Stat", None),
        "CriticalChanceBase": ("CritRate", "BaseAdd", "Stat", None),
        "CriticalDamageBase": ("CritDamage", "BaseAdd", "Stat", None),
        "StatusResistanceBase": ("EffectResistance", "BaseAdd", "Stat", None),
        "StatusProbabilityBase": ("EffectHitRate", "BaseAdd", "Stat", None),
        "BreakDamageAddedRatioBase": ("BreakEffect", "BaseAdd", "Stat", None),
        "ElationDamageAddedRatioBase": ("Atk", "DamageBoost", "ElationDamage", None),
        "FireAddedRatio": ("Atk", "DamageBoost", "OrdinaryDamage", "Fire"),
        "IceAddedRatio": ("Atk", "DamageBoost", "OrdinaryDamage", "Ice"),
        "ImaginaryAddedRatio": ("Atk", "DamageBoost", "OrdinaryDamage", "Imaginary"),
        "PhysicalAddedRatio": ("Atk", "DamageBoost", "OrdinaryDamage", "Physical"),
        "QuantumAddedRatio": ("Atk", "DamageBoost", "OrdinaryDamage", "Quantum"),
        "ThunderAddedRatio": ("Atk", "DamageBoost", "OrdinaryDamage", "Lightning"),
        "WindAddedRatio": ("Atk", "DamageBoost", "OrdinaryDamage", "Wind"),
    }
    try:
        return direct[property_type]
    except KeyError as error:
        raise ValueError(f"unsupported prepared minor-Trace property {property_type}") from error


def toughness_for(row: dict[str, Any], group: str) -> str | None:
    values = [Decimal(str(value)) for value in row.get("display_toughness", [])]
    if not values:
        return None
    index = {"Primary": 0, "BounceDraw": 0, "All": 1, "Adjacent": 2}[group]
    value = values[index] if index < len(values) else Decimal(0)
    if value == 0 and group == "All":
        value = next((candidate for candidate in values if candidate > 0), Decimal(0))
    return V1B.canonical_decimal(value) if value > 0 else None


def generated_rows(code: str) -> tuple[dict[str, list[dict[str, Any]]], list[dict[str, Any]], list[dict[str, Any]], list[str], int]:
    base, selected = partition(code)
    characters, abilities, traces, eidolons = sources(selected)
    frozen = V1B.identity_map()
    ids = internal_maps(base, abilities, traces, eidolons, DAMAGE_BY_PARTITION.get(code, {}))
    rows = {name: [] for name in OWNED_TABLES}
    internals = []
    selector_owner, selector_subject, stacking_group = base + 6_001, base + 6_002, base + 6_003
    rows["ModifierStackingGroup"].append({"id": stacking_group, "stable_key": f"{code.lower()}.trace-minor.additive", "aggregation": "Sum"})
    for selector_id, origin in ((selector_owner, "Owner"), (selector_subject, "CurrentSubject")):
        internals.append(V1B.identity(selector_id, f"selector.{code.lower()}.trace-minor.{origin.lower()}", "Selector", f"{code} Trace {origin} Selector", f"{code}行迹选择器", "Generic single-subject selector for exact minor-Trace modifiers."))
        rows["Selector"].append({
            "id": selector_id, "domain": "Battle", "origin": origin,
            "side_relationship": "SameSide", "life": "Alive", "presence": "Present",
            "reference_point": "CurrentState", "ordering": "StableId", "choice": "First",
            "minimum_count": 1, "maximum_count": 1, "allow_repeated_targets": False,
            "empty_pool_policy": "Fault",
        })

    for ability in sorted(abilities, key=lambda row: row["id"]):
        ability_id = ids["ability"][ability["id"]]
        kind = ABILITY_KIND_OVERRIDES.get(ability["id"], V1B.ability_kind(ability))
        pattern = target_pattern(ability)
        damage_shape = DAMAGE_BY_PARTITION.get(code, {}).get(ability["id"], [])
        internals.append(V1B.identity(ability_id, ability["id"], "Ability", ability["name_en"], ability["name_zh_cn"], "Complete prepared ability metadata, exact level parameters, resources and ordered hit structure."))
        rows["Ability"].append({
            "id": ability_id, "kind": kind, "target_pattern": pattern,
            "retarget_policy": "RecomputeEachHit" if pattern == "Bounce" else "CancelRemaining",
            "level_cap": max(1, int(ability["max_level"] or 1)),
            "cooldown_actions": max(0, int(ability.get("cooldown") or 0)),
            "semantic_tags_mask": V1B.semantic_mask(kind, ability),
        })
        rows["CharacterAbilityBinding"].append({
            "character_id": frozen[ability["character_id"]], "sequence": 0,
            "slot": V1B.ability_slot(kind), "ability_id": ability_id,
            "invested_level_cap": V1B.invested_level_cap(ability["kind"], max(1, int(ability["max_level"] or 1))),
        })
        for level in ability["levels"]:
            for parameter_index, value in enumerate(level["parameters"], start=1):
                rows["AbilityLevelParameter"].append({
                    "ability_id": ability_id, "effective_level": level["level"],
                    "parameter_key": f"parameter.{parameter_index:02d}",
                    "value_decimal": V1B.canonical_decimal(value),
                })
        delta_sequence = 1
        skill_points = SKILL_POINT_COST_OVERRIDES.get(ability["id"], ability.get("skill_point_cost"))
        is_basic = kind == "Basic"
        archer_skill = ability["id"] == "character.archer.ability.caladbolg-ii-fake-spiral-sword.bpskill"
        if is_basic or archer_skill or (skill_points is not None and Decimal(str(skill_points)) > 0):
            spend = archer_skill or (skill_points is not None and Decimal(str(skill_points)) > 0)
            amount = Decimal(str(skill_points)) if spend and skill_points is not None else Decimal(1)
            rows["AbilityResourceDelta"].append({
                "ability_id": ability_id, "sequence": delta_sequence,
                "resource_kind": "SkillPoints", "delta_kind": "Spend" if spend else "Gain",
                "timing": "ActionStarted" if spend else "AbilityResolved",
                "amount_decimal": V1B.canonical_decimal(abs(amount)),
            })
            delta_sequence += 1
        for resource_key, amount in CHARACTER_RESOURCE_COSTS.get(ability["id"], []):
            rows["AbilityResourceDelta"].append({
                "ability_id": ability_id, "sequence": delta_sequence,
                "resource_kind": "CharacterResource", "character_resource_key": resource_key,
                "delta_kind": "Spend", "timing": "ActionStarted",
                "amount_decimal": V1B.canonical_decimal(amount),
            })
            delta_sequence += 1
        energy = ability.get("energy_gain")
        if energy not in (None, "0", 0):
            rows["AbilityResourceDelta"].append({
                "ability_id": ability_id, "sequence": delta_sequence,
                "resource_kind": "Energy", "delta_kind": "Gain", "timing": "AbilityResolved",
                "amount_decimal": V1B.canonical_decimal(energy),
            })
        rows["AbilityPhase"].append({"ability_id": ability_id, "sequence": 1, "kind": "Hits" if damage_shape else "Resolved"})
        if damage_shape:
            plan_id = ids["hit_plan"][ability["id"]]
            internals.append(V1B.identity(plan_id, f"program.hit-plan.{ability['id']}", "Program", f"{ability['name_en']} Hit Plan", f"{ability['name_zh_cn']}命中计划", "Exact ordered damage and Toughness structure from the prepared ability record."))
            expanded = [(group, parameter) for group, parameter, count in damage_shape for _ in range(count)]
            shares = V1B.split_shares([
                (group, Decimal(1) / Decimal(len(expanded)))
                for group, _parameter in expanded
            ])
            rows["HitPlan"].append({"id": plan_id, "target_pattern": pattern, "retarget_policy": "RecomputeEachHit" if pattern == "Bounce" else "CancelRemaining", "declared_hit_count": len(expanded)})
            for sequence, ((group, parameter), (_share_group, share)) in enumerate(zip(expanded, shares), start=1):
                hit = {
                    "hit_plan_id": plan_id, "sequence": sequence, "target_group": group,
                    "damage_ratio_decimal": share,
                    "toughness_ratio_decimal": share,
                    "crit_policy": "PerTarget", "damage_parameter_key_override": f"parameter.{parameter:02d}",
                    "damage_operation_ratio_decimal": "1",
                }
                toughness = toughness_for(ability, group)
                if toughness is not None:
                    hit["toughness_amount_decimal"] = toughness
                rows["HitPlanHit"].append(hit)
            rows["AbilityHitPlanBinding"].append({
                "ability_id": ability_id, "phase_sequence": 1, "hit_plan_id": plan_id,
                "damage_parameter_key": f"parameter.{expanded[0][1]:02d}",
                "damage_scaling_stat": SCALING_STATS.get(ability["id"], "Atk"),
                "damage_class": "Elation" if ability["kind"] == "ElationDamage" else "Ordinary",
                "element": next(row["element"] for row in characters if row["id"] == ability["character_id"]),
            })

    by_character: dict[int, list[dict[str, Any]]] = {}
    for binding in rows["CharacterAbilityBinding"]:
        by_character.setdefault(int(binding["character_id"]), []).append(binding)
    for bindings in by_character.values():
        bindings.sort(key=lambda row: int(row["ability_id"]))
        for sequence, binding in enumerate(bindings, start=1):
            binding["sequence"] = sequence

    for character in sorted(characters, key=lambda row: row["id"]):
        character_id = frozen[character["id"]]
        path = {"The Hunt": "Hunt", "Warrior": "Destruction"}.get(character["path"], character["path"])
        rows["Character"].append({"id": character_id, "rarity": character["rarity"], "path": path, "element": character["element"], "base_energy_decimal": BASE_ENERGY_OVERRIDES.get(character["id"], character["max_energy"] or "0"), "base_aggro_decimal": character["promotions"][0]["aggro"]})
        for promotion, stat in enumerate(character["promotions"]):
            first_level = 1 if promotion == 0 else promotion * 10 + 10
            # ExactPreviousRelease rows carry an explicit promotion ordinal;
            # their level ceiling follows the same frozen 20/30/.../80 ladder.
            max_level = int(stat.get("max_level", (promotion + 2) * 10))
            for level in range(first_level, max_level + 1):
                offset = Decimal(level - 1)
                rows["CharacterStat"].append({
                    "character_id": character_id, "level": level, "promotion": promotion,
                    "hp_decimal": V1B.canonical_decimal(Decimal(stat["hp_base"]) + Decimal(stat["hp_per_level"]) * offset),
                    "atk_decimal": V1B.canonical_decimal(Decimal(stat["atk_base"]) + Decimal(stat["atk_per_level"]) * offset),
                    "def_decimal": V1B.canonical_decimal(Decimal(stat["def_base"]) + Decimal(stat["def_per_level"]) * offset),
                    "spd_decimal": V1B.canonical_decimal(stat["spd"]),
                })
        for sequence, (key, maximum, initial) in enumerate(CHARACTER_RESOURCES.get(character["id"], []), start=1):
            rows["CharacterResource"].append({"character_id": character_id, "sequence": sequence, "stable_key": key, "maximum_decimal": maximum, "initial_decimal": initial})

    point_to_trace = {source_id: row["id"] for row in traces for source_id in row["source_point_ids"]}
    minor_index = 0
    for trace in sorted(traces, key=lambda row: row["id"]):
        trace_id = ids["trace"][trace["id"]]
        internals.append(V1B.identity(trace_id, trace["id"], "Trace", trace["name_en"], trace["name_zh_cn"], "Complete battle-relevant Trace identity, graph edge and prepared mechanic payload."))
        prerequisites = sorted({ids["trace"][point_to_trace[source]] for source in trace["prerequisites"] if source in point_to_trace})
        rows["TraceNode"].append({"id": trace_id, "character_id": frozen[trace["character_id"]], "kind": trace_kind(trace), "promotion_requirement": 0, "prerequisite_trace_ids": "|".join(str(value) for value in prerequisites) or None})
        patch_sequence = 1
        for property_type, value in sorted(set(trace_status(trace))):
            minor_index += 1
            modifier_id, expression_id = base + 4_001 + minor_index, base + 5_001 + minor_index
            stat, stage, purpose, element = modifier_spec(property_type)
            internals.append(V1B.identity(modifier_id, f"modifier.{trace['id']}.{property_type}", "Modifier", f"{trace['name_en']} {property_type}", f"{trace['name_zh_cn']} {property_type}", "Exact prepared minor-Trace stat addition compiled as a persistent modifier."))
            value_kind = "Scalar" if property_type == "SpeedDelta" else "Ratio"
            rows["ValueExpression"].append({"id": expression_id, "stable_key": f"{code.lower()}.trace-minor.value.{minor_index:03d}", "result_kind": value_kind, "node": json.dumps({"type": f"{value_kind}Literal", "value_decimal": V1B.canonical_decimal(value)}, separators=(",", ":"))})
            rows["ModifierDefinition"].append({"id": modifier_id, "owner_selector_id": selector_owner, "subject_selector_id": selector_subject, "stat": stat, "formula_stage": stage, "formula_purpose": purpose, "value_expression_id": expression_id, "value_domain": value_kind, "stacking_group_id": stacking_group, "priority": 0, "cap_formula_stage": stage, "snapshot_policy": "Dynamic", "duration_scope": "Battle"})
            if element is not None:
                rows["ModifierFilter"].append({"modifier_id": modifier_id, "sequence": 1, "filter": json.dumps({"type": "Element", "element": element}, separators=(",", ":"))})
            rows["TracePatch"].append({"trace_id": trace_id, "sequence": patch_sequence, "patch": json.dumps({"type": "AddModifier", "modifier_identity_id": modifier_id}, separators=(",", ":"))})
            patch_sequence += 1
        for source_skill_id in trace["level_up_skill_source_ids"]:
            matching = next((row for row in abilities if source_skill_id in row["source_skill_ids"]), None)
            if (
                matching is not None
                and matching["kind"] not in ("Maze", "MazeNormal")
                and not trace["default_unlocked"]
            ):
                rows["TracePatch"].append({"trace_id": trace_id, "sequence": patch_sequence, "patch": json.dumps({"type": "AdjustAbilityLevel", "ability_id": ids["ability"][matching["id"]], "bonus": 1, "cap_delta": 1}, separators=(",", ":"))})
                patch_sequence += 1

    ability_by_source = {source_id: row for row in abilities for source_id in row["source_skill_ids"]}
    for eidolon in sorted(eidolons, key=lambda row: row["id"]):
        eidolon_id = ids["eidolon"][eidolon["id"]]
        internals.append(V1B.identity(eidolon_id, eidolon["id"], "Eidolon", eidolon["name_en"], eidolon["name_zh_cn"], "Complete E1-E6 identity and exact prepared ability-level patches."))
        rows["Eidolon"].append({"id": eidolon_id, "character_id": frozen[eidolon["character_id"]], "rank": eidolon["rank"]})
        additions_by_ability: dict[str, int] = {}
        for addition in eidolon["skill_level_additions"]:
            ability = ability_by_source.get(addition["source_skill_id"])
            if ability is None:
                continue
            additions_by_ability[ability["id"]] = max(
                additions_by_ability.get(ability["id"], 0),
                int(addition["levels"]),
            )
        for sequence, (ability_key, levels) in enumerate(sorted(additions_by_ability.items()), start=1):
            rows["EidolonPatch"].append({"eidolon_id": eidolon_id, "sequence": sequence, "patch": json.dumps({"type": "AdjustAbilityLevel", "ability_id": ids["ability"][ability_key], "bonus": levels, "cap_delta": levels}, separators=(",", ":"))})

    for table_rows in rows.values():
        table_rows.sort(key=lambda row: tuple(str(value) for value in row.values()))
    return rows, internals, abilities + traces + eidolons, selected, base


def owned_predicate(name: str, base: int, selected: list[str]) -> Callable[[dict[str, Any]], bool]:
    field = {
        "Ability": "id", "AbilityHitPlanBinding": "ability_id", "AbilityLevelParameter": "ability_id",
        "AbilityPhase": "ability_id", "AbilityResourceDelta": "ability_id", "Character": "id",
        "CharacterAbilityBinding": "ability_id", "CharacterResource": "character_id", "CharacterStat": "character_id",
        "Eidolon": "id", "EidolonPatch": "eidolon_id", "HitPlan": "id", "HitPlanHit": "hit_plan_id",
        "ModifierDefinition": "id", "ModifierFilter": "modifier_id", "ModifierStackingGroup": "id",
        "Selector": "id", "TraceNode": "id", "TracePatch": "trace_id", "ValueExpression": "id",
    }[name]
    frozen_ids = {V1B.identity_map()[key] for key in selected}
    if name in ("Character", "CharacterResource", "CharacterStat"):
        return lambda row: int(row[field]) in frozen_ids
    return lambda row: base <= int(row[field]) <= base + 9_999


def merged_table(name: str, authored: list[dict[str, Any]], base: int, selected: list[str]) -> list[dict[str, Any]]:
    _, existing = V1B.workbook_rows(name)
    owns = owned_predicate(name, base, selected)
    owned_positions = [index for index, row in enumerate(existing) if owns(row)]
    insertion = min(owned_positions) if owned_positions else len(existing)
    retained = [dict(row) for row in existing if not owns(row)]
    retained[insertion:insertion] = authored
    return retained


def update_metadata(code: str, internals: list[dict[str, Any]], source_rows: list[dict[str, Any]], selected: list[str], base: int) -> tuple[list[dict[str, Any]], list[dict[str, Any]]]:
    _, identities = V1B.workbook_rows("ContentIdentity")
    selected_set = set(selected)
    retained = [dict(row) for row in identities if not (base <= int(row["id"]) <= base + 9_999)]
    for row in retained:
        if str(row["stable_key"]) in selected_set:
            row["enabled"] = True
            row["coverage_state"] = "GoldenVerified"
            row["summary_en"] = str(row["summary_en"]).replace(" Catalog identity only; executable rows remain pending.", " Complete production statistics, abilities, Traces and E1-E6 rows are present.")
            row["summary_zh_cn"] = str(row["summary_zh_cn"]).replace(" 当前仅为目录身份；可执行数据尚待转录。", " 已具备完整生产属性、技能、行迹与星魂数据。")
    retained.extend(internals)
    retained.sort(key=lambda row: int(row["id"]))
    source_by_id = {row["id"]: row for row in source_rows}
    _, bindings = V1B.workbook_rows("ContentEvidenceBinding")
    selected_ids = {V1B.identity_map()[key] for key in selected}
    kept = [dict(row) for row in bindings if not (base <= int(row["content_id"]) <= base + 9_999) and not (int(row["content_id"]) in selected_ids and int(row["sequence"]) >= 2)]
    for record in internals:
        source = source_by_id.get(record["stable_key"])
        quality = source.get("quality", "ExactStructured") if source else "ExactStructured"
        mechanism = source.get("mechanism_quality", quality) if source else quality
        if mechanism == "ExactPreviousRelease":
            mechanism = "ExactPreviousReleaseText"
        kept.append({"content_id": record["id"], "sequence": 1, "fact_key": f"{code.lower()}.prepared:{record['stable_key']}", "source_record_id": 1, "evidence_record_id": 3, "quality": quality, "mechanism_quality": mechanism})
    for stable_key in sorted(selected):
        kept.append({"content_id": V1B.identity_map()[stable_key], "sequence": 2, "fact_key": f"{code.lower()}.executable:{stable_key}", "source_record_id": 1, "evidence_record_id": 3, "quality": "ExactStructured", "mechanism_quality": "ExactStructured"})
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
    rows, internals, source_rows, selected, base = generated_rows(code)
    expected = {name: merged_table(name, rows[name], base, selected) for name in OWNED_TABLES}
    identities, evidence = update_metadata(code, internals, source_rows, selected, base)
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
        print(f"Authored frozen {code} character partition into production workbooks.")
    else:
        for name in OWNED_TABLES:
            V1B.check_exact(name, expected[name])
        V1B.check_exact("ContentIdentity", identities)
        V1B.check_exact("ContentEvidenceBinding", evidence)
        V1B.check_exact("ConfigManifest", manifest_rows)
        print(f"Frozen {code} character workbooks match deterministic authoring output.")


if __name__ == "__main__":
    main()
