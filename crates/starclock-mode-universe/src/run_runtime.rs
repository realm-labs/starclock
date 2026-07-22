//! Shared Standard Universe run resources and typed non-battle content runtime.

use starclock_activity::{
    ActivityCondition, ActivityExpression, ActivityOperation, ActivitySlotId, ActivityValue,
};

use crate::{
    catalog::UniverseCatalog,
    digest::Encoder,
    id::{AbilityTreeNodeId, OccurrenceChoiceId, OccurrenceVariantId, ServiceId},
    occurrence::{OccurrenceCost, OccurrenceOutcome, OccurrenceParameterVector},
    path::ExactParameter,
    progression::{AbilityEffectClass, AbilityTreeEffect, ServiceKind, ServiceParameter},
};

pub const RUN_RUNTIME_REVISION: &str = "standard-universe-run-runtime-v1";
pub const MAX_COSMIC_FRAGMENTS: i64 = 4_294_967_295;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CosmicFragments(i64);

impl CosmicFragments {
    pub fn new(value: i64) -> Result<Self, RunRuntimeError> {
        if !(0..=MAX_COSMIC_FRAGMENTS).contains(&value) {
            return Err(RunRuntimeError::InvalidFragmentAmount);
        }
        Ok(Self(value))
    }
    #[must_use]
    pub const fn get(self) -> i64 {
        self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OccurrenceRuntimeChoice {
    id: OccurrenceChoiceId,
    variant: OccurrenceVariantId,
    stable_key: Box<str>,
    condition_keys: Box<[Box<str>]>,
    next_node_key: Option<Box<str>>,
    parameter_vectors: Box<[OccurrenceParameterVector]>,
    costs: Box<[OccurrenceCost]>,
    outcomes: Box<[OccurrenceOutcome]>,
}

impl OccurrenceRuntimeChoice {
    #[must_use]
    pub const fn id(&self) -> OccurrenceChoiceId {
        self.id
    }
    #[must_use]
    pub const fn variant(&self) -> OccurrenceVariantId {
        self.variant
    }
    #[must_use]
    pub fn stable_key(&self) -> &str {
        &self.stable_key
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
    pub fn parameter_vectors(&self) -> &[OccurrenceParameterVector] {
        &self.parameter_vectors
    }
    #[must_use]
    pub fn costs(&self) -> &[OccurrenceCost] {
        &self.costs
    }
    #[must_use]
    pub fn outcomes(&self) -> &[OccurrenceOutcome] {
        &self.outcomes
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ServiceRuntimeDefinition {
    id: ServiceId,
    stable_key: Box<str>,
    kind: ServiceKind,
    currency_key: Option<Box<str>>,
    price_formula_key: Option<Box<str>>,
    offer_pool_key: Option<Box<str>>,
    rule_key: Box<str>,
    parameters: Box<[ServiceParameter]>,
}

impl ServiceRuntimeDefinition {
    #[must_use]
    pub const fn id(&self) -> ServiceId {
        self.id
    }
    #[must_use]
    pub fn stable_key(&self) -> &str {
        &self.stable_key
    }
    #[must_use]
    pub const fn kind(&self) -> ServiceKind {
        self.kind
    }
    #[must_use]
    pub fn currency_key(&self) -> Option<&str> {
        self.currency_key.as_deref()
    }
    #[must_use]
    pub fn price_formula_key(&self) -> Option<&str> {
        self.price_formula_key.as_deref()
    }
    #[must_use]
    pub fn offer_pool_key(&self) -> Option<&str> {
        self.offer_pool_key.as_deref()
    }
    #[must_use]
    pub fn rule_key(&self) -> &str {
        &self.rule_key
    }
    #[must_use]
    pub fn parameters(&self) -> &[ServiceParameter] {
        &self.parameters
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AbilityTreeRuleContribution {
    id: AbilityTreeNodeId,
    stable_key: Box<str>,
    effect_class: AbilityEffectClass,
    rule_key: Box<str>,
    external_unlock_keys: Box<[Box<str>]>,
    effects: Box<[AbilityTreeEffect]>,
    parameters: Box<[ExactParameter]>,
}

impl AbilityTreeRuleContribution {
    #[must_use]
    pub const fn id(&self) -> AbilityTreeNodeId {
        self.id
    }
    #[must_use]
    pub fn stable_key(&self) -> &str {
        &self.stable_key
    }
    #[must_use]
    pub const fn effect_class(&self) -> AbilityEffectClass {
        self.effect_class
    }
    #[must_use]
    pub fn rule_key(&self) -> &str {
        &self.rule_key
    }
    #[must_use]
    pub fn external_unlock_keys(&self) -> &[Box<str>] {
        &self.external_unlock_keys
    }
    #[must_use]
    pub fn effects(&self) -> &[AbilityTreeEffect] {
        &self.effects
    }
    #[must_use]
    pub fn parameters(&self) -> &[ExactParameter] {
        &self.parameters
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AbilityTreeContributionSet {
    entries: Box<[AbilityTreeRuleContribution]>,
    digest: [u8; 32],
}

impl AbilityTreeContributionSet {
    #[must_use]
    pub fn entries(&self) -> &[AbilityTreeRuleContribution] {
        &self.entries
    }
    #[must_use]
    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RunRuntimeCatalog {
    occurrences: Box<[OccurrenceRuntimeChoice]>,
    services: Box<[ServiceRuntimeDefinition]>,
    abilities: Box<[AbilityTreeRuleContribution]>,
    digest: [u8; 32],
}

impl RunRuntimeCatalog {
    pub fn compile(catalog: &UniverseCatalog) -> Result<Self, RunRuntimeError> {
        let mut occurrences = catalog
            .occurrence_choices()
            .iter()
            .map(|choice| OccurrenceRuntimeChoice {
                id: choice.id(),
                variant: choice.variant(),
                stable_key: choice.stable_key().into(),
                condition_keys: choice.condition_keys().to_vec().into_boxed_slice(),
                next_node_key: choice.next_node_key().map(Into::into),
                parameter_vectors: choice.parameter_vectors().to_vec().into_boxed_slice(),
                costs: choice.costs().to_vec().into_boxed_slice(),
                outcomes: choice.outcomes().to_vec().into_boxed_slice(),
            })
            .collect::<Vec<_>>();
        occurrences.sort_by_key(OccurrenceRuntimeChoice::id);
        let mut services = catalog
            .services()
            .iter()
            .map(|service| ServiceRuntimeDefinition {
                id: service.id(),
                stable_key: service.stable_key().into(),
                kind: service.kind(),
                currency_key: service.currency_key().map(Into::into),
                price_formula_key: service.price_formula_key().map(Into::into),
                offer_pool_key: service.offer_pool_key().map(Into::into),
                rule_key: service.rule_key().into(),
                parameters: service.parameters().to_vec().into_boxed_slice(),
            })
            .collect::<Vec<_>>();
        services.sort_by_key(ServiceRuntimeDefinition::id);
        let mut abilities = catalog
            .ability_tree_nodes()
            .iter()
            .map(|node| AbilityTreeRuleContribution {
                id: node.id(),
                stable_key: node.stable_key().into(),
                effect_class: node.effect_class(),
                rule_key: node.rule_key().into(),
                external_unlock_keys: node.external_unlock_keys().to_vec().into_boxed_slice(),
                effects: node.effects().to_vec().into_boxed_slice(),
                parameters: node.parameters().to_vec().into_boxed_slice(),
            })
            .collect::<Vec<_>>();
        abilities.sort_by_key(AbilityTreeRuleContribution::id);
        if occurrences.len() != 321
            || services.len() != 94
            || abilities.len() != 42
            || has_duplicate(&occurrences, OccurrenceRuntimeChoice::id)
            || has_duplicate(&services, ServiceRuntimeDefinition::id)
            || has_duplicate(&abilities, AbilityTreeRuleContribution::id)
        {
            return Err(RunRuntimeError::InvalidDenominator);
        }
        let digest = catalog_digest(&occurrences, &services, &abilities);
        Ok(Self {
            occurrences: occurrences.into_boxed_slice(),
            services: services.into_boxed_slice(),
            abilities: abilities.into_boxed_slice(),
            digest,
        })
    }

    #[must_use]
    pub fn occurrence_choices(&self) -> &[OccurrenceRuntimeChoice] {
        &self.occurrences
    }
    #[must_use]
    pub fn services(&self) -> &[ServiceRuntimeDefinition] {
        &self.services
    }
    #[must_use]
    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }

    pub fn ability_contributions(
        &self,
        selected: &[AbilityTreeNodeId],
    ) -> Result<AbilityTreeContributionSet, RunRuntimeError> {
        let mut entries = Vec::with_capacity(selected.len());
        for id in selected {
            let entry = self
                .abilities
                .binary_search_by_key(id, AbilityTreeRuleContribution::id)
                .ok()
                .map(|index| self.abilities[index].clone())
                .ok_or(RunRuntimeError::UnknownAbilityTreeNode(*id))?;
            entries.push(entry);
        }
        entries.sort_by_key(AbilityTreeRuleContribution::id);
        if entries.windows(2).any(|pair| pair[0].id == pair[1].id) {
            return Err(RunRuntimeError::DuplicateAbilityTreeNode);
        }
        let digest = ability_digest(&entries);
        Ok(AbilityTreeContributionSet {
            entries: entries.into_boxed_slice(),
            digest,
        })
    }

    pub fn credit_fragments(
        slot: ActivitySlotId,
        amount: CosmicFragments,
    ) -> Box<[ActivityOperation]> {
        vec![ActivityOperation::AddToSlot {
            slot,
            delta: integer(amount.get()),
        }]
        .into_boxed_slice()
    }

    pub fn spend_fragments(
        slot: ActivitySlotId,
        amount: CosmicFragments,
    ) -> Box<[ActivityOperation]> {
        vec![
            ActivityOperation::Require(ActivityCondition::Not(Box::new(
                ActivityCondition::LessThan(ActivityExpression::Slot(slot), integer(amount.get())),
            ))),
            ActivityOperation::AddToSlot {
                slot,
                delta: integer(-amount.get()),
            },
        ]
        .into_boxed_slice()
    }
}

fn integer(value: i64) -> ActivityExpression {
    ActivityExpression::Literal(ActivityValue::BoundedInteger(value))
}

fn has_duplicate<T, I: Copy + Eq>(values: &[T], id: impl Fn(&T) -> I) -> bool {
    values.windows(2).any(|pair| id(&pair[0]) == id(&pair[1]))
}

fn catalog_digest(
    occurrences: &[OccurrenceRuntimeChoice],
    services: &[ServiceRuntimeDefinition],
    abilities: &[AbilityTreeRuleContribution],
) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock-universe-run-runtime-catalog-v1");
    encoder.text(RUN_RUNTIME_REVISION);
    encoder.u32(occurrences.len() as u32);
    for choice in occurrences {
        encoder.u32(choice.id.get());
        encoder.u32(choice.variant.get());
        encoder.text(&choice.stable_key);
        encode_texts(&mut encoder, &choice.condition_keys);
        encoder.text(choice.next_node_key.as_deref().unwrap_or(""));
        encoder.u32(choice.parameter_vectors.len() as u32);
        for vector in &choice.parameter_vectors {
            encoder.text(vector.source_option_id());
            encode_parameters(&mut encoder, vector.values());
        }
        encoder.u32(choice.costs.len() as u32);
        for cost in &choice.costs {
            encoder.u8(cost.operation() as u8);
            encoder.u32(cost.targets().len() as u32);
            for target in cost.targets() {
                encoder.u8(*target as u8);
            }
        }
        encoder.u32(choice.outcomes.len() as u32);
        for outcome in &choice.outcomes {
            encode_outcome(&mut encoder, outcome);
        }
    }
    encoder.u32(services.len() as u32);
    for service in services {
        encoder.u32(service.id.get());
        encoder.text(&service.stable_key);
        encoder.u8(service.kind as u8);
        encoder.text(service.currency_key.as_deref().unwrap_or(""));
        encoder.text(service.price_formula_key.as_deref().unwrap_or(""));
        encoder.text(service.offer_pool_key.as_deref().unwrap_or(""));
        encoder.text(&service.rule_key);
        encoder.u32(service.parameters.len() as u32);
        for parameter in &service.parameters {
            encoder.text(parameter.key());
            encoder.text(parameter.value());
        }
    }
    encoder.u32(abilities.len() as u32);
    for ability in abilities {
        encode_ability(&mut encoder, ability);
    }
    encoder.finish()
}

fn encode_outcome(encoder: &mut Encoder, outcome: &OccurrenceOutcome) {
    encoder.u32(outcome.operations().len() as u32);
    for value in outcome.operations() {
        encoder.u8(*value as u8);
    }
    encoder.u32(outcome.targets().len() as u32);
    for value in outcome.targets() {
        encoder.u8(*value as u8);
    }
    encoder.u32(outcome.numeric_literals().len() as u32);
    for value in outcome.numeric_literals() {
        encode_parameter(encoder, value.value());
        encoder.u8(value.unit() as u8);
    }
    encode_texts(encoder, outcome.parameter_refs());
    encode_parameters(encoder, outcome.chance_percentages());
    encoder.u8(outcome.random_policy().map_or(0, |value| value as u8 + 1));
}

fn ability_digest(entries: &[AbilityTreeRuleContribution]) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock-universe-ability-tree-contributions-v1");
    encoder.u32(entries.len() as u32);
    for entry in entries {
        encode_ability(&mut encoder, entry);
    }
    encoder.finish()
}

fn encode_ability(encoder: &mut Encoder, ability: &AbilityTreeRuleContribution) {
    encoder.u32(ability.id.get());
    encoder.text(&ability.stable_key);
    encoder.u8(ability.effect_class as u8);
    encoder.text(&ability.rule_key);
    encode_texts(encoder, &ability.external_unlock_keys);
    encoder.u32(ability.effects.len() as u32);
    for effect in &ability.effects {
        encoder.u8(effect.operation() as u8);
        encoder.text(effect.target_key());
        encode_parameter(encoder, effect.value());
        encoder.u8(effect.unit() as u8);
        encoder.text(effect.condition().unwrap_or(""));
    }
    encode_parameters(encoder, &ability.parameters);
}

fn encode_texts(encoder: &mut Encoder, values: &[Box<str>]) {
    encoder.u32(values.len() as u32);
    for value in values {
        encoder.text(value);
    }
}

fn encode_parameters(encoder: &mut Encoder, values: &[ExactParameter]) {
    encoder.u32(values.len() as u32);
    for value in values {
        encode_parameter(encoder, *value);
    }
}

fn encode_parameter(encoder: &mut Encoder, value: ExactParameter) {
    encoder.i64(value.coefficient());
    encoder.u8(value.scale());
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RunRuntimeError {
    InvalidDenominator,
    InvalidFragmentAmount,
    UnknownAbilityTreeNode(AbilityTreeNodeId),
    DuplicateAbilityTreeNode,
}
