//! Source-joined one-step evolution edges with separate runtime ownership aliases.

use std::collections::{BTreeMap, BTreeSet};

use super::source_keys;
use crate::divergent_universe::DivergentUniverseBundleCandidate;
use crate::divergent_universe_curio_catalog::{
    DivergentUniverseCurioCategory, DivergentUniverseCurioId,
};
use crate::divergent_universe_decisions::{
    CurioEvolutionDefinition, CurioEvolutionPolicy, DecisionDataError, key,
};
use crate::divergent_universe_decisions_generated::{
    SoraConfig, du_curio_evolution_policy::DuCurioEvolutionPolicy,
};
use crate::divergent_universe_service_catalog::DivergentUniverseOccurrenceId;

pub(super) fn compile(
    config: &SoraConfig,
    reference: &DivergentUniverseBundleCandidate,
) -> Result<Box<[CurioEvolutionDefinition]>, DecisionDataError> {
    let catalog = reference.curio_catalog();
    let mut result = Vec::new();
    let mut predecessors = BTreeSet::new();
    let mut targets = BTreeSet::new();
    for row in config.du_curio_evolutions().ordered_rows() {
        let from = catalog
            .states()
            .iter()
            .find(|state| state.id.as_str() == row.from_state)
            .ok_or(DecisionDataError::InvalidReference)?;
        let to = catalog
            .states()
            .iter()
            .find(|state| state.id.as_str() == row.to_state)
            .ok_or(DecisionDataError::InvalidReference)?;
        let owner = DivergentUniverseCurioId::new(row.owner.as_str())
            .map_err(|_| DecisionDataError::InvalidReference)?;
        let occurrence = DivergentUniverseOccurrenceId::new(row.occurrence.as_str())
            .map_err(|_| DecisionDataError::InvalidReference)?;
        if row.id <= 0 || !predecessors.insert(from.id.clone()) || !targets.insert(to.id.clone()) {
            return Err(DecisionDataError::InvalidIdentity);
        }
        // Strict adjacent rarity prevents cycles and skipped tiers independently
        // of transport order or upstream numeric identifiers.
        if !matches!(
            (from.category, to.category),
            (
                DivergentUniverseCurioCategory::Common,
                DivergentUniverseCurioCategory::Rare
            ) | (
                DivergentUniverseCurioCategory::Rare,
                DivergentUniverseCurioCategory::Legendary
            )
        ) || to.curio.is_some()
            || !config
                .du_curio_acquisitions()
                .ordered_rows()
                .any(|grant| grant.state_key == row.to_state)
            || !catalog.curios().iter().any(|curio| curio.id == owner)
            || !reference
                .service_catalog()
                .occurrences()
                .iter()
                .any(|event| event.id == occurrence)
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
        result.push(CurioEvolutionDefinition {
            key: key(&row.stable_key)?,
            from: from.id.clone(),
            to: to.id.clone(),
            owner,
            occurrence,
            policy: match row.policy {
                DuCurioEvolutionPolicy::ActiveSameOwnerResetAndAcquire => {
                    CurioEvolutionPolicy::VersionedProjectPolicyActiveSameOwnerResetAndAcquire
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
    let aliases = result
        .iter()
        .map(|edge| (&edge.to, &edge.owner))
        .collect::<BTreeMap<_, _>>();
    for edge in &result {
        let owner = catalog
            .states()
            .iter()
            .find(|state| state.id == edge.from)
            .and_then(|state| state.curio.as_ref())
            .or_else(|| aliases.get(&edge.from).copied());
        if owner != Some(&edge.owner) {
            return Err(DecisionDataError::InvalidReference);
        }
    }
    result.sort_by(|left, right| left.key.cmp(&right.key));
    Ok(result.into_boxed_slice())
}
