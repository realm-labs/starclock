use crate::swarm_disaster_content::SwarmDisasterContentErrorKind;
use crate::swarm_disaster_generated::{
    SoraConfig, swarm_disaster_adventure_outcome::SwarmDisasterAdventureOutcome,
    swarm_disaster_blessing::SwarmDisasterBlessing,
    swarm_disaster_blessing_level::SwarmDisasterBlessingLevel,
    swarm_disaster_curio::SwarmDisasterCurio, swarm_disaster_curio_rule::SwarmDisasterCurioRule,
    swarm_disaster_curio_state::SwarmDisasterCurioState,
    swarm_disaster_currency::SwarmDisasterCurrency,
    swarm_disaster_occurrence::SwarmDisasterOccurrence,
    swarm_disaster_occurrence_choice::SwarmDisasterOccurrenceChoice,
    swarm_disaster_occurrence_variant::SwarmDisasterOccurrenceVariant,
    swarm_disaster_pool_membership::SwarmDisasterPoolMembership,
    swarm_disaster_service::SwarmDisasterService,
    swarm_disaster_service_rule::SwarmDisasterServiceRule,
};

use super::{
    json, metadata, nonempty, nonnegative_u16, positive, positive_u8, scalar, stable, text_list,
    text_values,
};
use crate::swarm_disaster_content::{SwarmDisasterContentError, types::*};

pub(super) struct InventoryTables {
    pub(super) blessings: Box<[BlessingDefinition]>,
    pub(super) blessing_levels: Box<[BlessingLevelDefinition]>,
    pub(super) pool_memberships: Box<[PoolMembershipDefinition]>,
    pub(super) curios: Box<[CurioDefinition]>,
    pub(super) curio_states: Box<[CurioStateDefinition]>,
    pub(super) curio_rules: Box<[CurioRuleDefinition]>,
    pub(super) occurrences: Box<[OccurrenceDefinition]>,
    pub(super) occurrence_variants: Box<[OccurrenceVariantDefinition]>,
    pub(super) occurrence_choices: Box<[OccurrenceChoiceDefinition]>,
    pub(super) services: Box<[ServiceDefinition]>,
    pub(super) adventure_outcomes: Box<[AdventureOutcomeDefinition]>,
    pub(super) currencies: Box<[CurrencyDefinition]>,
    pub(super) service_rules: Box<[ServiceRuleDefinition]>,
}

macro_rules! rows {
    ($source:expr, $accessor:ident, $lower:ident) => {
        $source
            .$accessor()
            .ordered_rows()
            .map($lower)
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice()
    };
}

pub(super) fn lower(source: &SoraConfig) -> Result<InventoryTables, SwarmDisasterContentError> {
    Ok(InventoryTables {
        blessings: rows!(source, swarm_disaster_blessing, blessing),
        blessing_levels: rows!(source, swarm_disaster_blessing_level, blessing_level),
        pool_memberships: rows!(source, swarm_disaster_pool_membership, pool_membership),
        curios: rows!(source, swarm_disaster_curio, curio),
        curio_states: rows!(source, swarm_disaster_curio_state, curio_state),
        curio_rules: rows!(source, swarm_disaster_curio_rule, curio_rule),
        occurrences: rows!(source, swarm_disaster_occurrence, occurrence),
        occurrence_variants: rows!(
            source,
            swarm_disaster_occurrence_variant,
            occurrence_variant
        ),
        occurrence_choices: rows!(source, swarm_disaster_occurrence_choice, occurrence_choice),
        services: rows!(source, swarm_disaster_service, service),
        adventure_outcomes: rows!(source, swarm_disaster_adventure_outcome, adventure_outcome),
        currencies: rows!(source, swarm_disaster_currency, currency),
        service_rules: rows!(source, swarm_disaster_service_rule, service_rule),
    })
}

fn blessing(row: &SwarmDisasterBlessing) -> Result<BlessingDefinition, SwarmDisasterContentError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    Ok(BlessingDefinition {
        id: BlessingId(positive(row.id, &row.stable_key)?),
        key: stable(&row.stable_key, &row.stable_key)?,
        shared_key: stable(&row.shared_blessing_id, &row.stable_key)?,
        path_key: stable(&row.path_id, &row.stable_key)?,
        rarity: positive_u8(row.rarity, &row.stable_key)?,
        level_keys: text_list(&row.level_ids, &row.stable_key)?,
        pool_rules: json(&row.pool_rules_json, &row.stable_key)?,
    })
}

fn blessing_level(
    row: &SwarmDisasterBlessingLevel,
) -> Result<BlessingLevelDefinition, SwarmDisasterContentError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    Ok(BlessingLevelDefinition {
        id: BlessingLevelId(positive(row.id, &row.stable_key)?),
        key: stable(&row.stable_key, &row.stable_key)?,
        blessing: BlessingId(positive(row.blessing_id, &row.stable_key)?),
        shared_blessing_key: stable(&row.shared_blessing_id, &row.stable_key)?,
        shared_level_key: stable(&row.shared_blessing_level_id, &row.stable_key)?,
        level: positive_u8(row.level, &row.stable_key)?,
        parameters: row.parameter_values.as_deref().map_or_else(
            || Ok(Vec::<Box<str>>::new().into_boxed_slice()),
            |values| text_values(values, &row.stable_key),
        )?,
        effect_program: json(&row.effect_program_json, &row.stable_key)?,
    })
}

fn pool_membership(
    row: &SwarmDisasterPoolMembership,
) -> Result<PoolMembershipDefinition, SwarmDisasterContentError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    Ok(PoolMembershipDefinition {
        id: PoolMembershipId(positive(row.id, &row.stable_key)?),
        key: stable(&row.stable_key, &row.stable_key)?,
        pool_key: stable(&row.pool_id, &row.stable_key)?,
        member_kind: nonempty(&row.member_kind, &row.stable_key)?,
        member_key: stable(&row.member_id, &row.stable_key)?,
        eligibility: json(&row.eligibility_json, &row.stable_key)?,
        weight_policy: json(&row.weight_policy_json, &row.stable_key)?,
    })
}

fn curio(row: &SwarmDisasterCurio) -> Result<CurioDefinition, SwarmDisasterContentError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    Ok(CurioDefinition {
        id: CurioId(positive(row.id, &row.stable_key)?),
        key: stable(&row.stable_key, &row.stable_key)?,
        mode_copy_key: stable(&row.mode_copy_id, &row.stable_key)?,
        pool_category: nonempty(&row.pool_category, &row.stable_key)?,
        pool_rules: json(&row.pool_rules_json, &row.stable_key)?,
        initial_state: CurioStateId(positive(row.initial_state_id, &row.stable_key)?),
    })
}

fn curio_state(
    row: &SwarmDisasterCurioState,
) -> Result<CurioStateDefinition, SwarmDisasterContentError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    Ok(CurioStateDefinition {
        id: CurioStateId(positive(row.id, &row.stable_key)?),
        key: stable(&row.stable_key, &row.stable_key)?,
        curio: CurioId(positive(row.curio_id, &row.stable_key)?),
        state: nonempty(&row.state, &row.stable_key)?,
        charges: row
            .charges
            .as_deref()
            .map(|value| scalar(value, &row.stable_key))
            .transpose()?,
        effect_program: json(&row.effect_program_json, &row.stable_key)?,
        lifecycle: json(&row.lifecycle_json, &row.stable_key)?,
        repair_target: json(&row.repair_target_json, &row.stable_key)?,
    })
}

fn curio_rule(
    row: &SwarmDisasterCurioRule,
) -> Result<CurioRuleDefinition, SwarmDisasterContentError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    Ok(CurioRuleDefinition {
        id: CurioRuleId(positive(row.id, &row.stable_key)?),
        key: stable(&row.stable_key, &row.stable_key)?,
        curio: CurioId(positive(row.curio_id, &row.stable_key)?),
        state: CurioStateId(positive(row.state_id, &row.stable_key)?),
        trigger_phase: nonempty(&row.trigger_phase, &row.stable_key)?,
        trigger: json(&row.trigger_json, &row.stable_key)?,
        lifecycle: json(&row.lifecycle_json, &row.stable_key)?,
        replacement_policy: json(&row.replacement_policy_json, &row.stable_key)?,
    })
}

fn occurrence(
    row: &SwarmDisasterOccurrence,
) -> Result<OccurrenceDefinition, SwarmDisasterContentError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    Ok(OccurrenceDefinition {
        id: OccurrenceId(positive(row.id, &row.stable_key)?),
        key: stable(&row.stable_key, &row.stable_key)?,
        order: nonnegative_u16(row.handbook_order, &row.stable_key)?,
        source_event_type: nonempty(&row.source_event_type, &row.stable_key)?,
        variant_keys: text_list(&row.variant_ids, &row.stable_key)?,
        pool_rules: json(&row.pool_rules_json, &row.stable_key)?,
    })
}

fn occurrence_variant(
    row: &SwarmDisasterOccurrenceVariant,
) -> Result<OccurrenceVariantDefinition, SwarmDisasterContentError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    Ok(OccurrenceVariantDefinition {
        id: OccurrenceVariantId(positive(row.id, &row.stable_key)?),
        key: stable(&row.stable_key, &row.stable_key)?,
        occurrence_keys: text_list(&row.occurrence_ids, &row.stable_key)?,
        choice_keys: text_list(&row.choice_ids, &row.stable_key)?,
        graph: json(&row.graph_refs_json, &row.stable_key)?,
    })
}

fn occurrence_choice(
    row: &SwarmDisasterOccurrenceChoice,
) -> Result<OccurrenceChoiceDefinition, SwarmDisasterContentError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    json(&row.parameter_vectors_json, &row.stable_key)?;
    json(&row.text_digests_json, &row.stable_key)?;
    Ok(OccurrenceChoiceDefinition {
        id: OccurrenceChoiceId(positive(row.id, &row.stable_key)?),
        key: stable(&row.stable_key, &row.stable_key)?,
        variant: OccurrenceVariantId(positive(row.variant_id, &row.stable_key)?),
        ordinal: positive(row.ordinal, &row.stable_key)?
            .try_into()
            .map_err(|_| {
                super::error(SwarmDisasterContentErrorKind::Identifier, &row.stable_key)
            })?,
        node_ordinal: nonnegative_u16(row.node_ordinal, &row.stable_key)?,
        option_ordinal: nonnegative_u16(row.option_ordinal, &row.stable_key)?,
        conditions: json(&row.conditions_json, &row.stable_key)?,
        costs: json(&row.costs_json, &row.stable_key)?,
        outcomes: json(&row.ordered_outcomes_json, &row.stable_key)?,
        display: json(&row.dynamic_display_options_json, &row.stable_key)?,
    })
}

fn service(row: &SwarmDisasterService) -> Result<ServiceDefinition, SwarmDisasterContentError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    Ok(ServiceDefinition {
        id: ServiceId(positive(row.id, &row.stable_key)?),
        key: stable(&row.stable_key, &row.stable_key)?,
        shared_key: stable(&row.shared_service_id, &row.stable_key)?,
        service_kind: nonempty(&row.service_kind, &row.stable_key)?,
        resource_key: row
            .resource_id
            .as_deref()
            .map(|value| stable(value, &row.stable_key))
            .transpose()?,
        parameters: json(&row.parameters_json, &row.stable_key)?,
        eligibility: json(&row.eligibility_json, &row.stable_key)?,
        price_policy: json(&row.price_policy_json, &row.stable_key)?,
    })
}

fn adventure_outcome(
    row: &SwarmDisasterAdventureOutcome,
) -> Result<AdventureOutcomeDefinition, SwarmDisasterContentError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    Ok(AdventureOutcomeDefinition {
        id: AdventureOutcomeId(positive(row.id, &row.stable_key)?),
        key: stable(&row.stable_key, &row.stable_key)?,
        adventure_type: nonempty(&row.adventure_type, &row.stable_key)?,
        parameter_group: stable(&row.parameter_group_id, &row.stable_key)?,
        tier: nonempty(&row.tier, &row.stable_key)?,
        offered_result: json(&row.offered_result_json, &row.stable_key)?,
        reward_program: json(&row.reward_program_json, &row.stable_key)?,
    })
}

fn currency(row: &SwarmDisasterCurrency) -> Result<CurrencyDefinition, SwarmDisasterContentError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    Ok(CurrencyDefinition {
        id: CurrencyId(positive(row.id, &row.stable_key)?),
        key: stable(&row.stable_key, &row.stable_key)?,
        resource_key: stable(&row.resource_id, &row.stable_key)?,
        initial_value: scalar(&row.initial_value, &row.stable_key)?,
        cap_policy: json(&row.cap_policy_json, &row.stable_key)?,
    })
}

fn service_rule(
    row: &SwarmDisasterServiceRule,
) -> Result<ServiceRuleDefinition, SwarmDisasterContentError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    Ok(ServiceRuleDefinition {
        id: ServiceRuleId(positive(row.id, &row.stable_key)?),
        key: stable(&row.stable_key, &row.stable_key)?,
        service_key: stable(&row.service_id, &row.stable_key)?,
        conditions: json(&row.conditions_json, &row.stable_key)?,
        costs: json(&row.costs_json, &row.stable_key)?,
        operations: json(&row.ordered_operations_json, &row.stable_key)?,
    })
}
