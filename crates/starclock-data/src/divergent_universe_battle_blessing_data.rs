//! Bounded normal-battle offer policy and exact suppression state/effect joins.

use std::collections::BTreeSet;

use super::source_keys;
use crate::{
    divergent_universe::DivergentUniverseBundleCandidate,
    divergent_universe_curio_catalog::DivergentUniverseCurioStateId,
    divergent_universe_decisions::{
        BattleBlessingPolicy, BattleBlessingPolicyKind, DecisionDataError, RewardRarityRange, key,
    },
    divergent_universe_decisions_generated::{
        SoraConfig, du_battle_blessing_policy::DuBattleBlessingPolicy,
    },
};

pub(super) fn compile(
    config: &SoraConfig,
    reference: &DivergentUniverseBundleCandidate,
) -> Result<Box<[BattleBlessingPolicy]>, DecisionDataError> {
    let mut result = Vec::new();
    for row in config.du_battle_blessings().ordered_rows() {
        if row.id <= 0
            || !(1..=8).contains(&row.offer_width)
            || !(1..=3).contains(&row.minimum_rarity)
            || !(row.minimum_rarity..=3).contains(&row.maximum_rarity)
        {
            return Err(DecisionDataError::InvalidReward);
        }
        if row.summary_en.is_empty()
            || row.summary_zh_cn.is_empty()
            || row.policy_note.is_empty()
            || row.replacement_condition.is_empty()
        {
            return Err(DecisionDataError::InvalidPolicy);
        }
        if row.suppression_states.len() != row.suppression_effects.len()
            || row.suppression_states.len() > 64
        {
            return Err(DecisionDataError::InvalidReference);
        }
        let mut states = BTreeSet::new();
        for (state, effect) in row.suppression_states.iter().zip(&row.suppression_effects) {
            let state = DivergentUniverseCurioStateId::new(state.as_str())
                .map_err(|_| DecisionDataError::InvalidReference)?;
            let definition = reference
                .curio_catalog()
                .states()
                .iter()
                .find(|value| value.id == state)
                .ok_or(DecisionDataError::InvalidReference)?;
            if definition.curio.is_none()
                || definition.effect_ids.len() != 1
                || definition.effect_ids[0].as_ref() != effect
            {
                return Err(DecisionDataError::InvalidReference);
            }
            if !states.insert(state) {
                return Err(DecisionDataError::InvalidIdentity);
            }
        }
        result.push(BattleBlessingPolicy {
            key: key(&row.stable_key)?,
            kind: match row.policy {
                DuBattleBlessingPolicy::UniformUnownedSingleSelectionAvailableSubset => BattleBlessingPolicyKind::VersionedProjectPolicyUniformUnownedSingleSelectionAvailableSubset,
            },
            offer_width: u16::try_from(row.offer_width).map_err(|_| DecisionDataError::InvalidReward)?,
            rarity: RewardRarityRange {
                minimum: u8::try_from(row.minimum_rarity).map_err(|_| DecisionDataError::InvalidReward)?,
                maximum: u8::try_from(row.maximum_rarity).map_err(|_| DecisionDataError::InvalidReward)?,
            },
            suppression_states: states.into_iter().collect(),
            summary_en: row.summary_en.clone().into(), summary_zh_cn: row.summary_zh_cn.clone().into(),
            policy_note: row.policy_note.clone().into(), replacement_condition: row.replacement_condition.clone().into(),
            sources: source_keys(config, &row.source_ids)?,
        });
    }
    if result.is_empty() {
        return Err(DecisionDataError::InvalidPolicy);
    }
    result.sort_by(|left, right| left.key.cmp(&right.key));
    Ok(result.into_boxed_slice())
}
