//! Communing choice counters, dimensions, and Pathstrider cabinet rewards.

use std::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;
use starclock_activity::{
    ActivityCondition, ActivityExpression, ActivityOperation, ActivityProgramDefinition,
    ActivityProgramId, ActivitySlotId, ActivityTransactionState, ActivityValue,
};

use crate::{
    error::{UniverseCatalogLoadError, UniverseCatalogLoadErrorKind},
    swarm_disaster_unique::runtime_access::{
        SwarmCabinetRuntimeInput, SwarmCommuningAdjustmentRuntimeInput,
        SwarmCommuningChoiceRuntimeInput, SwarmCommuningDimensionRuntimeInput,
        SwarmCommuningRuntimeInput,
    },
};

use super::state::{COMMUNING, COMMUNING_CHOICE, PROGRESSION};

#[path = "communing_validation.rs"]
mod validation;

const CHOICE_PROGRAM_BASE: u32 = 0x5360_0000;
const CABINET_PROGRAM_BASE: u32 = 0x5370_0000;
const CHOICE_ROW_BASE: u64 = 0x0001_0000;
const CHOICE_STAGE_BASE: u64 = 0x0002_0000;
const CHOICE_AEON_BASE: u64 = 0x0003_0000;
const CABINET_PROGRESS_BASE: u64 = 0x2000_0000;

#[derive(Clone, Debug)]
pub(super) struct CommuningRuntimeCatalog {
    choices: Box<[RuntimeChoice]>,
    dimensions: Box<[RuntimeDimension]>,
    cabinets: Box<[RuntimeCabinet]>,
}

#[derive(Clone, Debug)]
struct RuntimeChoice {
    id: u32,
    key: Box<str>,
    source_id: u32,
    story_stage: u16,
    aeon_id: u32,
    rogue_npc_id: u32,
}

#[derive(Clone, Debug)]
struct RuntimeDimension {
    id: u32,
    key: Box<str>,
    shared_path: Box<str>,
    maximum: i64,
}

#[derive(Clone, Debug)]
struct RuntimeCabinet {
    id: u32,
    key: Box<str>,
    source_id: u32,
    sort: u16,
    kind: CabinetKind,
    objective_id: Box<str>,
    prerequisites: Box<[u32]>,
    unlocks: Box<[u32]>,
    adjustments: Box<[RuntimeAdjustment]>,
    description_parameters: Box<[i64]>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct RuntimeAdjustment {
    id: u32,
    ordinal: u16,
    dimension_id: u32,
    delta: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CabinetKind {
    Normal,
    Hidden,
}

impl CommuningRuntimeCatalog {
    pub(super) fn compile(
        input: SwarmCommuningRuntimeInput,
    ) -> Result<Self, UniverseCatalogLoadError> {
        let mut dimensions = input
            .dimensions
            .iter()
            .map(compile_dimension)
            .collect::<Result<Vec<_>, _>>()?;
        dimensions.sort_unstable_by_key(|dimension| dimension.id);
        validate_dimensions(&dimensions)?;

        let mut choices = input
            .choices
            .iter()
            .map(|choice| compile_choice(choice, &dimensions))
            .collect::<Result<Vec<_>, _>>()?;
        choices.sort_unstable_by_key(|choice| (choice.story_stage, choice.aeon_id, choice.id));
        validate_choices(&choices)?;

        let cabinet_ids = input
            .cabinets
            .iter()
            .map(|cabinet| (cabinet.key.as_ref(), cabinet.id))
            .collect::<BTreeMap<_, _>>();
        if cabinet_ids.len() != input.cabinets.len() {
            return Err(invalid("duplicate Swarm Pathstrider cabinet key"));
        }
        let grouped_adjustments = group_adjustments(&input.adjustments, &dimensions)?;
        let mut cabinets = input
            .cabinets
            .iter()
            .map(|cabinet| {
                compile_cabinet(cabinet, &cabinet_ids, &grouped_adjustments, &dimensions)
            })
            .collect::<Result<Vec<_>, _>>()?;
        cabinets.sort_unstable_by_key(|cabinet| (cabinet.sort, cabinet.id));
        validation::validate_cabinets(&cabinets)?;

        Ok(Self {
            choices: choices.into_boxed_slice(),
            dimensions: dimensions.into_boxed_slice(),
            cabinets: cabinets.into_boxed_slice(),
        })
    }

    pub(super) fn choices(&self, story_stage: u16) -> impl Iterator<Item = &str> {
        self.choices
            .iter()
            .filter(move |choice| choice.story_stage == story_stage)
            .map(|choice| choice.key.as_ref())
    }

    pub(super) fn choice_available(
        &self,
        state: &ActivityTransactionState,
        story_stage: u16,
        key: &str,
    ) -> Result<bool, UniverseCatalogLoadError> {
        let choice = self.choice(story_stage, key)?;
        let stage = counter_value(state, COMMUNING_CHOICE, stage_key(story_stage))?;
        let row = counter_value(state, COMMUNING_CHOICE, choice_key(choice.id))?;
        if stage < 0 || row < 0 || ![0, 1].contains(&row) {
            return Err(invalid("invalid Swarm Communing choice state"));
        }
        Ok(stage == 0 && row == 0)
    }

    pub(super) fn compile_choice(
        &self,
        state: &ActivityTransactionState,
        story_stage: u16,
        key: &str,
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        let choice = self.choice(story_stage, key)?;
        if !self.choice_available(state, story_stage, key)? {
            return Err(reference("Swarm Communing story stage is already resolved"));
        }
        let aeon_key = aeon_key(choice.aeon_id);
        let count = counter_value(state, COMMUNING_CHOICE, aeon_key)?;
        let next = count
            .checked_add(1)
            .ok_or_else(|| invalid("Swarm Communing choice counter overflow"))?;
        ActivityProgramDefinition::new(
            program_id(CHOICE_PROGRAM_BASE, choice.id)?,
            vec![
                require_counter(COMMUNING_CHOICE, stage_key(story_stage), 0),
                require_counter(COMMUNING_CHOICE, choice_key(choice.id), 0),
                set_counter(COMMUNING_CHOICE, choice_key(choice.id), 1),
                set_counter(
                    COMMUNING_CHOICE,
                    stage_key(story_stage),
                    i64::from(choice.id),
                ),
                require_counter(COMMUNING_CHOICE, aeon_key, count),
                set_counter(COMMUNING_CHOICE, aeon_key, next),
            ],
        )
        .map_err(|_| invalid("invalid Swarm Communing choice program"))
    }

    pub(super) fn choice_count(
        &self,
        state: &ActivityTransactionState,
        shared_path: &str,
    ) -> Result<i64, UniverseCatalogLoadError> {
        let dimension = self.dimension(shared_path)?;
        counter_value(state, COMMUNING_CHOICE, aeon_key(dimension.id))
    }

    pub(super) fn dimension_points(
        &self,
        state: &ActivityTransactionState,
        key: &str,
    ) -> Result<i64, UniverseCatalogLoadError> {
        let dimension = self.dimension(key)?;
        let value = counter_value(state, COMMUNING, u64::from(dimension.id))?;
        if !(0..=dimension.maximum).contains(&value) {
            return Err(invalid("invalid Swarm Communing dimension state"));
        }
        Ok(value)
    }

    pub(super) fn dimension_maximum(&self, key: &str) -> Option<i64> {
        self.dimensions
            .iter()
            .find(|dimension| {
                dimension.key.as_ref() == key || dimension.shared_path.as_ref() == key
            })
            .map(|dimension| dimension.maximum)
    }

    pub(super) fn available_cabinets<'a>(
        &'a self,
        state: &ActivityTransactionState,
    ) -> Result<Box<[&'a str]>, UniverseCatalogLoadError> {
        self.cabinets
            .iter()
            .filter_map(
                |cabinet| match self.cabinet_available_definition(state, cabinet) {
                    Ok(true) => Some(Ok(cabinet.key.as_ref())),
                    Ok(false) => None,
                    Err(error) => Some(Err(error)),
                },
            )
            .collect::<Result<Vec<_>, _>>()
            .map(Vec::into_boxed_slice)
    }

    pub(super) fn cabinet_available(
        &self,
        state: &ActivityTransactionState,
        key: &str,
    ) -> Result<bool, UniverseCatalogLoadError> {
        self.cabinet_available_definition(state, self.cabinet(key)?)
    }

    pub(super) fn compile_cabinet_completion(
        &self,
        state: &ActivityTransactionState,
        key: &str,
        completed_objective: &str,
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        let cabinet = self.cabinet(key)?;
        if cabinet.objective_id.as_ref() != completed_objective {
            return Err(reference("objective does not authorize this Swarm cabinet"));
        }
        if !self.cabinet_available_definition(state, cabinet)? {
            return Err(reference("Swarm Pathstrider cabinet is not eligible"));
        }
        let mut operations = Vec::with_capacity(4 + cabinet.adjustments.len() * 2);
        operations.push(require_counter(PROGRESSION, cabinet_key(cabinet.id), 0));
        for prerequisite in &cabinet.prerequisites {
            operations.push(require_counter(PROGRESSION, cabinet_key(*prerequisite), 1));
        }
        operations.push(set_counter(PROGRESSION, cabinet_key(cabinet.id), 1));
        let mut working = self
            .dimensions
            .iter()
            .map(|dimension| {
                counter_value(state, COMMUNING, u64::from(dimension.id))
                    .map(|value| (dimension.id, value))
            })
            .collect::<Result<BTreeMap<_, _>, _>>()?;
        for adjustment in &cabinet.adjustments {
            let dimension = self
                .dimensions
                .iter()
                .find(|dimension| dimension.id == adjustment.dimension_id)
                .ok_or_else(|| invalid("unknown Swarm adjustment dimension"))?;
            let current = *working
                .get(&dimension.id)
                .ok_or_else(|| invalid("missing Swarm adjustment dimension"))?;
            if !(0..=dimension.maximum).contains(&current) {
                return Err(invalid("invalid Swarm Communing dimension state"));
            }
            let next = current
                .checked_add(adjustment.delta)
                .map(|value| value.min(dimension.maximum))
                .ok_or_else(|| invalid("Swarm Communing point overflow"))?;
            operations.push(require_counter(COMMUNING, u64::from(dimension.id), current));
            operations.push(set_counter(COMMUNING, u64::from(dimension.id), next));
            working.insert(dimension.id, next);
        }
        ActivityProgramDefinition::new(program_id(CABINET_PROGRAM_BASE, cabinet.id)?, operations)
            .map_err(|_| invalid("invalid Swarm cabinet completion program"))
    }

    pub(super) fn cabinet_objective(&self, key: &str) -> Option<&str> {
        self.cabinets
            .iter()
            .find(|cabinet| cabinet.key.as_ref() == key)
            .map(|cabinet| cabinet.objective_id.as_ref())
    }

    pub(super) fn cabinet_prerequisites(
        &self,
        key: &str,
    ) -> Option<impl ExactSizeIterator<Item = &str>> {
        let cabinet = self
            .cabinets
            .iter()
            .find(|cabinet| cabinet.key.as_ref() == key)?;
        Some(cabinet.prerequisites.iter().map(|id| {
            self.cabinets
                .iter()
                .find(|candidate| candidate.id == *id)
                .expect("validated cabinet prerequisite exists")
                .key
                .as_ref()
        }))
    }

    fn choice(
        &self,
        story_stage: u16,
        key: &str,
    ) -> Result<&RuntimeChoice, UniverseCatalogLoadError> {
        self.choices
            .iter()
            .find(|choice| choice.story_stage == story_stage && choice.key.as_ref() == key)
            .ok_or_else(|| reference("unknown Swarm Communing choice for story stage"))
    }

    fn dimension(&self, key: &str) -> Result<&RuntimeDimension, UniverseCatalogLoadError> {
        self.dimensions
            .iter()
            .find(|dimension| {
                dimension.key.as_ref() == key || dimension.shared_path.as_ref() == key
            })
            .ok_or_else(|| reference("unknown Swarm Communing dimension"))
    }

    fn cabinet(&self, key: &str) -> Result<&RuntimeCabinet, UniverseCatalogLoadError> {
        self.cabinets
            .iter()
            .find(|cabinet| cabinet.key.as_ref() == key)
            .ok_or_else(|| reference("unknown Swarm Pathstrider cabinet"))
    }

    fn cabinet_available_definition(
        &self,
        state: &ActivityTransactionState,
        cabinet: &RuntimeCabinet,
    ) -> Result<bool, UniverseCatalogLoadError> {
        let completed = counter_value(state, PROGRESSION, cabinet_key(cabinet.id))?;
        if ![0, 1].contains(&completed) {
            return Err(invalid("invalid Swarm cabinet completion state"));
        }
        if completed == 1 {
            return Ok(false);
        }
        for prerequisite in &cabinet.prerequisites {
            match counter_value(state, PROGRESSION, cabinet_key(*prerequisite))? {
                1 => {}
                0 => return Ok(false),
                _ => return Err(invalid("invalid Swarm cabinet prerequisite state")),
            }
        }
        Ok(true)
    }

    #[cfg(test)]
    pub(super) fn denominators(&self) -> (usize, usize, usize, usize, usize, usize) {
        (
            self.choices.len(),
            self.dimensions.len(),
            self.cabinets.len(),
            self.cabinets
                .iter()
                .map(|cabinet| cabinet.adjustments.len())
                .sum(),
            self.cabinets
                .iter()
                .map(|cabinet| cabinet.prerequisites.len())
                .sum(),
            self.cabinets
                .iter()
                .map(|cabinet| cabinet.description_parameters.len())
                .sum(),
        )
    }
}

#[derive(Deserialize)]
struct ChoiceEligibility {
    branch_available: bool,
    story_stage: u16,
}

#[derive(Deserialize)]
struct ChoiceOperation {
    order: u16,
    operation: String,
    counter_id: String,
    delta: String,
    once_scope: String,
}

#[derive(Deserialize)]
struct PointDelta {
    dimension_id: String,
    delta: String,
}

fn compile_dimension(
    input: &SwarmCommuningDimensionRuntimeInput,
) -> Result<RuntimeDimension, UniverseCatalogLoadError> {
    if input.maximum != 20
        || input.key.as_ref() != format!("swarm-disaster.communing-dimension.{}", input.id)
    {
        return Err(invalid("Swarm Communing dimension policy drift"));
    }
    Ok(RuntimeDimension {
        id: input.id,
        key: input.key.clone(),
        shared_path: input.shared_path.clone(),
        maximum: i64::from(input.maximum),
    })
}

fn validate_dimensions(dimensions: &[RuntimeDimension]) -> Result<(), UniverseCatalogLoadError> {
    if dimensions.len() != 7
        || dimensions
            .iter()
            .map(|dimension| dimension.id)
            .collect::<Vec<_>>()
            != (1..=7).collect::<Vec<_>>()
        || dimensions
            .iter()
            .map(|dimension| dimension.key.as_ref())
            .collect::<BTreeSet<_>>()
            .len()
            != 7
        || dimensions
            .iter()
            .map(|dimension| dimension.shared_path.as_ref())
            .collect::<BTreeSet<_>>()
            .len()
            != 7
    {
        return Err(invalid("Swarm Communing dimension denominator drift"));
    }
    Ok(())
}

fn compile_choice(
    input: &SwarmCommuningChoiceRuntimeInput,
    dimensions: &[RuntimeDimension],
) -> Result<RuntimeChoice, UniverseCatalogLoadError> {
    let source_id = canonical_u32(&input.source_id)?;
    let aeon_id = canonical_u32(&input.aeon_id)?;
    let rogue_npc_id = canonical_u32(&input.rogue_npc_id)?;
    let eligibility = serde_json::from_str::<ChoiceEligibility>(&input.eligibility)
        .map_err(|_| invalid("invalid Swarm Communing choice eligibility"))?;
    let point_deltas = serde_json::from_str::<Vec<PointDelta>>(&input.point_deltas)
        .map_err(|_| invalid("invalid Swarm Communing choice point deltas"))?;
    let operations = serde_json::from_str::<Vec<ChoiceOperation>>(&input.operations)
        .map_err(|_| invalid("invalid Swarm Communing choice operations"))?;
    let operation = operations
        .first()
        .filter(|_| operations.len() == 1)
        .ok_or_else(|| invalid("Swarm Communing choice operation denominator drift"))?;
    if input.key.as_ref() != format!("swarm-disaster.communing-choice.{source_id}")
        || ![4, 6, 7].contains(&input.story_stage)
        || eligibility.story_stage != input.story_stage
        || !eligibility.branch_available
        || !point_deltas.is_empty()
        || !(1..=7).contains(&aeon_id)
        || dimensions
            .iter()
            .find(|dimension| dimension.id == aeon_id)
            .is_none_or(|dimension| dimension.shared_path != input.shared_path)
        || operation.order != 0
        || operation.operation != "IncrementAeonChoiceCounter"
        || operation.counter_id != format!("swarm-disaster.aeon-choice-counter.{aeon_id}")
        || operation.delta != "1"
        || operation.once_scope != format!("MainStoryBranch:{source_id}")
        || rogue_npc_id == 0
    {
        return Err(invalid("Swarm Communing choice policy drift"));
    }
    Ok(RuntimeChoice {
        id: input.id,
        key: input.key.clone(),
        source_id,
        story_stage: input.story_stage,
        aeon_id,
        rogue_npc_id,
    })
}

fn validate_choices(choices: &[RuntimeChoice]) -> Result<(), UniverseCatalogLoadError> {
    if choices.len() != 21
        || choices
            .iter()
            .map(|choice| (choice.story_stage, choice.aeon_id))
            .collect::<BTreeSet<_>>()
            .len()
            != 21
        || choices
            .iter()
            .map(|choice| choice.source_id)
            .collect::<BTreeSet<_>>()
            .len()
            != 21
        || choices
            .iter()
            .map(|choice| choice.rogue_npc_id)
            .collect::<BTreeSet<_>>()
            .len()
            != 21
    {
        return Err(invalid("Swarm Communing choice denominator drift"));
    }
    Ok(())
}

fn group_adjustments(
    input: &[SwarmCommuningAdjustmentRuntimeInput],
    dimensions: &[RuntimeDimension],
) -> Result<BTreeMap<u32, Vec<RuntimeAdjustment>>, UniverseCatalogLoadError> {
    let mut grouped = BTreeMap::<u32, Vec<RuntimeAdjustment>>::new();
    let mut ids = BTreeSet::new();
    for row in input {
        let source_id = canonical_u32(&row.source_id)?;
        let delta = canonical_i64(&row.delta)?;
        if row.key.as_ref()
            != format!(
                "swarm-disaster.communing-adjustment.cabinet.{source_id}.{}",
                row.ordinal
            )
            || row.source_kind.as_ref() != "PathstriderCabinet"
            || delta <= 0
            || !ids.insert(row.id)
            || dimensions
                .iter()
                .all(|dimension| dimension.id != row.dimension_id)
        {
            return Err(invalid("Swarm Communing adjustment policy drift"));
        }
        grouped
            .entry(source_id)
            .or_default()
            .push(RuntimeAdjustment {
                id: row.id,
                ordinal: row.ordinal,
                dimension_id: row.dimension_id,
                delta,
            });
    }
    if input.len() != 55 || grouped.len() != 31 {
        return Err(invalid("Swarm Communing adjustment denominator drift"));
    }
    for rows in grouped.values_mut() {
        rows.sort_unstable_by_key(|row| (row.ordinal, row.id));
        if rows
            .iter()
            .enumerate()
            .any(|(index, row)| usize::from(row.ordinal) != index)
        {
            return Err(invalid("Swarm Communing adjustment order drift"));
        }
    }
    Ok(grouped)
}

fn compile_cabinet(
    input: &SwarmCabinetRuntimeInput,
    ids: &BTreeMap<&str, u32>,
    grouped: &BTreeMap<u32, Vec<RuntimeAdjustment>>,
    dimensions: &[RuntimeDimension],
) -> Result<RuntimeCabinet, UniverseCatalogLoadError> {
    let source_id = canonical_u32(&input.source_id)?;
    let kind = match input.cabinet_type.as_ref() {
        "Normal" => CabinetKind::Normal,
        "Hide" => CabinetKind::Hidden,
        _ => return Err(invalid("unknown Swarm cabinet kind")),
    };
    let prerequisites = map_cabinet_keys(&input.prerequisite_keys, ids)?;
    let unlocks = map_cabinet_keys(&input.unlock_keys, ids)?;
    let point_deltas = serde_json::from_str::<Vec<PointDelta>>(&input.point_deltas)
        .map_err(|_| invalid("invalid Swarm cabinet point deltas"))?;
    let adjustments = grouped
        .get(&source_id)
        .ok_or_else(|| invalid("missing Swarm cabinet adjustments"))?;
    if input.key.as_ref() != format!("swarm-disaster.pathstrider-cabinet.{source_id}")
        || point_deltas.len() != adjustments.len()
        || point_deltas
            .iter()
            .zip(adjustments)
            .any(|(point, adjustment)| {
                dimensions
                    .iter()
                    .find(|dimension| dimension.id == adjustment.dimension_id)
                    .is_none_or(|dimension| dimension.key.as_ref() != point.dimension_id)
                    || canonical_i64(&point.delta).ok() != Some(adjustment.delta)
            })
    {
        return Err(invalid("Swarm cabinet adjustment projection drift"));
    }
    Ok(RuntimeCabinet {
        id: input.id,
        key: input.key.clone(),
        source_id,
        sort: input.sort,
        kind,
        objective_id: input.objective_id.clone(),
        prerequisites,
        unlocks,
        adjustments: adjustments.clone().into_boxed_slice(),
        description_parameters: input
            .description_parameters
            .iter()
            .map(|value| canonical_i64(value))
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
    })
}

fn map_cabinet_keys(
    keys: &[Box<str>],
    ids: &BTreeMap<&str, u32>,
) -> Result<Box<[u32]>, UniverseCatalogLoadError> {
    let values = keys
        .iter()
        .map(|key| {
            ids.get(key.as_ref())
                .copied()
                .ok_or_else(|| invalid("unknown Swarm cabinet edge"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    if values.iter().collect::<BTreeSet<_>>().len() != values.len() {
        return Err(invalid("duplicate Swarm cabinet edge"));
    }
    Ok(values.into_boxed_slice())
}

fn canonical_u32(value: &str) -> Result<u32, UniverseCatalogLoadError> {
    canonical_i64(value)
        .ok()
        .and_then(|value| u32::try_from(value).ok())
        .filter(|value| *value != 0)
        .ok_or_else(|| invalid("invalid positive Swarm Communing integer"))
}

fn canonical_i64(value: &str) -> Result<i64, UniverseCatalogLoadError> {
    let unsigned = value.strip_prefix('-').unwrap_or(value);
    let canonical = unsigned == "0"
        || (!unsigned.starts_with('0') && unsigned.bytes().all(|byte| byte.is_ascii_digit()));
    if value.is_empty() || value.starts_with('+') || value == "-0" || !canonical {
        return Err(invalid("non-canonical Swarm Communing integer"));
    }
    value
        .parse::<i64>()
        .map_err(|_| invalid("invalid Swarm Communing integer"))
}

const fn choice_key(id: u32) -> u64 {
    CHOICE_ROW_BASE + id as u64
}

const fn stage_key(stage: u16) -> u64 {
    CHOICE_STAGE_BASE + stage as u64
}

const fn aeon_key(id: u32) -> u64 {
    CHOICE_AEON_BASE + id as u64
}

const fn cabinet_key(id: u32) -> u64 {
    CABINET_PROGRESS_BASE + id as u64
}

fn program_id(base: u32, id: u32) -> Result<ActivityProgramId, UniverseCatalogLoadError> {
    base.checked_add(id)
        .and_then(ActivityProgramId::new)
        .ok_or_else(|| invalid("invalid Swarm Communing program ID"))
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

fn integer(value: i64) -> ActivityExpression {
    ActivityExpression::Literal(ActivityValue::BoundedInteger(value))
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
        _ => Err(invalid("invalid Swarm Communing state slot")),
    }
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
