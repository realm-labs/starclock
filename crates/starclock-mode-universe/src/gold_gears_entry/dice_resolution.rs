//! Custom Dice lifecycle, Path values and isolated roll resolution.

use std::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;
use starclock_activity::{
    ActivityCondition, ActivityExpression, ActivityOperation, ActivityProgramDefinition,
    ActivityProgramId, ActivityRngLabel, ActivityRngStreams, ActivitySlotId,
    ActivityTransactionState, ActivityValue,
};

use crate::gold_gears_unique::{
    CanonicalScalar, DiceDefinition, GoldAndGearsUniqueCatalog, NeuralNode,
};

use super::{
    GoldAndGearsEntryError,
    state_layout::{
        DEFERRED_DICE_INITIAL_BASE, DEFERRED_EFFECTS_SLOT, DICE_RESOLUTION_CANDIDATE_COUNT_KEY,
        DICE_RESOLUTION_DRAW_INDEX_KEY, DICE_RESOLUTION_FACE_KEY, DICE_RESOLUTION_KIND_KEY,
        DICE_RESOLUTION_PREVIOUS_FACE_KEY, DICE_RESOLUTION_SLOT, RESOURCE_COSMIC_FRAGMENTS_KEY,
        RESOURCE_DICE_CHEATS_KEY, RESOURCE_DICE_REROLLS_KEY, RUN_RESOURCES_SLOT,
    },
};

pub const GOLD_AND_GEARS_DICE_RUNTIME_REVISION: &str = "gold-and-gears-dice-runtime-v1";

pub(super) const DICE_ROLL_PURPOSE: u16 = 0x4751;
pub(super) const DICE_REROLL_PURPOSE: u16 = 0x4752;

const ROLL_PROGRAM_ID: u32 = 0x4750_0001;
const REROLL_PROGRAM_ID: u32 = 0x4750_0002;
const CHEAT_PROGRAM_ID: u32 = 0x4750_0003;
const PLANE_START_PROGRAM_BASE: u32 = 0x4760_0000;
const EXCLUDE_PREVIOUS_OPERATION: &str = "ExcludePreviousRerollResult";
const REROLL_POLICY_ID: &str = "neural-network-reroll-empty-candidate-v1";

const RESOLUTION_ROLL: i64 = 1;
const RESOLUTION_REROLL: i64 = 2;
const RESOLUTION_CHEAT: i64 = 3;
const RESOLUTION_NO_CANDIDATE: i64 = 4;

#[derive(Clone, Debug)]
pub(super) struct DiceRuntimeCatalog {
    dice: Box<[RuntimeDice]>,
    reroll_exclusion_node: u32,
}

#[derive(Clone, Debug)]
struct RuntimeDice {
    id: u32,
    key: Box<str>,
    kind: DiceKind,
    initial_effect_ids: Box<[Box<str>]>,
    passive_effect_ids: Box<[Box<str>]>,
    initial: RuntimeEffectPart,
    passive: RuntimeEffectPart,
    path_trigger: RuntimeEffectPart,
    path_values: Box<[RuntimePathValue]>,
    initial_plane_mask: u8,
    plane_start_fragments: i64,
    plane_start_cheats: i64,
}

#[derive(Clone, Debug)]
struct RuntimeEffectPart {
    parameters_scaled: Box<[i64]>,
}

#[derive(Clone, Debug)]
struct RuntimePathValue {
    id: u32,
    path: Box<str>,
    boost_stat: Box<str>,
    trigger_interval: i64,
    boost_value_scaled: i64,
    boost_unit: Box<str>,
}

#[derive(Clone, Debug)]
pub(super) struct CompiledDiceRuntime {
    pub(super) dice_id: u32,
    pub(super) kind: DiceKind,
    pub(super) initial_effect_ids: Box<[Box<str>]>,
    pub(super) passive_effect_ids: Box<[Box<str>]>,
    pub(super) initial_parameters_scaled: Box<[i64]>,
    pub(super) passive_parameters_scaled: Box<[i64]>,
    pub(super) path_trigger_parameters_scaled: Box<[i64]>,
    pub(super) path_value_id: u32,
    pub(super) path_boost_stat: Box<str>,
    pub(super) path_trigger_interval: i64,
    pub(super) path_boost_value_scaled: i64,
    pub(super) path_boost_unit: Box<str>,
    pub(super) reroll_excludes_previous: bool,
    initial_plane_mask: u8,
    plane_start_fragments: i64,
    plane_start_cheats: i64,
}

impl DiceRuntimeCatalog {
    pub(super) fn compile(
        catalog: &GoldAndGearsUniqueCatalog,
    ) -> Result<Self, GoldAndGearsEntryError> {
        let paths = catalog
            .paths
            .iter()
            .map(|path| path.identity.stable_key.as_ref())
            .collect::<BTreeSet<_>>();
        let values_by_dice = path_values_by_dice(catalog, &paths)?;
        let mut dice = Vec::with_capacity(catalog.dice.len());
        for definition in &catalog.dice {
            let parts = decode_effect_parts(definition)?;
            let [initial, passive, path_trigger] = parts.as_slice() else {
                return Err(GoldAndGearsEntryError::InvalidDiceRuntime);
            };
            if initial.role != DiceEffectRole::Initial
                || passive.role != DiceEffectRole::Passive
                || path_trigger.role != DiceEffectRole::PathBoost
            {
                return Err(GoldAndGearsEntryError::InvalidDiceRuntime);
            }
            let path_values = values_by_dice
                .get(&definition.identity.id.0)
                .ok_or(GoldAndGearsEntryError::InvalidDiceRuntime)?;
            let (initial_plane_mask, plane_start_fragments, plane_start_cheats) =
                initial_policy(&definition.identity.source_id, &initial.parameters_scaled)?;
            let kind = dice_kind(
                &definition.identity.source_id,
                &passive.parameters_scaled,
                &path_trigger.parameters_scaled,
            )?;
            dice.push(RuntimeDice {
                id: definition.identity.id.0,
                key: definition.identity.stable_key.clone(),
                kind,
                initial_effect_ids: numeric_effect_ids(&definition.initial_effects)?,
                passive_effect_ids: numeric_effect_ids(&definition.passive_effects)?,
                initial: initial.clone().into_runtime(),
                passive: passive.clone().into_runtime(),
                path_trigger: path_trigger.clone().into_runtime(),
                path_values: path_values.clone().into_boxed_slice(),
                initial_plane_mask,
                plane_start_fragments,
                plane_start_cheats,
            });
        }
        if dice.len() != 12
            || dice
                .iter()
                .any(|definition| definition.path_values.len() != paths.len())
        {
            return Err(GoldAndGearsEntryError::InvalidDiceRuntime);
        }
        let reroll_exclusion_node = reroll_exclusion_node(catalog)?;
        Ok(Self {
            dice: dice.into_boxed_slice(),
            reroll_exclusion_node,
        })
    }

    pub(super) fn select(
        &self,
        dice_key: &str,
        path_key: &str,
        neural: &[&NeuralNode],
    ) -> Result<CompiledDiceRuntime, GoldAndGearsEntryError> {
        let dice = self
            .dice
            .iter()
            .find(|dice| dice.key.as_ref() == dice_key)
            .ok_or(GoldAndGearsEntryError::InvalidDiceRuntime)?;
        let path = dice
            .path_values
            .iter()
            .find(|path| path.path.as_ref() == path_key)
            .ok_or(GoldAndGearsEntryError::InvalidDiceRuntime)?;
        Ok(CompiledDiceRuntime {
            dice_id: dice.id,
            kind: dice.kind,
            initial_effect_ids: dice.initial_effect_ids.clone(),
            passive_effect_ids: dice.passive_effect_ids.clone(),
            initial_parameters_scaled: dice.initial.parameters_scaled.clone(),
            passive_parameters_scaled: dice.passive.parameters_scaled.clone(),
            path_trigger_parameters_scaled: dice.path_trigger.parameters_scaled.clone(),
            path_value_id: path.id,
            path_boost_stat: path.boost_stat.clone(),
            path_trigger_interval: path.trigger_interval,
            path_boost_value_scaled: path.boost_value_scaled,
            path_boost_unit: path.boost_unit.clone(),
            reroll_excludes_previous: neural
                .iter()
                .any(|node| node.identity.id.0 == self.reroll_exclusion_node),
            initial_plane_mask: dice.initial_plane_mask,
            plane_start_fragments: dice.plane_start_fragments,
            plane_start_cheats: dice.plane_start_cheats,
        })
    }

    #[cfg(test)]
    pub(super) fn denominators(&self) -> (usize, usize, usize) {
        (
            self.dice.len(),
            self.dice.iter().map(|dice| dice.path_values.len()).sum(),
            self.dice
                .iter()
                .map(|dice| dice.initial_effect_ids.len() + dice.passive_effect_ids.len())
                .sum(),
        )
    }
}

pub(super) fn compile_roll(
    state: &ActivityTransactionState,
    faces: &[(Box<str>, u32)],
    rng: &mut ActivityRngStreams,
) -> Result<ActivityProgramDefinition, GoldAndGearsEntryError> {
    let previous = resolution_face_id(state).unwrap_or(0);
    choose_face(
        state,
        rng,
        DICE_ROLL_PURPOSE,
        faces,
        ResolutionRequest {
            program_id: ROLL_PROGRAM_ID,
            previous,
            kind: RESOLUTION_ROLL,
        },
        Vec::new(),
    )
}

pub(super) fn compile_reroll(
    state: &ActivityTransactionState,
    faces: &[(Box<str>, u32)],
    exclude_previous: bool,
    rng: &mut ActivityRngStreams,
) -> Result<ActivityProgramDefinition, GoldAndGearsEntryError> {
    let previous = resolution_face_id(state).ok_or(GoldAndGearsEntryError::MissingDiceResult)?;
    require_resource(
        state,
        RESOURCE_DICE_REROLLS_KEY,
        GoldAndGearsEntryError::NoDiceRerolls,
    )?;
    let candidates = faces
        .iter()
        .filter(|(_, id)| !exclude_previous || *id != previous)
        .cloned()
        .collect::<Vec<_>>();
    let resource_operations = vec![
        require_positive_resource(RESOURCE_DICE_REROLLS_KEY),
        ActivityOperation::AddCounter {
            slot: slot(RUN_RESOURCES_SLOT),
            key: RESOURCE_DICE_REROLLS_KEY,
            delta: integer(-1),
        },
    ];
    if candidates.is_empty() {
        return resolution_program(
            state,
            ResolutionSpec {
                program_id: REROLL_PROGRAM_ID,
                selected: previous,
                previous,
                kind: RESOLUTION_NO_CANDIDATE,
                candidate_count: 0,
                draw_index: 0,
            },
            resource_operations,
        );
    }
    choose_face(
        state,
        rng,
        DICE_REROLL_PURPOSE,
        &candidates,
        ResolutionRequest {
            program_id: REROLL_PROGRAM_ID,
            previous,
            kind: RESOLUTION_REROLL,
        },
        resource_operations,
    )
}

pub(super) fn compile_cheat(
    state: &ActivityTransactionState,
    faces: &[(Box<str>, u32)],
    selected: &str,
) -> Result<ActivityProgramDefinition, GoldAndGearsEntryError> {
    require_resource(
        state,
        RESOURCE_DICE_CHEATS_KEY,
        GoldAndGearsEntryError::NoDiceCheats,
    )?;
    let selected_id = faces
        .iter()
        .find(|(key, _)| key.as_ref() == selected)
        .map(|(_, id)| *id)
        .ok_or_else(|| GoldAndGearsEntryError::DiceFaceNotInLoadout(selected.into()))?;
    resolution_program(
        state,
        ResolutionSpec {
            program_id: CHEAT_PROGRAM_ID,
            selected: selected_id,
            previous: resolution_face_id(state).unwrap_or(0),
            kind: RESOLUTION_CHEAT,
            candidate_count: u32::try_from(faces.len())
                .map_err(|_| GoldAndGearsEntryError::InvalidDiceRuntime)?,
            draw_index: 0,
        },
        vec![
            require_positive_resource(RESOURCE_DICE_CHEATS_KEY),
            ActivityOperation::AddCounter {
                slot: slot(RUN_RESOURCES_SLOT),
                key: RESOURCE_DICE_CHEATS_KEY,
                delta: integer(-1),
            },
        ],
    )
}

pub(super) fn compile_plane_start(
    dice: &CompiledDiceRuntime,
    plane_layer: u8,
) -> Result<Option<ActivityProgramDefinition>, GoldAndGearsEntryError> {
    if !(1..=3).contains(&plane_layer) {
        return Err(GoldAndGearsEntryError::InvalidDicePlane);
    }
    let bit = 1_u8
        .checked_shl(u32::from(plane_layer - 1))
        .ok_or(GoldAndGearsEntryError::InvalidDicePlane)?;
    if dice.initial_plane_mask & bit == 0 {
        return Ok(None);
    }
    let marker = DEFERRED_DICE_INITIAL_BASE
        .checked_add(u64::from(dice.dice_id) * 4)
        .and_then(|key| key.checked_add(u64::from(plane_layer)))
        .ok_or(GoldAndGearsEntryError::InvalidDiceRuntime)?;
    let mut operations = vec![
        ActivityOperation::Require(ActivityCondition::Equal(
            counter(DEFERRED_EFFECTS_SLOT, marker),
            integer(0),
        )),
        ActivityOperation::AddCounter {
            slot: slot(DEFERRED_EFFECTS_SLOT),
            key: marker,
            delta: integer(1),
        },
    ];
    if dice.plane_start_fragments != 0 {
        operations.push(ActivityOperation::AddCounter {
            slot: slot(RUN_RESOURCES_SLOT),
            key: RESOURCE_COSMIC_FRAGMENTS_KEY,
            delta: integer(dice.plane_start_fragments),
        });
    }
    if dice.plane_start_cheats != 0 {
        operations.push(ActivityOperation::AddCounter {
            slot: slot(RUN_RESOURCES_SLOT),
            key: RESOURCE_DICE_CHEATS_KEY,
            delta: integer(dice.plane_start_cheats),
        });
    }
    let id = PLANE_START_PROGRAM_BASE
        .checked_add(dice.dice_id * 4)
        .and_then(|id| id.checked_add(u32::from(plane_layer)))
        .and_then(ActivityProgramId::new)
        .ok_or(GoldAndGearsEntryError::InvalidDiceRuntime)?;
    ActivityProgramDefinition::new(id, operations)
        .map(Some)
        .map_err(|_| GoldAndGearsEntryError::InvalidDiceRuntime)
}

pub(super) fn resolution_face<'a>(
    state: &ActivityTransactionState,
    faces: &'a [(Box<str>, u32)],
) -> Option<&'a str> {
    let id = resolution_face_id(state)?;
    faces
        .iter()
        .find(|(_, candidate)| *candidate == id)
        .map(|(key, _)| key.as_ref())
}

pub(super) fn resolution_kind(state: &ActivityTransactionState) -> Option<u8> {
    counter_value(state, DICE_RESOLUTION_SLOT, DICE_RESOLUTION_KIND_KEY)
        .and_then(|value| u8::try_from(value).ok())
        .filter(|value| *value != 0)
}

fn choose_face(
    state: &ActivityTransactionState,
    rng: &mut ActivityRngStreams,
    purpose: u16,
    faces: &[(Box<str>, u32)],
    request: ResolutionRequest,
    prefix: Vec<ActivityOperation>,
) -> Result<ActivityProgramDefinition, GoldAndGearsEntryError> {
    let mut candidates = faces.to_vec();
    candidates.sort_by_key(|(_, id)| *id);
    let candidate_count =
        u32::try_from(candidates.len()).map_err(|_| GoldAndGearsEntryError::InvalidDiceRuntime)?;
    if candidate_count == 0 {
        return Err(GoldAndGearsEntryError::InvalidDiceRuntime);
    }
    rng.transact(|working| {
        let draw = working
            .choose_index(ActivityRngLabel::Spawn, purpose, candidate_count)
            .map_err(|_| GoldAndGearsEntryError::InvalidDiceRuntime)?
            .ok_or(GoldAndGearsEntryError::InvalidDiceRuntime)?;
        let selected = candidates
            .get(draw.value() as usize)
            .ok_or(GoldAndGearsEntryError::InvalidDiceRuntime)?
            .1;
        let draw_index = draw
            .index()
            .checked_add(1)
            .ok_or(GoldAndGearsEntryError::InvalidDiceRuntime)?;
        resolution_program(
            state,
            ResolutionSpec {
                program_id: request.program_id,
                selected,
                previous: request.previous,
                kind: request.kind,
                candidate_count,
                draw_index,
            },
            prefix,
        )
    })
}

fn resolution_program(
    state: &ActivityTransactionState,
    resolution: ResolutionSpec,
    mut prefix: Vec<ActivityOperation>,
) -> Result<ActivityProgramDefinition, GoldAndGearsEntryError> {
    let next = [
        (DICE_RESOLUTION_FACE_KEY, i64::from(resolution.selected)),
        (
            DICE_RESOLUTION_PREVIOUS_FACE_KEY,
            i64::from(resolution.previous),
        ),
        (DICE_RESOLUTION_KIND_KEY, resolution.kind),
        (
            DICE_RESOLUTION_CANDIDATE_COUNT_KEY,
            i64::from(resolution.candidate_count),
        ),
        (
            DICE_RESOLUTION_DRAW_INDEX_KEY,
            i64::try_from(resolution.draw_index)
                .map_err(|_| GoldAndGearsEntryError::InvalidDiceRuntime)?,
        ),
    ];
    for (key, value) in next {
        let current = counter_value(state, DICE_RESOLUTION_SLOT, key).unwrap_or(0);
        prefix.push(ActivityOperation::Require(ActivityCondition::Equal(
            counter(DICE_RESOLUTION_SLOT, key),
            integer(current),
        )));
        prefix.push(ActivityOperation::AddCounter {
            slot: slot(DICE_RESOLUTION_SLOT),
            key,
            delta: integer(value - current),
        });
    }
    ActivityProgramDefinition::new(
        ActivityProgramId::new(resolution.program_id)
            .ok_or(GoldAndGearsEntryError::InvalidDiceRuntime)?,
        prefix,
    )
    .map_err(|_| GoldAndGearsEntryError::InvalidDiceRuntime)
}

#[derive(Clone, Copy)]
struct ResolutionRequest {
    program_id: u32,
    previous: u32,
    kind: i64,
}

#[derive(Clone, Copy)]
struct ResolutionSpec {
    program_id: u32,
    selected: u32,
    previous: u32,
    kind: i64,
    candidate_count: u32,
    draw_index: u64,
}

fn path_values_by_dice(
    catalog: &GoldAndGearsUniqueCatalog,
    paths: &BTreeSet<&str>,
) -> Result<BTreeMap<u32, Vec<RuntimePathValue>>, GoldAndGearsEntryError> {
    let mut values = BTreeMap::<u32, Vec<RuntimePathValue>>::new();
    for value in &catalog.dice_path_values {
        if !paths.contains(value.path_key.as_ref())
            || value.trigger_interval.as_ref() == "0"
            || value.boost_unit.as_ref() != "SourceRatioFormattedAsPercent"
        {
            return Err(GoldAndGearsEntryError::InvalidDiceRuntime);
        }
        let trigger_interval = value
            .trigger_interval
            .parse::<i64>()
            .ok()
            .filter(|interval| *interval > 0)
            .ok_or(GoldAndGearsEntryError::InvalidDiceRuntime)?;
        let boost_value_scaled = scaled(&value.boost_value)?;
        values
            .entry(value.dice.0)
            .or_default()
            .push(RuntimePathValue {
                id: value.identity.id.0,
                path: value.path_key.clone(),
                boost_stat: value.boost_stat.clone(),
                trigger_interval,
                boost_value_scaled,
                boost_unit: value.boost_unit.clone(),
            });
    }
    for entries in values.values_mut() {
        entries.sort_by(|left, right| left.path.cmp(&right.path));
        if entries.len() != paths.len()
            || entries.windows(2).any(|pair| pair[0].path == pair[1].path)
        {
            return Err(GoldAndGearsEntryError::InvalidDiceRuntime);
        }
    }
    Ok(values)
}

fn decode_effect_parts(
    definition: &DiceDefinition,
) -> Result<Vec<DecodedEffectPart>, GoldAndGearsEntryError> {
    let authored = serde_json::from_str::<Vec<AuthoredEffectPart>>(&definition.effect_parts_json)
        .map_err(|_| GoldAndGearsEntryError::InvalidDiceRuntime)?;
    authored
        .into_iter()
        .map(|part| {
            let role = match part.role.as_ref() {
                "InitialEffect" => DiceEffectRole::Initial,
                "PassiveEffect" => DiceEffectRole::Passive,
                "PathBoostTrigger" => DiceEffectRole::PathBoost,
                _ => return Err(GoldAndGearsEntryError::InvalidDiceRuntime),
            };
            let parameters_scaled = part
                .parameters
                .iter()
                .map(|value| scaled_text(value))
                .collect::<Result<Vec<_>, _>>()?
                .into_boxed_slice();
            Ok(DecodedEffectPart {
                role,
                parameters_scaled,
            })
        })
        .collect()
}

fn numeric_effect_ids(values: &[Box<str>]) -> Result<Box<[Box<str>]>, GoldAndGearsEntryError> {
    let mut ids = values.to_vec();
    ids.sort_unstable();
    if ids.windows(2).any(|pair| pair[0] == pair[1])
        || ids
            .iter()
            .any(|id| id.parse::<u64>().ok().is_none_or(|value| value == 0))
    {
        return Err(GoldAndGearsEntryError::InvalidDiceRuntime);
    }
    Ok(ids.into_boxed_slice())
}

fn initial_policy(
    source: &str,
    parameters: &[i64],
) -> Result<(u8, i64, i64), GoldAndGearsEntryError> {
    const FIRST_SECOND: u8 = 0b011;
    const FIRST_THIRD: u8 = 0b101;
    const EVERY_PLANE: u8 = 0b111;
    match (source, parameters) {
        ("101", []) => Ok((FIRST_SECOND, 0, 0)),
        ("102", [2_000_000]) => Ok((FIRST_SECOND, 0, 0)),
        ("103" | "201", [1_000_000]) => Ok((FIRST_SECOND, 0, 0)),
        ("202" | "301" | "302" | "303" | "402", [2_000_000]) => Ok((FIRST_SECOND, 0, 0)),
        ("203", []) => Ok((FIRST_THIRD, 0, 0)),
        ("401", [2_000_000, 100_000_000]) => Ok((FIRST_SECOND, 100, 0)),
        ("403", [1_000_000]) => Ok((EVERY_PLANE, 0, 1)),
        _ => Err(GoldAndGearsEntryError::InvalidDiceRuntime),
    }
}

fn dice_kind(
    source: &str,
    passive: &[i64],
    path_trigger: &[i64],
) -> Result<DiceKind, GoldAndGearsEntryError> {
    let kind = match (source, passive, path_trigger) {
        ("101", [80_000_000], [1_000_000]) => DiceKind::Trotter,
        ("102", [], [1_000_000]) => DiceKind::Knowledge,
        ("103", [1_000_000], [50_000_000]) => DiceKind::Beacon,
        ("201", [1_000_000], [1_000_000]) => DiceKind::Occurrence,
        ("202", [1_000_000], [1_000_000]) => DiceKind::Elite,
        ("203", [], [1_000_000]) => DiceKind::Domain,
        ("301", [5_000_000, 1_000_000, 10_000_000], []) => DiceKind::Countdown,
        ("302", [15_000_000], [1_000_000]) => DiceKind::KnowledgeProtection,
        ("303", [50_000_000], [1_000_000]) => DiceKind::KnowledgeCollapse,
        ("401", [300_000], [100_000_000]) => DiceKind::Transaction,
        ("402", [40_000_000], [1_000_000]) => DiceKind::Curio,
        ("403", [1_000_000], []) => DiceKind::GeneralBuff,
        _ => return Err(GoldAndGearsEntryError::InvalidDiceRuntime),
    };
    Ok(kind)
}

fn reroll_exclusion_node(
    catalog: &GoldAndGearsUniqueCatalog,
) -> Result<u32, GoldAndGearsEntryError> {
    let mut result = None;
    for node in &catalog.neural_nodes {
        let contributions =
            serde_json::from_str::<ContributionEnvelope>(&node.effect_contributions_json)
                .map(ContributionEnvelope::into_vec)
                .map_err(|_| GoldAndGearsEntryError::InvalidDiceRuntime)?;
        for contribution in contributions {
            if contribution.operation.as_ref() != EXCLUDE_PREVIOUS_OPERATION {
                continue;
            }
            if result.is_some()
                || contribution.scope.as_deref() != Some("Activity")
                || contribution.target.as_deref() != Some("dice-face-result")
                || contribution.exclusion.as_deref() != Some("PreviousResult")
                || contribution.selection_policy.as_ref().is_none_or(|policy| {
                    policy.policy_id.as_ref() != REROLL_POLICY_ID
                        || policy.candidate_order.as_ref() != "stable-dice-face-id-ascending"
                        || policy.draw_mode.as_ref() != "seeded-from-eligible-candidates"
                        || policy.empty_candidate_behavior.as_ref()
                            != "KeepPreviousAndConsumeAttempt"
                        || policy.replacement_condition.is_empty()
                })
            {
                return Err(GoldAndGearsEntryError::InvalidDiceRuntime);
            }
            result = Some(node.identity.id.0);
        }
    }
    result.ok_or(GoldAndGearsEntryError::InvalidDiceRuntime)
}

fn scaled(value: &CanonicalScalar) -> Result<i64, GoldAndGearsEntryError> {
    scaled_text(&value.0)
}

fn scaled_text(value: &str) -> Result<i64, GoldAndGearsEntryError> {
    let negative = value.starts_with('-');
    let unsigned = value.strip_prefix('-').unwrap_or(value);
    let (integer, fraction) = unsigned
        .split_once('.')
        .map_or((unsigned, ""), |parts| parts);
    let whole = integer
        .parse::<i64>()
        .map_err(|_| GoldAndGearsEntryError::InvalidDiceRuntime)?;
    let mut fraction_text = fraction.to_owned();
    if fraction_text.len() > 6 {
        return Err(GoldAndGearsEntryError::InvalidDiceRuntime);
    }
    fraction_text.extend(core::iter::repeat_n('0', 6 - fraction_text.len()));
    let fractional = if fraction_text.is_empty() {
        0
    } else {
        fraction_text
            .parse::<i64>()
            .map_err(|_| GoldAndGearsEntryError::InvalidDiceRuntime)?
    };
    let magnitude = whole
        .checked_mul(1_000_000)
        .and_then(|value| value.checked_add(fractional))
        .ok_or(GoldAndGearsEntryError::InvalidDiceRuntime)?;
    if negative {
        magnitude
            .checked_neg()
            .ok_or(GoldAndGearsEntryError::InvalidDiceRuntime)
    } else {
        Ok(magnitude)
    }
}

fn require_resource(
    state: &ActivityTransactionState,
    key: u64,
    error: GoldAndGearsEntryError,
) -> Result<(), GoldAndGearsEntryError> {
    if counter_value(state, RUN_RESOURCES_SLOT, key).is_none_or(|value| value <= 0) {
        return Err(error);
    }
    Ok(())
}

fn require_positive_resource(key: u64) -> ActivityOperation {
    ActivityOperation::Require(ActivityCondition::LessThan(
        integer(0),
        counter(RUN_RESOURCES_SLOT, key),
    ))
}

fn resolution_face_id(state: &ActivityTransactionState) -> Option<u32> {
    counter_value(state, DICE_RESOLUTION_SLOT, DICE_RESOLUTION_FACE_KEY)
        .and_then(|value| u32::try_from(value).ok())
        .filter(|value| *value != 0)
}

fn counter_value(state: &ActivityTransactionState, slot_id: u32, key: u64) -> Option<i64> {
    match state.slot(slot(slot_id)) {
        Some(ActivityValue::BoundedCounterMap(values)) => values
            .binary_search_by_key(&key, |(candidate, _)| *candidate)
            .ok()
            .map(|index| values[index].1),
        _ => None,
    }
}

fn counter(slot_id: u32, key: u64) -> ActivityExpression {
    ActivityExpression::CounterValue {
        slot: slot(slot_id),
        key,
    }
}

fn integer(value: i64) -> ActivityExpression {
    ActivityExpression::Literal(ActivityValue::BoundedInteger(value))
}

fn slot(raw: u32) -> ActivitySlotId {
    ActivitySlotId::new(raw).expect("static Gold and Gears slot is non-zero")
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DiceEffectRole {
    Initial,
    Passive,
    PathBoost,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum DiceKind {
    Trotter,
    Knowledge,
    Beacon,
    Occurrence,
    Elite,
    Domain,
    Countdown,
    KnowledgeProtection,
    KnowledgeCollapse,
    Transaction,
    Curio,
    GeneralBuff,
}

#[derive(Clone, Debug)]
struct DecodedEffectPart {
    role: DiceEffectRole,
    parameters_scaled: Box<[i64]>,
}

impl DecodedEffectPart {
    fn into_runtime(self) -> RuntimeEffectPart {
        RuntimeEffectPart {
            parameters_scaled: self.parameters_scaled,
        }
    }
}

#[derive(Deserialize)]
struct AuthoredEffectPart {
    role: Box<str>,
    parameters: Box<[Box<str>]>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum ContributionEnvelope {
    Many(Vec<NeuralContribution>),
    One(NeuralContribution),
}

impl ContributionEnvelope {
    fn into_vec(self) -> Vec<NeuralContribution> {
        match self {
            Self::Many(values) => values,
            Self::One(value) => vec![value],
        }
    }
}

#[derive(Deserialize)]
struct NeuralContribution {
    operation: Box<str>,
    #[serde(default)]
    scope: Option<Box<str>>,
    #[serde(default)]
    target: Option<Box<str>>,
    #[serde(default)]
    exclusion: Option<Box<str>>,
    #[serde(default)]
    selection_policy: Option<RerollSelectionPolicy>,
}

#[derive(Deserialize)]
struct RerollSelectionPolicy {
    policy_id: Box<str>,
    candidate_order: Box<str>,
    draw_mode: Box<str>,
    empty_candidate_behavior: Box<str>,
    replacement_condition: Box<str>,
}
