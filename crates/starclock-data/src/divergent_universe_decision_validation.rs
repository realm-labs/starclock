use std::collections::BTreeSet;

#[path = "divergent_universe_curio_acquisition_data.rs"]
mod acquisitions;
#[path = "divergent_universe_battle_blessing_data.rs"]
mod battle_blessings;
#[path = "divergent_universe_battle_fragment_data.rs"]
mod battle_fragments;
#[path = "divergent_universe_battle_route_data.rs"]
mod battle_routes;
#[path = "divergent_universe_battle_weight_data.rs"]
mod battle_weights;
#[path = "divergent_universe_curio_battle_grant_data.rs"]
mod curio_battle_grants;
#[path = "divergent_universe_curio_battle_reaction_data.rs"]
mod curio_battle_reactions;
#[path = "divergent_universe_curio_battle_stat_data.rs"]
mod curio_battle_stats;
#[path = "divergent_universe_curio_domain_grant_data.rs"]
mod curio_domain_grants;
#[path = "divergent_universe_curio_evolution_data.rs"]
mod curio_evolutions;
#[path = "divergent_universe_curio_expiry_data.rs"]
mod curio_expiries;
#[path = "divergent_universe_curio_victory_blessing_data.rs"]
mod curio_victory_blessings;
#[path = "divergent_universe_domain_choice_data.rs"]
mod domain_choices;
#[path = "divergent_universe_encounter_pool_data.rs"]
mod encounter_pool;
#[path = "divergent_universe_equation_expansion_data.rs"]
mod equation_expansion;
#[path = "divergent_universe_equation_grant_data.rs"]
mod equation_grants;
#[path = "divergent_universe_evolution_event_data.rs"]
mod evolution_events;
#[path = "divergent_universe_fragment_gain_data.rs"]
mod fragment_gains;
#[path = "divergent_universe_initial_equation_data.rs"]
mod initial_equations;
#[path = "divergent_universe_tawot_service_data.rs"]
mod tawot_services;

use crate::divergent_universe::DivergentUniverseBundleCandidate;
use crate::divergent_universe_decisions_generated::{
    SoraConfig, du_decision_binding::DuDecisionBinding, du_decision_evidence::DuDecisionEvidence,
    du_decision_exhaustion::DuDecisionExhaustion, du_decision_sampling::DuDecisionSampling,
};
use crate::divergent_universe_domain_decks;
use crate::divergent_universe_domain_layout;
use crate::divergent_universe_service_catalog::{
    DivergentUniverseOccurrenceId, DivergentUniverseOccurrenceVariantId,
};

use super::{
    DecisionCatalog, DecisionChoice, DecisionChoiceId, DecisionDataError, DecisionEvidence,
    DecisionExhaustion, DecisionOccurrence, DecisionOutcome, DecisionPolicy, DecisionPolicyKind,
    DecisionSource, key, reward, unique,
};

pub(super) fn compile(
    config: &SoraConfig,
    reference: &DivergentUniverseBundleCandidate,
) -> Result<DecisionCatalog, DecisionDataError> {
    unique(
        config
            .du_domain_layout()
            .ordered_rows()
            .map(|row| row.stable_key.as_str())
            .chain(
                config
                    .du_battle_fragments()
                    .ordered_rows()
                    .map(|row| row.stable_key.as_str()),
            )
            .chain(
                config
                    .du_curio_domain_grants()
                    .ordered_rows()
                    .map(|row| row.stable_key.as_str()),
            )
            .chain(
                config
                    .du_tawot_services()
                    .ordered_rows()
                    .map(|row| row.stable_key.as_str()),
            )
            .chain(
                config
                    .du_curio_battle_reactions()
                    .ordered_rows()
                    .map(|row| row.stable_key.as_str()),
            )
            .chain(
                config
                    .du_curio_battle_stats()
                    .ordered_rows()
                    .map(|row| row.stable_key.as_str()),
            )
            .chain(
                config
                    .du_curio_domain_expiries()
                    .ordered_rows()
                    .map(|row| row.stable_key.as_str()),
            )
            .chain(
                config
                    .du_decision_sources()
                    .ordered_rows()
                    .map(|row| row.stable_key.as_str()),
            )
            .chain(
                config
                    .du_decision_policies()
                    .ordered_rows()
                    .map(|row| row.stable_key.as_str()),
            )
            .chain(
                config
                    .du_decision_occurrences()
                    .ordered_rows()
                    .map(|row| row.stable_key.as_str()),
            )
            .chain(
                config
                    .du_decision_choices()
                    .ordered_rows()
                    .map(|row| row.stable_key.as_str()),
            )
            .chain(
                config
                    .du_decision_outcomes()
                    .ordered_rows()
                    .map(|row| row.stable_key.as_str()),
            )
            .chain(
                config
                    .du_curio_acquisitions()
                    .ordered_rows()
                    .map(|row| row.stable_key.as_str()),
            )
            .chain(
                config
                    .du_curio_fragment_gains()
                    .ordered_rows()
                    .map(|row| row.stable_key.as_str()),
            )
            .chain(
                config
                    .du_battle_blessings()
                    .ordered_rows()
                    .map(|row| row.stable_key.as_str()),
            )
            .chain(
                config
                    .du_curio_battle_weights()
                    .ordered_rows()
                    .map(|row| row.stable_key.as_str()),
            )
            .chain(
                config
                    .du_equation_grants()
                    .ordered_rows()
                    .map(|row| row.stable_key.as_str()),
            )
            .chain(
                config
                    .du_equation_expansion_rewards()
                    .ordered_rows()
                    .map(|row| row.stable_key.as_str()),
            )
            .chain(
                config
                    .du_initial_equations()
                    .ordered_rows()
                    .map(|row| row.stable_key.as_str()),
            )
            .chain(
                config
                    .du_battle_routes()
                    .ordered_rows()
                    .map(|row| row.stable_key.as_str()),
            )
            .chain(
                config
                    .du_encounter_pools()
                    .ordered_rows()
                    .map(|row| row.stable_key.as_str()),
            )
            .chain(
                config
                    .du_curio_battle_grants()
                    .ordered_rows()
                    .map(|row| row.stable_key.as_str()),
            )
            .chain(
                config
                    .du_curio_evolutions()
                    .ordered_rows()
                    .map(|row| row.stable_key.as_str()),
            )
            .chain(
                config
                    .du_evolution_events()
                    .ordered_rows()
                    .map(|row| row.stable_key.as_str()),
            )
            .chain(
                config
                    .du_curio_victory_blessings()
                    .ordered_rows()
                    .map(|row| row.stable_key.as_str()),
            )
            .chain(
                config
                    .du_domain_choices()
                    .ordered_rows()
                    .map(|row| row.stable_key.as_str()),
            )
            .chain(
                config
                    .du_evolution_options()
                    .ordered_rows()
                    .map(|row| row.stable_key.as_str()),
            ),
    )?;
    let sources = config
        .du_decision_sources()
        .ordered_rows()
        .map(|row| {
            if row.id <= 0
                || row.game_version != "4.4"
                || row.sha256.len() != 64
                || !row
                    .sha256
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
                || !row.url.starts_with("https://")
                || row.revision.is_empty()
                || row.locator.is_empty()
                || row.note.is_empty()
                || row.access_date.len() != 10
            {
                return Err(DecisionDataError::InvalidProvenance);
            }
            Ok(DecisionSource {
                key: key(&row.stable_key)?,
                url: row.url.clone().into(),
                revision: row.revision.clone().into(),
                game_version: row.game_version.clone().into(),
                access_date: row.access_date.clone().into(),
                locator: row.locator.clone().into(),
                sha256: row.sha256.clone().into(),
                note: row.note.clone().into(),
                quality: match row.quality {
                    DuDecisionEvidence::ExactStructured => DecisionEvidence::ExactStructured,
                    DuDecisionEvidence::ObservedCommunity => DecisionEvidence::ObservedCommunity,
                },
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    for row in config.du_decision_policies().ordered_rows() {
        if row.id <= 0
            || row.game_version != "4.4"
            || row.note.is_empty()
            || row.replacement_condition.is_empty()
        {
            return Err(DecisionDataError::InvalidPolicy);
        }
        source_keys(config, &row.source_ids)?;
    }
    for row in config.du_decision_choices().ordered_rows() {
        if row.id <= 0
            || config
                .du_decision_occurrences()
                .get(&row.occurrence_id)
                .is_none()
        {
            return Err(DecisionDataError::InvalidReference);
        }
    }
    for row in config.du_decision_outcomes().ordered_rows() {
        if row.id <= 0 || config.du_decision_choices().get(&row.choice_id).is_none() {
            return Err(DecisionDataError::InvalidReference);
        }
    }
    let mut variants = BTreeSet::new();
    let mut occurrences = Vec::new();
    for row in config.du_decision_occurrences().ordered_rows() {
        if row.id <= 0 || !variants.insert(&row.reference_variant) {
            return Err(DecisionDataError::InvalidIdentity);
        }
        let occurrence = DivergentUniverseOccurrenceId::new(row.reference_occurrence.as_str())
            .map_err(|_| DecisionDataError::InvalidReference)?;
        let variant = DivergentUniverseOccurrenceVariantId::new(row.reference_variant.as_str())
            .map_err(|_| DecisionDataError::InvalidReference)?;
        let handbook = reference
            .service_catalog()
            .occurrences()
            .iter()
            .find(|value| value.id == occurrence)
            .ok_or(DecisionDataError::InvalidReference)?;
        let npc = reference
            .service_catalog()
            .variants()
            .iter()
            .find(|value| value.id == variant)
            .ok_or(DecisionDataError::InvalidReference)?;
        if !handbook.variants.contains(&variant) || !npc.occurrences.contains(&occurrence) {
            return Err(DecisionDataError::InvalidReference);
        }
        let policy = config
            .du_decision_policies()
            .get(&row.policy_id)
            .ok_or(DecisionDataError::InvalidReference)?;
        let kind = match (policy.binding, policy.sampling) {
            (
                DuDecisionBinding::VersionedProjectPolicy,
                DuDecisionSampling::UniformUnownedCurrentCatalog,
            ) => DecisionPolicyKind::VersionedProjectPolicyUniformUnownedCurrentCatalog,
        };
        let exhaustion = match policy.exhaustion {
            DuDecisionExhaustion::DisableChoice => DecisionExhaustion::DisableChoice,
        };
        let mut choices = config
            .du_decision_choices()
            .ordered_rows()
            .filter(|choice| choice.occurrence_id == row.id)
            .collect::<Vec<_>>();
        choices.sort_by_key(|choice| choice.ordinal);
        ordinals(choices.iter().map(|choice| choice.ordinal))?;
        let choices = choices
            .into_iter()
            .map(|choice| {
                if choice.fragment_cost < 0
                    || choice.name_en.is_empty()
                    || choice.name_zh_cn.is_empty()
                {
                    return Err(DecisionDataError::InvalidReward);
                }
                let mut outcomes = config
                    .du_decision_outcomes()
                    .ordered_rows()
                    .filter(|outcome| outcome.choice_id == choice.id)
                    .collect::<Vec<_>>();
                outcomes.sort_by_key(|outcome| outcome.ordinal);
                ordinals(outcomes.iter().map(|outcome| outcome.ordinal))?;
                let outcomes = outcomes
                    .into_iter()
                    .map(|outcome| {
                        Ok(DecisionOutcome {
                            key: key(&outcome.stable_key)?,
                            reward: reward(
                                outcome.kind,
                                outcome.amount,
                                outcome.minimum_rarity,
                                outcome.maximum_rarity,
                            )?,
                            source: source_key(config, outcome.source_id)?,
                        })
                    })
                    .collect::<Result<Vec<_>, DecisionDataError>>()?;
                Ok(DecisionChoice {
                    id: DecisionChoiceId(key(&choice.stable_key)?),
                    ordinal: u16::try_from(choice.ordinal)
                        .map_err(|_| DecisionDataError::InvalidOrder)?,
                    name_en: choice.name_en.clone().into(),
                    name_zh_cn: choice.name_zh_cn.clone().into(),
                    fragment_cost: u64::try_from(choice.fragment_cost)
                        .map_err(|_| DecisionDataError::InvalidReward)?,
                    source: source_key(config, choice.source_id)?,
                    outcomes: outcomes.into_boxed_slice(),
                })
            })
            .collect::<Result<Vec<_>, DecisionDataError>>()?;
        occurrences.push(DecisionOccurrence {
            key: key(&row.stable_key)?,
            occurrence,
            variant,
            policy: DecisionPolicy {
                key: key(&policy.stable_key)?,
                kind,
                exhaustion,
                note: policy.note.clone().into(),
                replacement_condition: policy.replacement_condition.clone().into(),
                sources: source_keys(config, &policy.source_ids)?,
            },
            sources: source_keys(config, &row.source_ids)?,
            choices: choices.into_boxed_slice(),
        });
    }
    if occurrences.is_empty() {
        return Err(DecisionDataError::InvalidReference);
    }
    occurrences.sort_by(|left, right| left.key.cmp(&right.key));
    let curio_evolutions = curio_evolutions::compile(config, reference)?;
    let curio_battle_grants = curio_battle_grants::compile(config, reference, &curio_evolutions)?;
    let evolution_events = evolution_events::compile(config, &curio_evolutions)?;
    let curio_domain_expiries = curio_expiries::compile(config, reference)?;
    let curio_victory_blessings = curio_victory_blessings::compile(
        config,
        reference,
        &curio_evolutions,
        &curio_domain_expiries,
    )?;
    let domain_choices = domain_choices::compile(config)?;
    let battle_fragments = battle_fragments::compile(config)?;
    let curio_domain_grants =
        curio_domain_grants::compile(config, reference, &curio_domain_expiries)?;
    Ok(DecisionCatalog {
        domain_decks: divergent_universe_domain_decks::compile(config)?,
        domain_layout: divergent_universe_domain_layout::compile(config, reference)?,
        digest: [0; 32],
        sources: sources.into_boxed_slice(),
        occurrences: occurrences.into_boxed_slice(),
        curio_acquisitions: acquisitions::compile(config, reference)?,
        curio_fragment_gains: fragment_gains::compile(config, reference)?,
        curio_domain_expiries,
        curio_domain_grants,
        curio_battle_stats: curio_battle_stats::compile(config, reference)?,
        curio_battle_reactions: curio_battle_reactions::compile(config, reference)?,
        tawot_services: tawot_services::compile(config, reference)?,
        battle_blessings: battle_blessings::compile(config, reference)?,
        curio_battle_weights: battle_weights::compile(config, reference)?,
        equation_grants: equation_grants::compile(config, reference)?,
        equation_expansion_rewards: equation_expansion::compile(config, reference)?,
        initial_equations: initial_equations::compile(config, reference)?,
        battle_route: battle_routes::compile(config)?,
        encounter_pool: encounter_pool::compile(config, reference)?,
        curio_battle_grants,
        curio_victory_blessings,
        domain_choices,
        battle_fragments,
        curio_evolutions,
        evolution_events,
    })
}

fn source_key(config: &SoraConfig, id: i32) -> Result<Box<str>, DecisionDataError> {
    key(&config
        .du_decision_sources()
        .get(&id)
        .ok_or(DecisionDataError::InvalidReference)?
        .stable_key)
}

fn source_keys(config: &SoraConfig, ids: &[i32]) -> Result<Box<[Box<str>]>, DecisionDataError> {
    if ids.is_empty() || ids.iter().collect::<BTreeSet<_>>().len() != ids.len() {
        return Err(DecisionDataError::InvalidProvenance);
    }
    ids.iter()
        .map(|id| source_key(config, *id))
        .collect::<Result<Vec<_>, _>>()
        .map(Vec::into_boxed_slice)
}

pub(super) fn ordinals(values: impl Iterator<Item = i32>) -> Result<(), DecisionDataError> {
    let values = values.collect::<Vec<_>>();
    if values.is_empty()
        || values.len() > 64
        || values
            .iter()
            .enumerate()
            .any(|(index, value)| i32::try_from(index + 1) != Ok(*value))
    {
        return Err(DecisionDataError::InvalidOrder);
    }
    Ok(())
}
