//! Explicit candidate pool joins without promoting weekly display reachability.

use super::source_keys;
use crate::{
    divergent_universe::DivergentUniverseBundleCandidate,
    divergent_universe_decisions::{
        DecisionDataError, EncounterPoolPolicy, EncounterPoolPolicyKind, key,
    },
    divergent_universe_decisions_generated::{
        SoraConfig, du_encounter_pool_policy::DuEncounterPoolPolicy,
    },
    divergent_universe_encounter_catalog::DivergentUniverseEncounterGroupId,
};

pub(super) fn compile(
    config: &SoraConfig,
    reference: &DivergentUniverseBundleCandidate,
) -> Result<EncounterPoolPolicy, DecisionDataError> {
    let rows = config
        .du_encounter_pools()
        .ordered_rows()
        .collect::<Vec<_>>();
    let [row] = rows.as_slice() else {
        return Err(DecisionDataError::InvalidPolicy);
    };
    if row.id <= 0
        || row.summary_en.is_empty()
        || row.summary_zh_cn.is_empty()
        || row.policy_note.is_empty()
        || row.replacement_condition.is_empty()
        || row.candidate_stages.is_empty()
        || row.candidate_stages.len() > 256
        || row
            .candidate_stages
            .windows(2)
            .any(|pair| pair[0] >= pair[1])
    {
        return Err(DecisionDataError::InvalidPolicy);
    }
    let group = DivergentUniverseEncounterGroupId::new(row.encounter_group.as_str())
        .map_err(|_| DecisionDataError::InvalidReference)?;
    let source = reference
        .encounter_catalog()
        .groups()
        .iter()
        .find(|value| value.id == group)
        .ok_or(DecisionDataError::InvalidReference)?;
    if !row.candidate_stages.contains(&row.first_stage)
        || row.candidate_stages.iter().any(|stage| {
            !source
                .candidate_stage_ids
                .iter()
                .any(|candidate| candidate.as_ref() == stage)
        })
    {
        return Err(DecisionDataError::InvalidReference);
    }
    Ok(EncounterPoolPolicy {
        key: key(&row.stable_key)?,
        kind: match row.policy {
            DuEncounterPoolPolicy::FixedFirstUniformLaterLayersWithReplacement =>
                EncounterPoolPolicyKind::VersionedProjectPolicyFixedFirstUniformLaterLayersWithReplacement,
        },
        encounter_group: group,
        first_stage: row.first_stage.clone().into(),
        candidate_stages: row.candidate_stages.iter().cloned().map(Into::into).collect(),
        summary_en: row.summary_en.clone().into(),
        summary_zh_cn: row.summary_zh_cn.clone().into(),
        policy_note: row.policy_note.clone().into(),
        replacement_condition: row.replacement_condition.clone().into(),
        sources: source_keys(config, &row.source_ids)?,
    })
}
