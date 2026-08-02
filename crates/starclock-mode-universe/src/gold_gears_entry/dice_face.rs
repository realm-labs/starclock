//! Private lowering for all released Custom Dice face executor descriptors.

use std::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;
use starclock_activity::{
    ActivityCondition, ActivityExpression, ActivityOperation, ActivityProgramDefinition,
    ActivityProgramId, ActivityRngLabel, ActivityRngStreams, ActivitySlotId,
    ActivityTransactionState, ActivityValue, NodeId,
};

use crate::gold_gears_unique::{DiceFace, GoldAndGearsUniqueCatalog};

use super::{
    GoldAndGearsEntryError,
    state_layout::{
        DEFERRED_DICE_FACE_CODE_BASE, DEFERRED_DICE_FACE_EFFECT_BASE,
        DEFERRED_DICE_FACE_STAGE_BASE, DEFERRED_DICE_FACE_TARGET_BASE, DEFERRED_DICE_FACE_USE_BASE,
        DEFERRED_EFFECTS_SLOT, DICE_RESOLUTION_FACE_KEY, DICE_RESOLUTION_SLOT,
    },
};

const TARGET_POLICY_ID: &str = "dice-face-target-resolution-v1";
const FACE_PROGRAM_BASE: u32 = 0x4780_0000;
pub(super) const DICE_FACE_TARGET_PURPOSE: u16 = 0x4753;

#[derive(Clone, Debug)]
pub(super) struct DiceFaceRuntimeCatalog {
    faces: Box<[RuntimeDiceFace]>,
}

#[derive(Clone, Debug)]
pub(super) struct RuntimeDiceFace {
    id: u32,
    key: Box<str>,
    activation: FaceActivation,
    target: FaceTargetMode,
    parameters_scaled: Box<[i64]>,
    effect_ids: Box<[u64]>,
    mechanical_codes: Box<[FaceMechanicalCode]>,
    no_target: NoTargetBehavior,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FaceActivation {
    Immediate,
    AfterMovement,
    NextBattle,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FaceTargetMode {
    Global,
    Explicit(FaceSelector),
    Random { selector: FaceSelector, maximum: u8 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FaceSelector {
    Any,
    NonBoss,
    Combat,
    CombatOrElite,
    CombatEliteOccurrenceOrReward,
    CombatWithBeacon,
    EliteOrAdventure,
    EliteOrReward,
    EliteAdventureOrReward,
    OccurrenceOrReward,
    RewardOrAdventure,
    Knowledge,
    WithoutKnowledge,
    AdjacentCurrent,
    AboutToCollapse,
    KnowledgeNonBlankOrBoss,
    Beacon,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum FaceMechanicalCode {
    ActionPoint,
    BlockChange,
    Buff,
    BuffProMax,
    Coin,
    Mark,
    Miracle,
    Move,
    Replicate,
    SpecialType,
}

impl FaceMechanicalCode {
    const fn key(self) -> u64 {
        match self {
            Self::ActionPoint => 1,
            Self::BlockChange => 2,
            Self::Buff => 3,
            Self::BuffProMax => 4,
            Self::Coin => 5,
            Self::Mark => 6,
            Self::Miracle => 7,
            Self::Move => 8,
            Self::Replicate => 9,
            Self::SpecialType => 10,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum NoTargetBehavior {
    FailClosed,
    NoEffect,
}

impl DiceFaceRuntimeCatalog {
    pub(super) fn compile(
        catalog: &GoldAndGearsUniqueCatalog,
    ) -> Result<Self, GoldAndGearsEntryError> {
        let tags = catalog
            .dice_face_tags
            .iter()
            .map(|tag| {
                parse_code(&tag.mechanical_code)
                    .map(|code| (tag.identity.stable_key.as_ref(), code))
            })
            .collect::<Result<BTreeMap<_, _>, _>>()?;
        if tags.len() != 10 {
            return Err(GoldAndGearsEntryError::InvalidDiceFaceRuntime);
        }

        let mut faces = Vec::with_capacity(catalog.dice_faces.len());
        for face in &catalog.dice_faces {
            faces.push(runtime_face(face, &tags)?);
        }
        faces.sort_by_key(|face| face.id);
        if faces.len() != 80
            || faces.windows(2).any(|pair| pair[0].id == pair[1].id)
            || faces
                .iter()
                .map(|face| face.key.as_ref())
                .collect::<BTreeSet<_>>()
                .len()
                != 80
        {
            return Err(GoldAndGearsEntryError::InvalidDiceFaceRuntime);
        }
        Ok(Self {
            faces: faces.into_boxed_slice(),
        })
    }

    pub(super) fn select(
        &self,
        selected: &[&DiceFace],
    ) -> Result<Box<[RuntimeDiceFace]>, GoldAndGearsEntryError> {
        selected
            .iter()
            .map(|face| {
                self.faces
                    .iter()
                    .find(|runtime| runtime.id == face.identity.id.0)
                    .cloned()
                    .ok_or(GoldAndGearsEntryError::InvalidDiceFaceRuntime)
            })
            .collect::<Result<Vec<_>, _>>()
            .map(Vec::into_boxed_slice)
    }

    #[cfg(test)]
    pub(super) fn denominators(&self) -> (usize, usize, usize, usize) {
        (
            self.faces.len(),
            self.faces.iter().map(|face| face.effect_ids.len()).sum(),
            self.faces
                .iter()
                .map(|face| face.mechanical_codes.len())
                .sum(),
            self.faces
                .iter()
                .map(|face| face.parameters_scaled.len())
                .sum(),
        )
    }

    #[cfg(test)]
    pub(super) fn coverage(&self) -> ([usize; 3], [usize; 3], [usize; 2]) {
        let mut stages = [0; 3];
        let mut targets = [0; 3];
        let mut empty = [0; 2];
        for face in &self.faces {
            stages[usize::from(face.activation_stage() - 1)] += 1;
            targets[match face.target {
                FaceTargetMode::Global => 0,
                FaceTargetMode::Explicit(_) => 1,
                FaceTargetMode::Random { .. } => 2,
            }] += 1;
            empty[usize::from(face.no_target == NoTargetBehavior::NoEffect)] += 1;
        }
        (stages, targets, empty)
    }
}

impl RuntimeDiceFace {
    pub(super) fn compile_empty_content(
        &self,
        state: &ActivityTransactionState,
    ) -> Result<ActivityProgramDefinition, GoldAndGearsEntryError> {
        if self.no_target != NoTargetBehavior::NoEffect {
            return Err(GoldAndGearsEntryError::NoLegalDiceFaceTarget);
        }
        if counter_value(state, DICE_RESOLUTION_SLOT, DICE_RESOLUTION_FACE_KEY)
            != Some(i64::from(self.id))
        {
            return Err(GoldAndGearsEntryError::DiceFaceNotRolled);
        }
        let id = FACE_PROGRAM_BASE
            .checked_add(self.id)
            .and_then(ActivityProgramId::new)
            .ok_or(GoldAndGearsEntryError::InvalidDiceFaceRuntime)?;
        ActivityProgramDefinition::new(
            id,
            vec![
                ActivityOperation::Require(ActivityCondition::Equal(
                    counter(DICE_RESOLUTION_SLOT, DICE_RESOLUTION_FACE_KEY),
                    integer(i64::from(self.id)),
                )),
                add_counter(
                    DEFERRED_EFFECTS_SLOT,
                    DEFERRED_DICE_FACE_USE_BASE + u64::from(self.id),
                    1,
                ),
            ],
        )
        .map_err(|_| GoldAndGearsEntryError::InvalidDiceFaceRuntime)
    }

    pub(super) fn compile_activation(
        &self,
        state: &ActivityTransactionState,
        candidates: &[NodeId],
        explicit_target: Option<NodeId>,
        rng: &mut ActivityRngStreams,
    ) -> Result<ActivityProgramDefinition, GoldAndGearsEntryError> {
        if counter_value(state, DICE_RESOLUTION_SLOT, DICE_RESOLUTION_FACE_KEY)
            != Some(i64::from(self.id))
        {
            return Err(GoldAndGearsEntryError::DiceFaceNotRolled);
        }
        let mut canonical = candidates.to_vec();
        canonical.sort_unstable();
        if canonical.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(GoldAndGearsEntryError::InvalidDiceFaceTarget);
        }
        let targets = match self.target {
            FaceTargetMode::Global => {
                if explicit_target.is_some() {
                    return Err(GoldAndGearsEntryError::InvalidDiceFaceTarget);
                }
                Vec::new()
            }
            FaceTargetMode::Explicit(_) => {
                let target =
                    explicit_target.ok_or(GoldAndGearsEntryError::InvalidDiceFaceTarget)?;
                if canonical.binary_search(&target).is_err() {
                    return Err(GoldAndGearsEntryError::InvalidDiceFaceTarget);
                }
                vec![target]
            }
            FaceTargetMode::Random { maximum, .. } => {
                if explicit_target.is_some() {
                    return Err(GoldAndGearsEntryError::InvalidDiceFaceTarget);
                }
                if canonical.is_empty() {
                    return Err(GoldAndGearsEntryError::NoLegalDiceFaceTarget);
                }
                rng.transact(|working| {
                    let mut remaining = canonical;
                    let mut selected = Vec::new();
                    while selected.len() < usize::from(maximum) && !remaining.is_empty() {
                        let count = u32::try_from(remaining.len())
                            .map_err(|_| GoldAndGearsEntryError::InvalidDiceFaceRuntime)?;
                        let draw = working
                            .choose_index(ActivityRngLabel::Spawn, DICE_FACE_TARGET_PURPOSE, count)
                            .map_err(|_| GoldAndGearsEntryError::InvalidDiceFaceRuntime)?
                            .ok_or(GoldAndGearsEntryError::NoLegalDiceFaceTarget)?;
                        selected.push(remaining.remove(draw.value() as usize));
                    }
                    Ok(selected)
                })?
            }
        };
        let mut operations = vec![
            ActivityOperation::Require(ActivityCondition::Equal(
                counter(DICE_RESOLUTION_SLOT, DICE_RESOLUTION_FACE_KEY),
                integer(i64::from(self.id)),
            )),
            add_counter(
                DEFERRED_EFFECTS_SLOT,
                DEFERRED_DICE_FACE_USE_BASE + u64::from(self.id),
                1,
            ),
        ];
        for effect in &self.effect_ids {
            operations.push(add_counter(
                DEFERRED_EFFECTS_SLOT,
                DEFERRED_DICE_FACE_EFFECT_BASE
                    .checked_add(*effect)
                    .ok_or(GoldAndGearsEntryError::InvalidDiceFaceRuntime)?,
                1,
            ));
        }
        operations.push(add_counter(
            DEFERRED_EFFECTS_SLOT,
            DEFERRED_DICE_FACE_STAGE_BASE + u64::from(self.activation_stage()),
            1,
        ));
        for code in &self.mechanical_codes {
            operations.push(add_counter(
                DEFERRED_EFFECTS_SLOT,
                DEFERRED_DICE_FACE_CODE_BASE + code.key(),
                1,
            ));
        }
        for target in targets {
            operations.push(add_counter(
                DEFERRED_EFFECTS_SLOT,
                DEFERRED_DICE_FACE_TARGET_BASE + u64::from(target.get()),
                1,
            ));
        }
        let id = FACE_PROGRAM_BASE
            .checked_add(self.id)
            .and_then(ActivityProgramId::new)
            .ok_or(GoldAndGearsEntryError::InvalidDiceFaceRuntime)?;
        ActivityProgramDefinition::new(id, operations)
            .map_err(|_| GoldAndGearsEntryError::InvalidDiceFaceRuntime)
    }

    pub(super) const fn activation_stage(&self) -> u8 {
        match self.activation {
            FaceActivation::Immediate => 1,
            FaceActivation::AfterMovement => 2,
            FaceActivation::NextBattle => 3,
        }
    }

    pub(super) const fn target_contract(&self) -> &'static str {
        match self.target {
            FaceTargetMode::Global => "global-or-event-derived",
            FaceTargetMode::Explicit(_) => "caller-explicit-eligible-node",
            FaceTargetMode::Random { .. } => "spawn-random-eligible-node",
        }
    }

    pub(super) const fn random_target_maximum(&self) -> Option<u8> {
        match self.target {
            FaceTargetMode::Random { maximum, .. } => Some(maximum),
            FaceTargetMode::Global | FaceTargetMode::Explicit(_) => None,
        }
    }

    pub(super) const fn selector_name(&self) -> &'static str {
        let selector = match self.target {
            FaceTargetMode::Global => return "event-derived",
            FaceTargetMode::Explicit(selector) | FaceTargetMode::Random { selector, .. } => {
                selector
            }
        };
        match selector {
            FaceSelector::Any => "any-domain",
            FaceSelector::NonBoss => "non-boss-domain",
            FaceSelector::Combat => "combat-domain",
            FaceSelector::CombatOrElite => "combat-or-elite-domain",
            FaceSelector::CombatEliteOccurrenceOrReward => {
                "combat-elite-occurrence-or-reward-domain"
            }
            FaceSelector::CombatWithBeacon => "combat-domain-with-beacon",
            FaceSelector::EliteOrAdventure => "elite-or-adventure-domain",
            FaceSelector::EliteOrReward => "elite-or-reward-domain",
            FaceSelector::EliteAdventureOrReward => "elite-adventure-or-reward-domain",
            FaceSelector::OccurrenceOrReward => "occurrence-or-reward-domain",
            FaceSelector::RewardOrAdventure => "reward-or-adventure-domain",
            FaceSelector::Knowledge => "knowledge-domain",
            FaceSelector::WithoutKnowledge => "domain-without-knowledge",
            FaceSelector::AdjacentCurrent => "adjacent-current-domain",
            FaceSelector::AboutToCollapse => "about-to-collapse-domain",
            FaceSelector::KnowledgeNonBlankOrBoss => "knowledge-nonblank-nonboss-domain",
            FaceSelector::Beacon => "beacon-domain",
        }
    }

    pub(super) fn parameters_scaled(&self) -> &[i64] {
        &self.parameters_scaled
    }

    pub(super) fn effect_ids(&self) -> &[u64] {
        &self.effect_ids
    }

    pub(super) fn mechanical_codes(&self) -> impl ExactSizeIterator<Item = &'static str> + '_ {
        self.mechanical_codes.iter().map(|code| match code {
            FaceMechanicalCode::ActionPoint => "ActionPoint",
            FaceMechanicalCode::BlockChange => "BlockChange",
            FaceMechanicalCode::Buff => "Buff",
            FaceMechanicalCode::BuffProMax => "BuffProMax",
            FaceMechanicalCode::Coin => "Coin",
            FaceMechanicalCode::Mark => "Mark",
            FaceMechanicalCode::Miracle => "Miracle",
            FaceMechanicalCode::Move => "Move",
            FaceMechanicalCode::Replicate => "Replicate",
            FaceMechanicalCode::SpecialType => "SpecialType",
        })
    }

    pub(super) const fn no_target_behavior(&self) -> &'static str {
        match self.no_target {
            NoTargetBehavior::FailClosed => "FailClosed",
            NoTargetBehavior::NoEffect => "NoEffect",
        }
    }
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

fn add_counter(slot_id: u32, key: u64, delta: i64) -> ActivityOperation {
    ActivityOperation::AddCounter {
        slot: slot(slot_id),
        key,
        delta: integer(delta),
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

fn runtime_face(
    face: &DiceFace,
    tags: &BTreeMap<&str, FaceMechanicalCode>,
) -> Result<RuntimeDiceFace, GoldAndGearsEntryError> {
    let policy = serde_json::from_str::<TargetPolicy>(&face.target_policy_json)
        .map_err(|_| GoldAndGearsEntryError::InvalidDiceFaceRuntime)?;
    if policy.policy_id.as_ref() != TARGET_POLICY_ID
        || policy.selector_validation.as_ref() != "released-selector-exact"
        || policy.candidate_order.as_ref() != "stable-node-or-content-id-ascending"
        || policy.operation_order.as_ref() != "authored-effect-order"
        || policy.equal_priority_order.as_ref() != "target-stable-id-ascending"
        || policy.unpublished_empty_set_behavior.as_ref() != "FailClosed"
        || policy.evidence_quality.as_ref() != "ProjectPolicy"
        || policy.replacement_condition.is_empty()
    {
        return Err(GoldAndGearsEntryError::InvalidDiceFaceRuntime);
    }
    let activation = match face.activation_stage {
        1 => FaceActivation::Immediate,
        2 => FaceActivation::AfterMovement,
        3 => FaceActivation::NextBattle,
        _ => return Err(GoldAndGearsEntryError::InvalidDiceFaceRuntime),
    };
    let no_target = match face.no_target_behavior.as_ref() {
        "FailClosed" => NoTargetBehavior::FailClosed,
        "NoEffect" => NoTargetBehavior::NoEffect,
        _ => return Err(GoldAndGearsEntryError::InvalidDiceFaceRuntime),
    };
    let mechanical_codes = face
        .mechanical_codes
        .iter()
        .map(|code| parse_code(code))
        .collect::<Result<BTreeSet<_>, _>>()?;
    let mapped_codes = face
        .filter_tag_sources
        .iter()
        .map(|key| {
            tags.get(key.as_ref())
                .copied()
                .ok_or(GoldAndGearsEntryError::InvalidDiceFaceRuntime)
        })
        .collect::<Result<BTreeSet<_>, _>>()?;
    if mechanical_codes != mapped_codes {
        return Err(GoldAndGearsEntryError::InvalidDiceFaceRuntime);
    }
    let parameters_scaled = face
        .parameters
        .iter()
        .map(|value| scaled(value))
        .collect::<Result<Vec<_>, _>>()?
        .into_boxed_slice();
    let effect_ids = face
        .effect_ids
        .iter()
        .map(|value| {
            value
                .parse::<u64>()
                .ok()
                .filter(|value| *value != 0)
                .ok_or(GoldAndGearsEntryError::InvalidDiceFaceRuntime)
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_boxed_slice();
    Ok(RuntimeDiceFace {
        id: face.identity.id.0,
        key: face.identity.stable_key.clone(),
        activation,
        target: target_mode(&face.identity.source_id, &parameters_scaled)?,
        parameters_scaled,
        effect_ids,
        mechanical_codes: mechanical_codes.into_iter().collect(),
        no_target,
    })
}

fn target_mode(source: &str, parameters: &[i64]) -> Result<FaceTargetMode, GoldAndGearsEntryError> {
    let explicit = match source {
        "2004" | "2006" | "2008" | "2009" => Some(FaceSelector::NonBoss),
        "2007" => Some(FaceSelector::NonBoss),
        "2010" => Some(FaceSelector::Any),
        "2011" => Some(FaceSelector::Combat),
        "2013" | "2001" => Some(FaceSelector::CombatOrElite),
        "2016" => Some(FaceSelector::CombatWithBeacon),
        "2051" => Some(FaceSelector::EliteOrReward),
        "2065" => Some(FaceSelector::EliteAdventureOrReward),
        "2002" => Some(FaceSelector::OccurrenceOrReward),
        "2072" => Some(FaceSelector::RewardOrAdventure),
        "2073" | "2030" | "2077" => Some(FaceSelector::Knowledge),
        "2079" => Some(FaceSelector::KnowledgeNonBlankOrBoss),
        "2074" => Some(FaceSelector::Any),
        "2102" | "2052" => Some(FaceSelector::Beacon),
        "2014" => Some(FaceSelector::CombatEliteOccurrenceOrReward),
        _ => None,
    };
    if let Some(selector) = explicit {
        return Ok(FaceTargetMode::Explicit(selector));
    }
    let random = match source {
        "2005" | "2017" | "2063" => Some((FaceSelector::NonBoss, 1)),
        "2012" => Some((FaceSelector::EliteOrAdventure, 1)),
        "2019" | "2056" => Some((FaceSelector::Any, 1)),
        "2022" => Some((FaceSelector::CombatOrElite, 1)),
        "2027" => Some((FaceSelector::WithoutKnowledge, 1)),
        "2032" => Some((FaceSelector::AdjacentCurrent, 1)),
        "2023" => Some((FaceSelector::Knowledge, parameter_count(parameters, 0)?)),
        "2031" => Some((
            FaceSelector::AboutToCollapse,
            parameter_count(parameters, 0)?,
        )),
        "2024" => Some((
            FaceSelector::WithoutKnowledge,
            parameter_count(parameters, 0)?,
        )),
        "2025" => Some((FaceSelector::Any, parameter_count(parameters, 0)?)),
        _ => None,
    };
    if let Some((selector, maximum)) = random {
        return Ok(FaceTargetMode::Random { selector, maximum });
    }
    match source {
        "2080" | "2081" | "2082" | "2083" | "2084" | "2085" | "2086" | "2087" | "2088" | "2089"
        | "2090" | "2091" | "2092" | "2093" | "2018" | "2033" | "2037" | "2039" | "2040"
        | "2041" | "2042" | "2046" | "2047" | "2048" | "2057" | "2058" | "2064" | "2068"
        | "2101" | "2103" | "2003" | "2026" | "2034" | "2035" | "2038" | "2043" | "2053"
        | "2054" | "2061" | "2062" | "2066" | "2067" | "2070" | "2071" | "2078" => {
            Ok(FaceTargetMode::Global)
        }
        _ => Err(GoldAndGearsEntryError::InvalidDiceFaceRuntime),
    }
}

fn parameter_count(parameters: &[i64], index: usize) -> Result<u8, GoldAndGearsEntryError> {
    parameters
        .get(index)
        .and_then(|value| value.checked_div(1_000_000))
        .and_then(|value| u8::try_from(value).ok())
        .filter(|value| *value != 0)
        .ok_or(GoldAndGearsEntryError::InvalidDiceFaceRuntime)
}

fn parse_code(value: &str) -> Result<FaceMechanicalCode, GoldAndGearsEntryError> {
    match value {
        "ActionPoint" => Ok(FaceMechanicalCode::ActionPoint),
        "BlockChange" => Ok(FaceMechanicalCode::BlockChange),
        "Buff" => Ok(FaceMechanicalCode::Buff),
        "BuffProMax" => Ok(FaceMechanicalCode::BuffProMax),
        "Coin" => Ok(FaceMechanicalCode::Coin),
        "Mark" => Ok(FaceMechanicalCode::Mark),
        "Miracle" => Ok(FaceMechanicalCode::Miracle),
        "Move" => Ok(FaceMechanicalCode::Move),
        "Replicate" => Ok(FaceMechanicalCode::Replicate),
        "SpecialType" => Ok(FaceMechanicalCode::SpecialType),
        _ => Err(GoldAndGearsEntryError::InvalidDiceFaceRuntime),
    }
}

fn scaled(value: &str) -> Result<i64, GoldAndGearsEntryError> {
    let (integer, fraction) = value.split_once('.').map_or((value, ""), |parts| parts);
    if fraction.len() > 6 {
        return Err(GoldAndGearsEntryError::InvalidDiceFaceRuntime);
    }
    let whole = integer
        .parse::<i64>()
        .map_err(|_| GoldAndGearsEntryError::InvalidDiceFaceRuntime)?;
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
        .ok_or(GoldAndGearsEntryError::InvalidDiceFaceRuntime)
}

#[derive(Deserialize)]
struct TargetPolicy {
    policy_id: Box<str>,
    selector_validation: Box<str>,
    candidate_order: Box<str>,
    operation_order: Box<str>,
    equal_priority_order: Box<str>,
    unpublished_empty_set_behavior: Box<str>,
    evidence_quality: Box<str>,
    replacement_condition: Box<str>,
}
