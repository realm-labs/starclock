//! Source-joined full-HP victory grants, with explicit scheduling policy.

use super::source_keys;
use crate::{
    divergent_universe::DivergentUniverseBundleCandidate,
    divergent_universe_decisions::{
        CurioBattleGrantDefinition, CurioBattleGrantPolicy, CurioEvolutionDefinition,
        DecisionDataError, key,
    },
    divergent_universe_decisions_generated::{
        SoraConfig, du_curio_battle_grant_policy::DuCurioBattleGrantPolicy,
    },
};
use std::collections::BTreeSet;

pub(super) fn compile(
    config: &SoraConfig,
    reference: &DivergentUniverseBundleCandidate,
    evolutions: &[CurioEvolutionDefinition],
) -> Result<Box<[CurioBattleGrantDefinition]>, DecisionDataError> {
    let mut result = Vec::new();
    let mut states = BTreeSet::new();
    for row in config.du_curio_battle_grants().ordered_rows() {
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
        if !(1..=1_000_000).contains(&row.amount_per_full_hp) {
            return Err(DecisionDataError::InvalidReward);
        }
        let parameter = row
            .amount_parameter
            .checked_sub(1)
            .and_then(|index| usize::try_from(index).ok())
            .and_then(|index| state.effect_parameters.get(index));
        if row.id <= 0
            || state.effect_ids.len() != 1
            || state.effect_ids[0].as_ref() != row.effect_id
            || parameter.map(AsRef::as_ref) != Some(row.amount_per_full_hp.to_string().as_str())
        {
            return Err(DecisionDataError::InvalidReference);
        }
        if row.summary_en.is_empty()
            || row.summary_zh_cn.is_empty()
            || row.policy_note.is_empty()
            || row.replacement_condition.is_empty()
        {
            return Err(DecisionDataError::InvalidPolicy);
        }
        result.push(CurioBattleGrantDefinition {
            key: key(&row.stable_key)?,
            state: state.id.clone(),
            effect_id: row.effect_id.clone().into(),
            amount_per_full_hp: u32::try_from(row.amount_per_full_hp)
                .map_err(|_| DecisionDataError::InvalidReward)?,
            policy: match row.policy {
                DuCurioBattleGrantPolicy::FullHpPresentRosterAfterCarry => {
                    CurioBattleGrantPolicy::VersionedProjectPolicyFullHpPresentRosterAfterCarry
                }
            },
            summary_en: row.summary_en.clone().into(),
            summary_zh_cn: row.summary_zh_cn.clone().into(),
            policy_note: row.policy_note.clone().into(),
            replacement_condition: row.replacement_condition.clone().into(),
            sources: source_keys(config, &row.source_ids)?,
        });
    }
    if result.is_empty() {
        return Err(DecisionDataError::InvalidReference);
    }
    result.sort_by(|left, right| left.key.cmp(&right.key));
    Ok(result.into_boxed_slice())
}
