//! Typed execution descriptors for all released Swarm Audience Die faces.

use std::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;
use starclock_activity::{
    ActivityExpression, ActivityOperation, ActivityProgramDefinition, ActivityProgramId,
    ActivityRngLabel, ActivityRngStreams, ActivitySlotId, ActivityTransactionState, ActivityValue,
    NodeId,
};

use crate::{
    error::{UniverseCatalogLoadError, UniverseCatalogLoadErrorKind},
    swarm_disaster_unique::runtime_access::{
        SwarmAudienceFaceRuntimeInput, SwarmDiceTargetRuntimeInput,
    },
};

use super::{
    dice_control::CompiledDiceControls, face_operation::FaceOperation,
    map_overlay::MapRuntimeCatalog, state::DEFERRED,
};

const PROGRAM_BASE: u32 = 0x5350_0000;
pub(super) const TARGET_PURPOSE: u16 = 0x5323;

const USE_BASE: u64 = 0x0010_0000;
const OPERATION_BASE: u64 = 0x0011_0000;
const STAGE_BASE: u64 = 0x0012_0000;
const DURATION_BASE: u64 = 0x0013_0000;
const PARAMETER_COUNT_BASE: u64 = 0x0020_0000;
const PARAMETER_VALUE_BASE: u64 = 0x0021_0000;
const DESCRIPTION_COUNT_BASE: u64 = 0x0030_0000;
const DESCRIPTION_VALUE_BASE: u64 = 0x0031_0000;
const TURN_DURATION_BASE: u64 = 0x0032_0000;
const EFFECT_REFERENCE_BASE: u64 = 0x0040_0000;
const TARGET_BASE: u64 = 0x0500_0000;
const NO_OP_BASE: u64 = 0x0600_0000;
const GRAPH_EFFECT_BASE: u64 = 0x0610_0000;
const BATTLE_CONTRIBUTION_BASE: u64 = 0x0620_0000;
pub(super) const MERCY_TARGET_BASE: u64 = 0x0700_0000;

#[derive(Clone, Debug)]
pub(super) struct DiceFaceRuntimeCatalog {
    faces: Box<[RuntimeDiceFace]>,
}

#[derive(Clone, Debug)]
pub(super) struct RuntimeDiceFace {
    id: u32,
    key: Box<str>,
    activation: FaceActivation,
    duration: EffectDuration,
    target: FaceTargetMode,
    operation: FaceOperation,
    parameters_scaled: Box<[i64]>,
    description_scaled: Box<[i64]>,
    turn_duration: Option<u16>,
    effect_references: Box<[u32]>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FaceActivation {
    Immediate,
    AfterMovement,
    BattleContribution,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum EffectDuration {
    Immediate,
    CurrentMovement,
    AfterMovement,
    NextBattle,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FaceTargetMode {
    Derived,
    Explicit(FaceSelector),
    Random { selector: FaceSelector, maximum: u8 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum FaceSelector {
    Any,
    NonBoss,
    Combat,
    Elite,
    Occurrence,
    CombatSwarmElite,
    CombatSwarm,
    Swarm,
    Boss,
    WithoutMercy,
}

impl DiceFaceRuntimeCatalog {
    pub(super) fn compile(
        faces: &[SwarmAudienceFaceRuntimeInput],
        targets: &[SwarmDiceTargetRuntimeInput],
    ) -> Result<Self, UniverseCatalogLoadError> {
        let targets = targets
            .iter()
            .map(|target| Ok((target.id, compile_target(target)?)))
            .collect::<Result<BTreeMap<_, _>, UniverseCatalogLoadError>>()?;
        let mut compiled = faces
            .iter()
            .map(|face| {
                let target = targets
                    .get(&face.target_id)
                    .ok_or_else(|| invalid("unknown Swarm dice-face target"))?;
                compile_face(face, target)
            })
            .collect::<Result<Vec<_>, _>>()?;
        compiled.sort_unstable_by_key(|face| face.id);
        if targets.len() != 42
            || compiled.len() != 42
            || compiled.windows(2).any(|pair| pair[0].id == pair[1].id)
            || compiled
                .iter()
                .map(|face| face.operation)
                .collect::<BTreeSet<_>>()
                .len()
                != 33
        {
            return Err(invalid("Swarm dice-face denominator drift"));
        }
        let catalog = Self {
            faces: compiled.into_boxed_slice(),
        };
        if catalog.denominators() != (42, 42, 59, 23, 63)
            || catalog.coverage() != ([27, 8, 7], [25, 12, 5], [25, 2, 8, 7], 5)
        {
            return Err(invalid("Swarm dice-face coverage drift"));
        }
        Ok(catalog)
    }

    pub(super) fn compile_activation(
        &self,
        controls: &CompiledDiceControls,
        map: &MapRuntimeCatalog,
        state: &ActivityTransactionState,
        explicit_target: Option<NodeId>,
        rng: &mut ActivityRngStreams,
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        let id = controls
            .resolution_face_id(state)
            .ok_or_else(|| reference("Swarm dice face has not been rolled"))?;
        let face = self
            .faces
            .iter()
            .find(|face| face.id == id)
            .ok_or_else(|| reference("rolled Swarm dice face is unknown"))?;
        let candidates = match face.target {
            FaceTargetMode::Derived => Box::new([]),
            FaceTargetMode::Explicit(selector) | FaceTargetMode::Random { selector, .. } => {
                map.dice_face_candidates(state, selector)?
            }
        };
        face.compile_activation(
            controls.activation_prefix(state, face.id)?,
            &candidates,
            explicit_target,
            rng,
        )
    }

    pub(super) fn face(&self, key: &str) -> Option<&RuntimeDiceFace> {
        self.faces.iter().find(|face| face.key.as_ref() == key)
    }

    pub(super) fn denominators(&self) -> (usize, usize, usize, usize, usize) {
        (
            self.faces.len(),
            self.faces.len(),
            self.faces
                .iter()
                .map(|face| face.parameters_scaled.len())
                .sum(),
            self.faces
                .iter()
                .map(|face| face.description_scaled.len())
                .sum(),
            self.faces
                .iter()
                .map(|face| face.effect_references.len())
                .sum(),
        )
    }

    pub(super) fn coverage(&self) -> ([usize; 3], [usize; 3], [usize; 4], usize) {
        let mut stages = [0; 3];
        let mut targets = [0; 3];
        let mut durations = [0; 4];
        let mut finite_turns = 0;
        for face in &self.faces {
            stages[usize::from(face.activation.stage() - 1)] += 1;
            targets[match face.target {
                FaceTargetMode::Derived => 0,
                FaceTargetMode::Explicit(_) => 1,
                FaceTargetMode::Random { .. } => 2,
            }] += 1;
            durations[usize::from(face.duration.code() - 1)] += 1;
            finite_turns += usize::from(face.turn_duration.is_some());
        }
        (stages, targets, durations, finite_turns)
    }
}

impl RuntimeDiceFace {
    fn compile_activation(
        &self,
        mut operations: Vec<ActivityOperation>,
        candidates: &[NodeId],
        explicit_target: Option<NodeId>,
        rng: &mut ActivityRngStreams,
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        let (targets, no_op) = self.resolve_targets(candidates, explicit_target, rng)?;
        operations.extend([
            add_counter(USE_BASE + u64::from(self.id), 1),
            add_counter(OPERATION_BASE + self.operation.code(), 1),
            add_counter(STAGE_BASE + u64::from(self.activation.stage()), 1),
            add_counter(DURATION_BASE + u64::from(self.duration.code()), 1),
        ]);
        operations.extend(self.descriptor_operations()?);
        if no_op {
            operations.push(add_counter(NO_OP_BASE + u64::from(self.id), 1));
        } else if self.activation == FaceActivation::BattleContribution {
            operations.push(add_counter(
                BATTLE_CONTRIBUTION_BASE + u64::from(self.id),
                1,
            ));
        } else {
            operations.push(add_counter(GRAPH_EFFECT_BASE + u64::from(self.id), 1));
        }
        for target in targets {
            operations.push(add_counter(TARGET_BASE + u64::from(target.get()), 1));
            if self.operation.is_mercy() {
                operations.push(add_counter(MERCY_TARGET_BASE + u64::from(target.get()), 1));
            }
        }
        ActivityProgramDefinition::new(
            ActivityProgramId::new(
                PROGRAM_BASE
                    .checked_add(self.id)
                    .ok_or_else(|| invalid("Swarm dice-face program ID overflow"))?,
            )
            .ok_or_else(|| invalid("invalid Swarm dice-face program ID"))?,
            operations,
        )
        .map_err(|_| invalid("invalid Swarm dice-face program"))
    }

    fn resolve_targets(
        &self,
        candidates: &[NodeId],
        explicit_target: Option<NodeId>,
        rng: &mut ActivityRngStreams,
    ) -> Result<(Vec<NodeId>, bool), UniverseCatalogLoadError> {
        match self.target {
            FaceTargetMode::Derived => {
                if explicit_target.is_some() {
                    return Err(reference(
                        "derived Swarm dice face rejects an explicit target",
                    ));
                }
                Ok((Vec::new(), false))
            }
            FaceTargetMode::Explicit(_) => {
                if candidates.is_empty() && explicit_target.is_none() {
                    return Ok((Vec::new(), true));
                }
                let target = explicit_target
                    .ok_or_else(|| reference("Swarm dice face requires an explicit target"))?;
                if !candidates.contains(&target) {
                    return Err(reference("invalid explicit Swarm dice-face target"));
                }
                Ok((vec![target], false))
            }
            FaceTargetMode::Random { maximum, .. } => {
                if explicit_target.is_some() {
                    return Err(reference(
                        "random Swarm dice face rejects an explicit target",
                    ));
                }
                if candidates.is_empty() {
                    return Ok((Vec::new(), true));
                }
                rng.transact(|working| {
                    let mut remaining = candidates.to_vec();
                    let mut selected = Vec::new();
                    while selected.len() < usize::from(maximum) && !remaining.is_empty() {
                        let count = u32::try_from(remaining.len())
                            .map_err(|_| invalid("too many Swarm dice-face candidates"))?;
                        let draw = working
                            .choose_index(ActivityRngLabel::Spawn, TARGET_PURPOSE, count)
                            .map_err(|_| invalid("Swarm dice-face target RNG failure"))?
                            .ok_or_else(|| invalid("missing Swarm dice-face target draw"))?;
                        selected.push(remaining.remove(draw.value() as usize));
                    }
                    Ok((selected, false))
                })
            }
        }
    }

    fn descriptor_operations(&self) -> Result<Vec<ActivityOperation>, UniverseCatalogLoadError> {
        let mut operations = vec![
            set_counter(
                PARAMETER_COUNT_BASE + u64::from(self.id),
                i64::try_from(self.parameters_scaled.len())
                    .map_err(|_| invalid("too many Swarm dice-face parameters"))?,
            ),
            set_counter(
                DESCRIPTION_COUNT_BASE + u64::from(self.id),
                i64::try_from(self.description_scaled.len())
                    .map_err(|_| invalid("too many Swarm dice-face description parameters"))?,
            ),
        ];
        for (index, value) in self.parameters_scaled.iter().enumerate() {
            operations.push(set_counter(
                indexed(PARAMETER_VALUE_BASE, self.id, index)?,
                *value,
            ));
        }
        for (index, value) in self.description_scaled.iter().enumerate() {
            operations.push(set_counter(
                indexed(DESCRIPTION_VALUE_BASE, self.id, index)?,
                *value,
            ));
        }
        for reference in &self.effect_references {
            operations.push(add_counter(
                EFFECT_REFERENCE_BASE + u64::from(*reference),
                1,
            ));
        }
        if let Some(turns) = self.turn_duration {
            operations.push(set_counter(
                TURN_DURATION_BASE + u64::from(self.id),
                i64::from(turns),
            ));
        }
        Ok(operations)
    }

    pub(super) const fn activation_stage(&self) -> u8 {
        self.activation.stage()
    }

    pub(super) const fn target_contract(&self) -> &'static str {
        match self.target {
            FaceTargetMode::Derived => "global-or-event-derived",
            FaceTargetMode::Explicit(_) => "caller-explicit-eligible-node",
            FaceTargetMode::Random { .. } => "spawn-random-eligible-node",
        }
    }

    pub(super) const fn selector_name(&self) -> &'static str {
        match self.target {
            FaceTargetMode::Derived => "event-derived",
            FaceTargetMode::Explicit(selector) | FaceTargetMode::Random { selector, .. } => {
                selector.name()
            }
        }
    }

    pub(super) const fn duration_name(&self) -> &'static str {
        self.duration.name()
    }

    pub(super) fn parameters_scaled(&self) -> &[i64] {
        &self.parameters_scaled
    }

    pub(super) fn description_scaled(&self) -> &[i64] {
        &self.description_scaled
    }

    pub(super) const fn turn_duration(&self) -> Option<u16> {
        self.turn_duration
    }

    pub(super) fn effect_references(&self) -> &[u32] {
        &self.effect_references
    }

    pub(super) const fn operation_name(&self) -> &'static str {
        self.operation.name()
    }
}

impl FaceActivation {
    const fn stage(self) -> u8 {
        match self {
            Self::Immediate => 1,
            Self::AfterMovement => 2,
            Self::BattleContribution => 3,
        }
    }
}

impl EffectDuration {
    const fn code(self) -> u8 {
        match self {
            Self::Immediate => 1,
            Self::CurrentMovement => 2,
            Self::AfterMovement => 3,
            Self::NextBattle => 4,
        }
    }

    const fn name(self) -> &'static str {
        match self {
            Self::Immediate => "Immediate",
            Self::CurrentMovement => "CurrentMovement",
            Self::AfterMovement => "AfterMovement",
            Self::NextBattle => "NextBattle",
        }
    }
}

impl FaceSelector {
    pub(super) const fn name(self) -> &'static str {
        match self {
            Self::Any => "any-active-domain",
            Self::NonBoss => "non-boss-domain",
            Self::Combat => "combat-domain",
            Self::Elite => "elite-domain",
            Self::Occurrence => "occurrence-domain",
            Self::CombatSwarmElite => "combat-swarm-or-elite-domain",
            Self::CombatSwarm => "combat-swarm-domain",
            Self::Swarm => "swarm-domain",
            Self::Boss => "boss-or-swarm-boss-domain",
            Self::WithoutMercy => "active-domain-without-mercy",
        }
    }
}

#[derive(Deserialize)]
struct EffectInput {
    order: u16,
    operation: String,
    parameters: Vec<String>,
    description_parameters: Vec<String>,
    extra_effect_refs: Vec<String>,
}

#[derive(Deserialize)]
struct CandidateFilterInput {
    effect_type: String,
    authored_parameters: Vec<String>,
}

struct RuntimeTarget {
    source_id: Box<str>,
    operation: FaceOperation,
    parameters: Box<[Box<str>]>,
}

fn compile_target(
    input: &SwarmDiceTargetRuntimeInput,
) -> Result<RuntimeTarget, UniverseCatalogLoadError> {
    let filter = serde_json::from_str::<CandidateFilterInput>(&input.candidate_filter)
        .map_err(|_| invalid("invalid Swarm dice target filter"))?;
    let cardinality = serde_json::from_str::<String>(&input.cardinality)
        .map_err(|_| invalid("invalid Swarm dice target cardinality"))?;
    let no_target = serde_json::from_str::<String>(&input.no_legal_target)
        .map_err(|_| invalid("invalid Swarm dice empty-target policy"))?;
    if input.ordering.as_ref() != "StableDomainThenNodeId"
        || cardinality != "AuthoredEffectDefined"
        || no_target != "NoOp"
        || input.key.as_ref() != format!("swarm-disaster.dice-target.{}", input.source_id)
    {
        return Err(invalid("Swarm dice target policy drift"));
    }
    Ok(RuntimeTarget {
        source_id: input.source_id.clone(),
        operation: FaceOperation::parse(&filter.effect_type)
            .ok_or_else(|| invalid("unknown Swarm dice-face operation"))?,
        parameters: filter
            .authored_parameters
            .into_iter()
            .map(String::into_boxed_str)
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    })
}

fn compile_face(
    input: &SwarmAudienceFaceRuntimeInput,
    target: &RuntimeTarget,
) -> Result<RuntimeDiceFace, UniverseCatalogLoadError> {
    let mut effects = serde_json::from_str::<Vec<EffectInput>>(&input.effect_program)
        .map_err(|_| invalid("invalid Swarm dice-face effect program"))?;
    if effects.len() != 1 {
        return Err(invalid("Swarm dice face must have exactly one effect"));
    }
    let effect = effects.pop().expect("one effect was validated");
    let operation = FaceOperation::parse(&effect.operation)
        .ok_or_else(|| invalid("unknown Swarm dice-face operation"))?;
    if effect.order != 0
        || input.source_id != target.source_id
        || operation != target.operation
        || effect.parameters.len() != target.parameters.len()
        || effect
            .parameters
            .iter()
            .zip(target.parameters.iter())
            .any(|(left, right)| left.as_str() != right.as_ref())
    {
        return Err(invalid("Swarm dice face and target drift"));
    }
    let activation = match input.activation_stage {
        1 => FaceActivation::Immediate,
        2 => FaceActivation::AfterMovement,
        3 => FaceActivation::BattleContribution,
        _ => return Err(invalid("unknown Swarm dice-face activation stage")),
    };
    let parameters_scaled = scaled_values(&effect.parameters)?;
    let description_scaled = scaled_values(&effect.description_parameters)?;
    let turn_duration = battle_turn_duration(activation, &description_scaled)?;
    let target = target_mode(operation, &effect.parameters)?;
    Ok(RuntimeDiceFace {
        id: input.id,
        key: input.key.clone(),
        activation,
        duration: duration(activation, operation),
        target,
        operation,
        parameters_scaled,
        description_scaled,
        turn_duration,
        effect_references: effect
            .extra_effect_refs
            .iter()
            .map(|value| effect_reference(value))
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
    })
}

fn battle_turn_duration(
    activation: FaceActivation,
    description: &[i64],
) -> Result<Option<u16>, UniverseCatalogLoadError> {
    if activation != FaceActivation::BattleContribution {
        return Ok(None);
    }
    description.last().map_or(Ok(None), |value| {
        if *value <= 0 || value % 1_000_000 != 0 {
            return Ok(None);
        }
        u16::try_from(value / 1_000_000)
            .map(Some)
            .map_err(|_| invalid("Swarm dice-face turn duration overflow"))
    })
}

fn target_mode(
    operation: FaceOperation,
    parameters: &[String],
) -> Result<FaceTargetMode, UniverseCatalogLoadError> {
    let explicit = match operation {
        FaceOperation::SelectCellToProtect
        | FaceOperation::ReplicateCellToAround
        | FaceOperation::SelectAndToFightCell
        | FaceOperation::SelectExceptCellGetHelp => Some(FaceSelector::NonBoss),
        FaceOperation::SetSpecialType => Some(FaceSelector::Any),
        FaceOperation::SetCellTypeAndTakeReward | FaceOperation::SetMarkType => {
            Some(FaceSelector::CombatSwarmElite)
        }
        FaceOperation::SelectCellGetHelp => match parameters.first().map(String::as_str) {
            Some("3") => Some(FaceSelector::Combat),
            Some("5") => Some(FaceSelector::Elite),
            Some("8") => Some(FaceSelector::Occurrence),
            _ => return Err(invalid("unknown Swarm Happiness selector")),
        },
        FaceOperation::SetAroundBlockType => Some(FaceSelector::Swarm),
        FaceOperation::MoveToSwarmGetBuff => Some(FaceSelector::Boss),
        _ => None,
    };
    if let Some(selector) = explicit {
        return Ok(FaceTargetMode::Explicit(selector));
    }
    let random = match operation {
        FaceOperation::ReplicateLastCell | FaceOperation::ToRandomBlockType => Some((
            FaceSelector::Any,
            parameter_maximum(parameters, 0).unwrap_or(1),
        )),
        FaceOperation::RandomSetSpecialType => Some((
            FaceSelector::WithoutMercy,
            parameter_maximum(parameters, 0)?,
        )),
        FaceOperation::SetMarkToRandomCell => Some((FaceSelector::CombatSwarmElite, 1)),
        FaceOperation::TriggerMark => Some((FaceSelector::CombatSwarm, 1)),
        _ => None,
    };
    Ok(
        random.map_or(FaceTargetMode::Derived, |(selector, maximum)| {
            FaceTargetMode::Random { selector, maximum }
        }),
    )
}

const fn duration(activation: FaceActivation, operation: FaceOperation) -> EffectDuration {
    if matches!(activation, FaceActivation::BattleContribution) {
        EffectDuration::NextBattle
    } else if matches!(activation, FaceActivation::AfterMovement) {
        EffectDuration::AfterMovement
    } else if matches!(
        operation,
        FaceOperation::AllowMoveToReplicateCell
            | FaceOperation::TrunEmptyToReward
            | FaceOperation::EnterEmptyGetMoney
            | FaceOperation::MoveMarkCellUpgradeReward
            | FaceOperation::SetColCanMove
            | FaceOperation::EnterCellTriggerBuff
            | FaceOperation::MoveToSwarmGetBuff
    ) {
        EffectDuration::CurrentMovement
    } else {
        EffectDuration::Immediate
    }
}

fn parameter_maximum(parameters: &[String], index: usize) -> Result<u8, UniverseCatalogLoadError> {
    parameters
        .get(index)
        .and_then(|value| value.parse::<u8>().ok())
        .filter(|value| *value != 0)
        .ok_or_else(|| invalid("invalid Swarm dice-face target cardinality"))
}

fn scaled_values(values: &[String]) -> Result<Box<[i64]>, UniverseCatalogLoadError> {
    values
        .iter()
        .map(|value| scaled(value))
        .collect::<Result<Vec<_>, _>>()
        .map(Vec::into_boxed_slice)
}

fn scaled(value: &str) -> Result<i64, UniverseCatalogLoadError> {
    let (whole, fraction) = value.split_once('.').map_or((value, ""), |parts| parts);
    let canonical_whole = whole == "0"
        || (!whole.starts_with('0') && whole.bytes().all(|byte| byte.is_ascii_digit()));
    if !canonical_whole
        || fraction.len() > 6
        || (!fraction.is_empty()
            && (fraction.ends_with('0') || !fraction.bytes().all(|byte| byte.is_ascii_digit())))
    {
        return Err(invalid("non-canonical Swarm dice-face scalar"));
    }
    let whole = whole
        .parse::<i64>()
        .map_err(|_| invalid("invalid Swarm dice-face scalar"))?;
    let mut fraction_text = fraction.to_owned();
    fraction_text.extend(core::iter::repeat_n('0', 6 - fraction.len()));
    whole
        .checked_mul(1_000_000)
        .and_then(|value| {
            fraction_text
                .parse::<i64>()
                .ok()
                .and_then(|fraction| value.checked_add(fraction))
        })
        .ok_or_else(|| invalid("Swarm dice-face scalar overflow"))
}

fn effect_reference(value: &str) -> Result<u32, UniverseCatalogLoadError> {
    value
        .strip_prefix("source-effect.")
        .and_then(|value| value.parse::<u32>().ok())
        .filter(|value| *value != 0)
        .ok_or_else(|| invalid("invalid Swarm dice-face effect reference"))
}

fn indexed(base: u64, face: u32, index: usize) -> Result<u64, UniverseCatalogLoadError> {
    let index = u64::try_from(index).map_err(|_| invalid("Swarm dice descriptor overflow"))?;
    base.checked_add(u64::from(face) * 16)
        .and_then(|value| value.checked_add(index))
        .ok_or_else(|| invalid("Swarm dice descriptor overflow"))
}

fn add_counter(key: u64, delta: i64) -> ActivityOperation {
    ActivityOperation::AddCounter {
        slot: slot(DEFERRED),
        key,
        delta: integer(delta),
    }
}

fn set_counter(key: u64, desired: i64) -> ActivityOperation {
    ActivityOperation::AddCounter {
        slot: slot(DEFERRED),
        key,
        delta: ActivityExpression::Subtract(
            Box::new(integer(desired)),
            Box::new(ActivityExpression::CounterValue {
                slot: slot(DEFERRED),
                key,
            }),
        ),
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
