use crate::divergent_universe::{DivergentUniverseDataError, debug_error};
use crate::divergent_universe_catalog::DivergentUniverseFlowCatalog;
use crate::divergent_universe_encounter_catalog::*;
use crate::divergent_universe_generated::SoraConfig;
use crate::divergent_universe_progression_catalog::DivergentUniverseProgressionCatalog;
use serde::Deserialize;
pub(super) fn lower_divergent_universe_encounters(
    config: &SoraConfig,
    flow: &DivergentUniverseFlowCatalog,
    progression: &DivergentUniverseProgressionCatalog,
) -> Result<DivergentUniverseEncounterCatalog, DivergentUniverseDataError> {
    let p = DivergentUniverseEncounterCatalogParts {
        sources: config
            .divergent_universe_encounter_source_obligations()
            .ordered_rows()
            .map(|r| {
                let v: SourcePayload = payload(&r.payload_json)?;
                Ok(DivergentUniverseEncounterSourceDefinition {
                    id: source_id(&r.stable_key)?,
                    parent_id: v.parent_id.into(),
                    parent_kind: v.parent_kind.into(),
                    encounter_groups: ids(v.encounter_group_ids, group_id)?,
                    stage_ids: texts(v.stage_ids),
                    map_entry_id: v.map_entry_id.into(),
                    room_type: v.room_type.map(String::into_boxed_str),
                    resolution_state: v.resolution_state.into(),
                    replacement_condition: v.replacement_condition.into(),
                    blocking: v.blocking,
                    runtime_lowered: v.runtime_lowered,
                })
            })
            .collect::<Result<_, _>>()?,
        groups: config
            .divergent_universe_encounter_groups()
            .ordered_rows()
            .map(|r| {
                let v: GroupPayload = payload(&r.payload_json)?;
                Ok(DivergentUniverseEncounterGroupDefinition {
                    id: group_id(&r.stable_key)?,
                    candidate_stage_ids: texts(v.candidate_stage_ids),
                    members: v
                        .members
                        .into_iter()
                        .map(|x| DivergentUniverseEncounterMember {
                            npc_monster_id: x.npc_monster_id.into(),
                            source_monster_id: x.source_monster_id.into(),
                            stage_id: x.stage_id.into(),
                            weight: x.weight.into(),
                        })
                        .collect::<Vec<_>>()
                        .into_boxed_slice(),
                    display_binding_ids: texts(v.display_binding_ids),
                    display_roles: texts(v.display_roles),
                    module_id: v.module_id.into(),
                    area_ids: texts(v.area_id),
                    difficulty_ids: texts(v.difficulty_id),
                    selection_policy: v.selection_policy.into(),
                    reachability_disposition: v.reachability_disposition.into(),
                    runtime_lowered: v.runtime_lowered,
                })
            })
            .collect::<Result<_, _>>()?,
        waves: config
            .divergent_universe_encounter_waves()
            .ordered_rows()
            .map(|r| {
                let v: WavePayload = payload(&r.payload_json)?;
                Ok(DivergentUniverseEncounterWaveDefinition {
                    id: wave_id(&r.stable_key)?,
                    stage_id: v.stage_id.into(),
                    wave_index: v.wave_index,
                    trigger: v.trigger.into(),
                    enemy_slots: ids(v.enemy_slot_ids, slot_id)?,
                    hard_level_group: v.hard_level_group.into(),
                    level: v.level.into(),
                    stage_ability_refs: texts(v.stage_ability_refs),
                    reachability_disposition: v.reachability_disposition.into(),
                    runtime_lowered: v.runtime_lowered,
                })
            })
            .collect::<Result<_, _>>()?,
        slots: config
            .divergent_universe_enemy_slots()
            .ordered_rows()
            .map(|r| {
                let v: SlotPayload = payload(&r.payload_json)?;
                Ok(DivergentUniverseEnemySlotDefinition {
                    id: slot_id(&r.stable_key)?,
                    wave: wave_id(&v.wave_id)?,
                    slot_index: v.slot_index,
                    source_slot: v.source_slot.into(),
                    source_monster_id: v.source_monster_id.into(),
                    monster_id: v.monster_id.into(),
                    enemy_id: v.enemy_id.into(),
                    level: v.level.into(),
                    ability_refs: texts(v.ability_refs),
                    reachability_disposition: v.reachability_disposition.into(),
                    runtime_lowered: v.runtime_lowered,
                })
            })
            .collect::<Result<_, _>>()?,
        boss_pools: config
            .divergent_universe_boss_pools()
            .ordered_rows()
            .map(|r| {
                let v: PoolPayload = payload(&r.payload_json)?;
                Ok(DivergentUniverseBossPoolDefinition {
                    id: pool_id(&r.stable_key)?,
                    encounter_group: group_id(&v.encounter_group_id)?,
                    weekly_modifier_id: v.weekly_modifier_id.into(),
                    source_group_id: v.source_group_id.into(),
                    display_slot: v.display_slot.into(),
                    display_variant: v.display_variant.into(),
                    candidate_monster_ids: texts(v.candidate_monster_ids),
                    candidate_stage_ids: texts(v.candidate_stage_ids),
                    module_id: v.module_id.into(),
                    area_id: v.area_id.into(),
                    difficulty_ids: texts(v.difficulty_id),
                    selection_policy: v.selection_policy.into(),
                    fallback: v.fallback.into(),
                    runtime_lowered: v.runtime_lowered,
                })
            })
            .collect::<Result<_, _>>()?,
        source_obligations: config
            .divergent_universe_coverage()
            .ordered_rows()
            .filter(|r| r.manifest_category.as_deref() == Some("encounter_source_obligations"))
            .count(),
    };
    DivergentUniverseEncounterCatalog::new(p, flow, progression).map_err(debug_error)
}
fn payload<T: for<'de> Deserialize<'de>>(v: &str) -> Result<T, DivergentUniverseDataError> {
    serde_json::from_str(v).map_err(debug_error)
}
fn texts(v: Vec<String>) -> Box<[Box<str>]> {
    v.into_iter()
        .map(String::into_boxed_str)
        .collect::<Vec<_>>()
        .into_boxed_slice()
}
fn ids<T>(
    v: Vec<String>,
    f: fn(&str) -> Result<T, DivergentUniverseDataError>,
) -> Result<Box<[T]>, DivergentUniverseDataError> {
    v.into_iter()
        .map(|x| f(&x))
        .collect::<Result<Vec<_>, _>>()
        .map(Vec::into_boxed_slice)
}
macro_rules! parser {
    ($f:ident,$t:ty) => {
        fn $f(v: &str) -> Result<$t, DivergentUniverseDataError> {
            <$t>::new(v).map_err(debug_error)
        }
    };
}
parser!(source_id, DivergentUniverseEncounterSourceId);
parser!(group_id, DivergentUniverseEncounterGroupId);
parser!(wave_id, DivergentUniverseEncounterWaveId);
parser!(slot_id, DivergentUniverseEnemySlotId);
parser!(pool_id, DivergentUniverseBossPoolId);
#[derive(Deserialize)]
struct SourcePayload {
    blocking: bool,
    encounter_group_ids: Vec<String>,
    #[serde(default)]
    map_entry_id: String,
    parent_id: String,
    parent_kind: String,
    replacement_condition: String,
    resolution_state: String,
    room_type: Option<String>,
    runtime_lowered: bool,
    stage_ids: Vec<String>,
}
#[derive(Deserialize)]
struct Member {
    npc_monster_id: String,
    source_monster_id: String,
    stage_id: String,
    weight: String,
}
#[derive(Deserialize)]
struct GroupPayload {
    area_id: Vec<String>,
    candidate_stage_ids: Vec<String>,
    difficulty_id: Vec<String>,
    display_binding_ids: Vec<String>,
    display_roles: Vec<String>,
    members: Vec<Member>,
    module_id: String,
    reachability_disposition: String,
    runtime_lowered: bool,
    selection_policy: String,
}
#[derive(Deserialize)]
struct WavePayload {
    enemy_slot_ids: Vec<String>,
    hard_level_group: String,
    level: String,
    reachability_disposition: String,
    runtime_lowered: bool,
    stage_ability_refs: Vec<String>,
    stage_id: String,
    trigger: String,
    wave_index: u16,
}
#[derive(Deserialize)]
struct SlotPayload {
    ability_refs: Vec<String>,
    enemy_id: String,
    level: String,
    monster_id: String,
    reachability_disposition: String,
    runtime_lowered: bool,
    slot_index: u16,
    source_monster_id: String,
    source_slot: String,
    wave_id: String,
}
#[derive(Deserialize)]
struct PoolPayload {
    area_id: String,
    candidate_monster_ids: Vec<String>,
    candidate_stage_ids: Vec<String>,
    difficulty_id: Vec<String>,
    display_slot: String,
    display_variant: String,
    encounter_group_id: String,
    fallback: String,
    module_id: String,
    runtime_lowered: bool,
    selection_policy: String,
    source_group_id: String,
    weekly_modifier_id: String,
}
