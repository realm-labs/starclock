"""World, topology, encounter and Activity seam workbook rows."""

from __future__ import annotations

from pathlib import Path

from workbook.data import canonical_json, joined, load_json, sha256_file, stable_ids


DOMAIN_KINDS = {
    "combat-primary": "CombatPrimary",
    "combat-secondary": "CombatSecondary",
    "occurrence": "Occurrence",
    "encounter": "Encounter",
    "respite": "Respite",
    "elite": "Elite",
    "boss": "Boss",
    "transaction": "Transaction",
    "adventure": "Adventure",
}


def _base_tables(root: Path) -> tuple[dict[str, list[dict]], dict[str, dict[str, int]], dict[str, list[dict]]]:
    names = ("worlds", "world-difficulties", "domains", "maps", "rooms", "encounter-groups", "encounter-pools")
    records = {name: load_json(root, f"{name}.json") for name in names}
    ids = {name: stable_ids(records[name]) for name in names}
    return records, ids, {}


def _identity_rows(root: Path, records: dict[str, list[dict]], ids: dict[str, dict[str, int]], rows: dict[str, list[dict]]) -> None:
    manifest = load_json(root, "manifest.json")
    pack_index = load_json(root, "pack-index.json")
    content_manifest = root / manifest["content_manifest"]
    rows["UniverseProfile"] = [{
        "id": 1,
        "stable_key": "universe.profile.standard-main-world.v4.4",
        "game_version": manifest["snapshot"]["game_version"],
        "snapshot_date": manifest["snapshot"]["access_date"],
        "content_manifest_sha256": sha256_file(content_manifest),
        "pack_sha256": pack_index["pack_sha256"],
        "world_count": len(records["worlds"]),
        "path_count": 9,
        "runtime_loading": manifest["runtime_loading"],
    }]
    rows["UniverseWorld"] = [{
        "id": ids["worlds"][record["id"]],
        "profile_id": 1,
        "stable_key": record["id"],
        "world_number": record["world_id"],
        "name_en": record["name_en"],
        "name_zh_cn": record["name_zh_cn"],
        "summary_en": record["summary_en"],
        "summary_zh_cn": record["summary_zh_cn"],
        "entry_rule_stable_key": record["entry_rule_id"],
        "terminal_rule_stable_key": record["terminal_rule_id"],
    } for record in records["worlds"]]


def _difficulty_rows(root: Path, records: dict[str, list[dict]], ids: dict[str, dict[str, int]], rows: dict[str, list[dict]]) -> None:
    enemy_keys = {record["id"] for record in load_json(root, "../v4.4/enemy-variants.json")}
    rows["UniverseDifficulty"] = []
    rows["UniverseDifficultyEnemy"] = []
    for record in records["world-difficulties"]:
        difficulty_id = ids["world-difficulties"][record["id"]]
        rows["UniverseDifficulty"].append({
            "id": difficulty_id,
            "stable_key": record["id"],
            "world_id": ids["worlds"][record["world_id"]],
            "source_area_id": record["source_ids"][0],
            "difficulty": record["difficulty"],
            "kind": record["profile_kind"],
            "recommended_level": record["recommended_level"],
            "recommended_elements": joined(record["recommended_elements"]),
            "score_curve_json": canonical_json(record["score_curve"]),
            "unlock_source_id": record["unlock_source_id"] or None,
        })
        sequence = 1
        for role, field in (("Boss", "boss_variant_ids"), ("Elite", "elite_variant_ids")):
            for enemy in record[field]:
                stable_key = enemy["enemy_variant_id"]
                if stable_key not in enemy_keys:
                    raise ValueError(f"missing Goal 01 EnemyVariant {stable_key}")
                rows["UniverseDifficultyEnemy"].append({
                    "difficulty_id": difficulty_id,
                    "sequence": sequence,
                    "role": role,
                    "source_monster_id": enemy["source_monster_id"],
                    "enemy_variant_stable_key": stable_key,
                    "level": enemy["level"],
                })
                sequence += 1


def _topology_rows(records: dict[str, list[dict]], ids: dict[str, dict[str, int]], rows: dict[str, list[dict]]) -> None:
    rows["UniverseDomain"] = [{
        "id": ids["domains"][record["id"]],
        "stable_key": record["id"],
        "source_type": int(record["source_ids"][0]),
        "kind": DOMAIN_KINDS[record["kind"]],
        "decision_policy": record["decision_policy"],
        "terminal": record["terminal"],
        "name_en": record["name_en"],
        "name_zh_cn": record["name_zh_cn"],
        "summary_en": record["summary_en"],
        "summary_zh_cn": record["summary_zh_cn"],
    } for record in records["domains"]]
    rows["UniverseMapNode"] = []
    rows["UniverseMapEdge"] = []
    for record in records["maps"]:
        node_id = ids["maps"][record["id"]]
        rows["UniverseMapNode"].append({
            "id": node_id,
            "stable_key": record["id"],
            "source_map_id": int(record["map_id"].rsplit(".", 1)[1]),
            "source_node_id": record["node_id"],
            "is_start": record["start"],
            "position_x": record["position_hint"]["x"],
            "position_y": record["position_hint"]["y"],
        })
        for sequence, target in enumerate(record["next_node_ids"], start=1):
            rows["UniverseMapEdge"].append({"source_node_id": node_id, "sequence": sequence, "target_node_id": ids["maps"][target]})


def _room_rows(records: dict[str, list[dict]], ids: dict[str, dict[str, int]], rows: dict[str, list[dict]]) -> None:
    group_by_source = {record["source_ids"][0]: record["id"] for record in records["encounter-groups"]}
    domains = {record["id"]: record for record in records["domains"]}
    rows["UniverseRoom"] = []
    rows["UniverseRoomContent"] = []
    for record in records["rooms"]:
        room_id = ids["rooms"][record["id"]]
        rows["UniverseRoom"].append({
            "id": room_id,
            "stable_key": record["id"],
            "domain_id": ids["domains"][record["domain_id"]],
            "source_room_id": record["source_ids"][0],
            "map_entrance": record["map_entrance"],
            "source_group_id": record["source_group_id"],
            "section_ids": joined(record["section_ids"]),
        })
        for sequence, content in enumerate(record["content_map"], start=1):
            group_key = group_by_source.get(content["content_source_id"])
            if group_key:
                kind = "EncounterGroup"
            elif domains[record["domain_id"]]["decision_policy"] == "ExternalCommand":
                kind = "ExternalDecision"
            else:
                kind = "FixedContent"
            rows["UniverseRoomContent"].append({
                "room_id": room_id,
                "sequence": sequence,
                "condition_key": content["group_id"],
                "source_content_id": content["content_source_id"],
                "kind": kind,
                "encounter_group_id": ids["encounter-groups"][group_key] if group_key else None,
            })


def _encounter_rows(root: Path, records: dict[str, list[dict]], ids: dict[str, dict[str, int]], rows: dict[str, list[dict]]) -> None:
    enemy_keys = {record["id"] for record in load_json(root, "../v4.4/enemy-variants.json")}
    rows["UniverseEncounterGroup"] = []
    rows["UniverseEncounterMember"] = []
    rows["UniverseEncounterWave"] = []
    rows["UniverseEncounterWaveEnemy"] = []
    member_id = 1
    wave_id = 1
    for group in records["encounter-groups"]:
        group_id = ids["encounter-groups"][group["id"]]
        rows["UniverseEncounterGroup"].append({
            "id": group_id,
            "stable_key": group["id"],
            "source_group_id": group["source_ids"][0],
            "wave_policy": group["wave_policy"],
            "boss_phase_policy": group["boss_phase_policy"],
            "name_en": group["name_en"],
            "name_zh_cn": group["name_zh_cn"],
            "summary_en": group["summary_en"],
            "summary_zh_cn": group["summary_zh_cn"],
        })
        for sequence, member in enumerate(group["weighted_member_ids"], start=1):
            rows["UniverseEncounterMember"].append({
                "id": member_id,
                "group_id": group_id,
                "sequence": sequence,
                "source_rogue_monster_id": member["source_rogue_monster_id"],
                "source_primary_monster_id": member["source_primary_monster_id"],
                "source_stage_id": member["source_stage_id"],
                "weight_decimal": member["weight"],
                "stage_level": member["stage_level"],
                "hard_level_group": member["hard_level_group"],
                "stage_ability_ids": joined(member["stage_ability_ids"]) or None,
                "drop_type": member["drop_type"] or None,
            })
            for wave_sequence, wave in enumerate(member["waves"], start=1):
                rows["UniverseEncounterWave"].append({"id": wave_id, "member_id": member_id, "sequence": wave_sequence})
                for enemy_sequence, enemy in enumerate(wave["enemy_variant_ids"], start=1):
                    stable_key = enemy["enemy_variant_id"]
                    if stable_key not in enemy_keys:
                        raise ValueError(f"missing Goal 01 EnemyVariant {stable_key}")
                    rows["UniverseEncounterWaveEnemy"].append({
                        "wave_id": wave_id,
                        "sequence": enemy_sequence,
                        "slot": enemy["slot"],
                        "source_monster_id": enemy["source_monster_id"],
                        "enemy_variant_stable_key": stable_key,
                    })
                wave_id += 1
            member_id += 1


def _encounter_pool_rows(records: dict[str, list[dict]], ids: dict[str, dict[str, int]], rows: dict[str, list[dict]]) -> None:
    rows["UniverseEncounterPool"] = []
    rows["UniverseEncounterPoolGroup"] = []
    rows["UniverseEncounterPoolFixed"] = []
    for record in records["encounter-pools"]:
        pool_id = ids["encounter-pools"][record["id"]]
        rows["UniverseEncounterPool"].append({
            "id": pool_id,
            "stable_key": record["id"],
            "room_id": ids["rooms"][record["room_id"]],
            "domain_kind": DOMAIN_KINDS[record["domain_kind"]],
            "map_entrance": record["map_entrance"],
            "selection_policy": record["selection_policy"],
            "source_primary_condition_key": record["source_primary_condition_key"],
            "name_en": record["name_en"],
            "name_zh_cn": record["name_zh_cn"],
            "summary_en": record["summary_en"],
            "summary_zh_cn": record["summary_zh_cn"],
        })
        for sequence, item in enumerate(record["weighted_group_ids"], start=1):
            rows["UniverseEncounterPoolGroup"].append({
                "pool_id": pool_id,
                "sequence": sequence,
                "condition_key": item["condition_key"],
                "group_id": ids["encounter-groups"][item["group_id"]],
                "weight_decimal": item["weight"],
            })
        for sequence, item in enumerate(record["fixed_content_entries"], start=1):
            rows["UniverseEncounterPoolFixed"].append({
                "pool_id": pool_id,
                "sequence": sequence,
                "condition_key": item["condition_key"],
                "source_content_id": item["source_content_id"],
            })


def _activity_rows(records: dict[str, list[dict]], ids: dict[str, dict[str, int]], rows: dict[str, list[dict]]) -> None:
    rows["UniverseActivityBinding"] = [{
        "id": 1,
        "stable_key": "universe.activity-binding.standard-main-world.v1",
        "profile_id": 1,
        "activity_stable_key": "activity.standard-simulated-universe.v1",
        "participant_digest_locked": True,
        "scoped_slots_supported": True,
        "fork_join_reserved": True,
        "battle_handoff_contract": "activity.battle-handoff.rule-bundle.v1",
        "external_outcome_contract": "activity.external-outcome.command.v1",
    }]
    rows["UniverseActivityDomainBinding"] = []
    for sequence, record in enumerate(sorted(records["domains"], key=lambda item: int(item["source_ids"][0])), start=1):
        if record["decision_policy"] == "BattleHandoff":
            decision = "BattleCommand"
        elif record["kind"] == "adventure":
            decision = "ExternalOutcome"
        else:
            decision = "RunCommand"
        rows["UniverseActivityDomainBinding"].append({
            "activity_binding_id": 1,
            "sequence": sequence,
            "domain_id": ids["domains"][record["id"]],
            "decision_kind": decision,
        })


def build_rows(root: Path) -> dict[str, list[dict]]:
    records, ids, rows = _base_tables(root)
    _identity_rows(root, records, ids, rows)
    _difficulty_rows(root, records, ids, rows)
    _topology_rows(records, ids, rows)
    _room_rows(records, ids, rows)
    _encounter_rows(root, records, ids, rows)
    _encounter_pool_rows(records, ids, rows)
    _activity_rows(records, ids, rows)
    return rows
