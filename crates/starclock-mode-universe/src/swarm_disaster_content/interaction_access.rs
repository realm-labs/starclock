//! Immutable Occurrence, Service and Adventure runtime input boundary.

use super::SwarmDisasterContentCatalog;

#[derive(Clone, Debug)]
pub(crate) struct InteractionRuntimeInput {
    pub(crate) occurrences: Box<[OccurrenceInput]>,
    pub(crate) variants: Box<[VariantInput]>,
    pub(crate) choices: Box<[ChoiceInput]>,
    pub(crate) services: Box<[ServiceInput]>,
    pub(crate) adventures: Box<[AdventureInput]>,
    pub(crate) currencies: Box<[CurrencyInput]>,
    pub(crate) service_rules: Box<[ServiceRuleInput]>,
}

#[derive(Clone, Debug)]
pub(crate) struct OccurrenceInput {
    pub(crate) id: u32,
    pub(crate) key: Box<str>,
    pub(crate) order: u16,
    pub(crate) source_event_type: Box<str>,
    pub(crate) variant_keys: Box<[Box<str>]>,
    pub(crate) pool_rules: Box<str>,
}

#[derive(Clone, Debug)]
pub(crate) struct VariantInput {
    pub(crate) id: u32,
    pub(crate) key: Box<str>,
    pub(crate) occurrence_keys: Box<[Box<str>]>,
    pub(crate) choice_keys: Box<[Box<str>]>,
    pub(crate) graph: Box<str>,
}

#[derive(Clone, Debug)]
pub(crate) struct ChoiceInput {
    pub(crate) id: u32,
    pub(crate) key: Box<str>,
    pub(crate) variant: u32,
    pub(crate) ordinal: u16,
    pub(crate) node_ordinal: u16,
    pub(crate) option_ordinal: u16,
    pub(crate) conditions: Box<str>,
    pub(crate) costs: Box<str>,
    pub(crate) outcomes: Box<str>,
    pub(crate) display: Box<str>,
}

#[derive(Clone, Debug)]
pub(crate) struct ServiceInput {
    pub(crate) id: u32,
    pub(crate) key: Box<str>,
    pub(crate) shared_key: Box<str>,
    pub(crate) service_kind: Box<str>,
    pub(crate) resource_key: Option<Box<str>>,
    pub(crate) parameters: Box<str>,
    pub(crate) eligibility: Box<str>,
    pub(crate) price_policy: Box<str>,
}

#[derive(Clone, Debug)]
pub(crate) struct AdventureInput {
    pub(crate) id: u32,
    pub(crate) key: Box<str>,
    pub(crate) adventure_type: Box<str>,
    pub(crate) parameter_group: Box<str>,
    pub(crate) tier: Box<str>,
    pub(crate) offered_result: Box<str>,
    pub(crate) reward_program: Box<str>,
}

#[derive(Clone, Debug)]
pub(crate) struct CurrencyInput {
    pub(crate) id: u32,
    pub(crate) key: Box<str>,
    pub(crate) resource_key: Box<str>,
    pub(crate) initial_value: Box<str>,
    pub(crate) cap_policy: Box<str>,
}

#[derive(Clone, Debug)]
pub(crate) struct ServiceRuleInput {
    pub(crate) id: u32,
    pub(crate) key: Box<str>,
    pub(crate) service_key: Box<str>,
    pub(crate) conditions: Box<str>,
    pub(crate) costs: Box<str>,
    pub(crate) operations: Box<str>,
}

impl SwarmDisasterContentCatalog {
    pub(crate) fn interaction_runtime_input(&self) -> InteractionRuntimeInput {
        InteractionRuntimeInput {
            occurrences: self
                .occurrences
                .iter()
                .map(|row| OccurrenceInput {
                    id: row.id.0,
                    key: row.key.clone(),
                    order: row.order,
                    source_event_type: row.source_event_type.clone(),
                    variant_keys: row.variant_keys.clone(),
                    pool_rules: row.pool_rules.clone(),
                })
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            variants: self
                .occurrence_variants
                .iter()
                .map(|row| VariantInput {
                    id: row.id.0,
                    key: row.key.clone(),
                    occurrence_keys: row.occurrence_keys.clone(),
                    choice_keys: row.choice_keys.clone(),
                    graph: row.graph.clone(),
                })
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            choices: self
                .occurrence_choices
                .iter()
                .map(|row| ChoiceInput {
                    id: row.id.0,
                    key: row.key.clone(),
                    variant: row.variant.0,
                    ordinal: row.ordinal,
                    node_ordinal: row.node_ordinal,
                    option_ordinal: row.option_ordinal,
                    conditions: row.conditions.clone(),
                    costs: row.costs.clone(),
                    outcomes: row.outcomes.clone(),
                    display: row.display.clone(),
                })
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            services: self
                .services
                .iter()
                .map(|row| ServiceInput {
                    id: row.id.0,
                    key: row.key.clone(),
                    shared_key: row.shared_key.clone(),
                    service_kind: row.service_kind.clone(),
                    resource_key: row.resource_key.clone(),
                    parameters: row.parameters.clone(),
                    eligibility: row.eligibility.clone(),
                    price_policy: row.price_policy.clone(),
                })
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            adventures: self
                .adventure_outcomes
                .iter()
                .map(|row| AdventureInput {
                    id: row.id.0,
                    key: row.key.clone(),
                    adventure_type: row.adventure_type.clone(),
                    parameter_group: row.parameter_group.clone(),
                    tier: row.tier.clone(),
                    offered_result: row.offered_result.clone(),
                    reward_program: row.reward_program.clone(),
                })
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            currencies: self
                .currencies
                .iter()
                .map(|row| CurrencyInput {
                    id: row.id.0,
                    key: row.key.clone(),
                    resource_key: row.resource_key.clone(),
                    initial_value: row.initial_value.clone(),
                    cap_policy: row.cap_policy.clone(),
                })
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            service_rules: self
                .service_rules
                .iter()
                .map(|row| ServiceRuleInput {
                    id: row.id.0,
                    key: row.key.clone(),
                    service_key: row.service_key.clone(),
                    conditions: row.conditions.clone(),
                    costs: row.costs.clone(),
                    operations: row.operations.clone(),
                })
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        }
    }
}
