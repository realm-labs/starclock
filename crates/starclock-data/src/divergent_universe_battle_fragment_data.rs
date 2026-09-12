//! Domain-exact coverage of provisional fixed base battle rewards.

use super::source_keys;
use crate::{
    divergent_universe_decisions::{
        BattleFragmentDefinition, BattleFragmentPolicy, BattleRewardDomain, DecisionDataError,
        DomainChoiceDefinition, key,
    },
    divergent_universe_decisions_generated::{
        SoraConfig, du_battle_fragment_policy::DuBattleFragmentPolicy,
        du_battle_reward_domain::DuBattleRewardDomain,
    },
};
use std::collections::BTreeSet;

pub(super) fn compile(
    config: &SoraConfig,
    choices: &[DomainChoiceDefinition],
) -> Result<Box<[BattleFragmentDefinition]>, DecisionDataError> {
    let mut domains = BTreeSet::new();
    let mut result = Vec::new();
    for row in config.du_battle_fragments().ordered_rows() {
        let domain = match row.domain {
            DuBattleRewardDomain::Combat => BattleRewardDomain::Combat,
            DuBattleRewardDomain::Elite => BattleRewardDomain::Elite,
            DuBattleRewardDomain::Aberration => BattleRewardDomain::Aberration,
            DuBattleRewardDomain::Boss => return Err(DecisionDataError::InvalidPolicy),
        };
        if row.id <= 0 || !domains.insert(domain) {
            return Err(DecisionDataError::InvalidIdentity);
        }
        if row.amount <= 0 {
            return Err(DecisionDataError::InvalidReward);
        }
        if row.policy_note.is_empty() || row.replacement_condition.is_empty() {
            return Err(DecisionDataError::InvalidPolicy);
        }
        result.push(BattleFragmentDefinition {
            key: key(&row.stable_key)?,
            domain,
            amount: u64::try_from(row.amount).map_err(|_| DecisionDataError::InvalidReward)?,
            policy: match row.policy {
                DuBattleFragmentPolicy::FixedVerifiedDomainCredit => {
                    BattleFragmentPolicy::VersionedProjectPolicyFixedVerifiedDomainCredit
                }
            },
            policy_note: row.policy_note.clone().into(),
            replacement_condition: row.replacement_condition.clone().into(),
            sources: source_keys(config, &row.source_ids)?,
        });
    }
    if domains != choices.iter().map(|choice| choice.domain).collect() {
        return Err(DecisionDataError::InvalidPolicy);
    }
    result.sort_by_key(|row| row.domain);
    Ok(result.into_boxed_slice())
}
