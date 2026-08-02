//! Generated-type-free encounter inputs decoded from validated Sora rows.

use serde::Deserialize;

use crate::error::{UniverseCatalogLoadError, UniverseCatalogLoadErrorKind};

use super::SwarmDisasterContentCatalog;

#[derive(Clone, Debug)]
pub(crate) struct EncounterRuntimeInput {
    pub(crate) groups: Box<[EncounterGroupInput]>,
    pub(crate) waves: Box<[EncounterWaveInput]>,
    pub(crate) slots: Box<[EnemySlotInput]>,
    pub(crate) boss_pools: Box<[BossPoolInput]>,
}

#[derive(Clone, Debug)]
pub(crate) struct EncounterGroupInput {
    pub(crate) id: u32,
    pub(crate) key: Box<str>,
    pub(crate) room_key: Option<Box<str>>,
    pub(crate) area_keys: Box<[Box<str>]>,
    pub(crate) boss_choice_keys: Box<[Box<str>]>,
    pub(crate) role: Box<str>,
    pub(crate) wave_keys: Box<[Box<str>]>,
    pub(crate) members: Box<[EncounterMemberInput]>,
}

#[derive(Clone, Debug)]
pub(crate) struct EncounterMemberInput {
    pub(crate) order: u16,
    pub(crate) source_rogue_monster_id: Box<str>,
    pub(crate) source_primary_monster_id: Box<str>,
    pub(crate) source_stage_id: Box<str>,
    pub(crate) weight: u64,
    pub(crate) wave_keys: Box<[Box<str>]>,
}

#[derive(Clone, Debug)]
pub(crate) struct EncounterWaveInput {
    pub(crate) id: u32,
    pub(crate) key: Box<str>,
    pub(crate) group_id: u32,
    pub(crate) ordinal: u16,
    pub(crate) slot_keys: Box<[Box<str>]>,
    pub(crate) stage_type: Box<str>,
    pub(crate) authored_level: u16,
    pub(crate) hard_level_group: u16,
}

#[derive(Clone, Debug)]
pub(crate) struct EnemySlotInput {
    pub(crate) id: u32,
    pub(crate) key: Box<str>,
    pub(crate) wave_id: u32,
    pub(crate) formation_index: u8,
    pub(crate) enemy_variant_key: Box<str>,
    pub(crate) boss_choice_keys: Box<[Box<str>]>,
}

#[derive(Clone, Debug)]
pub(crate) struct BossPoolInput {
    pub(crate) id: u32,
    pub(crate) key: Box<str>,
    pub(crate) difficulty_key: Box<str>,
    pub(crate) area_id: u32,
    pub(crate) tier: Box<str>,
    pub(crate) candidate_keys: Box<[Box<str>]>,
}

#[derive(Deserialize)]
struct MemberRow {
    order: u16,
    source_rogue_monster_id: Box<str>,
    source_primary_monster_id: Box<str>,
    source_stage_id: Box<str>,
    weight: Box<str>,
    wave_ids: Box<[Box<str>]>,
}

#[derive(Deserialize)]
struct GroupDifficultyPolicy {
    formal_area_ids: Box<[Box<str>]>,
    formal_difficulty_segment_ids: Box<[Box<str>]>,
    effective_level_policy_id: Box<str>,
    unresolved_behavior: Box<str>,
}

#[derive(Deserialize)]
struct GroupWeightPolicy {
    candidate_order: Box<str>,
    randomness: Box<str>,
    unresolved_behavior: Box<str>,
}

#[derive(Deserialize)]
struct WaveLevelPolicy {
    policy_id: Box<str>,
    authored_stage_level_is_fallback: bool,
    unresolved_area_or_plane_behavior: Box<str>,
}

#[derive(Deserialize)]
struct BossSelectionPolicy {
    randomness: Box<str>,
    unresolved_behavior: Box<str>,
}

impl SwarmDisasterContentCatalog {
    pub(crate) fn encounter_runtime_input(
        &self,
    ) -> Result<EncounterRuntimeInput, UniverseCatalogLoadError> {
        let groups = self
            .encounter_groups
            .iter()
            .map(|row| {
                let difficulty: GroupDifficultyPolicy = decode(&row.difficulty_binding)?;
                let weight_policy: GroupWeightPolicy = decode(&row.weight_policy)?;
                if difficulty.formal_area_ids.as_ref() != row.area_keys.as_ref()
                    || difficulty.formal_difficulty_segment_ids.len() != 15
                    || difficulty.effective_level_policy_id.as_ref()
                        != "swarm-disaster-difficulty-segment-by-area-and-plane-v1"
                    || difficulty.unresolved_behavior.as_ref() != "FailClosed"
                    || weight_policy.candidate_order.as_ref() != "source-group-member-order"
                    || weight_policy.randomness.as_ref() != "seeded-activity-stream"
                    || weight_policy.unresolved_behavior.as_ref() != "FailClosed"
                {
                    return Err(invalid("invalid Swarm encounter-group policy"));
                }
                let members = serde_json::from_str::<Vec<MemberRow>>(&row.members)
                    .map_err(|_| invalid("invalid Swarm encounter members"))?
                    .into_iter()
                    .map(|member| {
                        let weight = member
                            .weight
                            .parse::<u64>()
                            .ok()
                            .filter(|weight| *weight != 0)
                            .ok_or_else(|| invalid("invalid Swarm encounter weight"))?;
                        Ok(EncounterMemberInput {
                            order: member.order,
                            source_rogue_monster_id: member.source_rogue_monster_id,
                            source_primary_monster_id: member.source_primary_monster_id,
                            source_stage_id: member.source_stage_id,
                            weight,
                            wave_keys: member.wave_ids,
                        })
                    })
                    .collect::<Result<Vec<_>, UniverseCatalogLoadError>>()?;
                Ok(EncounterGroupInput {
                    id: row.id.0,
                    key: row.key.clone(),
                    room_key: row.room_key.clone(),
                    area_keys: row.area_keys.clone(),
                    boss_choice_keys: row.boss_choice_keys.clone(),
                    role: row.role.clone(),
                    wave_keys: row.wave_keys.clone(),
                    members: members.into_boxed_slice(),
                })
            })
            .collect::<Result<Vec<_>, UniverseCatalogLoadError>>()?;
        let waves = self
            .encounter_waves
            .iter()
            .map(|row| {
                let policy: WaveLevelPolicy = decode(&row.level_binding)?;
                if policy.policy_id.as_ref()
                    != "swarm-disaster-difficulty-segment-by-area-and-plane-v1"
                    || policy.authored_stage_level_is_fallback
                    || policy.unresolved_area_or_plane_behavior.as_ref() != "FailClosed"
                {
                    return Err(invalid("invalid Swarm encounter-wave level policy"));
                }
                Ok(EncounterWaveInput {
                    id: row.id.0,
                    key: row.key.clone(),
                    group_id: row.group.0,
                    ordinal: row.ordinal,
                    slot_keys: row.slot_keys.clone(),
                    stage_type: row.stage_type.clone(),
                    authored_level: row.authored_level,
                    hard_level_group: row.hard_level_group,
                })
            })
            .collect::<Result<Vec<_>, UniverseCatalogLoadError>>()?;
        let slots = self
            .enemy_slots
            .iter()
            .map(|row| EnemySlotInput {
                id: row.id.0,
                key: row.key.clone(),
                wave_id: row.wave.0,
                formation_index: row.formation_index,
                enemy_variant_key: row.enemy_variant_key.clone(),
                boss_choice_keys: row.boss_choice_keys.clone(),
            })
            .collect::<Vec<_>>();
        let boss_pools = self
            .boss_pools
            .iter()
            .map(|row| {
                let policy: BossSelectionPolicy = decode(&row.selection_policy)?;
                let consequences =
                    serde_json::from_str::<Vec<serde_json::Value>>(&row.consequences)
                        .map_err(|_| invalid("invalid Swarm boss-pool consequences"))?;
                if row.candidate_order.as_ref() != "source-group-id-ascending"
                    || policy.randomness.as_ref() != "seeded-activity-stream"
                    || policy.unresolved_behavior.as_ref() != "FailClosed"
                    || (row.tier.as_ref() == "FinalBoss" && consequences.len() != 1)
                {
                    return Err(invalid("invalid Swarm boss-pool policy"));
                }
                Ok(BossPoolInput {
                    id: row.id.0,
                    key: row.key.clone(),
                    difficulty_key: row.difficulty_key.clone(),
                    area_id: row.area_id,
                    tier: row.tier.clone(),
                    candidate_keys: row.candidate_keys.clone(),
                })
            })
            .collect::<Result<Vec<_>, UniverseCatalogLoadError>>()?;
        Ok(EncounterRuntimeInput {
            groups: groups.into_boxed_slice(),
            waves: waves.into_boxed_slice(),
            slots: slots.into_boxed_slice(),
            boss_pools: boss_pools.into_boxed_slice(),
        })
    }
}

fn decode<T: for<'de> Deserialize<'de>>(value: &str) -> Result<T, UniverseCatalogLoadError> {
    serde_json::from_str(value).map_err(|_| invalid("invalid Swarm encounter policy JSON"))
}

fn invalid(message: &'static str) -> UniverseCatalogLoadError {
    UniverseCatalogLoadError::new(UniverseCatalogLoadErrorKind::InvalidDefinition, message)
}
