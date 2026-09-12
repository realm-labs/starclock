//! Source operand joins and exact-once ownership of grant-before-discard policies.

use super::source_keys;
use crate::{
    divergent_universe::DivergentUniverseBundleCandidate,
    divergent_universe_decisions::{
        CurioDomainExpiryDefinition, CurioDomainExpiryPolicy, CurioDomainGrantDefinition,
        CurioDomainGrantPolicy, DecisionDataError, key,
    },
    divergent_universe_decisions_generated::{
        SoraConfig, du_curio_domain_grant_policy::DuCurioDomainGrantPolicy,
    },
};
use std::collections::BTreeSet;

pub(super) fn compile(
    config: &SoraConfig,
    reference: &DivergentUniverseBundleCandidate,
    expiries: &[CurioDomainExpiryDefinition],
) -> Result<Box<[CurioDomainGrantDefinition]>, DecisionDataError> {
    let mut seen = BTreeSet::new();
    let mut result = Vec::new();
    for row in config.du_curio_domain_grants().ordered_rows() {
        let linked = config
            .du_curio_domain_expiries()
            .get(&row.expiry_id)
            .ok_or(DecisionDataError::InvalidReference)?;
        let expiry = expiries
            .iter()
            .find(|expiry| expiry.key.as_ref() == linked.stable_key)
            .ok_or(DecisionDataError::InvalidReference)?;
        if expiry.policy != CurioDomainExpiryPolicy::VersionedProjectPolicyActiveFutureSelectedDomainsGrantThenDiscard {
            return Err(DecisionDataError::InvalidPolicy);
        }
        let state = reference
            .curio_catalog()
            .states()
            .iter()
            .find(|state| state.id == expiry.state)
            .ok_or(DecisionDataError::InvalidReference)?;
        let amount = u32::try_from(row.amount)
            .ok()
            .filter(|amount| *amount > 0)
            .ok_or(DecisionDataError::InvalidReward)?;
        let parameter = row
            .amount_parameter
            .checked_sub(1)
            .and_then(|index| usize::try_from(index).ok())
            .and_then(|index| state.effect_parameters.get(index));
        if parameter.map(AsRef::as_ref) != Some(amount.to_string().as_str()) {
            return Err(DecisionDataError::InvalidReference);
        }
        if row.id <= 0 || !seen.insert(expiry.state.clone()) {
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
            DuCurioDomainGrantPolicy::ActivePositiveAllowanceFragmentsBeforeDiscard =>
                CurioDomainGrantPolicy::VersionedProjectPolicyActivePositiveAllowanceFragmentsBeforeDiscard,
        };
        result.push(CurioDomainGrantDefinition {
            key: key(&row.stable_key)?,
            state: expiry.state.clone(),
            amount,
            policy,
            summary_en: row.summary_en.clone().into(),
            summary_zh_cn: row.summary_zh_cn.clone().into(),
            policy_note: row.policy_note.clone().into(),
            replacement_condition: row.replacement_condition.clone().into(),
            sources: source_keys(config, &row.source_ids)?,
        });
    }
    if expiries.iter().any(|expiry|
        expiry.policy == CurioDomainExpiryPolicy::VersionedProjectPolicyActiveFutureSelectedDomainsGrantThenDiscard
        && !seen.contains(&expiry.state)) {
        return Err(DecisionDataError::InvalidReference);
    }
    result.sort_by(|left, right| left.state.cmp(&right.state));
    Ok(result.into_boxed_slice())
}
