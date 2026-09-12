//! Source-linked public evolution options, separate from accepted state edges.

use super::{ordinals, source_keys};
use crate::divergent_universe_decisions::{
    CurioEvolutionDefinition, DecisionDataError, EvolutionEventDefinition, EvolutionEventPolicy,
    EvolutionOptionDefinition, EvolutionOptionEffect, RewardRarityRange, key,
};
use crate::divergent_universe_decisions_generated::{
    SoraConfig, du_evolution_event_policy::DuEvolutionEventPolicy,
    du_evolution_option_kind::DuEvolutionOptionKind, du_evolution_options::DuEvolutionOptions,
};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn compile(
    config: &SoraConfig,
    evolutions: &[CurioEvolutionDefinition],
) -> Result<Box<[EvolutionEventDefinition]>, DecisionDataError> {
    let mut layer_counts = BTreeMap::<i32, usize>::new();
    let mut layer_owners = BTreeSet::new();
    let mut edges = BTreeSet::new();
    let mut result = Vec::new();
    for row in config.du_evolution_options().ordered_rows() {
        if config.du_evolution_events().get(&row.event_id).is_none() {
            return Err(DecisionDataError::InvalidReference);
        }
    }
    for row in config.du_evolution_events().ordered_rows() {
        if row.id <= 0 || !edges.insert(row.evolution_id) {
            return Err(DecisionDataError::InvalidIdentity);
        }
        if !(2..=64).contains(&row.layer_ordinal)
            || row.policy_note.is_empty()
            || row.replacement_condition.is_empty()
        {
            return Err(DecisionDataError::InvalidPolicy);
        }
        let edge_key = &config
            .du_curio_evolutions()
            .get(&row.evolution_id)
            .ok_or(DecisionDataError::InvalidReference)?
            .stable_key;
        let evolution = evolutions
            .iter()
            .find(|edge| edge.key.as_ref() == edge_key)
            .ok_or(DecisionDataError::InvalidReference)?
            .clone();
        let count = layer_counts.entry(row.layer_ordinal).or_default();
        *count += 1;
        // A bounded sequence, not repeated upgrades of one owner in one layer.
        // The cap also bounds the nested shared-operation program's size.
        if *count > 3 || !layer_owners.insert((row.layer_ordinal, evolution.owner.clone())) {
            return Err(DecisionDataError::InvalidIdentity);
        }
        let mut options = config
            .du_evolution_options()
            .ordered_rows()
            .filter(|option| option.event_id == row.id)
            .collect::<Vec<_>>();
        options.sort_by_key(|option| option.ordinal);
        ordinals(options.iter().map(|option| option.ordinal))?;
        if options.len() != 3 {
            return Err(DecisionDataError::InvalidOrder);
        }
        result.push(EvolutionEventDefinition {
            key: key(&row.stable_key)?, evolution,
            layer_ordinal: u16::try_from(row.layer_ordinal).map_err(|_| DecisionDataError::InvalidOrder)?,
            options: options.into_iter().map(|row| option(config, row)).collect::<Result<Vec<_>, _>>()?.into_boxed_slice(),
            policy: match row.policy { DuEvolutionEventPolicy::ActiveTreasuresAtLayerEntryStableSequence => EvolutionEventPolicy::VersionedProjectPolicyActiveTreasuresAtLayerEntryStableSequence },
            policy_note: row.policy_note.clone().into(), replacement_condition: row.replacement_condition.clone().into(), sources: source_keys(config, &row.source_ids)?,
        });
    }
    if result.is_empty() {
        return Err(DecisionDataError::InvalidReference);
    }
    result.sort_by(|left, right| {
        (left.layer_ordinal, &left.key).cmp(&(right.layer_ordinal, &right.key))
    });
    Ok(result.into_boxed_slice())
}

fn option(
    config: &SoraConfig,
    row: &DuEvolutionOptions,
) -> Result<EvolutionOptionDefinition, DecisionDataError> {
    if row.id <= 0 || row.name_en.is_empty() || row.name_zh_cn.is_empty() {
        return Err(DecisionDataError::InvalidIdentity);
    }
    if row.fragment_cost < 0
        || row.fragment_grant < 0
        || !(0..=8).contains(&row.curio_count)
        || !(1..=3).contains(&row.minimum_rarity)
        || !(row.minimum_rarity..=3).contains(&row.maximum_rarity)
        || !(1..=1_000_000).contains(&row.success_denominator)
        || !(0..=row.success_denominator).contains(&row.success_numerator)
    {
        return Err(DecisionDataError::InvalidReward);
    }
    let count = u16::try_from(row.curio_count).map_err(|_| DecisionDataError::InvalidReward)?;
    let rarity = RewardRarityRange {
        minimum: u8::try_from(row.minimum_rarity).map_err(|_| DecisionDataError::InvalidReward)?,
        maximum: u8::try_from(row.maximum_rarity).map_err(|_| DecisionDataError::InvalidReward)?,
    };
    let effect = match row.kind {
        DuEvolutionOptionKind::Evolve
            if count == 0 && row.success_numerator == 1 && row.success_denominator == 1 =>
        {
            EvolutionOptionEffect::Evolve
        }
        DuEvolutionOptionKind::ChanceEvolution
            if count == 0
                && row.success_numerator > 0
                && row.success_numerator < row.success_denominator =>
        {
            EvolutionOptionEffect::Chance {
                numerator: u32::try_from(row.success_numerator)
                    .map_err(|_| DecisionDataError::InvalidReward)?,
                denominator: u32::try_from(row.success_denominator)
                    .map_err(|_| DecisionDataError::InvalidReward)?,
            }
        }
        DuEvolutionOptionKind::RandomCurios
            if count > 0 && row.success_numerator == 0 && row.success_denominator == 1 =>
        {
            EvolutionOptionEffect::Curios { count, rarity }
        }
        DuEvolutionOptionKind::Sacrifice
            if count > 0 && row.success_numerator == 0 && row.success_denominator == 1 =>
        {
            EvolutionOptionEffect::Sacrifice { count, rarity }
        }
        _ => return Err(DecisionDataError::InvalidReward),
    };
    Ok(EvolutionOptionDefinition {
        key: key(&row.stable_key)?,
        ordinal: u16::try_from(row.ordinal).map_err(|_| DecisionDataError::InvalidOrder)?,
        name_en: row.name_en.clone().into(),
        name_zh_cn: row.name_zh_cn.clone().into(),
        fragment_cost: u64::try_from(row.fragment_cost)
            .map_err(|_| DecisionDataError::InvalidReward)?,
        fragment_grant: u64::try_from(row.fragment_grant)
            .map_err(|_| DecisionDataError::InvalidReward)?,
        effect,
        sources: source_keys(config, &row.source_ids)?,
    })
}
