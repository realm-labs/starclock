//! Terminal rule bindings and atomic execution plans for Occurrence choices.

use starclock_activity::{
    ActivityCondition, ActivityExpression, ActivityOperation, ActivityProgramDefinition,
    ActivityProgramId, ActivitySlotId, ActivityValue,
};

use crate::{digest::Encoder, gold_gears_content::GoldAndGearsContentCatalog};

use super::{
    GoldAndGearsEntryError,
    api::{GoldAndGearsRuntimeFactory, GoldAndGearsRuntimeInstance},
    occurrence_types::{
        GoldAndGearsOccurrenceChoice, GoldAndGearsOccurrenceChoiceId, GoldAndGearsOccurrenceEffect,
        GoldAndGearsOccurrenceEffectPhase, GoldAndGearsOccurrenceExecutionPlan,
        GoldAndGearsOccurrenceRuleAccuracy, GoldAndGearsOccurrenceRuleBinding,
        GoldAndGearsOccurrenceRuleKind, GoldAndGearsOccurrenceRuleOwnership,
        GoldAndGearsOccurrenceSelection,
    },
    state_layout::DEFERRED_EFFECTS_SLOT,
};

const OCCURRENCE_PROGRAM_BASE: u32 = 0x4B00_0000;
const OCCURRENCE_APPLIED_BASE: u64 = 0x4747_4500_0000_0000;
const OCCURRENCE_COST_BASE: u64 = 0x4747_4600_0000_0000;
const OCCURRENCE_OUTCOME_BASE: u64 = 0x4747_4700_0000_0000;
const OCCURRENCE_SELECTION_BASE: u64 = 0x4747_4800_0000_0000;
const COST_KEY_STRIDE: u64 = 16;

pub(super) fn compile_rule_runtime(
    content: &GoldAndGearsContentCatalog,
    choices: &[GoldAndGearsOccurrenceChoice],
) -> Result<(Box<[GoldAndGearsOccurrenceRuleBinding]>, [u8; 32]), GoldAndGearsEntryError> {
    let mut bindings = Vec::with_capacity(384);
    for occurrence in &content.occurrences {
        bindings.push(binding(
            occurrence.rule.as_str(),
            occurrence.key.as_str(),
            GoldAndGearsOccurrenceRuleKind::Occurrence,
            GoldAndGearsOccurrenceRuleAccuracy::ExactPublic,
        )?);
    }
    for variant in &content.occurrence_variants {
        bindings.push(binding(
            variant.rule.as_str(),
            variant.key.as_str(),
            GoldAndGearsOccurrenceRuleKind::Variant,
            GoldAndGearsOccurrenceRuleAccuracy::ExactPublic,
        )?);
    }
    for source in &content.occurrence_choices {
        let choice = choices
            .iter()
            .find(|choice| choice.stable_key() == source.key.as_str())
            .ok_or(GoldAndGearsEntryError::InvalidOccurrenceRuntime)?;
        let accuracy = if choice.outcome().uses_seeded_uniform_policy() {
            GoldAndGearsOccurrenceRuleAccuracy::ProjectPolicy
        } else {
            GoldAndGearsOccurrenceRuleAccuracy::ExactPublic
        };
        bindings.push(binding(
            source.rule.as_str(),
            source.key.as_str(),
            GoldAndGearsOccurrenceRuleKind::Choice,
            accuracy,
        )?);
    }
    bindings.sort_by(|left, right| left.rule_id.cmp(&right.rule_id));
    if bindings.len() != 384
        || bindings
            .windows(2)
            .any(|pair| pair[0].rule_id >= pair[1].rule_id)
        || bindings
            .iter()
            .filter(|binding| binding.ownership == GoldAndGearsOccurrenceRuleOwnership::Shared)
            .count()
            != 51
        || bindings
            .iter()
            .filter(|binding| binding.accuracy == GoldAndGearsOccurrenceRuleAccuracy::ExactPublic)
            .count()
            != 341
    {
        return Err(GoldAndGearsEntryError::InvalidOccurrenceRuntime);
    }
    let digest = execution_digest(&bindings, choices);
    Ok((bindings.into_boxed_slice(), digest))
}

impl GoldAndGearsRuntimeFactory {
    #[must_use]
    pub fn occurrence_rule_bindings(&self) -> &[GoldAndGearsOccurrenceRuleBinding] {
        self.content_runtime.occurrences.rule_bindings()
    }

    #[must_use]
    pub fn occurrence_execution_digest(&self) -> [u8; 32] {
        self.content_runtime.occurrences.execution_digest()
    }
}

impl GoldAndGearsRuntimeInstance {
    #[must_use]
    pub fn occurrence_rule_bindings(&self) -> &[GoldAndGearsOccurrenceRuleBinding] {
        self.content_runtime.occurrences.rule_bindings()
    }

    #[must_use]
    pub fn occurrence_execution_digest(&self) -> [u8; 32] {
        self.content_runtime.occurrences.execution_digest()
    }

    pub fn compile_occurrence_choice_execution(
        &self,
        choice: GoldAndGearsOccurrenceChoiceId,
        selection: Option<&GoldAndGearsOccurrenceSelection>,
    ) -> Result<GoldAndGearsOccurrenceExecutionPlan, GoldAndGearsEntryError> {
        let choice_definition = self
            .content_runtime
            .occurrences
            .choice(choice)
            .ok_or(GoldAndGearsEntryError::UnknownOccurrenceChoice(choice))?;
        let selected = validate_selection(choice_definition, selection)?;
        let effects = effects(choice_definition)?;
        let applied_key = occurrence_key(OCCURRENCE_APPLIED_BASE, choice)?;
        let mut operations = Vec::with_capacity(effects.len() + 4);
        operations.push(require_counter(applied_key, 0));
        for effect in &effects {
            let key = match effect.phase {
                GoldAndGearsOccurrenceEffectPhase::Cost => cost_key(choice, effect.ordinal)?,
                GoldAndGearsOccurrenceEffectPhase::Outcome => {
                    occurrence_key(OCCURRENCE_OUTCOME_BASE, choice)?
                }
            };
            operations.push(add_counter(key, 1));
        }
        if !selected.is_empty() {
            operations.push(add_counter(
                occurrence_key(OCCURRENCE_SELECTION_BASE, choice)?,
                i64::try_from(selected.len())
                    .map_err(|_| GoldAndGearsEntryError::InvalidOccurrenceRuntime)?,
            ));
        }
        operations.push(add_counter(applied_key, 1));
        let program_id = OCCURRENCE_PROGRAM_BASE
            .checked_add(choice.get())
            .and_then(ActivityProgramId::new)
            .ok_or(GoldAndGearsEntryError::InvalidOccurrenceRuntime)?;
        let program = ActivityProgramDefinition::new(program_id, operations)
            .map_err(|_| GoldAndGearsEntryError::InvalidOccurrenceRuntime)?;
        Ok(GoldAndGearsOccurrenceExecutionPlan {
            choice,
            effects,
            selected,
            program,
        })
    }
}

fn binding(
    rule_id: &str,
    owner_id: &str,
    kind: GoldAndGearsOccurrenceRuleKind,
    accuracy: GoldAndGearsOccurrenceRuleAccuracy,
) -> Result<GoldAndGearsOccurrenceRuleBinding, GoldAndGearsEntryError> {
    if rule_id.is_empty() || owner_id.is_empty() {
        return Err(GoldAndGearsEntryError::InvalidOccurrenceRuntime);
    }
    let ownership = if owner_id.starts_with("universe.") {
        GoldAndGearsOccurrenceRuleOwnership::Shared
    } else if owner_id.starts_with("gold-gears.") {
        GoldAndGearsOccurrenceRuleOwnership::GoldAndGears
    } else {
        return Err(GoldAndGearsEntryError::InvalidOccurrenceRuntime);
    };
    Ok(GoldAndGearsOccurrenceRuleBinding {
        rule_id: rule_id.into(),
        owner_id: owner_id.into(),
        kind,
        ownership,
        accuracy,
    })
}

fn validate_selection(
    choice: &GoldAndGearsOccurrenceChoice,
    selection: Option<&GoldAndGearsOccurrenceSelection>,
) -> Result<Box<[u64]>, GoldAndGearsEntryError> {
    match (choice.outcome().uses_seeded_uniform_policy(), selection) {
        (false, None) => Ok(Box::new([])),
        (true, Some(selection))
            if selection.source_choice() == choice.id()
                && !selection.selected().is_empty()
                && selection.selected().iter().all(|value| *value != 0)
                && selection
                    .selected()
                    .windows(2)
                    .all(|pair| pair[0] < pair[1]) =>
        {
            Ok(selection.selected().into())
        }
        _ => Err(GoldAndGearsEntryError::InvalidOccurrenceSelection(
            choice.id(),
        )),
    }
}

fn effects(
    choice: &GoldAndGearsOccurrenceChoice,
) -> Result<Box<[GoldAndGearsOccurrenceEffect]>, GoldAndGearsEntryError> {
    let mut effects = Vec::with_capacity(choice.costs().len() + 1);
    for (index, cost) in choice.costs().iter().enumerate() {
        effects.push(GoldAndGearsOccurrenceEffect {
            phase: GoldAndGearsOccurrenceEffectPhase::Cost,
            ordinal: u16::try_from(index)
                .map_err(|_| GoldAndGearsEntryError::InvalidOccurrenceRuntime)?,
            operations: Box::new([cost.operation()]),
            targets: cost.targets().into(),
            numeric_literals: cost.numeric_literals().into(),
            parameter_refs: cost.parameter_refs().into(),
            chance_percentages: Box::new([]),
        });
    }
    effects.push(GoldAndGearsOccurrenceEffect {
        phase: GoldAndGearsOccurrenceEffectPhase::Outcome,
        ordinal: 0,
        operations: choice.outcome().operations().into(),
        targets: choice.outcome().targets().into(),
        numeric_literals: choice.outcome().numeric_literals().into(),
        parameter_refs: choice.outcome().parameter_refs().into(),
        chance_percentages: choice.outcome().chance_percentages().into(),
    });
    Ok(effects.into_boxed_slice())
}

fn occurrence_key(
    base: u64,
    choice: GoldAndGearsOccurrenceChoiceId,
) -> Result<u64, GoldAndGearsEntryError> {
    base.checked_add(u64::from(choice.get()))
        .ok_or(GoldAndGearsEntryError::InvalidOccurrenceRuntime)
}

fn cost_key(
    choice: GoldAndGearsOccurrenceChoiceId,
    ordinal: u16,
) -> Result<u64, GoldAndGearsEntryError> {
    OCCURRENCE_COST_BASE
        .checked_add(u64::from(choice.get()) * COST_KEY_STRIDE)
        .and_then(|key| key.checked_add(u64::from(ordinal)))
        .ok_or(GoldAndGearsEntryError::InvalidOccurrenceRuntime)
}

fn require_counter(key: u64, value: i64) -> ActivityOperation {
    ActivityOperation::Require(ActivityCondition::Equal(counter(key), integer(value)))
}

fn add_counter(key: u64, value: i64) -> ActivityOperation {
    ActivityOperation::AddCounter {
        slot: ActivitySlotId::new(DEFERRED_EFFECTS_SLOT)
            .expect("static deferred-effects slot is non-zero"),
        key,
        delta: integer(value),
    }
}

fn counter(key: u64) -> ActivityExpression {
    ActivityExpression::CounterValue {
        slot: ActivitySlotId::new(DEFERRED_EFFECTS_SLOT)
            .expect("static deferred-effects slot is non-zero"),
        key,
    }
}

fn integer(value: i64) -> ActivityExpression {
    ActivityExpression::Literal(ActivityValue::BoundedInteger(value))
}

fn execution_digest(
    bindings: &[GoldAndGearsOccurrenceRuleBinding],
    choices: &[GoldAndGearsOccurrenceChoice],
) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock-gold-gears-occurrence-execution");
    encoder.u32(bindings.len() as u32);
    for binding in bindings {
        encoder.text(&binding.rule_id);
        encoder.text(&binding.owner_id);
        encoder.u8(binding.kind as u8);
        encoder.u8(binding.ownership as u8);
        encoder.u8(binding.accuracy as u8);
    }
    encoder.u32(choices.len() as u32);
    for choice in choices {
        encoder.u32(choice.id().get());
        encoder.u32(choice.costs().len() as u32);
        encoder.u32(choice.outcome().operations().len() as u32);
        encoder.u32(choice.outcome().targets().len() as u32);
        encoder.bool(choice.outcome().uses_seeded_uniform_policy());
    }
    encoder.finish()
}
