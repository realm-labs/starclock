//! Audience Die definitions, unlock metadata and persistent Path graph rules.

use std::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;
use starclock_activity::{
    ActivityCondition, ActivityExpression, ActivityOperation, ActivityProgramDefinition,
    ActivityProgramId, ActivitySlotId, ActivityTransactionState, ActivityValue,
};

use crate::{
    error::{UniverseCatalogLoadError, UniverseCatalogLoadErrorKind},
    swarm_disaster_unique::runtime_access::{
        SwarmAudienceDieRuntimeInput, SwarmAudienceFaceRuntimeInput, SwarmAudiencePathRuntimeInput,
        SwarmAudienceRuntimeInput,
    },
};

use super::state::{AUDIENCE_DIE, CONTENT};

const INITIALIZATION_PROGRAM_BASE: u32 = 0x5370_0000;

const SELECTED_DIE_KEY: u64 = 1;
const SELECTED_PATH_KEY: u64 = 2;
const PATH_SORT_KEY: u64 = 3;
const UNLOCK_POLICY_KEY: u64 = 4;
const UNLOCK_ID_KEY: u64 = 5;
const INITIAL_MAZE_BUFF_KEY: u64 = 6;
const PASSIVE_KIND_KEY: u64 = 7;
const FACE_COUNT_KEY: u64 = 8;
const INITIAL_APPLIED_KEY: u64 = 9;
const PASSIVE_ACTIVE_KEY: u64 = 10;
const UNLOCK_AUTHORIZED_KEY: u64 = 11;
const FACE_RARITY_KEY_BASE: u64 = 0x1000_0000;
const ACTIVE_MAZE_BUFF_KEY_BASE: u64 = 0x2000_0000;

#[derive(Clone, Debug)]
pub(super) struct AudienceRuntimeCatalog {
    definitions: Box<[RuntimeAudienceDefinition]>,
    _rarity_count: usize,
    _face_count: usize,
    _required_unlock_count: usize,
    _available_without_unlock_count: usize,
    _initial_parameter_slots: usize,
    _passive_parameter_slots: usize,
}

#[derive(Clone, Debug)]
struct RuntimeAudienceDefinition {
    path_id: u32,
    _path_key: Box<str>,
    die_id: u32,
    die_key: Box<str>,
    shared_path: Box<str>,
    sort: u16,
    unlock: RuntimeUnlock,
    faces: Box<[RuntimeFace]>,
    initial: RuntimeEffect,
    passive: RuntimeEffect,
    passive_kind: PassiveKind,
    _description_parameters: Box<[Box<str>]>,
    _rogue_buff_type: u32,
    _battle_event_buff_group: u32,
    _battle_event_enhance_buff_group: u32,
    _extra_effect_refs: Box<[Box<str>]>,
}

#[derive(Clone, Debug)]
struct RuntimeFace {
    id: u32,
    key: Box<str>,
    rarity_rank: u8,
}

#[derive(Clone, Debug)]
struct RuntimeEffect {
    operation: Box<str>,
    parameters: Box<[Box<str>]>,
    secondary_parameters: Box<[Box<str>]>,
}

#[derive(Clone, Debug)]
enum RuntimeUnlock {
    Required { id: Box<str>, numeric_id: u32 },
    Available,
}

impl RuntimeUnlock {
    const fn policy_code(&self) -> i64 {
        match self {
            Self::Required { .. } => 1,
            Self::Available => 2,
        }
    }

    const fn numeric_id(&self) -> u32 {
        match self {
            Self::Required { numeric_id, .. } => *numeric_id,
            Self::Available => 0,
        }
    }

    fn id(&self) -> Option<&str> {
        match self {
            Self::Required { id, .. } => Some(id),
            Self::Available => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum PassiveKind {
    ProtectCellNoCollapse = 1,
    ExtraMoneyAndRandomSwap = 2,
    ReRandomEmptyCell = 3,
    FertileAeonGain = 4,
    ExtraMarkAndRandomSwap = 5,
    DestroyAeonGain = 6,
    GetHelpOnEnterCell = 7,
    RandomGenSwarm = 8,
}

impl PassiveKind {
    const fn code(self) -> i64 {
        self as i64
    }
}

#[derive(Clone, Debug)]
pub(super) struct CompiledAudienceRuntime {
    definition: RuntimeAudienceDefinition,
}

impl AudienceRuntimeCatalog {
    pub(super) fn compile(
        input: SwarmAudienceRuntimeInput,
    ) -> Result<Self, UniverseCatalogLoadError> {
        let rarities = rarity_ranks(&input)?;
        let faces = face_inputs(&input)?;
        let dice = input
            .dice
            .iter()
            .map(|die| (die.id, die))
            .collect::<BTreeMap<_, _>>();
        if dice.len() != input.dice.len() {
            return Err(invalid("duplicate Swarm Audience Die ID"));
        }
        let mut definitions = input
            .paths
            .iter()
            .map(|path| compile_definition(path, &dice, &faces, &rarities))
            .collect::<Result<Vec<_>, _>>()?;
        definitions.sort_unstable_by_key(|definition| definition.sort);
        let required_unlock_count = definitions
            .iter()
            .filter(|definition| matches!(definition.unlock, RuntimeUnlock::Required { .. }))
            .count();
        let available_without_unlock_count = definitions.len() - required_unlock_count;
        let initial_parameter_slots = definitions
            .iter()
            .map(|definition| {
                definition.initial.parameters.len() + definition.initial.secondary_parameters.len()
            })
            .sum();
        let passive_parameter_slots = definitions
            .iter()
            .map(|definition| {
                definition.passive.parameters.len() + definition.passive.secondary_parameters.len()
            })
            .sum();
        let used_faces = definitions
            .iter()
            .flat_map(|definition| definition.faces.iter().map(|face| face.id))
            .collect::<BTreeSet<_>>();
        let passive_kinds = definitions
            .iter()
            .map(|definition| definition.passive_kind)
            .collect::<BTreeSet<_>>();
        if input.paths.len() != 8
            || input.dice.len() != 8
            || input.rarities.len() != 3
            || input.faces.len() != 42
            || definitions.len() != 8
            || definitions
                .iter()
                .enumerate()
                .any(|(index, definition)| usize::from(definition.sort) != index + 1)
            || definitions
                .iter()
                .any(|definition| !(5..=6).contains(&definition.faces.len()))
            || used_faces.len() != 42
            || passive_kinds.len() != 8
            || required_unlock_count != 7
            || available_without_unlock_count != 1
            || initial_parameter_slots != 16
            || passive_parameter_slots != 26
        {
            return Err(invalid("Swarm Audience runtime denominator drift"));
        }
        Ok(Self {
            definitions: definitions.into_boxed_slice(),
            _rarity_count: rarities.len(),
            _face_count: used_faces.len(),
            _required_unlock_count: required_unlock_count,
            _available_without_unlock_count: available_without_unlock_count,
            _initial_parameter_slots: initial_parameter_slots,
            _passive_parameter_slots: passive_parameter_slots,
        })
    }

    pub(super) fn select(
        &self,
        shared_path: &str,
        die_key: &str,
        unlocked: &[Box<str>],
    ) -> Result<CompiledAudienceRuntime, UniverseCatalogLoadError> {
        let definition = self
            .definitions
            .iter()
            .find(|definition| {
                definition.shared_path.as_ref() == shared_path
                    && definition.die_key.as_ref() == die_key
            })
            .ok_or_else(|| invalid("unknown Swarm Audience Path and Die pair"))?;
        self.validate_unlocks(definition, unlocked)?;
        Ok(CompiledAudienceRuntime {
            definition: definition.clone(),
        })
    }

    #[cfg(test)]
    pub(super) fn denominators(&self) -> (usize, usize, usize, usize, usize, usize, usize) {
        (
            self.definitions.len(),
            self._rarity_count,
            self._face_count,
            self._required_unlock_count,
            self._available_without_unlock_count,
            self._initial_parameter_slots,
            self._passive_parameter_slots,
        )
    }

    #[cfg(test)]
    pub(super) fn ordered_paths(&self) -> impl ExactSizeIterator<Item = (&str, &str)> {
        self.definitions
            .iter()
            .map(|definition| (definition._path_key.as_ref(), definition.die_key.as_ref()))
    }

    fn validate_unlocks(
        &self,
        selected: &RuntimeAudienceDefinition,
        unlocked: &[Box<str>],
    ) -> Result<(), UniverseCatalogLoadError> {
        let known = self
            .definitions
            .iter()
            .filter_map(|definition| definition.unlock.id())
            .collect::<BTreeSet<_>>();
        let provided = unlocked
            .iter()
            .map(AsRef::as_ref)
            .collect::<BTreeSet<&str>>();
        if provided.len() != unlocked.len()
            || provided.iter().any(|id| !known.contains(id))
            || selected
                .unlock
                .id()
                .is_some_and(|required| !provided.contains(required))
        {
            return Err(invalid("Swarm Audience Path is not unlocked"));
        }
        Ok(())
    }
}

impl CompiledAudienceRuntime {
    pub(super) fn state_values(&self) -> Result<Box<[(u64, i64)]>, UniverseCatalogLoadError> {
        let definition = &self.definition;
        let mut values = vec![
            (SELECTED_DIE_KEY, i64::from(definition.die_id)),
            (SELECTED_PATH_KEY, i64::from(definition.path_id)),
            (PATH_SORT_KEY, i64::from(definition.sort)),
            (UNLOCK_POLICY_KEY, definition.unlock.policy_code()),
            (UNLOCK_ID_KEY, i64::from(definition.unlock.numeric_id())),
            (
                INITIAL_MAZE_BUFF_KEY,
                i64::from(initial_maze_buff(definition)?),
            ),
            (PASSIVE_KIND_KEY, definition.passive_kind.code()),
            (UNLOCK_AUTHORIZED_KEY, 1),
            (
                FACE_COUNT_KEY,
                i64::try_from(definition.faces.len())
                    .map_err(|_| invalid("Swarm Audience face count overflow"))?,
            ),
        ];
        for face in &definition.faces {
            values.push((face_key(face.id)?, i64::from(face.rarity_rank)));
        }
        values.sort_unstable_by_key(|(key, _)| *key);
        Ok(values.into_boxed_slice())
    }

    pub(super) fn compile_initialization(
        &self,
        state: &ActivityTransactionState,
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        let definition = &self.definition;
        let maze_buff = initial_maze_buff(definition)?;
        require_metadata(state, definition)?;
        let operations = vec![
            ActivityOperation::Require(ActivityCondition::All(
                vec![
                    equal(AUDIENCE_DIE, INITIAL_APPLIED_KEY, 0),
                    equal(AUDIENCE_DIE, PASSIVE_ACTIVE_KEY, 0),
                    equal(CONTENT, active_maze_buff_key(maze_buff)?, 0),
                ]
                .into_boxed_slice(),
            )),
            set_counter(AUDIENCE_DIE, INITIAL_APPLIED_KEY, 1),
            set_counter(CONTENT, active_maze_buff_key(maze_buff)?, 1),
            set_counter(
                AUDIENCE_DIE,
                PASSIVE_ACTIVE_KEY,
                definition.passive_kind.code(),
            ),
        ];
        program(
            INITIALIZATION_PROGRAM_BASE
                .checked_add(definition.path_id)
                .ok_or_else(|| invalid("Swarm Audience program ID overflow"))?,
            operations,
        )
    }

    pub(super) fn initialization_applied(
        &self,
        state: &ActivityTransactionState,
    ) -> Result<bool, UniverseCatalogLoadError> {
        require_metadata(state, &self.definition)?;
        Ok(
            counter_value(state, AUDIENCE_DIE, INITIAL_APPLIED_KEY)? == 1
                && counter_value(state, AUDIENCE_DIE, PASSIVE_ACTIVE_KEY)?
                    == self.definition.passive_kind.code(),
        )
    }

    pub(super) fn path_sort(&self) -> u16 {
        self.definition.sort
    }

    pub(super) fn unlock_id(&self) -> Option<&str> {
        self.definition.unlock.id()
    }

    pub(super) fn requires_unlock(&self) -> bool {
        matches!(self.definition.unlock, RuntimeUnlock::Required { .. })
    }

    pub(super) fn faces(&self) -> impl ExactSizeIterator<Item = &str> {
        self.definition.faces.iter().map(|face| face.key.as_ref())
    }

    pub(super) fn face_ids(&self) -> impl ExactSizeIterator<Item = (&str, u32)> {
        self.definition
            .faces
            .iter()
            .map(|face| (face.key.as_ref(), face.id))
    }

    pub(super) fn face_key(&self, id: u32) -> Option<&str> {
        self.definition
            .faces
            .iter()
            .find(|face| face.id == id)
            .map(|face| face.key.as_ref())
    }

    pub(super) fn initial_rule(&self) -> &str {
        &self.definition.initial.operation
    }

    pub(super) fn initial_parameters(&self) -> impl ExactSizeIterator<Item = &str> {
        self.definition.initial.parameters.iter().map(AsRef::as_ref)
    }

    pub(super) fn initial_secondary_parameters(&self) -> impl ExactSizeIterator<Item = &str> {
        self.definition
            .initial
            .secondary_parameters
            .iter()
            .map(AsRef::as_ref)
    }

    pub(super) fn passive_rule(&self) -> &str {
        &self.definition.passive.operation
    }

    pub(super) fn passive_parameters(&self) -> impl ExactSizeIterator<Item = &str> {
        self.definition.passive.parameters.iter().map(AsRef::as_ref)
    }

    pub(super) fn passive_secondary_parameters(&self) -> impl ExactSizeIterator<Item = &str> {
        self.definition
            .passive
            .secondary_parameters
            .iter()
            .map(AsRef::as_ref)
    }
}

fn compile_definition(
    path: &SwarmAudiencePathRuntimeInput,
    dice: &BTreeMap<u32, &SwarmAudienceDieRuntimeInput>,
    faces: &BTreeMap<&str, &SwarmAudienceFaceRuntimeInput>,
    rarities: &BTreeMap<u32, u8>,
) -> Result<RuntimeAudienceDefinition, UniverseCatalogLoadError> {
    let die = dice
        .get(&path.die_id)
        .copied()
        .ok_or_else(|| invalid("Swarm Audience Path references unknown Die"))?;
    let path_source = positive_u32(&path.source_id)?;
    let die_source = positive_u32(&die.source_id)?;
    if path_source != die_source
        || die.path_id != path.id
        || die.shared_path != path.shared_path
        || path.key.is_empty()
        || die.key.is_empty()
    {
        return Err(invalid("Swarm Audience Path and Die identity mismatch"));
    }
    let unlock = unlock(path, die)?;
    let initial = decode_effect(&path.initial_program, "RunStart")?;
    let passive = decode_effect(&path.passive_program, "AcceptedActivityOperation")?;
    if initial.operation.as_ref() != "AddMazeBuff"
        || initial.parameters.len() != 1
        || initial.secondary_parameters.len() != 1
        || initial.secondary_parameters[0].as_ref() != "0"
    {
        return Err(invalid("invalid Swarm Audience initial effect"));
    }
    positive_u32(&initial.parameters[0])?;
    let passive_kind = passive_kind(&passive.operation)?;
    let mut description_parameters = die.initial_effect_parameters.to_vec();
    description_parameters.extend(die.passive_description_parameters.iter().cloned());
    if description_parameters != path.description_parameters.as_ref()
        || die.extra_effect_refs != path.extra_effect_refs
    {
        return Err(invalid("Swarm Audience description parameter drift"));
    }
    validate_roll_policy(&die.roll_policy)?;
    let runtime_faces = die
        .face_keys
        .iter()
        .map(|key| {
            let face = faces
                .get(key.as_ref())
                .copied()
                .ok_or_else(|| invalid("unknown Swarm Audience Die face"))?;
            if face.die_id != die.id {
                return Err(invalid("Swarm Audience Die face ownership mismatch"));
            }
            let rarity_rank = rarities
                .get(&face.rarity_id)
                .copied()
                .ok_or_else(|| invalid("unknown Swarm Audience face rarity"))?;
            Ok((
                face.sort,
                face.id,
                RuntimeFace {
                    id: face.id,
                    key: face.key.clone(),
                    rarity_rank,
                },
            ))
        })
        .collect::<Result<Vec<_>, UniverseCatalogLoadError>>()?;
    if runtime_faces
        .iter()
        .map(|(_, id, _)| *id)
        .collect::<BTreeSet<_>>()
        .len()
        != runtime_faces.len()
        || !runtime_faces
            .windows(2)
            .all(|pair| (pair[0].0, pair[0].1) < (pair[1].0, pair[1].1))
    {
        return Err(invalid("Swarm Audience face order or membership drift"));
    }
    Ok(RuntimeAudienceDefinition {
        path_id: path.id,
        _path_key: path.key.clone(),
        die_id: die.id,
        die_key: die.key.clone(),
        shared_path: path.shared_path.clone(),
        sort: path.sort,
        unlock,
        faces: runtime_faces
            .into_iter()
            .map(|(_, _, face)| face)
            .collect::<Vec<_>>()
            .into_boxed_slice(),
        initial,
        passive,
        passive_kind,
        _description_parameters: path.description_parameters.clone(),
        _rogue_buff_type: positive_u32(&path.rogue_buff_type)?,
        _battle_event_buff_group: positive_u32(&path.battle_event_buff_group)?,
        _battle_event_enhance_buff_group: positive_u32(&path.battle_event_enhance_buff_group)?,
        _extra_effect_refs: path.extra_effect_refs.clone(),
    })
}

fn rarity_ranks(
    input: &SwarmAudienceRuntimeInput,
) -> Result<BTreeMap<u32, u8>, UniverseCatalogLoadError> {
    let ranks = input
        .rarities
        .iter()
        .map(|rarity| (rarity.id, rarity.rank))
        .collect::<BTreeMap<_, _>>();
    if ranks.len() != 3
        || input.rarities.iter().any(|rarity| rarity.key.is_empty())
        || ranks.values().copied().collect::<BTreeSet<_>>() != BTreeSet::from([1, 2, 3])
    {
        return Err(invalid("Swarm Audience rarity denominator drift"));
    }
    Ok(ranks)
}

fn face_inputs(
    input: &SwarmAudienceRuntimeInput,
) -> Result<BTreeMap<&str, &SwarmAudienceFaceRuntimeInput>, UniverseCatalogLoadError> {
    let faces = input
        .faces
        .iter()
        .map(|face| (face.key.as_ref(), face))
        .collect::<BTreeMap<_, _>>();
    if faces.len() != 42
        || input
            .faces
            .iter()
            .map(|face| face.id)
            .collect::<BTreeSet<_>>()
            .len()
            != 42
    {
        return Err(invalid("Swarm Audience face denominator drift"));
    }
    Ok(faces)
}

fn unlock(
    path: &SwarmAudiencePathRuntimeInput,
    die: &SwarmAudienceDieRuntimeInput,
) -> Result<RuntimeUnlock, UniverseCatalogLoadError> {
    if path.unlock_id != die.unlock_id {
        return Err(invalid("Swarm Audience unlock reference mismatch"));
    }
    let policy = serde_json::from_str::<String>(&path.unlock_policy)
        .map_err(|_| invalid("invalid Swarm Audience unlock policy"))?;
    match (policy.as_str(), path.unlock_id.as_deref()) {
        ("RequireAuthoredUnlockId", Some(id)) => Ok(RuntimeUnlock::Required {
            id: id.into(),
            numeric_id: positive_u32(id)?,
        }),
        ("AvailableWithoutAuthoredUnlock", None) => Ok(RuntimeUnlock::Available),
        _ => Err(invalid("inconsistent Swarm Audience unlock policy")),
    }
}

fn decode_effect(value: &str, boundary: &str) -> Result<RuntimeEffect, UniverseCatalogLoadError> {
    let decoded = serde_json::from_str::<Vec<EffectInput>>(value)
        .map_err(|_| invalid("invalid Swarm Audience effect program"))?;
    let [effect] = decoded.as_slice() else {
        return Err(invalid(
            "Swarm Audience effect program must contain one row",
        ));
    };
    if effect.order != 0 || effect.application_boundary != boundary || effect.operation.is_empty() {
        return Err(invalid("invalid Swarm Audience effect boundary"));
    }
    for parameter in effect
        .parameters
        .iter()
        .chain(effect.secondary_parameters.iter())
    {
        canonical_scalar(parameter)?;
    }
    Ok(RuntimeEffect {
        operation: effect.operation.clone().into_boxed_str(),
        parameters: boxed(&effect.parameters),
        secondary_parameters: boxed(&effect.secondary_parameters),
    })
}

fn passive_kind(value: &str) -> Result<PassiveKind, UniverseCatalogLoadError> {
    match value {
        "ProtectCellNoCollapse" => Ok(PassiveKind::ProtectCellNoCollapse),
        "ExtraMoneyAndRandomSwap" => Ok(PassiveKind::ExtraMoneyAndRandomSwap),
        "ReRandomEmptyCell" => Ok(PassiveKind::ReRandomEmptyCell),
        "FertileAeonGain" => Ok(PassiveKind::FertileAeonGain),
        "ExtraMarkAndRandomSwap" => Ok(PassiveKind::ExtraMarkAndRandomSwap),
        "DestroyAeonGain" => Ok(PassiveKind::DestroyAeonGain),
        "GetHelpOnEnterCell" => Ok(PassiveKind::GetHelpOnEnterCell),
        "RandomGenSwarm" => Ok(PassiveKind::RandomGenSwarm),
        _ => Err(invalid("unknown Swarm Audience passive rule")),
    }
}

fn validate_roll_policy(value: &str) -> Result<(), UniverseCatalogLoadError> {
    let policy = serde_json::from_str::<RollPolicyInput>(value)
        .map_err(|_| invalid("invalid Swarm Audience roll policy"))?;
    if policy.candidate_order != "AuthoredSortThenStableFaceId"
        || policy.control_rule_source != "G09-P1-B5"
        || policy.empty_face_set != "Reject"
    {
        return Err(invalid("unsupported Swarm Audience roll policy"));
    }
    Ok(())
}

fn require_metadata(
    state: &ActivityTransactionState,
    definition: &RuntimeAudienceDefinition,
) -> Result<(), UniverseCatalogLoadError> {
    let expected = [
        (SELECTED_DIE_KEY, i64::from(definition.die_id)),
        (SELECTED_PATH_KEY, i64::from(definition.path_id)),
        (PATH_SORT_KEY, i64::from(definition.sort)),
        (UNLOCK_POLICY_KEY, definition.unlock.policy_code()),
        (UNLOCK_ID_KEY, i64::from(definition.unlock.numeric_id())),
        (PASSIVE_KIND_KEY, definition.passive_kind.code()),
        (UNLOCK_AUTHORIZED_KEY, 1),
    ];
    if expected
        .iter()
        .any(|(key, value)| counter_value(state, AUDIENCE_DIE, *key) != Ok(*value))
    {
        return Err(invalid("Swarm Audience state metadata mismatch"));
    }
    Ok(())
}

fn initial_maze_buff(
    definition: &RuntimeAudienceDefinition,
) -> Result<u32, UniverseCatalogLoadError> {
    definition
        .initial
        .parameters
        .first()
        .ok_or_else(|| invalid("missing Swarm Audience maze buff"))
        .and_then(|value| positive_u32(value))
}

fn face_key(id: u32) -> Result<u64, UniverseCatalogLoadError> {
    FACE_RARITY_KEY_BASE
        .checked_add(u64::from(id))
        .ok_or_else(|| invalid("Swarm Audience face key overflow"))
}

fn active_maze_buff_key(id: u32) -> Result<u64, UniverseCatalogLoadError> {
    ACTIVE_MAZE_BUFF_KEY_BASE
        .checked_add(u64::from(id))
        .ok_or_else(|| invalid("Swarm Audience maze-buff key overflow"))
}

fn equal(slot_id: u32, key: u64, value: i64) -> ActivityCondition {
    ActivityCondition::Equal(counter(slot_id, key), integer(value))
}

fn set_counter(slot_id: u32, key: u64, desired: i64) -> ActivityOperation {
    ActivityOperation::AddCounter {
        slot: slot(slot_id),
        key,
        delta: ActivityExpression::Subtract(
            Box::new(integer(desired)),
            Box::new(counter(slot_id, key)),
        ),
    }
}

fn counter(slot_id: u32, key: u64) -> ActivityExpression {
    ActivityExpression::CounterValue {
        slot: slot(slot_id),
        key,
    }
}

fn counter_value(
    state: &ActivityTransactionState,
    slot_id: u32,
    key: u64,
) -> Result<i64, UniverseCatalogLoadError> {
    match state.slot(slot(slot_id)) {
        Some(ActivityValue::BoundedCounterMap(values)) => Ok(values
            .binary_search_by_key(&key, |(candidate, _)| *candidate)
            .ok()
            .map_or(0, |index| values[index].1)),
        _ => Err(invalid("invalid Swarm Audience state slot")),
    }
}

fn integer(value: i64) -> ActivityExpression {
    ActivityExpression::Literal(ActivityValue::BoundedInteger(value))
}

fn positive_u32(value: &str) -> Result<u32, UniverseCatalogLoadError> {
    canonical_scalar(value)?;
    value
        .parse::<u32>()
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| invalid("invalid positive Swarm Audience integer"))
}

fn canonical_scalar(value: &str) -> Result<(), UniverseCatalogLoadError> {
    let unsigned = value.strip_prefix('-').unwrap_or(value);
    let (whole, fraction) = unsigned
        .split_once('.')
        .map_or((unsigned, None), |(whole, fraction)| {
            (whole, Some(fraction))
        });
    let whole_valid = whole == "0"
        || (!whole.starts_with('0') && whole.bytes().all(|byte| byte.is_ascii_digit()));
    let fraction_valid = fraction.is_none_or(|fraction| {
        !fraction.is_empty()
            && !fraction.ends_with('0')
            && fraction.bytes().all(|byte| byte.is_ascii_digit())
    });
    if value.is_empty()
        || value == "-0"
        || value.starts_with('+')
        || !whole_valid
        || !fraction_valid
    {
        return Err(invalid("invalid canonical Swarm Audience scalar"));
    }
    Ok(())
}

fn boxed(values: &[String]) -> Box<[Box<str>]> {
    values
        .iter()
        .map(|value| value.clone().into_boxed_str())
        .collect::<Vec<_>>()
        .into_boxed_slice()
}

fn slot(raw: u32) -> ActivitySlotId {
    ActivitySlotId::new(raw).expect("static Swarm slot is non-zero")
}

fn program(
    raw: u32,
    operations: Vec<ActivityOperation>,
) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
    ActivityProgramDefinition::new(
        ActivityProgramId::new(raw).expect("static Swarm program ID is non-zero"),
        operations,
    )
    .map_err(|_| invalid("invalid Swarm Audience program"))
}

fn invalid(message: &'static str) -> UniverseCatalogLoadError {
    UniverseCatalogLoadError::new(UniverseCatalogLoadErrorKind::InvalidDefinition, message)
}

#[derive(Deserialize)]
struct EffectInput {
    order: u16,
    operation: String,
    parameters: Vec<String>,
    secondary_parameters: Vec<String>,
    application_boundary: String,
}

#[derive(Deserialize)]
struct RollPolicyInput {
    candidate_order: String,
    control_rule_source: String,
    empty_face_set: String,
}
