//! Audience Die roll controls and atomic resolution-state compilation.

use std::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;
use starclock_activity::{
    ActivityCondition, ActivityExpression, ActivityOperation, ActivityProgramDefinition,
    ActivityProgramId, ActivityRngLabel, ActivityRngStreams, ActivitySlotId,
    ActivityTransactionState, ActivityValue,
};

use crate::{
    error::{UniverseCatalogLoadError, UniverseCatalogLoadErrorKind},
    swarm_disaster_unique::runtime_access::SwarmDiceControlRuntimeInput,
};

use super::state::{AUDIENCE_DIE, DICE_RESOLUTION, RESOURCES};

pub(super) const ROLL_PURPOSE: u16 = 0x5321;
pub(super) const REROLL_PURPOSE: u16 = 0x5322;

const PROGRAM_BASE: u32 = 0x5320_0000;
const COSMIC_FRAGMENTS_KEY: u64 = 1;
pub(super) const REROLL_CHARGE_KEY: u64 = 2;
pub(super) const CHEAT_CHARGE_KEY: u64 = 3;
const ABANDON_AUTHORIZED_KEY: u64 = 12;

const SELECTED_FACE_KEY: u64 = 1;
const PREVIOUS_FACE_KEY: u64 = 2;
const RESOLUTION_KIND_KEY: u64 = 3;
const CANDIDATE_COUNT_KEY: u64 = 4;
const DRAW_INDEX_KEY: u64 = 5;
const PHASE_CLOSED_KEY: u64 = 6;

const RESOLUTION_ROLL: i64 = 1;
const RESOLUTION_REROLL: i64 = 2;
const RESOLUTION_CHEAT: i64 = 3;
const RESOLUTION_ABANDON: i64 = 4;

#[derive(Clone, Debug)]
pub(super) struct DiceControlRuntimeCatalog {
    controls: BTreeMap<ControlKind, RuntimeControl>,
    abandon_unlock: Box<str>,
}

#[derive(Clone, Debug)]
struct RuntimeControl {
    id: u32,
    _key: Box<str>,
    kind: ControlKind,
    cost: ResourceCost,
    _fallback: FallbackPolicy,
    abandon_reward: i64,
    unlock_id: Option<Box<str>>,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum ControlKind {
    Abandon,
    Cheat,
    Reroll,
    Roll,
}

impl ControlKind {
    const fn resolution(self) -> i64 {
        match self {
            Self::Roll => RESOLUTION_ROLL,
            Self::Reroll => RESOLUTION_REROLL,
            Self::Cheat => RESOLUTION_CHEAT,
            Self::Abandon => RESOLUTION_ABANDON,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ResourceKind {
    None,
    SelectedFace,
    RerollCharge,
    CheatCharge,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ResourceCost {
    resource: ResourceKind,
    amount: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FallbackPolicy {
    EmptyFaceSet,
    InsufficientCharge,
    InsufficientChargeOrInvalidFace,
    WithoutSelectedFace,
}

#[derive(Clone, Debug)]
pub(super) struct CompiledDiceControls {
    controls: BTreeMap<ControlKind, RuntimeControl>,
    abandon_authorized: bool,
}

impl DiceControlRuntimeCatalog {
    pub(super) fn compile(
        input: &[SwarmDiceControlRuntimeInput],
    ) -> Result<Self, UniverseCatalogLoadError> {
        let controls = input
            .iter()
            .map(compile_control)
            .collect::<Result<BTreeMap<_, _>, _>>()?;
        let expected = BTreeSet::from([
            ControlKind::Abandon,
            ControlKind::Cheat,
            ControlKind::Reroll,
            ControlKind::Roll,
        ]);
        if input.len() != 4
            || controls.len() != 4
            || controls.keys().copied().collect::<BTreeSet<_>>() != expected
        {
            return Err(invalid("Swarm dice-control denominator drift"));
        }
        let abandon = controls
            .get(&ControlKind::Abandon)
            .ok_or_else(|| invalid("missing Swarm abandon control"))?;
        let abandon_unlock = abandon
            .unlock_id
            .clone()
            .ok_or_else(|| invalid("missing Swarm abandon unlock"))?;
        Ok(Self {
            controls,
            abandon_unlock,
        })
    }

    pub(super) fn select(
        &self,
        unlocks: &[Box<str>],
    ) -> Result<CompiledDiceControls, UniverseCatalogLoadError> {
        let provided = unlocks
            .iter()
            .map(AsRef::as_ref)
            .collect::<BTreeSet<&str>>();
        if provided.len() != unlocks.len()
            || provided
                .iter()
                .any(|unlock| *unlock != self.abandon_unlock.as_ref())
        {
            return Err(reference("unknown or duplicate Swarm dice-control unlock"));
        }
        Ok(CompiledDiceControls {
            controls: self.controls.clone(),
            abandon_authorized: provided.contains(self.abandon_unlock.as_ref()),
        })
    }

    #[cfg(test)]
    pub(super) fn denominators(&self) -> (usize, &str) {
        (self.controls.len(), &self.abandon_unlock)
    }
}

impl CompiledDiceControls {
    pub(super) fn state_values(&self) -> Box<[(u64, i64)]> {
        if self.abandon_authorized {
            Box::new([(ABANDON_AUTHORIZED_KEY, 1)])
        } else {
            Box::new([])
        }
    }

    pub(super) fn compile_roll(
        &self,
        state: &ActivityTransactionState,
        faces: &[(&str, u32)],
        rng: &mut ActivityRngStreams,
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        require_open_without_face(state)?;
        self.choose_face(
            ControlKind::Roll,
            state,
            faces,
            ROLL_PURPOSE,
            Vec::new(),
            rng,
        )
    }

    pub(super) fn compile_reroll(
        &self,
        state: &ActivityTransactionState,
        faces: &[(&str, u32)],
        rng: &mut ActivityRngStreams,
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        require_open_selected_face(state, faces)?;
        let control = self.control(ControlKind::Reroll)?;
        require_charge(state, control.cost)?;
        self.choose_face(
            ControlKind::Reroll,
            state,
            faces,
            REROLL_PURPOSE,
            charge_operations(control.cost),
            rng,
        )
    }

    pub(super) fn compile_cheat(
        &self,
        state: &ActivityTransactionState,
        faces: &[(&str, u32)],
        selected: &str,
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        let previous = require_open_selected_face(state, faces)?;
        let control = self.control(ControlKind::Cheat)?;
        require_charge(state, control.cost)?;
        let selected = faces
            .iter()
            .find(|(key, _)| *key == selected)
            .map(|(_, id)| *id)
            .ok_or_else(|| reference("Swarm cheat face is not in the selected Die"))?;
        resolution_program(
            state,
            ResolutionSpec {
                control,
                selected,
                previous,
                candidate_count: candidate_count(faces)?,
                draw_index: 0,
                closed: 0,
            },
            charge_operations(control.cost),
        )
    }

    pub(super) fn compile_abandon(
        &self,
        state: &ActivityTransactionState,
        faces: &[(&str, u32)],
        reward_bonus: i64,
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        let previous = require_open_selected_face(state, faces)?;
        let control = self.control(ControlKind::Abandon)?;
        if !self.abandon_authorized
            || counter_value(state, AUDIENCE_DIE, ABANDON_AUTHORIZED_KEY)? != 1
        {
            return Err(reference("Swarm abandon control is not unlocked"));
        }
        let fragments = counter_value(state, RESOURCES, COSMIC_FRAGMENTS_KEY)?;
        let reward = control
            .abandon_reward
            .checked_add(reward_bonus)
            .filter(|value| *value >= 0)
            .ok_or_else(|| invalid("Swarm abandon reward contribution is invalid"))?;
        let next_fragments = fragments
            .checked_add(reward)
            .filter(|value| *value <= 1_000_000_000)
            .ok_or_else(|| invalid("Swarm abandon reward exceeds resource bounds"))?;
        resolution_program(
            state,
            ResolutionSpec {
                control,
                selected: 0,
                previous,
                candidate_count: candidate_count(faces)?,
                draw_index: 0,
                closed: 1,
            },
            vec![
                require_counter(RESOURCES, COSMIC_FRAGMENTS_KEY, fragments),
                set_counter(RESOURCES, COSMIC_FRAGMENTS_KEY, next_fragments),
            ],
        )
    }

    pub(super) fn resolution_face_id(&self, state: &ActivityTransactionState) -> Option<u32> {
        let selected = counter_value(state, DICE_RESOLUTION, SELECTED_FACE_KEY).ok()?;
        u32::try_from(selected).ok().filter(|id| *id != 0)
    }

    pub(super) fn resolution_kind(
        &self,
        state: &ActivityTransactionState,
    ) -> Result<Option<u8>, UniverseCatalogLoadError> {
        let value = counter_value(state, DICE_RESOLUTION, RESOLUTION_KIND_KEY)?;
        match value {
            0 => Ok(None),
            RESOLUTION_ROLL..=RESOLUTION_ABANDON => Ok(Some(
                u8::try_from(value).expect("bounded Swarm resolution code fits u8"),
            )),
            _ => Err(invalid("invalid Swarm dice resolution kind")),
        }
    }

    pub(super) fn activation_prefix(
        &self,
        state: &ActivityTransactionState,
        expected_face: u32,
    ) -> Result<Vec<ActivityOperation>, UniverseCatalogLoadError> {
        let selected = counter_value(state, DICE_RESOLUTION, SELECTED_FACE_KEY)?;
        let closed = counter_value(state, DICE_RESOLUTION, PHASE_CLOSED_KEY)?;
        if selected != i64::from(expected_face) || closed != 0 {
            return Err(reference("Swarm dice face is not available for activation"));
        }
        Ok(vec![
            require_counter(DICE_RESOLUTION, SELECTED_FACE_KEY, selected),
            require_counter(DICE_RESOLUTION, PHASE_CLOSED_KEY, 0),
            set_counter(DICE_RESOLUTION, PHASE_CLOSED_KEY, 1),
        ])
    }

    pub(super) fn roll_available(
        &self,
        state: &ActivityTransactionState,
        faces: &[(&str, u32)],
    ) -> Result<bool, UniverseCatalogLoadError> {
        self.control_available(state, faces, ControlKind::Roll)
    }

    pub(super) fn reroll_available(
        &self,
        state: &ActivityTransactionState,
        faces: &[(&str, u32)],
    ) -> Result<bool, UniverseCatalogLoadError> {
        self.control_available(state, faces, ControlKind::Reroll)
    }

    pub(super) fn cheat_available(
        &self,
        state: &ActivityTransactionState,
        faces: &[(&str, u32)],
    ) -> Result<bool, UniverseCatalogLoadError> {
        self.control_available(state, faces, ControlKind::Cheat)
    }

    pub(super) fn abandon_available(
        &self,
        state: &ActivityTransactionState,
        faces: &[(&str, u32)],
    ) -> Result<bool, UniverseCatalogLoadError> {
        self.control_available(state, faces, ControlKind::Abandon)
    }

    fn control_available(
        &self,
        state: &ActivityTransactionState,
        faces: &[(&str, u32)],
        kind: ControlKind,
    ) -> Result<bool, UniverseCatalogLoadError> {
        let face = counter_value(state, DICE_RESOLUTION, SELECTED_FACE_KEY)?;
        let closed = counter_value(state, DICE_RESOLUTION, PHASE_CLOSED_KEY)?;
        if face > 0 && !faces.iter().any(|(_, id)| i64::from(*id) == face) {
            return Err(invalid(
                "selected Swarm dice face is outside the authored Die",
            ));
        }
        if closed != 0 {
            return Ok(false);
        }
        match kind {
            ControlKind::Roll => Ok(face == 0),
            ControlKind::Reroll => Ok(face > 0 && charge(state, REROLL_CHARGE_KEY)? > 0),
            ControlKind::Cheat => Ok(face > 0 && charge(state, CHEAT_CHARGE_KEY)? > 0),
            ControlKind::Abandon => Ok(face > 0
                && self.abandon_authorized
                && counter_value(state, AUDIENCE_DIE, ABANDON_AUTHORIZED_KEY)? == 1),
        }
    }

    fn choose_face(
        &self,
        kind: ControlKind,
        state: &ActivityTransactionState,
        faces: &[(&str, u32)],
        purpose: u16,
        prefix: Vec<ActivityOperation>,
        rng: &mut ActivityRngStreams,
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        let control = self.control(kind)?;
        let count = candidate_count(faces)?;
        if count == 0 {
            return Err(reference("Swarm Audience Die has no roll candidates"));
        }
        let previous = counter_value(state, DICE_RESOLUTION, SELECTED_FACE_KEY)?;
        let previous =
            u32::try_from(previous).map_err(|_| invalid("invalid prior Swarm dice face"))?;
        rng.transact(|working| {
            let draw = working
                .choose_index(ActivityRngLabel::Spawn, purpose, count)
                .map_err(|_| invalid("Swarm dice RNG failure"))?
                .ok_or_else(|| invalid("missing Swarm dice RNG draw"))?;
            let selected = faces
                .get(draw.value() as usize)
                .map(|(_, id)| *id)
                .ok_or_else(|| invalid("Swarm dice draw mapping failure"))?;
            let draw_index = draw
                .index()
                .checked_add(1)
                .ok_or_else(|| invalid("Swarm dice draw index overflow"))?;
            resolution_program(
                state,
                ResolutionSpec {
                    control,
                    selected,
                    previous,
                    candidate_count: count,
                    draw_index,
                    closed: 0,
                },
                prefix,
            )
        })
    }

    fn control(&self, kind: ControlKind) -> Result<&RuntimeControl, UniverseCatalogLoadError> {
        self.controls
            .get(&kind)
            .ok_or_else(|| invalid("missing compiled Swarm dice control"))
    }
}

fn compile_control(
    input: &SwarmDiceControlRuntimeInput,
) -> Result<(ControlKind, RuntimeControl), UniverseCatalogLoadError> {
    let kind = control_kind(&input.operation)?;
    let cost = decode_cost(&input.resource_cost)?;
    let fallback = decode_fallback(&input.fallback_policy)?;
    let abandon_reward = decode_scalar(&input.abandon_reward)?;
    let expected = match kind {
        ControlKind::Abandon => (
            1,
            "swarm-disaster.dice-control.abandon",
            ResourceKind::SelectedFace,
            1,
            FallbackPolicy::WithoutSelectedFace,
            10,
            Some("1000022"),
        ),
        ControlKind::Cheat => (
            2,
            "swarm-disaster.dice-control.cheat",
            ResourceKind::CheatCharge,
            1,
            FallbackPolicy::InsufficientChargeOrInvalidFace,
            0,
            None,
        ),
        ControlKind::Reroll => (
            3,
            "swarm-disaster.dice-control.reroll",
            ResourceKind::RerollCharge,
            1,
            FallbackPolicy::InsufficientCharge,
            0,
            None,
        ),
        ControlKind::Roll => (
            4,
            "swarm-disaster.dice-control.roll",
            ResourceKind::None,
            0,
            FallbackPolicy::EmptyFaceSet,
            0,
            None,
        ),
    };
    if input.id != expected.0
        || input.key.as_ref() != expected.1
        || input.result_order.as_ref() != "AuthoredSortThenStableFaceId"
        || cost.resource != expected.2
        || cost.amount != expected.3
        || fallback != expected.4
        || abandon_reward != expected.5
        || input.unlock_id.as_deref() != expected.6
    {
        return Err(invalid("Swarm dice-control policy drift"));
    }
    Ok((
        kind,
        RuntimeControl {
            id: input.id,
            _key: input.key.clone(),
            kind,
            cost,
            _fallback: fallback,
            abandon_reward,
            unlock_id: input.unlock_id.clone(),
        },
    ))
}

fn control_kind(value: &str) -> Result<ControlKind, UniverseCatalogLoadError> {
    match value {
        "Abandon" => Ok(ControlKind::Abandon),
        "Cheat" => Ok(ControlKind::Cheat),
        "Reroll" => Ok(ControlKind::Reroll),
        "Roll" => Ok(ControlKind::Roll),
        _ => Err(invalid("unknown Swarm dice control")),
    }
}

fn decode_cost(value: &str) -> Result<ResourceCost, UniverseCatalogLoadError> {
    let input = serde_json::from_str::<ResourceCostInput>(value)
        .map_err(|_| invalid("invalid Swarm dice resource cost"))?;
    let resource = match input.resource.as_str() {
        "None" => ResourceKind::None,
        "SelectedFace" => ResourceKind::SelectedFace,
        "RerollCharge" => ResourceKind::RerollCharge,
        "CheatCharge" => ResourceKind::CheatCharge,
        _ => return Err(invalid("unknown Swarm dice resource")),
    };
    Ok(ResourceCost {
        resource,
        amount: decode_integer(&input.amount)?,
    })
}

fn decode_fallback(value: &str) -> Result<FallbackPolicy, UniverseCatalogLoadError> {
    let input = serde_json::from_str::<String>(value)
        .map_err(|_| invalid("invalid Swarm dice fallback"))?;
    match input.as_str() {
        "RejectEmptyFaceSet" => Ok(FallbackPolicy::EmptyFaceSet),
        "RejectInsufficientCharge" => Ok(FallbackPolicy::InsufficientCharge),
        "RejectInsufficientChargeOrInvalidFace" => {
            Ok(FallbackPolicy::InsufficientChargeOrInvalidFace)
        }
        "RejectWithoutSelectedFace" => Ok(FallbackPolicy::WithoutSelectedFace),
        _ => Err(invalid("unknown Swarm dice fallback")),
    }
}

fn decode_scalar(value: &str) -> Result<i64, UniverseCatalogLoadError> {
    let input =
        serde_json::from_str::<String>(value).map_err(|_| invalid("invalid Swarm dice scalar"))?;
    decode_integer(&input)
}

fn decode_integer(value: &str) -> Result<i64, UniverseCatalogLoadError> {
    let unsigned = value.strip_prefix('-').unwrap_or(value);
    let canonical = unsigned == "0"
        || (!unsigned.starts_with('0') && unsigned.bytes().all(|byte| byte.is_ascii_digit()));
    if value.is_empty() || value.starts_with('+') || value == "-0" || !canonical {
        return Err(invalid("non-canonical Swarm dice integer"));
    }
    value
        .parse::<i64>()
        .map_err(|_| invalid("invalid Swarm dice integer"))
}

fn require_open_without_face(
    state: &ActivityTransactionState,
) -> Result<(), UniverseCatalogLoadError> {
    if counter_value(state, DICE_RESOLUTION, SELECTED_FACE_KEY)? != 0
        || counter_value(state, DICE_RESOLUTION, PHASE_CLOSED_KEY)? != 0
    {
        return Err(reference("Swarm dice roll is not currently offered"));
    }
    Ok(())
}

fn require_open_selected_face(
    state: &ActivityTransactionState,
    faces: &[(&str, u32)],
) -> Result<u32, UniverseCatalogLoadError> {
    let selected = counter_value(state, DICE_RESOLUTION, SELECTED_FACE_KEY)?;
    let selected = u32::try_from(selected)
        .ok()
        .filter(|selected| *selected != 0)
        .ok_or_else(|| reference("Swarm dice control requires a selected face"))?;
    if counter_value(state, DICE_RESOLUTION, PHASE_CLOSED_KEY)? != 0
        || !faces.iter().any(|(_, id)| *id == selected)
    {
        return Err(reference("Swarm dice control is not currently offered"));
    }
    Ok(selected)
}

fn require_charge(
    state: &ActivityTransactionState,
    cost: ResourceCost,
) -> Result<(), UniverseCatalogLoadError> {
    let key = resource_key(cost.resource)
        .ok_or_else(|| invalid("Swarm dice control has no charge resource"))?;
    if charge(state, key)? < cost.amount {
        return Err(reference("insufficient Swarm dice-control charge"));
    }
    Ok(())
}

fn charge(state: &ActivityTransactionState, key: u64) -> Result<i64, UniverseCatalogLoadError> {
    counter_value(state, RESOURCES, key)
}

fn charge_operations(cost: ResourceCost) -> Vec<ActivityOperation> {
    let key = resource_key(cost.resource).expect("validated charged control has a resource key");
    vec![
        ActivityOperation::Require(ActivityCondition::LessThan(
            integer(0),
            counter(RESOURCES, key),
        )),
        ActivityOperation::AddCounter {
            slot: slot(RESOURCES),
            key,
            delta: integer(-cost.amount),
        },
    ]
}

const fn resource_key(resource: ResourceKind) -> Option<u64> {
    match resource {
        ResourceKind::RerollCharge => Some(REROLL_CHARGE_KEY),
        ResourceKind::CheatCharge => Some(CHEAT_CHARGE_KEY),
        ResourceKind::None | ResourceKind::SelectedFace => None,
    }
}

fn candidate_count(faces: &[(&str, u32)]) -> Result<u32, UniverseCatalogLoadError> {
    let count =
        u32::try_from(faces.len()).map_err(|_| invalid("too many Swarm Audience Die faces"))?;
    if faces
        .iter()
        .map(|(_, id)| *id)
        .collect::<BTreeSet<_>>()
        .len()
        != faces.len()
    {
        return Err(invalid("duplicate Swarm Audience Die face"));
    }
    Ok(count)
}

fn resolution_program(
    state: &ActivityTransactionState,
    spec: ResolutionSpec<'_>,
    mut prefix: Vec<ActivityOperation>,
) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
    let next = [
        (SELECTED_FACE_KEY, i64::from(spec.selected)),
        (PREVIOUS_FACE_KEY, i64::from(spec.previous)),
        (RESOLUTION_KIND_KEY, spec.control.kind.resolution()),
        (CANDIDATE_COUNT_KEY, i64::from(spec.candidate_count)),
        (
            DRAW_INDEX_KEY,
            i64::try_from(spec.draw_index).map_err(|_| invalid("Swarm dice draw overflow"))?,
        ),
        (PHASE_CLOSED_KEY, spec.closed),
    ];
    for (key, desired) in next {
        let current = counter_value(state, DICE_RESOLUTION, key)?;
        prefix.push(require_counter(DICE_RESOLUTION, key, current));
        prefix.push(set_counter(DICE_RESOLUTION, key, desired));
    }
    ActivityProgramDefinition::new(
        ActivityProgramId::new(
            PROGRAM_BASE
                .checked_add(spec.control.id)
                .ok_or_else(|| invalid("Swarm dice program ID overflow"))?,
        )
        .ok_or_else(|| invalid("invalid Swarm dice program ID"))?,
        prefix,
    )
    .map_err(|_| invalid("invalid Swarm dice control program"))
}

fn require_counter(slot_id: u32, key: u64, value: i64) -> ActivityOperation {
    ActivityOperation::Require(ActivityCondition::Equal(
        counter(slot_id, key),
        integer(value),
    ))
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
        _ => Err(invalid("invalid Swarm dice-control state slot")),
    }
}

fn integer(value: i64) -> ActivityExpression {
    ActivityExpression::Literal(ActivityValue::BoundedInteger(value))
}

fn slot(raw: u32) -> ActivitySlotId {
    ActivitySlotId::new(raw).expect("static Swarm slot is non-zero")
}

fn invalid(message: &'static str) -> UniverseCatalogLoadError {
    UniverseCatalogLoadError::new(UniverseCatalogLoadErrorKind::InvalidDefinition, message)
}

fn reference(message: &'static str) -> UniverseCatalogLoadError {
    UniverseCatalogLoadError::new(UniverseCatalogLoadErrorKind::InvalidReference, message)
}

struct ResolutionSpec<'a> {
    control: &'a RuntimeControl,
    selected: u32,
    previous: u32,
    candidate_count: u32,
    draw_index: u64,
    closed: i64,
}

#[derive(Deserialize)]
struct ResourceCostInput {
    resource: String,
    amount: String,
}
