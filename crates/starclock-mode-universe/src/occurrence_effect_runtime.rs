//! Closed typed execution plans for every Standard Universe Occurrence choice.

use crate::{
    catalog::UniverseCatalog,
    digest::Encoder,
    id::{OccurrenceChoiceId, OccurrenceVariantId},
    occurrence::{OccurrenceCost, OccurrenceOutcome, RandomOutcomePolicy},
    run_runtime::{OccurrenceRuntimeChoice, RunRuntimeCatalog},
};

pub const OCCURRENCE_EFFECT_RUNTIME_REVISION: &str =
    "standard-universe-occurrence-effect-runtime-v1";

#[derive(Clone, Debug, Eq, PartialEq)]
struct CompiledOccurrenceEffect {
    choice: OccurrenceChoiceId,
    variant: OccurrenceVariantId,
    source_key: Box<str>,
    condition_keys: Box<[Box<str>]>,
    next_node_key: Option<Box<str>>,
    costs: Box<[OccurrenceCost]>,
    outcome: OccurrenceOutcome,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppliedOccurrenceEffect {
    choice: OccurrenceChoiceId,
    variant: OccurrenceVariantId,
    source_key: Box<str>,
    condition_keys: Box<[Box<str>]>,
    next_node_key: Option<Box<str>>,
    costs: Box<[OccurrenceCost]>,
    outcome: OccurrenceOutcome,
}

impl AppliedOccurrenceEffect {
    #[must_use]
    pub const fn choice(&self) -> OccurrenceChoiceId {
        self.choice
    }
    #[must_use]
    pub const fn variant(&self) -> OccurrenceVariantId {
        self.variant
    }
    #[must_use]
    pub fn source_key(&self) -> &str {
        &self.source_key
    }
    #[must_use]
    pub fn condition_keys(&self) -> &[Box<str>] {
        &self.condition_keys
    }
    #[must_use]
    pub fn next_node_key(&self) -> Option<&str> {
        self.next_node_key.as_deref()
    }
    #[must_use]
    pub fn costs(&self) -> &[OccurrenceCost] {
        &self.costs
    }
    #[must_use]
    pub const fn outcome(&self) -> &OccurrenceOutcome {
        &self.outcome
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OccurrenceEffectRuntimeCatalog {
    programs: Box<[CompiledOccurrenceEffect]>,
    digest: [u8; 32],
}

impl OccurrenceEffectRuntimeCatalog {
    pub fn compile(
        catalog: &UniverseCatalog,
        runtime: &RunRuntimeCatalog,
    ) -> Result<Self, OccurrenceEffectRuntimeError> {
        if catalog.occurrences().len() != 59 || catalog.occurrence_variants().len() != 67 {
            return Err(OccurrenceEffectRuntimeError::InvalidDenominator);
        }
        let mut programs = runtime
            .occurrence_choices()
            .iter()
            .map(compile_choice)
            .collect::<Result<Vec<_>, _>>()?;
        programs.sort_by_key(|program| program.choice);
        let cost_count = programs
            .iter()
            .map(|program| program.costs.len())
            .sum::<usize>();
        let policy_count = programs
            .iter()
            .filter(|program| {
                program.outcome.random_policy()
                    == Some(RandomOutcomePolicy::StableUniformOrderedCandidates)
            })
            .count();
        if programs.len() != 321
            || cost_count != 70
            || policy_count != 52
            || programs
                .windows(2)
                .any(|pair| pair[0].choice == pair[1].choice)
        {
            return Err(OccurrenceEffectRuntimeError::InvalidDenominator);
        }
        let digest = catalog_digest(&programs);
        Ok(Self {
            programs: programs.into_boxed_slice(),
            digest,
        })
    }

    #[must_use]
    pub const fn content_count(&self) -> usize {
        447
    }
    #[must_use]
    pub const fn rule_count(&self) -> usize {
        0
    }
    #[must_use]
    pub const fn choice_count(&self) -> usize {
        321
    }
    #[must_use]
    pub const fn random_policy_count(&self) -> usize {
        52
    }
    #[must_use]
    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }
    #[must_use]
    pub fn choice_ids(&self) -> impl ExactSizeIterator<Item = OccurrenceChoiceId> + '_ {
        self.programs.iter().map(|program| program.choice)
    }

    pub fn execute(
        &self,
        choice: OccurrenceChoiceId,
    ) -> Result<AppliedOccurrenceEffect, OccurrenceEffectRuntimeError> {
        let program = self
            .programs
            .binary_search_by_key(&choice, |program| program.choice)
            .ok()
            .map(|index| &self.programs[index])
            .ok_or(OccurrenceEffectRuntimeError::UnknownChoice)?;
        Ok(AppliedOccurrenceEffect {
            choice: program.choice,
            variant: program.variant,
            source_key: program.source_key.clone(),
            condition_keys: program.condition_keys.clone(),
            next_node_key: program.next_node_key.clone(),
            costs: program.costs.clone(),
            outcome: program.outcome.clone(),
        })
    }
}

fn compile_choice(
    choice: &OccurrenceRuntimeChoice,
) -> Result<CompiledOccurrenceEffect, OccurrenceEffectRuntimeError> {
    let [outcome] = choice.outcomes() else {
        return Err(OccurrenceEffectRuntimeError::InvalidOutcome);
    };
    if outcome.operations().is_empty()
        || outcome
            .parameter_refs()
            .iter()
            .any(|value| value.is_empty())
        || outcome
            .chance_percentages()
            .iter()
            .any(|value| !valid_percentage(*value))
    {
        return Err(OccurrenceEffectRuntimeError::InvalidOutcome);
    }
    Ok(CompiledOccurrenceEffect {
        choice: choice.id(),
        variant: choice.variant(),
        source_key: choice.stable_key().into(),
        condition_keys: choice.condition_keys().to_vec().into_boxed_slice(),
        next_node_key: choice.next_node_key().map(Into::into),
        costs: choice.costs().to_vec().into_boxed_slice(),
        outcome: outcome.clone(),
    })
}

fn valid_percentage(value: crate::path::ExactParameter) -> bool {
    let Some(maximum) = 100_i64.checked_mul(10_i64.pow(u32::from(value.scale()))) else {
        return false;
    };
    (0..=maximum).contains(&value.coefficient())
}

fn catalog_digest(programs: &[CompiledOccurrenceEffect]) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock-universe-occurrence-effect-runtime-catalog-v1");
    encoder.text(OCCURRENCE_EFFECT_RUNTIME_REVISION);
    encoder.u32(programs.len() as u32);
    for program in programs {
        encoder.u32(program.choice.get());
        encoder.u32(program.variant.get());
        encoder.text(&program.source_key);
        encode_texts(&mut encoder, &program.condition_keys);
        encoder.text(program.next_node_key.as_deref().unwrap_or(""));
        encoder.u32(program.costs.len() as u32);
        for cost in &program.costs {
            encoder.u8(cost.operation() as u8);
            encoder.u32(cost.targets().len() as u32);
            for target in cost.targets() {
                encoder.u8(*target as u8);
            }
        }
        encode_outcome(&mut encoder, &program.outcome);
    }
    encoder.finish()
}

fn encode_outcome(encoder: &mut Encoder, outcome: &OccurrenceOutcome) {
    encoder.u32(outcome.operations().len() as u32);
    for operation in outcome.operations() {
        encoder.u8(*operation as u8);
    }
    encoder.u32(outcome.targets().len() as u32);
    for target in outcome.targets() {
        encoder.u8(*target as u8);
    }
    encoder.u32(outcome.numeric_literals().len() as u32);
    for value in outcome.numeric_literals() {
        encoder.i64(value.value().coefficient());
        encoder.u8(value.value().scale());
        encoder.u8(value.unit() as u8);
    }
    encode_texts(encoder, outcome.parameter_refs());
    encoder.u32(outcome.chance_percentages().len() as u32);
    for chance in outcome.chance_percentages() {
        encoder.i64(chance.coefficient());
        encoder.u8(chance.scale());
    }
    encoder.u8(outcome.random_policy().map_or(0, |value| value as u8 + 1));
}

fn encode_texts(encoder: &mut Encoder, values: &[Box<str>]) {
    encoder.u32(values.len() as u32);
    for value in values {
        encoder.text(value);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OccurrenceEffectRuntimeError {
    InvalidDenominator,
    InvalidOutcome,
    UnknownChoice,
}
