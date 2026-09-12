//! Validated state/effect/parameter joins for reviewed immediate Curio grants.

use std::collections::BTreeSet;

use super::source_keys;
use crate::{
    divergent_universe::DivergentUniverseBundleCandidate,
    divergent_universe_curio_catalog::DivergentUniverseCurioStateId,
    divergent_universe_decisions::{
        CurioAcquisitionDefinition, CurioAcquisitionGrant, CurioAcquisitionPolicy,
        DecisionDataError, RewardRarityRange, key,
    },
    divergent_universe_decisions_generated::{
        SoraConfig, du_curio_acquisition_kind::DuCurioAcquisitionKind,
        du_curio_acquisition_policy::DuCurioAcquisitionPolicy,
    },
    divergent_universe_equation_catalog::DivergentUniversePathType,
};

pub(super) fn compile(
    config: &SoraConfig,
    reference: &DivergentUniverseBundleCandidate,
) -> Result<Box<[CurioAcquisitionDefinition]>, DecisionDataError> {
    let mut seen = BTreeSet::new();
    let mut result = Vec::new();
    for row in config.du_curio_acquisitions().ordered_rows() {
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
            || definition.effect_ids.len() != 1
            || definition.effect_ids[0].as_ref() != row.effect_id
            || parameter.map(AsRef::as_ref) != Some(row.amount.as_str())
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
        if row.kind != DuCurioAcquisitionKind::RarityBlessings
            && (row.minimum_rarity.is_some() || row.maximum_rarity.is_some())
        {
            return Err(DecisionDataError::InvalidReward);
        }
        let grant = match row.kind {
            DuCurioAcquisitionKind::RarityBlessings => {
                let count = row
                    .amount
                    .parse::<u16>()
                    .map_err(|_| DecisionDataError::InvalidReward)?;
                let minimum = row.minimum_rarity.ok_or(DecisionDataError::InvalidReward)?;
                let maximum = row.maximum_rarity.ok_or(DecisionDataError::InvalidReward)?;
                if count == 0
                    || count > 64
                    || count.to_string() != row.amount
                    || !(1..=3).contains(&minimum)
                    || !(minimum..=3).contains(&maximum)
                    || row.path_types.is_some()
                {
                    return Err(DecisionDataError::InvalidReward);
                }
                CurioAcquisitionGrant::RarityBlessings {
                    count,
                    rarity: RewardRarityRange {
                        minimum: minimum as u8,
                        maximum: maximum as u8,
                    },
                }
            }
            DuCurioAcquisitionKind::FixedFragments => {
                if row.path_types.is_some() {
                    return Err(DecisionDataError::InvalidReward);
                }
                let amount = row
                    .amount
                    .parse::<u32>()
                    .map_err(|_| DecisionDataError::InvalidReward)?;
                if amount == 0 || amount > i32::MAX as u32 || amount.to_string() != row.amount {
                    return Err(DecisionDataError::InvalidReward);
                }
                CurioAcquisitionGrant::FixedFragments(u64::from(amount))
            }
            DuCurioAcquisitionKind::BalanceFraction => {
                if row.path_types.is_some() {
                    return Err(DecisionDataError::InvalidReward);
                }
                let digits = row
                    .amount
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
                let denominator = 10_u32.pow(
                    u32::try_from(digits.len()).map_err(|_| DecisionDataError::InvalidReward)?,
                );
                if numerator == 0 || numerator >= denominator {
                    return Err(DecisionDataError::InvalidReward);
                }
                CurioAcquisitionGrant::BalanceFraction {
                    numerator,
                    denominator,
                }
            }
            DuCurioAcquisitionKind::PathBlessings => {
                let count = row
                    .amount
                    .parse::<u16>()
                    .map_err(|_| DecisionDataError::InvalidReward)?;
                if count == 0 || count > 64 || count.to_string() != row.amount {
                    return Err(DecisionDataError::InvalidReward);
                }
                let raw_paths = row
                    .path_types
                    .as_ref()
                    .ok_or(DecisionDataError::InvalidReference)?;
                if raw_paths.is_empty()
                    || raw_paths.len() > reference.blessing_catalog().paths().len()
                {
                    return Err(DecisionDataError::InvalidReference);
                }
                let mut paths = raw_paths
                    .iter()
                    .map(|path| {
                        let path = DivergentUniversePathType::new(path.as_str())
                            .map_err(|_| DecisionDataError::InvalidReference)?;
                        if !reference
                            .blessing_catalog()
                            .paths()
                            .iter()
                            .any(|candidate| candidate.path == path)
                        {
                            return Err(DecisionDataError::InvalidReference);
                        }
                        Ok(path)
                    })
                    .collect::<Result<Vec<_>, DecisionDataError>>()?;
                paths.sort();
                if paths.windows(2).any(|pair| pair[0] == pair[1]) {
                    return Err(DecisionDataError::InvalidReference);
                }
                CurioAcquisitionGrant::PathBlessings {
                    count,
                    paths: paths.into_boxed_slice(),
                }
            }
        };
        let policy = match (row.kind, row.policy) {
            (DuCurioAcquisitionKind::FixedFragments | DuCurioAcquisitionKind::BalanceFraction, DuCurioAcquisitionPolicy::StableStateOrderFloorBeforeEachGrant) => {
                CurioAcquisitionPolicy::VersionedProjectPolicyStableStateOrderFloorBeforeEachGrant
            }
            (DuCurioAcquisitionKind::PathBlessings, DuCurioAcquisitionPolicy::StableStateOrderUniformUnownedPathRejectExhaustion) => CurioAcquisitionPolicy::VersionedProjectPolicyStableStateOrderUniformUnownedPathRejectExhaustion,
            (DuCurioAcquisitionKind::RarityBlessings, DuCurioAcquisitionPolicy::FeasibleRarityAssignments) => CurioAcquisitionPolicy::FeasibleRarityAssignments,
            _ => return Err(DecisionDataError::InvalidPolicy),
        };
        result.push(CurioAcquisitionDefinition {
            key: key(&row.stable_key)?,
            state,
            effect_id: row.effect_id.clone().into(),
            grant,
            policy,
            summary_en: row.summary_en.clone().into(),
            summary_zh_cn: row.summary_zh_cn.clone().into(),
            policy_note: row.policy_note.clone().into(),
            replacement_condition: row.replacement_condition.clone().into(),
            sources: source_keys(config, &row.source_ids)?,
        });
    }
    result.sort_by(|left, right| left.state.as_str().cmp(right.state.as_str()));
    Ok(result.into_boxed_slice())
}
