//! Source-checked domain limits, independent from scheduling policy.

use super::source_keys;
use crate::{
    divergent_universe::DivergentUniverseBundleCandidate,
    divergent_universe_curio_catalog::DivergentUniverseCurioStateId,
    divergent_universe_decisions::{
        CurioDomainExpiryDefinition, CurioDomainExpiryPolicy, DecisionDataError, key,
    },
    divergent_universe_decisions_generated::{
        SoraConfig, du_curio_domain_expiry_policy::DuCurioDomainExpiryPolicy,
    },
};
use std::collections::BTreeSet;

pub(super) fn compile(
    config: &SoraConfig,
    reference: &DivergentUniverseBundleCandidate,
) -> Result<Box<[CurioDomainExpiryDefinition]>, DecisionDataError> {
    let mut seen = BTreeSet::new();
    let mut result = Vec::new();
    for row in config.du_curio_domain_expiries().ordered_rows() {
        let state = DivergentUniverseCurioStateId::new(row.state_key.as_str())
            .map_err(|_| DecisionDataError::InvalidReference)?;
        let definition = reference
            .curio_catalog()
            .states()
            .iter()
            .find(|candidate| candidate.id == state)
            .ok_or(DecisionDataError::InvalidReference)?;
        let limit = u16::try_from(row.domain_limit)
            .ok()
            .filter(|limit| *limit > 0)
            .ok_or(DecisionDataError::InvalidReward)?;
        let parameter = row
            .limit_parameter
            .checked_sub(1)
            .and_then(|index| usize::try_from(index).ok())
            .and_then(|index| definition.effect_parameters.get(index));
        if row.id <= 0
            || definition.curio.is_none()
            || definition.effect_ids.len() != 1
            || definition.effect_ids[0].as_ref() != row.effect_id
            || parameter.map(AsRef::as_ref) != Some(limit.to_string().as_str())
        {
            return Err(DecisionDataError::InvalidReference);
        }
        if !seen.insert(state.clone()) {
            return Err(DecisionDataError::InvalidIdentity);
        }
        if row.summary_en.is_empty()
            || row.summary_zh_cn.is_empty()
            || row.policy_note.is_empty()
            || row.replacement_condition.is_empty()
        {
            return Err(DecisionDataError::InvalidPolicy);
        }
        let policy = match row.policy {
            DuCurioDomainExpiryPolicy::ActiveFutureSelectedDomainsDiscard => {
                CurioDomainExpiryPolicy::VersionedProjectPolicyActiveFutureSelectedDomainsDiscard
            }
            DuCurioDomainExpiryPolicy::ActiveFutureSelectedDomainsGrantThenDiscard => {
                CurioDomainExpiryPolicy::VersionedProjectPolicyActiveFutureSelectedDomainsGrantThenDiscard
            }
        };
        result.push(CurioDomainExpiryDefinition {
            key: key(&row.stable_key)?,
            state,
            effect_id: row.effect_id.clone().into(),
            domain_limit: limit,
            policy,
            summary_en: row.summary_en.clone().into(),
            summary_zh_cn: row.summary_zh_cn.clone().into(),
            policy_note: row.policy_note.clone().into(),
            replacement_condition: row.replacement_condition.clone().into(),
            sources: source_keys(config, &row.source_ids)?,
        });
    }
    result.sort_by(|left, right| left.state.cmp(&right.state));
    Ok(result.into_boxed_slice())
}
