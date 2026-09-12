//! Exact passive value/limit joins; battle assembly remains mode-owned.

use super::source_keys;
use crate::{
    divergent_universe::DivergentUniverseBundleCandidate,
    divergent_universe_decisions::{
        CurioBattleStat, CurioBattleStatDefinition, CurioBattleStatPolicy, DecisionDataError, key,
    },
    divergent_universe_decisions_generated::{
        SoraConfig, du_curio_battle_stat::DuCurioBattleStat,
        du_curio_battle_stat_policy::DuCurioBattleStatPolicy,
    },
};
use std::collections::BTreeSet;

pub(super) fn compile(
    config: &SoraConfig,
    reference: &DivergentUniverseBundleCandidate,
) -> Result<Box<[CurioBattleStatDefinition]>, DecisionDataError> {
    let mut seen = BTreeSet::new();
    let mut result = Vec::new();
    for row in config.du_curio_battle_stats().ordered_rows() {
        let state = reference
            .curio_catalog()
            .states()
            .iter()
            .find(|state| state.id.as_str() == row.state_key && state.curio.is_some())
            .ok_or(DecisionDataError::InvalidReference)?;
        let parameter = |index: i32| {
            index
                .checked_sub(1)
                .and_then(|index| usize::try_from(index).ok())
                .and_then(|index| state.effect_parameters.get(index))
                .map(AsRef::as_ref)
        };
        let limit = u16::try_from(row.battle_limit)
            .ok()
            .filter(|limit| *limit > 0)
            .ok_or(DecisionDataError::InvalidReward)?;
        if row.id <= 0
            || state.effect_ids.len() != 1
            || state.effect_ids[0].as_ref() != row.effect_id
            || parameter(row.bonus_parameter) != Some(row.bonus_fraction.as_str())
            || parameter(row.limit_parameter) != Some(limit.to_string().as_str())
        {
            return Err(DecisionDataError::InvalidReference);
        }
        // Six-place positive proper fractions lower losslessly into the shared
        // fixed-point scalar. This is a bounded stat-bonus authoring surface.
        let digits = row
            .bonus_fraction
            .strip_prefix("0.")
            .ok_or(DecisionDataError::InvalidReward)?;
        if digits.is_empty()
            || digits.len() > 6
            || digits.ends_with('0')
            || !digits.bytes().all(|byte| byte.is_ascii_digit())
            || digits.parse::<u32>().map_or(true, |value| value == 0)
        {
            return Err(DecisionDataError::InvalidReward);
        }
        if !seen.insert(state.id.clone()) {
            return Err(DecisionDataError::InvalidIdentity);
        }
        if row.summary_en.is_empty()
            || row.summary_zh_cn.is_empty()
            || row.policy_note.is_empty()
            || row.replacement_condition.is_empty()
        {
            return Err(DecisionDataError::InvalidPolicy);
        }
        result.push(CurioBattleStatDefinition {
            key: key(&row.stable_key)?,
            state: state.id.clone(),
            effect_id: row.effect_id.clone().into(),
            stat: match row.stat {
                DuCurioBattleStat::Speed => CurioBattleStat::Speed,
                DuCurioBattleStat::FinalDamage => CurioBattleStat::FinalDamage,
            },
            bonus_fraction: row.bonus_fraction.clone().into(),
            battle_limit: limit,
            policy: match row.policy {
                DuCurioBattleStatPolicy::BasePercentVerifiedBattleLifetime => {
                    if row.stat != DuCurioBattleStat::Speed { return Err(DecisionDataError::InvalidPolicy); }
                    CurioBattleStatPolicy::VersionedProjectPolicyBasePercentVerifiedBattleLifetime
                }
                DuCurioBattleStatPolicy::OutgoingFinalMultiplierVerifiedBattleLifetime => {
                    if row.stat != DuCurioBattleStat::FinalDamage { return Err(DecisionDataError::InvalidPolicy); }
                    CurioBattleStatPolicy::VersionedProjectPolicyOutgoingFinalMultiplierVerifiedBattleLifetime
                }
            },
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
