//! Exact state/effect/Path bindings; numeric preference is explicit policy.

use std::collections::BTreeSet;

use super::{acquisitions, source_keys};
use crate::{
    divergent_universe::DivergentUniverseBundleCandidate,
    divergent_universe_decisions::{
        CurioAcquisitionGrant, CurioBattleWeightDefinition, CurioBattleWeightPolicy,
        DecisionDataError, key,
    },
    divergent_universe_decisions_generated::{
        SoraConfig, du_curio_battle_weight_policy::DuCurioBattleWeightPolicy,
    },
};

pub(super) fn compile(
    config: &SoraConfig,
    reference: &DivergentUniverseBundleCandidate,
) -> Result<Box<[CurioBattleWeightDefinition]>, DecisionDataError> {
    let acquisitions = acquisitions::compile(config, reference)?;
    let mut result = Vec::new();
    let mut states = BTreeSet::new();
    for row in config.du_curio_battle_weights().ordered_rows() {
        if row.id <= 0 || !(1..=1_000_000).contains(&row.bonus_weight) {
            return Err(DecisionDataError::InvalidReward);
        }
        if row.summary_en.is_empty()
            || row.summary_zh_cn.is_empty()
            || row.policy_note.is_empty()
            || row.replacement_condition.is_empty()
        {
            return Err(DecisionDataError::InvalidPolicy);
        }
        let acquisition = acquisitions
            .iter()
            .find(|value| {
                value.state.as_str() == row.state_key && value.effect_id.as_ref() == row.effect_id
            })
            .ok_or(DecisionDataError::InvalidReference)?;
        let CurioAcquisitionGrant::PathBlessings { paths, .. } = &acquisition.grant else {
            return Err(DecisionDataError::InvalidReference);
        };
        let [path] = paths.as_ref() else {
            return Err(DecisionDataError::InvalidReference);
        };
        if path.as_str() != row.path_type
            || !reference
                .curio_catalog()
                .states()
                .iter()
                .any(|state| state.id == acquisition.state && state.curio.is_some())
        {
            return Err(DecisionDataError::InvalidReference);
        }
        if !states.insert(acquisition.state.clone()) {
            return Err(DecisionDataError::InvalidIdentity);
        }
        result.push(CurioBattleWeightDefinition {
            key: key(&row.stable_key)?, state: acquisition.state.clone(), path: path.clone(),
            effect_id: row.effect_id.clone().into(),
            summary_en: row.summary_en.clone().into(), summary_zh_cn: row.summary_zh_cn.clone().into(),
            bonus_weight: u32::try_from(row.bonus_weight).map_err(|_| DecisionDataError::InvalidReward)?,
            policy: match row.policy {
                DuCurioBattleWeightPolicy::ActiveStateAdditiveCandidateWeight => CurioBattleWeightPolicy::VersionedProjectPolicyActiveStateAdditiveCandidateWeight,
            },
            policy_note: row.policy_note.clone().into(), replacement_condition: row.replacement_condition.clone().into(),
            sources: source_keys(config, &row.source_ids)?,
        });
    }
    if result.is_empty() || result.len() > 64 {
        return Err(DecisionDataError::InvalidPolicy);
    }
    result.sort_by(|a, b| a.state.cmp(&b.state));
    Ok(result.into_boxed_slice())
}
