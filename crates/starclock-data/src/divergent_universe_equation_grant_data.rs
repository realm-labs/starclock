//! Reviewed Equation-acquisition trigger counts and replaceable selection policy.

use super::source_keys;
use crate::{
    divergent_universe::DivergentUniverseBundleCandidate,
    divergent_universe_decisions::{
        DecisionDataError, EquationGrantDefinition, EquationGrantPolicy, key,
    },
    divergent_universe_decisions_generated::{
        SoraConfig, du_equation_grant_policy::DuEquationGrantPolicy,
    },
};

pub(super) fn compile(
    config: &SoraConfig,
    reference: &DivergentUniverseBundleCandidate,
) -> Result<Box<[EquationGrantDefinition]>, DecisionDataError> {
    let rows = config
        .du_equation_grants()
        .ordered_rows()
        .collect::<Vec<_>>();
    let [row] = rows.as_slice() else {
        return Err(DecisionDataError::InvalidPolicy);
    };
    let state = reference
        .curio_catalog()
        .states()
        .iter()
        .find(|state| state.id.as_str() == row.state_key && state.curio.is_some())
        .ok_or(DecisionDataError::InvalidReference)?;
    let parameter = |index: i32| {
        index
            .checked_sub(1)
            .and_then(|value| usize::try_from(value).ok())
            .and_then(|index| state.effect_parameters.get(index))
            .map(AsRef::as_ref)
    };
    if row.id <= 0
        || state.effect_ids.len() != 1
        || state.effect_ids[0].as_ref() != row.effect_id
        || parameter(row.count_parameter) != Some(row.count.to_string().as_str())
        || parameter(row.limit_parameter) != Some(row.domain_limit.to_string().as_str())
    {
        return Err(DecisionDataError::InvalidReference);
    }
    if !(1..=8).contains(&row.count) || row.domain_limit != 1 {
        return Err(DecisionDataError::InvalidReward);
    }
    if row.summary_en.is_empty()
        || row.summary_zh_cn.is_empty()
        || row.policy_note.is_empty()
        || row.replacement_condition.is_empty()
    {
        return Err(DecisionDataError::InvalidPolicy);
    }
    Ok(vec![EquationGrantDefinition {
        key: key(&row.stable_key)?, state: state.id.clone(), effect_id: row.effect_id.clone().into(),
        count: u16::try_from(row.count).map_err(|_| DecisionDataError::InvalidReward)?,
        policy: match row.policy { DuEquationGrantPolicy::UniformMissingRecipeAvailableSubsetOnceLogicalDomain => EquationGrantPolicy::VersionedProjectPolicyUniformMissingRecipeAvailableSubsetOnceLogicalDomain },
        summary_en: row.summary_en.clone().into(), summary_zh_cn: row.summary_zh_cn.clone().into(),
        policy_note: row.policy_note.clone().into(), replacement_condition: row.replacement_condition.clone().into(),
        sources: source_keys(config, &row.source_ids)?,
    }].into_boxed_slice())
}
