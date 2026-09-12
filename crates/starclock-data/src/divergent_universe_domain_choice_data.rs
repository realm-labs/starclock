//! Validated selectable proxy domains, separate from frozen room membership.

use super::source_keys;
use crate::{
    divergent_universe_decisions::{
        BattleRewardDomain, DecisionDataError, DomainChoiceDefinition, DomainChoicePolicy, key,
    },
    divergent_universe_decisions_generated::{
        SoraConfig, du_battle_reward_domain::DuBattleRewardDomain,
        du_domain_choice_policy::DuDomainChoicePolicy,
    },
};
use std::collections::BTreeSet;

pub(super) fn compile(
    config: &SoraConfig,
) -> Result<Box<[DomainChoiceDefinition]>, DecisionDataError> {
    let mut result = Vec::new();
    let mut domains = BTreeSet::new();
    for row in config.du_domain_choices().ordered_rows() {
        let domain = match row.domain {
            DuBattleRewardDomain::Combat => BattleRewardDomain::Combat,
            DuBattleRewardDomain::Elite => BattleRewardDomain::Elite,
            DuBattleRewardDomain::Aberration => BattleRewardDomain::Aberration,
            // The current proxy policy does not enable boss topology/programs.
            DuBattleRewardDomain::Boss => return Err(DecisionDataError::InvalidPolicy),
        };
        if !domains.insert(domain) || !(1..=3).contains(&row.id) {
            return Err(DecisionDataError::InvalidIdentity);
        }
        if row.name_en.is_empty()
            || row.name_zh_cn.is_empty()
            || row.policy_note.is_empty()
            || row.replacement_condition.is_empty()
        {
            return Err(DecisionDataError::InvalidPolicy);
        }
        result.push(DomainChoiceDefinition {
            key: key(&row.stable_key)?,
            ordinal: u16::try_from(row.id).map_err(|_| DecisionDataError::InvalidIdentity)?,
            domain,
            name_en: row.name_en.clone().into(),
            name_zh_cn: row.name_zh_cn.clone().into(),
            policy: match row.policy {
                DuDomainChoicePolicy::ExplicitLaterLayerChoices => {
                    DomainChoicePolicy::VersionedProjectPolicyExplicitLaterLayerChoices
                }
            },
            policy_note: row.policy_note.clone().into(),
            replacement_condition: row.replacement_condition.clone().into(),
            sources: source_keys(config, &row.source_ids)?,
        });
    }
    if domains
        != BTreeSet::from([
            BattleRewardDomain::Combat,
            BattleRewardDomain::Elite,
            BattleRewardDomain::Aberration,
        ])
    {
        return Err(DecisionDataError::InvalidPolicy);
    }
    result.sort_by_key(|choice| choice.ordinal);
    Ok(result.into_boxed_slice())
}
