//! Canonical identity for encounter, pool and mechanic-rule definitions.

use crate::definition::LocalizedText;
use crate::digest::{Encoder, UniverseEncounterDefinitionsDigest};
use crate::encounter::{
    ContentPoolDefinition, DifficultyEnemyBinding, EncounterGroupDefinition,
    EncounterPoolDefinition, RoomContentBinding,
};
use crate::path::ExactParameter;
use crate::rule::MechanicRuleDefinition;

pub(crate) fn digest(
    groups: &[EncounterGroupDefinition],
    difficulty: &[DifficultyEnemyBinding],
    pools: &[EncounterPoolDefinition],
    room_content: &[RoomContentBinding],
    content_pools: &[ContentPoolDefinition],
    rules: &[MechanicRuleDefinition],
) -> UniverseEncounterDefinitionsDigest {
    let mut encoder = Encoder::new(b"starclock-standard-universe-encounter-definitions-v1");
    encoder.u32(groups.len() as u32);
    for group in groups {
        encoder.u32(group.id().get());
        encoder.text(group.stable_key());
        encoder.text(group.source_group_id());
        encoder.u8(group.wave_policy() as u8);
        encoder.u8(group.boss_phase_policy() as u8);
        localized(&mut encoder, group.text());
        encoder.u32(group.members().len() as u32);
        for member in group.members() {
            encoder.u32(member.id().get());
            encoder.text(member.source_rogue_monster_id());
            encoder.text(member.source_primary_monster_id());
            encoder.text(member.source_stage_id());
            parameter(&mut encoder, member.weight());
            encoder.u32(member.stage_level());
            encoder.u32(member.hard_level_group());
            texts(&mut encoder, member.stage_ability_ids());
            encoder.optional_text(member.drop_type());
            encoder.u32(member.waves().len() as u32);
            for wave in member.waves() {
                encoder.u32(wave.id().get());
                encoder.u32(wave.enemies().len() as u32);
                for enemy in wave.enemies() {
                    encoder.text(enemy.slot_key());
                    encoder.text(enemy.source_monster_id());
                    encoder.text(enemy.enemy_variant_key());
                }
            }
        }
    }
    encoder.u32(difficulty.len() as u32);
    for value in difficulty {
        encoder.u32(value.difficulty().get());
        encoder.u8(value.role() as u8);
        encoder.text(value.source_monster_id());
        encoder.text(value.enemy_variant_key());
        encoder.u32(value.level());
    }
    encoder.u32(pools.len() as u32);
    for pool in pools {
        encoder.u32(pool.id().get());
        encoder.text(pool.stable_key());
        encoder.u32(pool.room().get());
        encoder.u8(pool.domain_kind() as u8);
        encoder.text(pool.map_entrance());
        encoder.u8(pool.selection_policy() as u8);
        encoder.text(pool.source_primary_condition_key());
        localized(&mut encoder, pool.text());
        encoder.u32(pool.fixed().len() as u32);
        for fixed in pool.fixed() {
            encoder.text(fixed.condition_key());
            encoder.text(fixed.source_content_id());
        }
        encoder.u32(pool.weighted().len() as u32);
        for weighted in pool.weighted() {
            encoder.text(weighted.condition_key());
            encoder.u32(weighted.group().get());
            parameter(&mut encoder, weighted.weight());
        }
    }
    encoder.u32(room_content.len() as u32);
    for value in room_content {
        encoder.u32(value.room().get());
        encoder.text(value.condition_key());
        encoder.text(value.source_content_id());
        encoder.u8(value.kind() as u8);
        optional_u32(&mut encoder, value.encounter_group().map(|id| id.get()));
    }
    encoder.u32(content_pools.len() as u32);
    for pool in content_pools {
        encoder.u32(pool.id().get());
        encoder.text(pool.stable_key());
        encoder.u8(pool.kind() as u8);
        encoder.text(pool.ordering_key());
        encoder.bool(pool.replacement());
        encoder.u32(pool.entries().len() as u32);
        for entry in pool.entries() {
            encoder.text(entry.content_key());
            parameter(&mut encoder, entry.weight());
            encoder.optional_text(entry.condition());
        }
    }
    encoder.u32(rules.len() as u32);
    for rule in rules {
        encoder.u32(rule.id().get());
        encoder.text(rule.stable_key());
        encoder.text(rule.source_record_key());
        encoder.text(rule.source_file());
        encoder.u8(rule.kind() as u8);
        encoder.optional_text(rule.native_handler_key());
        encoder.optional_text(rule.source_binding_key());
        encoder.u32(rule.parameters().len() as u32);
        for parameter in rule.parameters() {
            optional_u32(&mut encoder, parameter.index());
            encoder.optional_text(parameter.key());
            encoder.text(parameter.value());
        }
        texts(&mut encoder, rule.mechanic_tags());
        encoder.optional_text(rule.approximation_replacement_condition());
        localized(&mut encoder, rule.text());
    }
    UniverseEncounterDefinitionsDigest::new(encoder.finish())
}

fn localized(encoder: &mut Encoder, value: &LocalizedText) {
    encoder.text(value.name_en());
    encoder.text(value.name_zh_cn());
    encoder.text(value.summary_en());
    encoder.text(value.summary_zh_cn());
}
fn texts(encoder: &mut Encoder, values: &[Box<str>]) {
    encoder.u32(values.len() as u32);
    for value in values {
        encoder.text(value);
    }
}
fn parameter(encoder: &mut Encoder, value: ExactParameter) {
    encoder.i64(value.coefficient());
    encoder.u8(value.scale());
}
fn optional_u32(encoder: &mut Encoder, value: Option<u32>) {
    encoder.bool(value.is_some());
    if let Some(value) = value {
        encoder.u32(value);
    }
}
