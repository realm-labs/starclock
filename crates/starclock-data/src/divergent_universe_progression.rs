use crate::divergent_universe::{DivergentUniverseDataError, debug_error};
use crate::divergent_universe_catalog::DivergentUniverseFlowCatalog;
use crate::divergent_universe_generated::SoraConfig;
use crate::divergent_universe_progression_catalog::*;
use serde::Deserialize;
const COVERAGE: [&str; 7] = [
    "astronomical_divisions",
    "astronomical_division_effects",
    "permanent_talents",
    "unlocks",
    "common_constants",
    "weekly_modifiers",
    "room_marks",
];

pub(super) fn lower_divergent_universe_progression(
    config: &SoraConfig,
    flow: &DivergentUniverseFlowCatalog,
) -> Result<DivergentUniverseProgressionCatalog, DivergentUniverseDataError> {
    let p = DivergentUniverseProgressionCatalogParts {
        protocols: config
            .divergent_universe_protocols()
            .ordered_rows()
            .map(|r| {
                let v: ProtocolPayload = payload(&r.payload_json)?;
                Ok(DivergentUniverseProtocolDefinition {
                    id: protocol_id(&r.stable_key)?,
                    level: v.protocol_level,
                    difficulty_changes: texts(v.difficulty_changes),
                    entry_rules: texts(v.entry_rules),
                    berserk_changes: texts(v.enemy_changes.berserk_changes),
                    boss_identity: v.enemy_changes.first_second_plane_boss_identity.into(),
                    plane_scaling: DivergentUniversePlaneScaling {
                        attack: v.enemy_changes.plane_scaled_maximum_increase.attack.into(),
                        max_hp: v.enemy_changes.plane_scaled_maximum_increase.max_hp.into(),
                        speed: v.enemy_changes.plane_scaled_maximum_increase.speed.into(),
                        max_toughness: v
                            .enemy_changes
                            .plane_scaled_maximum_increase
                            .max_toughness
                            .map(String::into_boxed_str),
                    },
                    source_parameters: texts(v.source_parameters),
                    runtime_lowered: v.runtime_lowered,
                })
            })
            .collect::<Result<_, _>>()?,
        divisions: config
            .divergent_universe_astronomical_divisions()
            .ordered_rows()
            .map(|r| {
                let v: DivisionPayload = payload(&r.payload_json)?;
                Ok(DivergentUniverseDivisionDefinition {
                    id: division_id(&r.stable_key)?,
                    level: v.division_level,
                    protocols: ids(v.effect_ids, protocol_id)?,
                    progress_boundary: v.progress_boundary.into(),
                    cognoculi_retention: v.cognoculi_retention.into(),
                    runtime_lowered: v.runtime_lowered,
                })
            })
            .collect::<Result<_, _>>()?,
        modes: config
            .divergent_universe_star_pioneer_practice()
            .ordered_rows()
            .map(|r| {
                let v: ModePayload = payload(&r.payload_json)?;
                Ok(DivergentUniverseAstronomicalModeDefinition {
                    id: mode_id(&r.stable_key)?,
                    mode_kind: v.mode_kind.into(),
                    available_content: texts(v.available_content),
                    entry_rules: texts(v.entry_rules),
                    reset_rules: texts(v.reset_rules),
                    runtime_lowered: v.runtime_lowered,
                })
            })
            .collect::<Result<_, _>>()?,
        cognoculi: config
            .divergent_universe_cognoculi()
            .ordered_rows()
            .map(|r| {
                let v: CognoculiPayload = payload(&r.payload_json)?;
                Ok(DivergentUniverseCognoculiDefinition {
                    id: cognoculi_id(&r.stable_key)?,
                    division: division_id(&v.division_id)?,
                    contribution_divisions: ids(v.contribution_ids, division_id)?,
                    retention: v.retention.into(),
                    gain: v.gain.into(),
                    loss: v.loss.into(),
                    division_floor: v.division_floor.into(),
                    runtime_lowered: v.runtime_lowered,
                })
            })
            .collect::<Result<_, _>>()?,
        talents: config
            .divergent_universe_permanent_talents()
            .ordered_rows()
            .map(|r| {
                let v: TalentPayload = payload(&r.payload_json)?;
                Ok(DivergentUniversePermanentTalentDefinition {
                    id: talent_id(&r.stable_key)?,
                    adjacent: ids(v.adjacent_talent_ids, talent_id)?,
                    prerequisites: ids(v.prerequisite_ids, talent_id)?,
                    prerequisite_resolution: v.prerequisite_resolution.into(),
                    cost: v
                        .cost
                        .into_iter()
                        .map(|x| DivergentUniverseTalentCost {
                            item_id: x.item_id.into(),
                            amount: x.amount.into(),
                        })
                        .collect::<Vec<_>>()
                        .into_boxed_slice(),
                    effects: ids(v.effect_ids, effect_id)?,
                    program: program(v.effect_program),
                    important: v.important,
                    scope: v.scope.into(),
                    runtime_lowered: v.runtime_lowered,
                })
            })
            .collect::<Result<_, _>>()?,
        unlocks: config
            .divergent_universe_unlocks()
            .ordered_rows()
            .map(|r| {
                let v: UnlockPayload = payload(&r.payload_json)?;
                Ok(DivergentUniverseUnlockDefinition {
                    id: unlock_id(&r.stable_key)?,
                    finish_condition_id: v.finish_condition_id.into(),
                    scope: v.scope.into(),
                    unlocked_content_ids: texts(v.unlocked_content_ids),
                    runtime_lowered: v.runtime_lowered,
                })
            })
            .collect::<Result<_, _>>()?,
        constants: config
            .divergent_universe_common_constants()
            .ordered_rows()
            .map(|r| {
                let v: ConstantPayload = payload(&r.payload_json)?;
                Ok(DivergentUniverseConstantDefinition {
                    id: constant_id(&r.stable_key)?,
                    canonical_value: match v.canonical_value {
                        ConstantValue::Scalar(value) => {
                            DivergentUniverseConstantValue::Scalar(value.into())
                        }
                        ConstantValue::Array(values) => {
                            DivergentUniverseConstantValue::Array(texts(values))
                        }
                    },
                    consumers: texts(v.consumer_ids),
                    exclusion_reason: optional(v.exclusion_reason),
                    value_kind: v.value_kind.into(),
                    runtime_lowered: v.runtime_lowered,
                })
            })
            .collect::<Result<_, _>>()?,
        weekly_modifiers: config
            .divergent_universe_weekly_modifiers()
            .ordered_rows()
            .map(|r| {
                let v: WeeklyPayload = payload(&r.payload_json)?;
                Ok(DivergentUniverseWeeklyModifierDefinition {
                    id: weekly_id(&r.stable_key)?,
                    content_ids: texts(v.content_ids),
                    detail_ids: texts(v.detail_ids),
                    effect_ids: texts(v.effect_ids),
                    enemy_groups: v
                        .enemy_group_refs
                        .into_iter()
                        .map(|x| DivergentUniverseWeeklyEnemyGroup {
                            slot: x.slot.into(),
                            source_group_id: x.source_group_id.into(),
                            variant: x.variant.into(),
                            resolution: x.resolution.into(),
                        })
                        .collect::<Vec<_>>()
                        .into_boxed_slice(),
                    reachability: v.reachability.into(),
                    runtime_lowered: v.runtime_lowered,
                })
            })
            .collect::<Result<_, _>>()?,
        room_marks: config
            .divergent_universe_room_marks()
            .ordered_rows()
            .map(|r| {
                let v: RoomMarkPayload = payload(&r.payload_json)?;
                Ok(DivergentUniverseRoomMarkDefinition {
                    id: room_mark_id(&r.stable_key)?,
                    room_type: v.room_type.into(),
                    mark_kind: v.mark_kind.into(),
                    transition_rules: texts(v.transition_rules),
                    fallback: v.fallback.into(),
                    runtime_lowered: v.runtime_lowered,
                })
            })
            .collect::<Result<_, _>>()?,
        effects: config
            .divergent_universe_progression_effects()
            .ordered_rows()
            .map(|r| {
                let v: ProgressionEffectPayload = payload(&r.payload_json)?;
                Ok(DivergentUniverseProgressionEffectDefinition {
                    id: effect_id(&r.stable_key)?,
                    activation: v.activation.into(),
                    contributions: v
                        .rule_contribution_ids
                        .into_iter()
                        .map(program)
                        .collect::<Vec<_>>()
                        .into_boxed_slice(),
                    scope: v.scope.into(),
                    runtime_lowered: v.runtime_lowered,
                })
            })
            .collect::<Result<_, _>>()?,
        source_obligations: config
            .divergent_universe_coverage()
            .ordered_rows()
            .filter(|r| {
                r.manifest_category
                    .as_deref()
                    .is_some_and(|x| COVERAGE.contains(&x))
            })
            .count(),
    };
    DivergentUniverseProgressionCatalog::new(p, flow).map_err(debug_error)
}
fn program(v: Program) -> DivergentUniverseProgressionProgram {
    DivergentUniverseProgressionProgram {
        condition: v.condition.into(),
        description_hash: v.description_hash.map(String::into_boxed_str),
        metric: v.metric.into(),
        operation: v.operation.into(),
        parameters: texts(v.parameters),
        scope: v.scope.map(String::into_boxed_str),
    }
}
fn optional(v: String) -> Option<Box<str>> {
    if v.is_empty() { None } else { Some(v.into()) }
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
parser!(protocol_id, DivergentUniverseProtocolId);
parser!(division_id, DivergentUniverseDivisionId);
parser!(mode_id, DivergentUniverseAstronomicalModeId);
parser!(cognoculi_id, DivergentUniverseCognoculiId);
parser!(talent_id, DivergentUniversePermanentTalentId);
parser!(unlock_id, DivergentUniverseUnlockId);
parser!(constant_id, DivergentUniverseConstantId);
parser!(weekly_id, DivergentUniverseWeeklyModifierId);
parser!(room_mark_id, DivergentUniverseRoomMarkId);
parser!(effect_id, DivergentUniverseProgressionEffectId);
#[derive(Deserialize)]
struct PlaneScaling {
    attack: String,
    max_hp: String,
    speed: String,
    max_toughness: Option<String>,
}
#[derive(Deserialize)]
struct EnemyChanges {
    berserk_changes: Vec<String>,
    first_second_plane_boss_identity: String,
    plane_scaled_maximum_increase: PlaneScaling,
}
#[derive(Deserialize)]
struct ProtocolPayload {
    difficulty_changes: Vec<String>,
    enemy_changes: EnemyChanges,
    entry_rules: Vec<String>,
    protocol_level: u16,
    runtime_lowered: bool,
    source_parameters: Vec<String>,
}
#[derive(Deserialize)]
struct DivisionPayload {
    cognoculi_retention: String,
    division_level: u16,
    effect_ids: Vec<String>,
    progress_boundary: String,
    runtime_lowered: bool,
}
#[derive(Deserialize)]
struct ModePayload {
    available_content: Vec<String>,
    entry_rules: Vec<String>,
    mode_kind: String,
    reset_rules: Vec<String>,
    runtime_lowered: bool,
}
#[derive(Deserialize)]
struct CognoculiPayload {
    contribution_ids: Vec<String>,
    division_floor: String,
    division_id: String,
    gain: String,
    loss: String,
    retention: String,
    runtime_lowered: bool,
}
#[derive(Deserialize)]
struct Cost {
    amount: String,
    item_id: String,
}
#[derive(Deserialize)]
struct Program {
    condition: String,
    description_hash: Option<String>,
    metric: String,
    operation: String,
    parameters: Vec<String>,
    scope: Option<String>,
}
#[derive(Deserialize)]
struct TalentPayload {
    adjacent_talent_ids: Vec<String>,
    cost: Vec<Cost>,
    effect_ids: Vec<String>,
    effect_program: Program,
    important: bool,
    prerequisite_ids: Vec<String>,
    prerequisite_resolution: String,
    runtime_lowered: bool,
    scope: String,
}
#[derive(Deserialize)]
struct UnlockPayload {
    finish_condition_id: String,
    runtime_lowered: bool,
    scope: String,
    unlocked_content_ids: Vec<String>,
}
#[derive(Deserialize)]
struct ConstantPayload {
    canonical_value: ConstantValue,
    consumer_ids: Vec<String>,
    exclusion_reason: String,
    runtime_lowered: bool,
    value_kind: String,
}
#[derive(Deserialize)]
#[serde(untagged)]
enum ConstantValue {
    Scalar(String),
    Array(Vec<String>),
}
#[derive(Deserialize)]
struct EnemyGroup {
    resolution: String,
    slot: String,
    source_group_id: String,
    variant: String,
}
#[derive(Deserialize)]
struct WeeklyPayload {
    content_ids: Vec<String>,
    detail_ids: Vec<String>,
    effect_ids: Vec<String>,
    enemy_group_refs: Vec<EnemyGroup>,
    reachability: String,
    runtime_lowered: bool,
}
#[derive(Deserialize)]
struct RoomMarkPayload {
    fallback: String,
    mark_kind: String,
    room_type: String,
    runtime_lowered: bool,
    transition_rules: Vec<String>,
}
#[derive(Deserialize)]
struct ProgressionEffectPayload {
    activation: String,
    rule_contribution_ids: Vec<Program>,
    runtime_lowered: bool,
    scope: String,
}
