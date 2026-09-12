//! Exact current event/handbook joins with separately authored service policy.
use super::source_keys;
use crate::{
    divergent_universe::DivergentUniverseBundleCandidate,
    divergent_universe_decisions::{
        DecisionDataError, TawotServiceDefinition, TawotServicePolicy, key,
    },
    divergent_universe_decisions_generated::{
        SoraConfig, du_tawot_service_policy::DuTawotServicePolicy,
    },
};
use std::collections::BTreeSet;

pub(super) fn compile(
    config: &SoraConfig,
    reference: &DivergentUniverseBundleCandidate,
) -> Result<Box<[TawotServiceDefinition]>, DecisionDataError> {
    let mut seen = BTreeSet::new();
    let mut result = Vec::new();
    for row in config.du_tawot_services().ordered_rows() {
        let variant = reference
            .service_catalog()
            .variants()
            .iter()
            .find(|variant| variant.id.as_str() == row.variant_key)
            .ok_or(DecisionDataError::InvalidReference)?;
        let occurrence = reference
            .service_catalog()
            .occurrences()
            .iter()
            .find(|event| {
                event.id.as_str() == row.occurrence_key && event.variants.contains(&variant.id)
            })
            .ok_or(DecisionDataError::InvalidReference)?;
        let curio = reference
            .curio_catalog()
            .curios()
            .iter()
            .find(|curio| curio.id.as_str() == row.curio_key)
            .ok_or(DecisionDataError::InvalidReference)?;
        let mut states = reference
            .curio_catalog()
            .states()
            .iter()
            .filter(|state| state.curio.as_ref() == Some(&curio.id))
            .map(|state| state.id.clone())
            .collect::<Vec<_>>();
        states.sort();
        if row.id <= 0 || !seen.insert((variant.id.clone(), row.forge_level)) {
            return Err(DecisionDataError::InvalidIdentity);
        }
        if !(2..=5).contains(&row.forge_level)
            || row.fragment_cost <= 0
            || !(1..=8).contains(&row.offer_width)
            || !(1..=8).contains(&row.purchase_limit)
            || usize::try_from(row.offer_width).map_or(true, |count| count > states.len())
            || row.summary_en.is_empty()
            || row.summary_zh_cn.is_empty()
            || row.policy_note.is_empty()
            || row.replacement_condition.is_empty()
        {
            return Err(DecisionDataError::InvalidPolicy);
        }
        result.push(TawotServiceDefinition {
            key: key(&row.stable_key)?, occurrence: occurrence.id.clone(), variant: variant.id.clone(),
            curio: curio.id.clone(), states: states.into_boxed_slice(),
            forge_level: u16::try_from(row.forge_level).map_err(|_| DecisionDataError::InvalidPolicy)?,
            fragment_cost: u32::try_from(row.fragment_cost).map_err(|_| DecisionDataError::InvalidPolicy)?,
            offer_width: u16::try_from(row.offer_width).map_err(|_| DecisionDataError::InvalidPolicy)?,
            purchase_limit: u16::try_from(row.purchase_limit).map_err(|_| DecisionDataError::InvalidPolicy)?,
            policy: match row.policy { DuTawotServicePolicy::UniformDistinctCurrentStatesPaidSelection => TawotServicePolicy::VersionedProjectPolicyUniformDistinctCurrentStatesPaidSelection },
            summary_en: row.summary_en.clone().into(), summary_zh_cn: row.summary_zh_cn.clone().into(),
            policy_note: row.policy_note.clone().into(), replacement_condition: row.replacement_condition.clone().into(),
            sources: source_keys(config, &row.source_ids)?,
        });
    }
    result.sort_by(|left, right| left.key.cmp(&right.key));
    Ok(result.into_boxed_slice())
}
