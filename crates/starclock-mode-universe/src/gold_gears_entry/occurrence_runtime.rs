//! Gold and Gears Occurrence choice graphs and bounded random target selection.

use serde::Deserialize;
use starclock_activity::{ActivityRngLabel, ActivityRngStreams};

use crate::{
    digest::Encoder,
    gold_gears_content::{
        GoldAndGearsContentCatalog,
        types::{Occurrence, OccurrenceChoice, OccurrenceVariant},
    },
};

use super::{
    GoldAndGearsEntryError,
    api::{GoldAndGearsRuntimeFactory, GoldAndGearsRuntimeInstance},
    occurrence_types::{
        GoldAndGearsAuthoredScalar, GoldAndGearsOccurrenceChoice, GoldAndGearsOccurrenceChoiceId,
        GoldAndGearsOccurrenceCost, GoldAndGearsOccurrenceDefinition,
        GoldAndGearsOccurrenceOperation, GoldAndGearsOccurrenceOutcome,
        GoldAndGearsOccurrenceSelection, GoldAndGearsOccurrenceTarget,
        GoldAndGearsOccurrenceVariantDefinition,
    },
};

pub const GOLD_AND_GEARS_OCCURRENCE_RUNTIME_REVISION: &str = "gold-and-gears-occurrence-runtime-v1";
pub const GOLD_AND_GEARS_OCCURRENCE_POLICY_REVISION: &str =
    "gold-and-gears-occurrence-random-outcome-policy-v1";
pub const GOLD_AND_GEARS_OCCURRENCE_POLICY_ACCURACY: &str =
    "DeterministicProjectPolicyNotObservedParity";

const OCCURRENCE_PURPOSE_BASE: u16 = 0x4800;

#[derive(Clone, Debug)]
pub(super) struct GoldAndGearsOccurrenceRuntimeCatalog {
    definitions: Box<[GoldAndGearsOccurrenceDefinition]>,
    variants: Box<[GoldAndGearsOccurrenceVariantDefinition]>,
    choices: Box<[GoldAndGearsOccurrenceChoice]>,
    digest: [u8; 32],
}

impl GoldAndGearsOccurrenceRuntimeCatalog {
    pub(super) fn compile(
        content: &GoldAndGearsContentCatalog,
    ) -> Result<Self, GoldAndGearsEntryError> {
        if content.occurrences.len() != 62
            || content.occurrence_variants.len() != 65
            || content.occurrence_choices.len() != 257
        {
            return Err(GoldAndGearsEntryError::InvalidOccurrenceRuntime);
        }
        let mut choices = content
            .occurrence_choices
            .iter()
            .map(|choice| compile_choice(content, choice))
            .collect::<Result<Vec<_>, _>>()?;
        choices.sort_by_key(|choice| choice.id);
        if choices.windows(2).any(|pair| pair[0].id == pair[1].id)
            || choices
                .iter()
                .filter(|choice| choice.outcome.seeded_uniform)
                .count()
                != 43
        {
            return Err(GoldAndGearsEntryError::InvalidOccurrenceRuntime);
        }
        let mut variants = content
            .occurrence_variants
            .iter()
            .map(|variant| compile_variant(content, variant, &choices))
            .collect::<Result<Vec<_>, _>>()?;
        variants.sort_by_key(|variant| variant.id);
        let mut definitions = content
            .occurrences
            .iter()
            .map(|occurrence| compile_occurrence(occurrence, &variants))
            .collect::<Result<Vec<_>, _>>()?;
        definitions.sort_by_key(|definition| definition.id);
        if variants.windows(2).any(|pair| pair[0].id == pair[1].id)
            || definitions.windows(2).any(|pair| pair[0].id == pair[1].id)
        {
            return Err(GoldAndGearsEntryError::InvalidOccurrenceRuntime);
        }
        let digest = catalog_digest(&definitions, &variants, &choices);
        Ok(Self {
            definitions: definitions.into_boxed_slice(),
            variants: variants.into_boxed_slice(),
            choices: choices.into_boxed_slice(),
            digest,
        })
    }

    pub(super) fn choices(&self) -> &[GoldAndGearsOccurrenceChoice] {
        &self.choices
    }

    pub(super) fn definitions(&self) -> &[GoldAndGearsOccurrenceDefinition] {
        &self.definitions
    }

    pub(super) fn variants(&self) -> &[GoldAndGearsOccurrenceVariantDefinition] {
        &self.variants
    }

    pub(super) const fn digest(&self) -> [u8; 32] {
        self.digest
    }

    fn choice(&self, id: GoldAndGearsOccurrenceChoiceId) -> Option<&GoldAndGearsOccurrenceChoice> {
        self.choices
            .binary_search_by_key(&id, |choice| choice.id)
            .ok()
            .map(|index| &self.choices[index])
    }
}

impl GoldAndGearsRuntimeFactory {
    #[must_use]
    pub fn occurrence_definitions(&self) -> &[GoldAndGearsOccurrenceDefinition] {
        self.content_runtime.occurrences.definitions()
    }

    #[must_use]
    pub fn occurrence_variants(&self) -> &[GoldAndGearsOccurrenceVariantDefinition] {
        self.content_runtime.occurrences.variants()
    }

    #[must_use]
    pub fn occurrence_choices(&self) -> &[GoldAndGearsOccurrenceChoice] {
        self.content_runtime.occurrences.choices()
    }

    #[must_use]
    pub fn occurrence_runtime_digest(&self) -> [u8; 32] {
        self.content_runtime.occurrences.digest()
    }
}

impl GoldAndGearsRuntimeInstance {
    #[must_use]
    pub fn occurrence_definitions(&self) -> &[GoldAndGearsOccurrenceDefinition] {
        self.content_runtime.occurrences.definitions()
    }

    #[must_use]
    pub fn occurrence_variants(&self) -> &[GoldAndGearsOccurrenceVariantDefinition] {
        self.content_runtime.occurrences.variants()
    }

    #[must_use]
    pub fn occurrence_choices(&self) -> &[GoldAndGearsOccurrenceChoice] {
        self.content_runtime.occurrences.choices()
    }

    #[must_use]
    pub fn occurrence_runtime_digest(&self) -> [u8; 32] {
        self.content_runtime.occurrences.digest()
    }

    pub fn select_occurrence_candidates(
        &self,
        choice: GoldAndGearsOccurrenceChoiceId,
        candidates: &[u64],
        maximum: u16,
        rng: &mut ActivityRngStreams,
    ) -> Result<GoldAndGearsOccurrenceSelection, GoldAndGearsEntryError> {
        let definition = self
            .content_runtime
            .occurrences
            .choice(choice)
            .ok_or(GoldAndGearsEntryError::UnknownOccurrenceChoice(choice))?;
        if !definition.outcome.seeded_uniform {
            return Err(GoldAndGearsEntryError::OccurrenceChoiceIsNotRandom(choice));
        }
        let mut candidates = candidates.to_vec();
        candidates.sort_unstable();
        if candidates.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(GoldAndGearsEntryError::InvalidOccurrenceCandidates);
        }
        if maximum == 0 || candidates.is_empty() {
            return Ok(GoldAndGearsOccurrenceSelection::new(choice, Box::new([])));
        }
        let purpose = OCCURRENCE_PURPOSE_BASE
            .checked_add(
                u16::try_from(choice.get())
                    .map_err(|_| GoldAndGearsEntryError::InvalidOccurrenceRuntime)?,
            )
            .ok_or(GoldAndGearsEntryError::InvalidOccurrenceRuntime)?;
        let selected = rng.transact(|working| {
            working
                .choose_weighted_without_replacement(
                    ActivityRngLabel::Occurrence,
                    purpose,
                    &vec![1; candidates.len()],
                    maximum,
                )
                .map_err(|_| GoldAndGearsEntryError::InvalidOccurrenceRuntime)
        })?;
        let mut selected = selected
            .iter()
            .map(|index| {
                candidates
                    .get(*index as usize)
                    .copied()
                    .ok_or(GoldAndGearsEntryError::InvalidOccurrenceRuntime)
            })
            .collect::<Result<Vec<_>, _>>()?;
        selected.sort_unstable();
        Ok(GoldAndGearsOccurrenceSelection::new(
            choice,
            selected.into_boxed_slice(),
        ))
    }
}

fn compile_occurrence(
    occurrence: &Occurrence,
    variants: &[GoldAndGearsOccurrenceVariantDefinition],
) -> Result<GoldAndGearsOccurrenceDefinition, GoldAndGearsEntryError> {
    let variants = occurrence
        .variants
        .iter()
        .map(|key| {
            variants
                .iter()
                .find(|variant| variant.stable_key.as_ref() == key.as_str())
                .map(|variant| variant.id)
                .ok_or(GoldAndGearsEntryError::InvalidOccurrenceRuntime)
        })
        .collect::<Result<Vec<_>, _>>()?;
    if variants.is_empty() || variants.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(GoldAndGearsEntryError::InvalidOccurrenceRuntime);
    }
    Ok(GoldAndGearsOccurrenceDefinition {
        id: positive_u32(occurrence.id)?,
        stable_key: occurrence.key.as_str().into(),
        variants: variants.into_boxed_slice(),
    })
}

fn compile_variant(
    content: &GoldAndGearsContentCatalog,
    variant: &OccurrenceVariant,
    choices: &[GoldAndGearsOccurrenceChoice],
) -> Result<GoldAndGearsOccurrenceVariantDefinition, GoldAndGearsEntryError> {
    if content
        .occurrences
        .iter()
        .all(|occurrence| occurrence.id != variant.occurrence_id)
        || variant.occurrence_keys.is_empty()
    {
        return Err(GoldAndGearsEntryError::InvalidOccurrenceRuntime);
    }
    let choices = variant
        .choices
        .iter()
        .map(|key| {
            choices
                .iter()
                .find(|choice| choice.stable_key.as_ref() == key.as_str())
                .map(|choice| choice.id)
                .ok_or(GoldAndGearsEntryError::InvalidOccurrenceRuntime)
        })
        .collect::<Result<Vec<_>, _>>()?;
    if choices.is_empty() || choices.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(GoldAndGearsEntryError::InvalidOccurrenceRuntime);
    }
    Ok(GoldAndGearsOccurrenceVariantDefinition {
        id: positive_u32(variant.id)?,
        stable_key: variant.key.as_str().into(),
        occurrence: positive_u32(variant.occurrence_id)?,
        occurrence_keys: variant
            .occurrence_keys
            .iter()
            .map(|key| key.as_str().into())
            .collect(),
        entry_node: variant.entry_node.as_str().into(),
        conditions: variant.conditions.clone(),
        choices: choices.into_boxed_slice(),
    })
}

fn compile_choice(
    content: &GoldAndGearsContentCatalog,
    choice: &OccurrenceChoice,
) -> Result<GoldAndGearsOccurrenceChoice, GoldAndGearsEntryError> {
    let variant = content
        .occurrence_variants
        .iter()
        .find(|variant| variant.id == choice.variant_id)
        .ok_or(GoldAndGearsEntryError::InvalidOccurrenceRuntime)?;
    validate_variant_choice(variant, choice)?;
    let [
        costs,
        outcomes,
        parameter_vectors,
        dynamic_display,
        quality_overrides,
    ] = choice.payloads.as_ref()
    else {
        return Err(GoldAndGearsEntryError::InvalidOccurrenceRuntime);
    };
    let costs: Vec<RawCost> = serde_json::from_str(costs.as_str())
        .map_err(|_| GoldAndGearsEntryError::InvalidOccurrenceRuntime)?;
    let outcomes: Vec<RawOutcome> = serde_json::from_str(outcomes.as_str())
        .map_err(|_| GoldAndGearsEntryError::InvalidOccurrenceRuntime)?;
    let empty_vectors: Vec<RawParameterVector> =
        serde_json::from_str(parameter_vectors.as_str())
            .map_err(|_| GoldAndGearsEntryError::InvalidOccurrenceRuntime)?;
    let dynamic: Vec<RawDynamicDisplay> = serde_json::from_str(dynamic_display.as_str())
        .map_err(|_| GoldAndGearsEntryError::InvalidOccurrenceRuntime)?;
    let quality: Vec<RawQualityOverride> = serde_json::from_str(quality_overrides.as_str())
        .map_err(|_| GoldAndGearsEntryError::InvalidOccurrenceRuntime)?;
    let [outcome] = outcomes.as_slice() else {
        return Err(GoldAndGearsEntryError::InvalidOccurrenceRuntime);
    };
    let seeded_uniform = outcome.probability_policy.as_ref() == "SeededUniformStableSourceOrder";
    if !empty_vectors.is_empty()
        || dynamic
            .iter()
            .any(|entry| entry.key.is_empty() || entry.display_id.is_empty())
        || quality.len() != usize::from(seeded_uniform)
        || quality.iter().any(|entry| {
            entry.field.as_ref() != "probability_policy"
                || entry.evidence_quality.as_ref() != "ProjectPolicy"
                || entry.policy_id.as_ref() != "occurrence-random-outcome-v1"
                || entry.replacement_condition.is_empty()
        })
    {
        return Err(GoldAndGearsEntryError::InvalidOccurrenceRuntime);
    }
    let id = GoldAndGearsOccurrenceChoiceId::new(
        u32::try_from(choice.id).map_err(|_| GoldAndGearsEntryError::InvalidOccurrenceRuntime)?,
    )
    .ok_or(GoldAndGearsEntryError::InvalidOccurrenceRuntime)?;
    Ok(GoldAndGearsOccurrenceChoice {
        id,
        stable_key: choice.key.as_str().into(),
        source_id: choice
            .source_id
            .parse()
            .map_err(|_| GoldAndGearsEntryError::InvalidOccurrenceRuntime)?,
        variant_key: variant.key.as_str().into(),
        node_index: positive_u16(choice.node_index)?,
        choice_index: positive_u16(choice.choice_index)?,
        option_index: positive_u16(choice.option_index)?,
        conditions: choice.conditions.clone(),
        next_node: choice.next_node.as_ref().map(|key| key.as_str().into()),
        costs: costs
            .iter()
            .map(compile_cost)
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
        outcome: compile_outcome(outcome)?,
    })
}

fn validate_variant_choice(
    variant: &OccurrenceVariant,
    choice: &OccurrenceChoice,
) -> Result<(), GoldAndGearsEntryError> {
    if variant.choices.iter().all(|key| key != &choice.key)
        || variant.entry_node.as_str().is_empty()
        || variant.conditions.windows(2).any(|pair| pair[0] >= pair[1])
        || choice.conditions.windows(2).any(|pair| pair[0] >= pair[1])
    {
        Err(GoldAndGearsEntryError::InvalidOccurrenceRuntime)
    } else {
        Ok(())
    }
}

fn compile_cost(raw: &RawCost) -> Result<GoldAndGearsOccurrenceCost, GoldAndGearsEntryError> {
    let operation = operation(&raw.kind)?;
    if !matches!(
        operation,
        GoldAndGearsOccurrenceOperation::Consume
            | GoldAndGearsOccurrenceOperation::Discard
            | GoldAndGearsOccurrenceOperation::Lose
    ) {
        return Err(GoldAndGearsEntryError::InvalidOccurrenceRuntime);
    }
    Ok(GoldAndGearsOccurrenceCost {
        operation,
        targets: raw
            .targets
            .iter()
            .map(|target| occurrence_target(target))
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
        numeric_literals: raw
            .numeric_literals
            .iter()
            .map(|value| scalar(value, false))
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
        parameter_refs: raw.parameter_refs.iter().copied().collect(),
    })
}

fn compile_outcome(
    raw: &RawOutcome,
) -> Result<GoldAndGearsOccurrenceOutcome, GoldAndGearsEntryError> {
    if raw.kinds.is_empty()
        || !matches!(
            raw.probability_policy.as_ref(),
            "ExactPrintedPercentagesOrDeterministic" | "SeededUniformStableSourceOrder"
        )
        || (raw.probability_policy.as_ref() == "SeededUniformStableSourceOrder")
            != (raw.unresolved_candidate_pool.as_ref() == "FailClosed")
    {
        return Err(GoldAndGearsEntryError::InvalidOccurrenceRuntime);
    }
    let chance_percentages = raw
        .chance_percentages
        .iter()
        .map(|value| scalar(value, true))
        .collect::<Result<Vec<_>, _>>()?;
    if chance_percentages.iter().any(|value| {
        value.coefficient() < 0
            || value.coefficient() > 100_i64 * 10_i64.pow(u32::from(value.scale()))
    }) {
        return Err(GoldAndGearsEntryError::InvalidOccurrenceRuntime);
    }
    Ok(GoldAndGearsOccurrenceOutcome {
        operations: raw
            .kinds
            .iter()
            .map(|kind| operation(kind))
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
        targets: raw
            .targets
            .iter()
            .map(|target| occurrence_target(target))
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
        numeric_literals: raw
            .numeric_literals
            .iter()
            .map(|value| scalar(value, false))
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
        parameter_refs: raw.parameter_refs.iter().copied().collect(),
        chance_percentages: chance_percentages.into_boxed_slice(),
        seeded_uniform: raw.probability_policy.as_ref() == "SeededUniformStableSourceOrder",
    })
}

fn operation(value: &str) -> Result<GoldAndGearsOccurrenceOperation, GoldAndGearsEntryError> {
    match value {
        "Battle" => Ok(GoldAndGearsOccurrenceOperation::Battle),
        "Consume" => Ok(GoldAndGearsOccurrenceOperation::Consume),
        "Discard" => Ok(GoldAndGearsOccurrenceOperation::Discard),
        "Enhance" => Ok(GoldAndGearsOccurrenceOperation::Enhance),
        "Lose" => Ok(GoldAndGearsOccurrenceOperation::Lose),
        "NoOp" => Ok(GoldAndGearsOccurrenceOperation::NoOp),
        "Obtain" => Ok(GoldAndGearsOccurrenceOperation::Obtain),
        "Repair" => Ok(GoldAndGearsOccurrenceOperation::Repair),
        "Replace" => Ok(GoldAndGearsOccurrenceOperation::Replace),
        "Restore" => Ok(GoldAndGearsOccurrenceOperation::Restore),
        "Select" => Ok(GoldAndGearsOccurrenceOperation::Select),
        "Special" => Ok(GoldAndGearsOccurrenceOperation::Special),
        _ => Err(GoldAndGearsEntryError::InvalidOccurrenceRuntime),
    }
}

fn occurrence_target(value: &str) -> Result<GoldAndGearsOccurrenceTarget, GoldAndGearsEntryError> {
    match value {
        "Blessing" => Ok(GoldAndGearsOccurrenceTarget::Blessing),
        "Character" => Ok(GoldAndGearsOccurrenceTarget::Character),
        "CosmicFragments" => Ok(GoldAndGearsOccurrenceTarget::CosmicFragments),
        "Curio" => Ok(GoldAndGearsOccurrenceTarget::Curio),
        "DiceReroll" => Ok(GoldAndGearsOccurrenceTarget::DiceReroll),
        "HP" => Ok(GoldAndGearsOccurrenceTarget::Hp),
        _ => Err(GoldAndGearsEntryError::InvalidOccurrenceRuntime),
    }
}

fn scalar(
    value: &str,
    force_percent: bool,
) -> Result<GoldAndGearsAuthoredScalar, GoldAndGearsEntryError> {
    let (value, suffix_percent) = value
        .strip_suffix('%')
        .map_or((value, false), |value| (value, true));
    let negative = value.starts_with('-');
    let unsigned = value.strip_prefix('-').unwrap_or(value);
    let (whole, fraction) = unsigned
        .split_once('.')
        .map_or((unsigned, ""), |parts| parts);
    let scale = u8::try_from(fraction.len())
        .map_err(|_| GoldAndGearsEntryError::InvalidOccurrenceRuntime)?;
    let coefficient = format!("{whole}{fraction}")
        .parse::<i64>()
        .map_err(|_| GoldAndGearsEntryError::InvalidOccurrenceRuntime)?;
    Ok(GoldAndGearsAuthoredScalar::new(
        if negative { -coefficient } else { coefficient },
        scale,
        force_percent || suffix_percent,
    ))
}

fn positive_u16(value: i32) -> Result<u16, GoldAndGearsEntryError> {
    u16::try_from(value)
        .ok()
        .filter(|value| *value > 0)
        .ok_or(GoldAndGearsEntryError::InvalidOccurrenceRuntime)
}

fn positive_u32(value: i32) -> Result<u32, GoldAndGearsEntryError> {
    u32::try_from(value)
        .ok()
        .filter(|value| *value > 0)
        .ok_or(GoldAndGearsEntryError::InvalidOccurrenceRuntime)
}

fn catalog_digest(
    definitions: &[GoldAndGearsOccurrenceDefinition],
    variants: &[GoldAndGearsOccurrenceVariantDefinition],
    choices: &[GoldAndGearsOccurrenceChoice],
) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock-gold-gears-occurrence-runtime-v1");
    encoder.text(GOLD_AND_GEARS_OCCURRENCE_RUNTIME_REVISION);
    encoder.text(GOLD_AND_GEARS_OCCURRENCE_POLICY_REVISION);
    encoder.u32(definitions.len() as u32);
    for definition in definitions {
        encoder.u32(definition.id);
        encoder.text(&definition.stable_key);
        encoder.u32(definition.variants.len() as u32);
        for variant in &definition.variants {
            encoder.u32(*variant);
        }
    }
    encoder.u32(variants.len() as u32);
    for variant in variants {
        encoder.u32(variant.id);
        encoder.text(&variant.stable_key);
        encoder.u32(variant.occurrence);
        encoder.u32(variant.occurrence_keys.len() as u32);
        for key in &variant.occurrence_keys {
            encoder.text(key);
        }
        encoder.text(&variant.entry_node);
        encoder.u32(variant.conditions.len() as u32);
        for condition in &variant.conditions {
            encoder.text(condition);
        }
        encoder.u32(variant.choices.len() as u32);
        for choice in &variant.choices {
            encoder.u32(choice.get());
        }
    }
    encoder.u32(choices.len() as u32);
    for choice in choices {
        encoder.u32(choice.id.get());
        encoder.text(&choice.stable_key);
        encoder.u32(choice.source_id);
        encoder.text(&choice.variant_key);
        encoder.u32(u32::from(choice.node_index));
        encoder.u32(u32::from(choice.choice_index));
        encoder.u32(u32::from(choice.option_index));
        encoder.u32(choice.conditions.len() as u32);
        for condition in &choice.conditions {
            encoder.text(condition);
        }
        encoder.text(choice.next_node.as_deref().unwrap_or(""));
        encoder.u32(choice.costs.len() as u32);
        for cost in &choice.costs {
            encoder.u8(cost.operation as u8);
            encoder.u32(cost.targets.len() as u32);
            for target in &cost.targets {
                encoder.u8(*target as u8);
            }
            encoder.u32(cost.numeric_literals.len() as u32);
            for value in &cost.numeric_literals {
                encoder.i64(value.coefficient());
                encoder.u8(value.scale());
                encoder.bool(value.is_percent());
            }
            encoder.u32(cost.parameter_refs.len() as u32);
            for value in &cost.parameter_refs {
                encoder.u32(*value);
            }
        }
        encoder.u32(choice.outcome.operations.len() as u32);
        for operation in &choice.outcome.operations {
            encoder.u8(*operation as u8);
        }
        encoder.u32(choice.outcome.targets.len() as u32);
        for target in &choice.outcome.targets {
            encoder.u8(*target as u8);
        }
        encoder.u32(choice.outcome.numeric_literals.len() as u32);
        for value in &choice.outcome.numeric_literals {
            encoder.i64(value.coefficient());
            encoder.u8(value.scale());
            encoder.bool(value.is_percent());
        }
        encoder.u32(choice.outcome.parameter_refs.len() as u32);
        for value in &choice.outcome.parameter_refs {
            encoder.u32(*value);
        }
        encoder.u32(choice.outcome.chance_percentages.len() as u32);
        for value in &choice.outcome.chance_percentages {
            encoder.i64(value.coefficient());
            encoder.u8(value.scale());
        }
        encoder.bool(choice.outcome.seeded_uniform);
    }
    encoder.finish()
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCost {
    kind: Box<str>,
    targets: Box<[Box<str>]>,
    numeric_literals: Box<[Box<str>]>,
    parameter_refs: Box<[u32]>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawOutcome {
    kinds: Box<[Box<str>]>,
    targets: Box<[Box<str>]>,
    numeric_literals: Box<[Box<str>]>,
    parameter_refs: Box<[u32]>,
    chance_percentages: Box<[Box<str>]>,
    probability_policy: Box<str>,
    unresolved_candidate_pool: Box<str>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawParameterVector {}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawDynamicDisplay {
    key: Box<str>,
    display_id: Box<str>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawQualityOverride {
    field: Box<str>,
    evidence_quality: Box<str>,
    policy_id: Box<str>,
    replacement_condition: Box<str>,
}
