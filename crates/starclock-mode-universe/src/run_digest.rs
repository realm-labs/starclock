//! Canonical identity for run-choice and progression definitions.

use crate::definition::LocalizedText;
use crate::digest::{Encoder, UniverseRunDefinitionsDigest};
use crate::occurrence::{
    AuthoredScalar, OccurrenceChoiceDefinition, OccurrenceDefinition, OccurrenceOutcome,
    OccurrenceVariantDefinition,
};
use crate::path::ExactParameter;
use crate::progression::{AbilityTreeNodeDefinition, ServiceDefinition};

pub(crate) fn digest(
    occurrences: &[OccurrenceDefinition],
    variants: &[OccurrenceVariantDefinition],
    choices: &[OccurrenceChoiceDefinition],
    services: &[ServiceDefinition],
    ability_nodes: &[AbilityTreeNodeDefinition],
) -> UniverseRunDefinitionsDigest {
    let mut encoder = Encoder::new(b"starclock-standard-universe-run-definitions-v1");
    encode_occurrences(&mut encoder, occurrences, variants, choices);
    encode_services(&mut encoder, services);
    encode_ability_nodes(&mut encoder, ability_nodes);
    UniverseRunDefinitionsDigest::new(encoder.finish())
}

fn encode_occurrences(
    encoder: &mut Encoder,
    occurrences: &[OccurrenceDefinition],
    variants: &[OccurrenceVariantDefinition],
    choices: &[OccurrenceChoiceDefinition],
) {
    encoder.u32(occurrences.len() as u32);
    for value in occurrences {
        encoder.u32(value.id().get());
        encoder.text(value.stable_key());
        encoder.text(value.choice_graph_key());
        texts(encoder, value.pool_tags());
        encoder.bool(value.index_only());
        localized(encoder, value.text());
        ids(encoder, value.variants().iter().map(|id| id.get()));
    }
    encoder.u32(variants.len() as u32);
    for value in variants {
        encoder.u32(value.id().get());
        encoder.text(value.stable_key());
        encoder.u32(value.occurrence().get());
        encoder.text(value.entry_node_key());
        texts(encoder, value.condition_keys());
        encoder.text(value.source_dialogue_type());
        localized(encoder, value.text());
        ids(encoder, value.choices().iter().map(|id| id.get()));
    }
    encoder.u32(choices.len() as u32);
    for value in choices {
        encoder.u32(value.id().get());
        encoder.text(value.stable_key());
        encoder.u32(value.variant().get());
        texts(encoder, value.condition_keys());
        encoder.optional_text(value.next_node_key());
        localized(encoder, value.text());
        for digest in value
            .label_digests()
            .into_iter()
            .chain(value.result_digests())
        {
            encoder.digest(digest);
        }
        encoder.u32(value.parameter_vectors().len() as u32);
        for vector in value.parameter_vectors() {
            encoder.text(vector.source_option_id());
            parameters(encoder, vector.values());
        }
        encoder.u32(value.costs().len() as u32);
        for cost in value.costs() {
            encoder.u8(cost.operation() as u8);
            encoder.u32(cost.targets().len() as u32);
            for target in cost.targets() {
                encoder.u8(*target as u8);
            }
        }
        encoder.u32(value.outcomes().len() as u32);
        for outcome in value.outcomes() {
            encode_outcome(encoder, outcome);
        }
    }
}

fn encode_outcome(encoder: &mut Encoder, value: &OccurrenceOutcome) {
    encoder.u32(value.operations().len() as u32);
    for operation in value.operations() {
        encoder.u8(*operation as u8);
    }
    encoder.u32(value.targets().len() as u32);
    for target in value.targets() {
        encoder.u8(*target as u8);
    }
    encoder.u32(value.numeric_literals().len() as u32);
    for scalar in value.numeric_literals() {
        authored_scalar(encoder, *scalar);
    }
    texts(encoder, value.parameter_refs());
    parameters(encoder, value.chance_percentages());
    encoder.bool(value.random_policy().is_some());
    if let Some(policy) = value.random_policy() {
        encoder.u8(policy as u8);
    }
}

fn encode_services(encoder: &mut Encoder, values: &[ServiceDefinition]) {
    encoder.u32(values.len() as u32);
    for value in values {
        encoder.u32(value.id().get());
        encoder.text(value.stable_key());
        encoder.u8(value.kind() as u8);
        encoder.optional_text(value.currency_key());
        encoder.optional_text(value.price_formula_key());
        encoder.optional_text(value.offer_pool_key());
        encoder.text(value.rule_key());
        localized(encoder, value.text());
        encoder.u32(value.parameters().len() as u32);
        for parameter in value.parameters() {
            encoder.text(parameter.key());
            encoder.text(parameter.value());
        }
    }
}

fn encode_ability_nodes(encoder: &mut Encoder, values: &[AbilityTreeNodeDefinition]) {
    encoder.u32(values.len() as u32);
    for value in values {
        encoder.u32(value.id().get());
        encoder.text(value.stable_key());
        encoder.bool(value.important());
        encoder.u8(value.effect_class() as u8);
        encoder.text(value.effect_tag_en());
        encoder.text(value.effect_tag_zh_cn());
        texts(encoder, value.external_unlock_keys());
        encoder.text(value.rule_key());
        localized(encoder, value.text());
        ids(encoder, value.prerequisites().iter().map(|id| id.get()));
        encoder.u32(value.costs().len() as u32);
        for cost in value.costs() {
            encoder.text(cost.source_item_id());
            parameter(encoder, cost.amount());
        }
        encoder.u32(value.effects().len() as u32);
        for effect in value.effects() {
            encoder.u8(effect.operation() as u8);
            encoder.text(effect.target_key());
            parameter(encoder, effect.value());
            encoder.u8(effect.unit() as u8);
            encoder.optional_text(effect.condition());
        }
        parameters(encoder, value.parameters());
    }
}

fn localized(encoder: &mut Encoder, value: &LocalizedText) {
    encoder.text(value.name_en());
    encoder.text(value.name_zh_cn());
    encoder.text(value.summary_en());
    encoder.text(value.summary_zh_cn());
}
fn texts(encoder: &mut Encoder, values: &[Box<str>]) {
    encoder.u32(values.len() as u32);
    for value in values {
        encoder.text(value);
    }
}
fn ids(encoder: &mut Encoder, values: impl ExactSizeIterator<Item = u32>) {
    encoder.u32(values.len() as u32);
    for value in values {
        encoder.u32(value);
    }
}
fn parameter(encoder: &mut Encoder, value: ExactParameter) {
    encoder.i64(value.coefficient());
    encoder.u8(value.scale());
}
fn parameters(encoder: &mut Encoder, values: &[ExactParameter]) {
    encoder.u32(values.len() as u32);
    for value in values {
        parameter(encoder, *value);
    }
}
fn authored_scalar(encoder: &mut Encoder, value: AuthoredScalar) {
    parameter(encoder, value.value());
    encoder.u8(value.unit() as u8);
}
