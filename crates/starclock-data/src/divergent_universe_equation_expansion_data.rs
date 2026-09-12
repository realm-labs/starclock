//! Released expansion operands, separately validated from acquisition rewards.

use super::source_keys;
use crate::{
    divergent_universe::DivergentUniverseBundleCandidate,
    divergent_universe_decisions::{
        DecisionDataError, EquationExpansionRewardDefinition, EquationExpansionRewardPolicy, key,
    },
    divergent_universe_decisions_generated::{
        SoraConfig, du_expansion_reward_policy::DuExpansionRewardPolicy,
    },
};

pub(super) fn compile(
    config: &SoraConfig,
    reference: &DivergentUniverseBundleCandidate,
) -> Result<Box<[EquationExpansionRewardDefinition]>, DecisionDataError> {
    let rows = config
        .du_equation_expansion_rewards()
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
    if row.id <= 0 || state.effect_ids.len() != 1
        || state.effect_ids[0].as_ref() != row.effect_id
        || parameter(row.count_parameter) != Some(row.count.to_string().as_str())
        || parameter(row.limit_parameter) != Some(row.trigger_limit.to_string().as_str())
        // The reference classifier is only PassiveWhileOwned. The reviewed
        // released text, not that broad classifier, admits this expansion rule.
        || row.state_key != "divergent-universe.curio-state.9074"
        || row.effect_id != "2074"
        || state.charges.as_ref() != row.trigger_limit.to_string()
        || i32::from(state.counter_parameter_index) != row.limit_parameter
    {
        return Err(DecisionDataError::InvalidReference);
    }
    if row.count != 1 || row.trigger_limit != 3 {
        return Err(DecisionDataError::InvalidReward);
    }
    if row.minimum_rarity != 1
        || row.maximum_rarity != 3
        || row.summary_en.is_empty()
        || row.summary_zh_cn.is_empty()
        || row.policy_note.is_empty()
        || row.replacement_condition.is_empty()
    {
        return Err(DecisionDataError::InvalidPolicy);
    }
    let sources = source_keys(config, &row.source_ids)?;
    if sources.iter().map(AsRef::as_ref).collect::<Vec<&str>>()
        != [
            "du.source.equation-expansion.state",
            "du.source.equation-expansion.parameters",
            "du.source.equation-expansion.text",
        ]
    {
        return Err(DecisionDataError::InvalidReference);
    }
    let definition = EquationExpansionRewardDefinition {
        key: key(&row.stable_key)?,
        state: state.id.clone(),
        effect_id: row.effect_id.clone().into(),
        count: u16::try_from(row.count).map_err(|_| DecisionDataError::InvalidReward)?,
        trigger_limit: u16::try_from(row.trigger_limit).map_err(|_| DecisionDataError::InvalidReward)?,
        policy: match row.policy {
            DuExpansionRewardPolicy::ActivePreStateUniformUnownedBoundedCascade =>
                EquationExpansionRewardPolicy::VersionedProjectPolicyActivePreStateUniformUnownedBoundedCascade,
        },
        summary_en: row.summary_en.clone().into(),
        summary_zh_cn: row.summary_zh_cn.clone().into(),
        policy_note: row.policy_note.clone().into(),
        replacement_condition: row.replacement_condition.clone().into(),
        sources,
    };
    Ok(vec![definition].into_boxed_slice())
}
