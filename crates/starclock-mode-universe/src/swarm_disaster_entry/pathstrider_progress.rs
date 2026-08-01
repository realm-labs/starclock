//! Pathstrider external progress, DLC unlock flags and chapter availability.

use std::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;
use starclock_activity::{
    ActivityCondition, ActivityExpression, ActivityOperation, ActivityProgramDefinition,
    ActivityProgramId, ActivitySlotId, ActivityTransactionState, ActivityValue,
};

use crate::{
    error::{UniverseCatalogLoadError, UniverseCatalogLoadErrorKind},
    swarm_disaster_unique::runtime_access::{
        SwarmChapterRuntimeInput, SwarmPathstriderRuntimeInput,
    },
};

use super::{SwarmDisasterRuntimeInstance, state::PROGRESSION};

const FINISH_PROGRESS_BASE: u64 = 0x4000_0000;
const UNLOCK_FLAG_BASE: u64 = 0x4100_0000;
const CHAPTER_AVAILABLE_BASE: u64 = 0x5000_0000;
const FINISH_PROGRAM_BASE: u32 = 0x5344_4200;
const CHAPTER_PROGRAM_BASE: u32 = 0x5344_4300;

#[derive(Clone, Debug)]
pub(super) struct PathstriderRuntimeCatalog {
    objectives: Box<[RuntimeObjective]>,
    finishes: Box<[RuntimeFinish]>,
    chapters: Box<[RuntimeChapter]>,
}

#[derive(Clone, Debug)]
struct RuntimeObjective {
    condition: Box<str>,
    cabinet: Box<str>,
}

#[derive(Clone, Debug)]
struct RuntimeFinish {
    id: u32,
    key: Box<str>,
    finish_type: Box<str>,
    comparison: Box<str>,
    parameters: Box<[Box<str>]>,
    target: u32,
    unlocks: Box<[RuntimeUnlock]>,
}

#[derive(Clone, Debug)]
struct RuntimeUnlock {
    id: u32,
    key: Box<str>,
    _flag: Box<str>,
}

#[derive(Clone, Debug)]
struct RuntimeChapter {
    id: u32,
    key: Box<str>,
    dimension_id: Option<u32>,
    layer: u8,
    threshold: Option<u16>,
    unresolved_bonus: bool,
}

impl PathstriderRuntimeCatalog {
    pub(super) fn compile(
        input: SwarmPathstriderRuntimeInput,
    ) -> Result<Self, UniverseCatalogLoadError> {
        let objectives = compile_objectives(&input)?;
        let (finishes, _, _) = compile_finishes(&input)?;
        let chapters = compile_chapters(&input.chapters)?;
        Ok(Self {
            objectives,
            finishes,
            chapters,
        })
    }

    fn objective_cabinet(&self, condition: &str) -> Result<&str, UniverseCatalogLoadError> {
        self.objectives
            .iter()
            .find(|objective| objective.condition.as_ref() == condition)
            .map(|objective| objective.cabinet.as_ref())
            .ok_or_else(|| reference("unknown Pathstrider external quest condition"))
    }

    fn compile_progress(
        &self,
        state: &ActivityTransactionState,
        condition: &str,
        observed_progress: u32,
    ) -> Result<Option<ActivityProgramDefinition>, UniverseCatalogLoadError> {
        let finish = self
            .finishes
            .iter()
            .find(|finish| finish.key.as_ref() == condition)
            .ok_or_else(|| reference("Pathstrider FinishWay is not enabled for Swarm"))?;
        let progress_key = finish_progress_key(finish.id);
        let current = counter_value(state, progress_key)?;
        let observed = i64::from(observed_progress);
        if observed < current {
            return Err(reference("Pathstrider progress cannot be revoked"));
        }
        let mut operations = Vec::new();
        if observed != current {
            operations.push(require_counter(progress_key, current));
            operations.push(add_counter(progress_key, observed - current));
        }
        if observed_progress >= finish.target {
            for unlock in &finish.unlocks {
                let key = unlock_flag_key(unlock.id);
                match counter_value(state, key)? {
                    0 => {
                        operations.push(require_counter(key, 0));
                        operations.push(add_counter(key, 1));
                    }
                    1 => {}
                    _ => return Err(invalid("invalid Pathstrider unlock flag state")),
                }
            }
        }
        if operations.is_empty() {
            return Ok(None);
        }
        program(FINISH_PROGRAM_BASE + finish.id, operations).map(Some)
    }

    fn finish_conditions(&self) -> impl ExactSizeIterator<Item = (&str, &str, &str, u32)> {
        self.finishes.iter().map(|finish| {
            (
                finish.key.as_ref(),
                finish.finish_type.as_ref(),
                finish.comparison.as_ref(),
                finish.target,
            )
        })
    }

    fn finish_parameters(&self, condition: &str) -> Option<impl ExactSizeIterator<Item = &str>> {
        self.finishes
            .iter()
            .find(|finish| finish.key.as_ref() == condition)
            .map(|finish| finish.parameters.iter().map(Box::as_ref))
    }

    fn unlock_applied(
        &self,
        state: &ActivityTransactionState,
        unlock_key: &str,
    ) -> Result<bool, UniverseCatalogLoadError> {
        let unlock = self
            .finishes
            .iter()
            .flat_map(|finish| finish.unlocks.iter())
            .find(|unlock| unlock.key.as_ref() == unlock_key)
            .ok_or_else(|| reference("Pathstrider unlock is not enabled for Swarm"))?;
        match counter_value(state, unlock_flag_key(unlock.id))? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(invalid("invalid Pathstrider unlock flag state")),
        }
    }

    fn compile_chapter_availability(
        &self,
        instance: &SwarmDisasterRuntimeInstance,
        state: &ActivityTransactionState,
    ) -> Result<Option<ActivityProgramDefinition>, UniverseCatalogLoadError> {
        let layer = instance
            .graph
            .node(state.current_node())
            .ok_or_else(|| invalid("current Swarm node is outside the graph"))?
            .section()
            .get();
        let layer = u8::try_from(layer)
            .ok()
            .filter(|layer| (1..=3).contains(layer))
            .ok_or_else(|| invalid("current Swarm node has an invalid plane"))?;
        let mut operations = Vec::new();
        for chapter in self
            .chapters
            .iter()
            .filter(|chapter| chapter.layer <= layer)
        {
            let key = chapter_available_key(chapter.id);
            match counter_value(state, key)? {
                1 => continue,
                0 => {}
                _ => return Err(invalid("invalid mechanical chapter availability state")),
            }
            if let (Some(dimension_id), Some(threshold)) = (chapter.dimension_id, chapter.threshold)
            {
                let points = communing_points(state, dimension_id)?;
                if points < i64::from(threshold) {
                    continue;
                }
                operations.push(require_communing(dimension_id, points));
            }
            operations.push(require_counter(key, 0));
            operations.push(add_counter(key, 1));
        }
        if operations.is_empty() {
            return Ok(None);
        }
        program(CHAPTER_PROGRAM_BASE + u32::from(layer), operations).map(Some)
    }

    fn chapters(&self) -> impl ExactSizeIterator<Item = (&str, u8, Option<(u32, u16)>, bool)> {
        self.chapters.iter().map(|chapter| {
            (
                chapter.key.as_ref(),
                chapter.layer,
                chapter.dimension_id.zip(chapter.threshold),
                chapter.unresolved_bonus,
            )
        })
    }

    fn chapter_available(
        &self,
        state: &ActivityTransactionState,
        key: &str,
    ) -> Result<bool, UniverseCatalogLoadError> {
        let chapter = self
            .chapters
            .iter()
            .find(|chapter| chapter.key.as_ref() == key)
            .ok_or_else(|| reference("unknown mechanical chapter"))?;
        match counter_value(state, chapter_available_key(chapter.id))? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(invalid("invalid mechanical chapter availability state")),
        }
    }

    #[cfg(test)]
    pub(super) fn denominators(&self) -> (usize, usize, usize, usize, usize) {
        (
            self.objectives.len(),
            self.finishes.len(),
            87,
            self.finishes.iter().map(|row| row.unlocks.len()).sum(),
            95,
        )
    }
}

impl SwarmDisasterRuntimeInstance {
    /// Completes the cabinet owned by one exact external quest condition.
    pub fn compile_pathstrider_objective_completion(
        &self,
        state: &ActivityTransactionState,
        external_condition: &str,
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        let cabinet = self.pathstrider.objective_cabinet(external_condition)?;
        let objective = external_condition
            .strip_prefix("swarm-disaster.external-quest-condition.")
            .ok_or_else(|| reference("invalid Pathstrider external quest condition"))?;
        self.communing
            .compile_cabinet_completion(state, cabinet, objective)
    }

    /// Enabled released FinishWay descriptors in stable condition order.
    pub fn pathstrider_finish_conditions(
        &self,
    ) -> impl ExactSizeIterator<Item = (&str, &str, &str, u32)> {
        self.pathstrider.finish_conditions()
    }

    /// Canonical locator parameters for one enabled FinishWay.
    pub fn pathstrider_finish_parameters(
        &self,
        condition: &str,
    ) -> Option<impl ExactSizeIterator<Item = &str>> {
        self.pathstrider.finish_parameters(condition)
    }

    /// Records nondecreasing external progress after an accepted Activity
    /// operation and commits every released non-revocable unlock exactly once.
    pub fn compile_pathstrider_progress(
        &self,
        state: &ActivityTransactionState,
        condition: &str,
        observed_progress: u32,
    ) -> Result<Option<ActivityProgramDefinition>, UniverseCatalogLoadError> {
        self.pathstrider
            .compile_progress(state, condition, observed_progress)
    }

    /// Whether one explicitly Swarm-enabled DLC unlock has committed.
    pub fn pathstrider_unlock_applied(
        &self,
        state: &ActivityTransactionState,
        unlock: &str,
    ) -> Result<bool, UniverseCatalogLoadError> {
        self.pathstrider.unlock_applied(state, unlock)
    }

    /// Chapter locators with plane/point thresholds and unresolved-bonus flag.
    pub fn mechanical_chapters(
        &self,
    ) -> impl ExactSizeIterator<Item = (&str, u8, Option<(u32, u16)>, bool)> {
        self.pathstrider.chapters()
    }

    /// Makes every newly eligible mechanical chapter available atomically.
    pub fn compile_mechanical_chapter_availability(
        &self,
        state: &ActivityTransactionState,
    ) -> Result<Option<ActivityProgramDefinition>, UniverseCatalogLoadError> {
        self.pathstrider.compile_chapter_availability(self, state)
    }

    /// Whether one mechanical chapter availability flag has committed.
    pub fn mechanical_chapter_available(
        &self,
        state: &ActivityTransactionState,
        chapter: &str,
    ) -> Result<bool, UniverseCatalogLoadError> {
        self.pathstrider.chapter_available(state, chapter)
    }
}

fn compile_objectives(
    input: &SwarmPathstriderRuntimeInput,
) -> Result<Box<[RuntimeObjective]>, UniverseCatalogLoadError> {
    let cabinets = input
        .cabinet_objectives
        .iter()
        .map(|row| (row.id, row))
        .collect::<BTreeMap<_, _>>();
    let mut objectives = Vec::with_capacity(input.objectives.len());
    let mut conditions = BTreeSet::new();
    for objective in &input.objectives {
        let cabinet = cabinets
            .get(&objective.cabinet_id)
            .ok_or_else(|| invalid("Pathstrider objective cabinet is missing"))?;
        let policy = serde_json::from_str::<ObjectivePolicy>(&objective.progress_policy)
            .map_err(|_| invalid("invalid Pathstrider objective policy"))?;
        let objective_id = objective
            .finish_key
            .strip_prefix("swarm-disaster.external-quest-condition.")
            .ok_or_else(|| invalid("invalid external Pathstrider objective"))?;
        let cabinet_key = cabinet
            .key
            .strip_prefix("swarm-disaster.pathstrider-cabinet.")
            .ok_or_else(|| invalid("invalid Pathstrider cabinet key"))?;
        if objective.id == 0
            || objective.key.as_ref()
                != format!("swarm-disaster.pathstrider-objective.{cabinet_key}")
            || cabinet.objective_id.as_ref() != objective_id
            || policy.source.as_ref() != "ExternalQuestCompletion"
            || policy.comparison.as_ref() != "Completed"
            || policy.update_boundary.as_ref() != "AfterAcceptedActivityOperation"
            || policy.once_scope.as_ref() != format!("PathstriderQuest:{objective_id}")
            || policy.description_parameters.is_empty()
            || !conditions.insert(objective.finish_key.as_ref())
        {
            return Err(invalid("Pathstrider external objective contract drift"));
        }
        objectives.push(RuntimeObjective {
            condition: objective.finish_key.clone(),
            cabinet: cabinet.key.clone(),
        });
    }
    objectives.sort_unstable_by(|left, right| left.condition.cmp(&right.condition));
    if objectives.len() != 31 || cabinets.len() != 31 {
        return Err(invalid("Pathstrider objective denominator drift"));
    }
    Ok(objectives.into_boxed_slice())
}

fn compile_finishes(
    input: &SwarmPathstriderRuntimeInput,
) -> Result<(Box<[RuntimeFinish]>, usize, usize), UniverseCatalogLoadError> {
    let unlocks = input
        .unlocks
        .iter()
        .map(|row| (row.key.as_ref(), row))
        .collect::<BTreeMap<_, _>>();
    let mut used = BTreeSet::new();
    let mut enabled = Vec::new();
    let mut disabled_finishes = 0;
    let mut disabled_unlocks = 0;
    for finish in &input.finishes {
        let target = positive_u32(&finish.target)?;
        let parameters = serde_json::from_str::<FinishParameters>(&finish.parameters)
            .map_err(|_| invalid("invalid Pathstrider FinishWay parameters"))?;
        let mut compiled_unlocks = Vec::new();
        for key in &finish.unlock_keys {
            let unlock = unlocks
                .get(key.as_ref())
                .ok_or_else(|| invalid("Pathstrider FinishWay unlock is missing"))?;
            let consequence = serde_json::from_str::<UnlockConsequence>(&unlock.consequence)
                .map_err(|_| invalid("invalid Pathstrider unlock consequence"))?;
            let suffix = unlock
                .key
                .strip_prefix("swarm-disaster.pathstrider-unlock.")
                .ok_or_else(|| invalid("invalid Pathstrider unlock key"))?;
            if unlock.finish_id != finish.id
                || consequence.operation.as_ref() != "SetDlcUnlockFlag"
                || consequence.once_scope.as_ref() != format!("DlcUnlock:{suffix}")
                || consequence.unlock_flag_id.as_ref()
                    != format!("swarm-disaster.dlc-unlock-flag.{suffix}")
                || consequence.revocable
                || consequence.enabled_for_swarm_compilation != finish.enabled
                || !used.insert(unlock.id)
            {
                return Err(invalid("Pathstrider unlock contract drift"));
            }
            if finish.enabled {
                compiled_unlocks.push(RuntimeUnlock {
                    id: unlock.id,
                    key: unlock.key.clone(),
                    _flag: consequence.unlock_flag_id,
                });
            } else {
                disabled_unlocks += 1;
            }
        }
        if finish.enabled {
            validate_enabled_finish(finish, &parameters, target)?;
            enabled.push(RuntimeFinish {
                id: finish.id,
                key: finish.key.clone(),
                finish_type: finish.finish_type.clone(),
                comparison: finish.comparison.clone(),
                parameters: parameters.flatten()?,
                target,
                unlocks: compiled_unlocks.into_boxed_slice(),
            });
        } else {
            disabled_finishes += 1;
        }
    }
    enabled.sort_unstable_by_key(|finish| finish.id);
    if input.finishes.len() != 102
        || input.unlocks.len() != 110
        || used.len() != 110
        || enabled.len() != 15
        || disabled_finishes != 87
        || enabled.iter().map(|row| row.unlocks.len()).sum::<usize>() != 15
        || disabled_unlocks != 95
    {
        return Err(invalid("Pathstrider FinishWay denominator drift"));
    }
    Ok((
        enabled.into_boxed_slice(),
        disabled_finishes,
        disabled_unlocks,
    ))
}

fn validate_enabled_finish(
    finish: &crate::swarm_disaster_unique::runtime_access::SwarmPathstriderFinishRuntimeInput,
    parameters: &FinishParameters,
    target: u32,
) -> Result<(), UniverseCatalogLoadError> {
    let valid = match (finish.finish_type.as_ref(), finish.comparison.as_ref()) {
        ("RogueDLCFinishCnt", "GreaterEqual") => !parameters.integer.is_empty(),
        ("FinishMission" | "FinishQuest" | "RogueDLCFinishMainStory", "ListContain") => {
            parameters.integers.len() == 1 && target == 1
        }
        ("RogueFinishUnlock", "ListContain") => parameters.integers.len() == 6 && target == 5,
        ("RogueDLCFinishMainStoryCnt", "NoPara") => {
            parameters.integer.is_empty() && parameters.integers.is_empty() && target == 13
        }
        _ => false,
    };
    if valid && parameters.items.is_empty() && parameters.text.is_empty() {
        Ok(())
    } else {
        Err(invalid("enabled Pathstrider FinishWay contract drift"))
    }
}

fn compile_chapters(
    input: &[SwarmChapterRuntimeInput],
) -> Result<Box<[RuntimeChapter]>, UniverseCatalogLoadError> {
    let mut chapters = Vec::with_capacity(input.len());
    let mut unresolved = 0;
    for chapter in input {
        let unlock = serde_json::from_str::<ChapterUnlock>(&chapter.mechanical_unlock)
            .map_err(|_| invalid("invalid mechanical chapter program"))?;
        let threshold = chapter.threshold.as_deref().map(positive_u16).transpose()?;
        if chapter.id == 0
            || !(1..=3).contains(&chapter.layer)
            || chapter.dimension_id.is_some() != threshold.is_some()
            || chapter
                .dimension_id
                .is_some_and(|id| !(1..=7).contains(&id))
            || threshold.is_some_and(|value| value > 20)
            || unlock.operation.as_ref() != "MakeMechanicalChapterAvailable"
            || unlock.chapter_id != chapter.key
            || !matches!(
                unlock.presentation_toast_type.as_ref(),
                "" | "Effect" | "Buff"
            )
            || !unlock.bonus_payload.is_empty()
            || (unlock.bonus_declared
                && unlock.simulation_payload_status.as_ref() != "UnresolvedFailClosed")
            || (!unlock.bonus_declared
                && unlock.simulation_payload_status.as_ref() != "ChapterAvailabilityOnly")
        {
            return Err(invalid("mechanical chapter contract drift"));
        }
        unresolved += usize::from(unlock.bonus_declared);
        chapters.push(RuntimeChapter {
            id: chapter.id,
            key: chapter.key.clone(),
            dimension_id: chapter.dimension_id,
            layer: chapter.layer,
            threshold,
            unresolved_bonus: unlock.bonus_declared,
        });
    }
    chapters.sort_unstable_by_key(|chapter| (chapter.layer, chapter.id));
    if chapters.len() != 13 || unresolved != 3 {
        return Err(invalid("mechanical chapter denominator drift"));
    }
    Ok(chapters.into_boxed_slice())
}

fn finish_progress_key(id: u32) -> u64 {
    FINISH_PROGRESS_BASE + u64::from(id)
}
fn unlock_flag_key(id: u32) -> u64 {
    UNLOCK_FLAG_BASE + u64::from(id)
}
fn chapter_available_key(id: u32) -> u64 {
    CHAPTER_AVAILABLE_BASE + u64::from(id)
}
fn counter_value(
    state: &ActivityTransactionState,
    key: u64,
) -> Result<i64, UniverseCatalogLoadError> {
    match state.slot(slot(PROGRESSION)) {
        Some(ActivityValue::BoundedCounterMap(values)) => Ok(values
            .binary_search_by_key(&key, |(candidate, _)| *candidate)
            .ok()
            .map_or(0, |index| values[index].1)),
        _ => Err(invalid("invalid Pathstrider progression slot")),
    }
}
fn communing_points(
    state: &ActivityTransactionState,
    dimension_id: u32,
) -> Result<i64, UniverseCatalogLoadError> {
    match state.slot(slot(super::state::COMMUNING)) {
        Some(ActivityValue::BoundedCounterMap(values)) => Ok(values
            .binary_search_by_key(&u64::from(dimension_id), |(candidate, _)| *candidate)
            .ok()
            .map_or(0, |index| values[index].1)),
        _ => Err(invalid("invalid Communing dimension slot")),
    }
}
fn require_counter(key: u64, value: i64) -> ActivityOperation {
    ActivityOperation::Require(ActivityCondition::Equal(counter(key), integer(value)))
}
fn require_communing(dimension_id: u32, value: i64) -> ActivityOperation {
    ActivityOperation::Require(ActivityCondition::Equal(
        ActivityExpression::CounterValue {
            slot: slot(super::state::COMMUNING),
            key: u64::from(dimension_id),
        },
        integer(value),
    ))
}
fn add_counter(key: u64, delta: i64) -> ActivityOperation {
    ActivityOperation::AddCounter {
        slot: slot(PROGRESSION),
        key,
        delta: integer(delta),
    }
}
fn counter(key: u64) -> ActivityExpression {
    ActivityExpression::CounterValue {
        slot: slot(PROGRESSION),
        key,
    }
}
fn integer(value: i64) -> ActivityExpression {
    ActivityExpression::Literal(ActivityValue::BoundedInteger(value))
}
fn slot(raw: u32) -> ActivitySlotId {
    ActivitySlotId::new(raw).expect("static Swarm slot ID is non-zero")
}
fn program(
    raw: u32,
    operations: Vec<ActivityOperation>,
) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
    ActivityProgramDefinition::new(
        ActivityProgramId::new(raw).expect("static Swarm program ID is non-zero"),
        operations,
    )
    .map_err(|_| invalid("invalid Pathstrider Activity program"))
}
fn positive_u32(value: &str) -> Result<u32, UniverseCatalogLoadError> {
    value
        .parse::<u32>()
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| invalid("invalid positive Pathstrider integer"))
}
fn positive_u16(value: &str) -> Result<u16, UniverseCatalogLoadError> {
    value
        .parse::<u16>()
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| invalid("invalid positive chapter threshold"))
}
fn invalid(message: &'static str) -> UniverseCatalogLoadError {
    UniverseCatalogLoadError::new(UniverseCatalogLoadErrorKind::InvalidDefinition, message)
}
fn reference(message: &'static str) -> UniverseCatalogLoadError {
    UniverseCatalogLoadError::new(UniverseCatalogLoadErrorKind::InvalidReference, message)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ObjectivePolicy {
    comparison: Box<str>,
    description_parameters: Box<[Box<str>]>,
    once_scope: Box<str>,
    source: Box<str>,
    update_boundary: Box<str>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FinishParameters {
    integer: Box<str>,
    integers: Box<[Box<str>]>,
    items: Box<[serde_json::Value]>,
    text: Box<str>,
}
impl FinishParameters {
    fn flatten(&self) -> Result<Box<[Box<str>]>, UniverseCatalogLoadError> {
        let mut values = Vec::new();
        if !self.integer.is_empty() {
            values.push(self.integer.clone());
        }
        values.extend(self.integers.iter().cloned());
        if self.items.is_empty() && self.text.is_empty() {
            Ok(values.into_boxed_slice())
        } else {
            Err(invalid("unsupported enabled FinishWay parameter kind"))
        }
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct UnlockConsequence {
    enabled_for_swarm_compilation: bool,
    once_scope: Box<str>,
    operation: Box<str>,
    revocable: bool,
    unlock_flag_id: Box<str>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ChapterUnlock {
    bonus_declared: bool,
    bonus_payload: Box<str>,
    chapter_id: Box<str>,
    operation: Box<str>,
    presentation_toast_type: Box<str>,
    simulation_payload_status: Box<str>,
}
