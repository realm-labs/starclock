use crate::swarm_disaster_generated::{
    SoraConfig, swarm_disaster_boss_pool::SwarmDisasterBossPool,
    swarm_disaster_encounter_group::SwarmDisasterEncounterGroup,
    swarm_disaster_encounter_wave::SwarmDisasterEncounterWave,
    swarm_disaster_enemy_slot::SwarmDisasterEnemySlot,
};

use super::{
    json, metadata, nonempty, optional_text_list, positive, positive_u8, positive_u16, stable,
    text_list,
};
use crate::swarm_disaster_content::{SwarmDisasterContentError, types::*};

pub(super) type EncounterTables = (
    Box<[EncounterGroupDefinition]>,
    Box<[EncounterWaveDefinition]>,
    Box<[EnemySlotDefinition]>,
    Box<[BossPoolDefinition]>,
);

pub(super) fn lower(source: &SoraConfig) -> Result<EncounterTables, SwarmDisasterContentError> {
    Ok((
        source
            .swarm_disaster_encounter_group()
            .ordered_rows()
            .map(encounter_group)
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
        source
            .swarm_disaster_encounter_wave()
            .ordered_rows()
            .map(encounter_wave)
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
        source
            .swarm_disaster_enemy_slot()
            .ordered_rows()
            .map(enemy_slot)
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
        source
            .swarm_disaster_boss_pool()
            .ordered_rows()
            .map(boss_pool)
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
    ))
}

fn encounter_group(
    row: &SwarmDisasterEncounterGroup,
) -> Result<EncounterGroupDefinition, SwarmDisasterContentError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    json(&row.room_scope_json, &row.stable_key)?;
    Ok(EncounterGroupDefinition {
        id: EncounterGroupId(positive(row.id, &row.stable_key)?),
        key: stable(&row.stable_key, &row.stable_key)?,
        room_key: row
            .room_id
            .as_deref()
            .map(|value| stable(value, &row.stable_key))
            .transpose()?,
        area_keys: optional_text_list(row.eligible_area_ids.as_deref(), &row.stable_key)?,
        boss_choice_keys: optional_text_list(
            row.displayed_boss_choice_ids.as_deref(),
            &row.stable_key,
        )?,
        role: nonempty(&row.encounter_role, &row.stable_key)?,
        wave_keys: text_list(&row.wave_ids, &row.stable_key)?,
        difficulty_binding: json(&row.difficulty_binding_json, &row.stable_key)?,
        members: json(&row.weighted_members_json, &row.stable_key)?,
        weight_policy: json(&row.weight_policy_json, &row.stable_key)?,
    })
}

fn encounter_wave(
    row: &SwarmDisasterEncounterWave,
) -> Result<EncounterWaveDefinition, SwarmDisasterContentError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    Ok(EncounterWaveDefinition {
        id: EncounterWaveId(positive(row.id, &row.stable_key)?),
        key: stable(&row.stable_key, &row.stable_key)?,
        group: EncounterGroupId(positive(row.encounter_group_id, &row.stable_key)?),
        ordinal: positive_u16(row.ordinal, &row.stable_key)?,
        slot_keys: text_list(&row.enemy_slot_ids, &row.stable_key)?,
        stage_type: nonempty(&row.stage_type, &row.stable_key)?,
        authored_level: positive_u16(row.authored_stage_level, &row.stable_key)?,
        hard_level_group: positive_u16(row.hard_level_group, &row.stable_key)?,
        level_binding: json(&row.level_binding_json, &row.stable_key)?,
    })
}

fn enemy_slot(
    row: &SwarmDisasterEnemySlot,
) -> Result<EnemySlotDefinition, SwarmDisasterContentError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    nonempty(&row.source_slot, &row.stable_key)?;
    nonempty(&row.source_monster_id, &row.stable_key)?;
    Ok(EnemySlotDefinition {
        id: EnemySlotId(positive(row.id, &row.stable_key)?),
        key: stable(&row.stable_key, &row.stable_key)?,
        wave: EncounterWaveId(positive(row.wave_id, &row.stable_key)?),
        wave_key: stable(&row.encounter_wave_id, &row.stable_key)?,
        formation_index: positive_u8(row.formation_index, &row.stable_key)?,
        enemy_variant_key: stable(&row.enemy_variant_id, &row.stable_key)?,
        boss_choice_keys: optional_text_list(row.boss_choice_ids.as_deref(), &row.stable_key)?,
    })
}

fn boss_pool(row: &SwarmDisasterBossPool) -> Result<BossPoolDefinition, SwarmDisasterContentError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    Ok(BossPoolDefinition {
        id: BossPoolId(positive(row.id, &row.stable_key)?),
        key: stable(&row.stable_key, &row.stable_key)?,
        difficulty_key: stable(&row.difficulty_id, &row.stable_key)?,
        area_id: positive(row.area_id, &row.stable_key)?,
        tier: nonempty(&row.pool_tier, &row.stable_key)?,
        candidate_keys: text_list(&row.candidate_ids, &row.stable_key)?,
        candidate_order: nonempty(&row.candidate_order, &row.stable_key)?,
        consequences: json(&row.choice_consequences_json, &row.stable_key)?,
        selection_policy: json(&row.selection_policy_json, &row.stable_key)?,
    })
}
