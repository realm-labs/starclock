//! Source-joined victory counts with independently authored domains and rarity.

use super::source_keys;
use crate::{
    divergent_universe::DivergentUniverseBundleCandidate,
    divergent_universe_decisions::{
        BattleRewardDomain, CurioDomainExpiryDefinition, CurioDomainExpiryPolicy,
        CurioEvolutionDefinition, CurioVictoryBlessingDefinition, CurioVictoryBlessingPolicy,
        DecisionDataError, RewardRarityRange, key,
    },
    divergent_universe_decisions_generated::{
        SoraConfig, du_battle_reward_domain::DuBattleRewardDomain,
        du_victory_blessing_policy::DuVictoryBlessingPolicy,
    },
};
use std::collections::BTreeSet;

pub(super) fn compile(
    config: &SoraConfig,
    reference: &DivergentUniverseBundleCandidate,
    evolutions: &[CurioEvolutionDefinition],
    expiries: &[CurioDomainExpiryDefinition],
) -> Result<Box<[CurioVictoryBlessingDefinition]>, DecisionDataError> {
    let mut definitions = Vec::new();
    let mut states = BTreeSet::new();
    for row in config.du_curio_victory_blessings().ordered_rows() {
        let state = reference
            .curio_catalog()
            .states()
            .iter()
            .find(|state| {
                state.id.as_str() == row.state_key
                    && (state.curio.is_some() || evolutions.iter().any(|edge| edge.to == state.id))
            })
            .ok_or(DecisionDataError::InvalidReference)?;
        if !states.insert(state.id.clone()) {
            return Err(DecisionDataError::InvalidIdentity);
        }
        if !(1..=64).contains(&row.count) {
            return Err(DecisionDataError::InvalidReward);
        }
        let parameter = row
            .count_parameter
            .checked_sub(1)
            .and_then(|index| usize::try_from(index).ok())
            .and_then(|index| state.effect_parameters.get(index));
        if row.id <= 0
            || state.effect_ids.len() != 1
            || state.effect_ids[0].as_ref() != row.effect_id
            || parameter.map(AsRef::as_ref) != Some(row.count.to_string().as_str())
        {
            return Err(DecisionDataError::InvalidReference);
        }
        if !(1..=3).contains(&row.minimum_rarity)
            || !(row.minimum_rarity..=3).contains(&row.maximum_rarity)
        {
            return Err(DecisionDataError::InvalidReward);
        }
        let domains = row
            .domains
            .iter()
            .map(|domain| match domain {
                DuBattleRewardDomain::Combat => BattleRewardDomain::Combat,
                DuBattleRewardDomain::Elite => BattleRewardDomain::Elite,
                DuBattleRewardDomain::Aberration => BattleRewardDomain::Aberration,
                DuBattleRewardDomain::Boss => BattleRewardDomain::Boss,
            })
            .collect::<BTreeSet<_>>();
        if domains.is_empty()
            || domains.len() != row.domains.len()
            || row.summary_en.is_empty()
            || row.summary_zh_cn.is_empty()
            || row.policy_note.is_empty()
            || row.replacement_condition.is_empty()
        {
            return Err(DecisionDataError::InvalidPolicy);
        }
        definitions.push(CurioVictoryBlessingDefinition {
            key: key(&row.stable_key)?, state: state.id.clone(),
            effect_id: row.effect_id.clone().into(),
            count: u16::try_from(row.count).map_err(|_| DecisionDataError::InvalidReward)?,
            rarity: RewardRarityRange {
                minimum: u8::try_from(row.minimum_rarity).map_err(|_| DecisionDataError::InvalidReward)?,
                maximum: u8::try_from(row.maximum_rarity).map_err(|_| DecisionDataError::InvalidReward)?,
            },
            domains: domains.into_iter().collect(),
            policy: match row.policy {
                DuVictoryBlessingPolicy::BoundDomainUniformUnownedAvailableSubset => {
                    if expiries.iter().any(|expiry| expiry.state == state.id) {
                        return Err(DecisionDataError::InvalidPolicy);
                    }
                    CurioVictoryBlessingPolicy::VersionedProjectPolicyBoundDomainUniformUnownedAvailableSubset
                }
                DuVictoryBlessingPolicy::PositiveDomainAllowanceUniformUnownedAvailableSubset => {
                    if !expiries.iter().any(|expiry| expiry.state == state.id
                        && expiry.effect_id.as_ref() == row.effect_id
                        && expiry.policy == CurioDomainExpiryPolicy::VersionedProjectPolicyActiveFutureSelectedDomainsDiscard) {
                        return Err(DecisionDataError::InvalidReference);
                    }
                    if row.minimum_rarity != 1 || row.maximum_rarity != 3 || row.domains.len() != 4 {
                        return Err(DecisionDataError::InvalidPolicy);
                    }
                    CurioVictoryBlessingPolicy::VersionedProjectPolicyPositiveDomainAllowanceUniformUnownedAvailableSubset
                }
            },
            summary_en: row.summary_en.clone().into(), summary_zh_cn: row.summary_zh_cn.clone().into(),
            policy_note: row.policy_note.clone().into(),
            replacement_condition: row.replacement_condition.clone().into(),
            sources: source_keys(config, &row.source_ids)?,
        });
    }
    if definitions.is_empty() {
        return Err(DecisionDataError::InvalidReference);
    }
    definitions.sort_by(|left, right| left.state.cmp(&right.state));
    Ok(definitions.into_boxed_slice())
}
