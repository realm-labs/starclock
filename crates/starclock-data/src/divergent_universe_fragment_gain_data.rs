//! Exact state/effect/rate joins for the reviewed global fragment-gain component.

use std::collections::BTreeSet;

use super::source_keys;
use crate::{
    divergent_universe::DivergentUniverseBundleCandidate,
    divergent_universe_curio_catalog::DivergentUniverseCurioStateId,
    divergent_universe_decisions::{
        CurioFragmentGainDefinition, CurioFragmentGainPolicy, DecisionDataError, key,
    },
    divergent_universe_decisions_generated::{
        SoraConfig, du_curio_fragment_gain_policy::DuCurioFragmentGainPolicy,
    },
};

pub(super) fn compile(
    config: &SoraConfig,
    reference: &DivergentUniverseBundleCandidate,
) -> Result<Box<[CurioFragmentGainDefinition]>, DecisionDataError> {
    let mut seen = BTreeSet::new();
    let mut result = Vec::new();
    for row in config.du_curio_fragment_gains().ordered_rows() {
        let state = DivergentUniverseCurioStateId::new(row.state_key.as_str())
            .map_err(|_| DecisionDataError::InvalidReference)?;
        let definition = reference
            .curio_catalog()
            .states()
            .iter()
            .find(|candidate| candidate.id == state)
            .ok_or(DecisionDataError::InvalidReference)?;
        let parameter = row
            .parameter_index
            .checked_sub(1)
            .and_then(|index| usize::try_from(index).ok())
            .and_then(|index| definition.effect_parameters.get(index));
        if row.id <= 0
            || definition.curio.is_none()
            || definition.effect_ids.len() != 1
            || definition.effect_ids[0].as_ref() != row.effect_id
            || parameter.map(AsRef::as_ref) != Some(row.bonus_fraction.as_str())
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
        let digits = row
            .bonus_fraction
            .strip_prefix("0.")
            .ok_or(DecisionDataError::InvalidReward)?;
        if digits.is_empty()
            || digits.len() > 6
            || digits.ends_with('0')
            || !digits.bytes().all(|byte| byte.is_ascii_digit())
        {
            return Err(DecisionDataError::InvalidReward);
        }
        let numerator = digits
            .parse::<u32>()
            .map_err(|_| DecisionDataError::InvalidReward)?;
        let denominator =
            10_u32.pow(u32::try_from(digits.len()).map_err(|_| DecisionDataError::InvalidReward)?);
        if numerator == 0 || numerator >= denominator {
            return Err(DecisionDataError::InvalidReward);
        }
        let policy = match row.policy { DuCurioFragmentGainPolicy::ActiveStateAdditiveOriginalBaseFloorEachBonus => CurioFragmentGainPolicy::VersionedProjectPolicyActiveStateAdditiveOriginalBaseFloorEachBonus };
        result.push(CurioFragmentGainDefinition {
            key: key(&row.stable_key)?,
            state,
            effect_id: row.effect_id.clone().into(),
            numerator,
            denominator,
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
