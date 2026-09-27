//! Explicit Reward-card substitution; source count is separate from pool policy.

use std::collections::BTreeSet;

use super::{DecisionDataError, DecisionOccurrence, key, validation::source_keys};
use crate::divergent_universe_decisions_generated::{
    SoraConfig, du_decision_evidence::DuDecisionEvidence, du_domain_card_kind::DuDomainCardKind,
    du_reward_occurrence_policy::DuRewardOccurrencePolicy,
};
use crate::divergent_universe_service_catalog::DivergentUniverseOccurrenceVariantId;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RewardOccurrenceId(Box<str>);

impl RewardOccurrenceId {
    /// Validates the project namespace, not existence or original event membership.
    pub fn new(value: impl Into<Box<str>>) -> Result<Self, DecisionDataError> {
        let value = value.into();
        if !value.starts_with("du.reward-room.") || value.len() == "du.reward-room.".len() {
            return Err(DecisionDataError::InvalidIdentity);
        }
        Ok(Self(key(&value)?))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RewardOccurrencePolicy {
    VersionedProjectPolicySingleExplicitRewardOnlySubstituteNoPoolSampling,
}

/// One free, nonempty authored occurrence at a reviewed level-one Reward card.
/// The selected variant is a policy substitute, not a source pool membership fact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RewardOccurrenceDefinition {
    pub id: RewardOccurrenceId,
    pub preset_source: Box<str>,
    pub level: u16,
    pub variant: DivergentUniverseOccurrenceVariantId,
    pub policy: RewardOccurrencePolicy,
    pub summary_en: Box<str>,
    pub summary_zh_cn: Box<str>,
    pub policy_note: Box<str>,
    pub replacement_condition: Box<str>,
    pub sources: Box<[Box<str>]>,
}

pub(super) fn compile(
    config: &SoraConfig,
    occurrences: &[DecisionOccurrence],
) -> Result<Box<[RewardOccurrenceDefinition]>, DecisionDataError> {
    let mut seen = BTreeSet::new();
    let mut result = Vec::new();
    for row in config.du_reward_occurrences().ordered_rows() {
        if row.id <= 0 || !seen.insert(&row.preset_source) {
            return Err(DecisionDataError::InvalidIdentity);
        }
        if row.level != 1 {
            return Err(DecisionDataError::InvalidPolicy);
        }
        if !config.du_domain_cards().ordered_rows().any(|card| {
            card.preset_source == row.preset_source
                && card.level == row.level
                && card.kind == DuDomainCardKind::Reward
        }) {
            return Err(DecisionDataError::InvalidReference);
        }
        let selected = config
            .du_decision_occurrences()
            .get(&row.occurrence_id)
            .ok_or(DecisionDataError::InvalidReference)?;
        let occurrence = occurrences
            .iter()
            .find(|occurrence| occurrence.key.as_ref() == selected.stable_key)
            .ok_or(DecisionDataError::InvalidReference)?;
        if occurrence.choices.is_empty()
            || occurrence
                .choices
                .iter()
                .any(|choice| choice.fragment_cost != 0 || choice.outcomes.is_empty())
            || [
                &row.summary_en,
                &row.summary_zh_cn,
                &row.policy_note,
                &row.replacement_condition,
            ]
            .iter()
            .any(|text| text.trim().is_empty())
        {
            return Err(DecisionDataError::InvalidPolicy);
        }
        let sources = source_keys(config, &row.source_ids)?;
        for table in [
            "ExcelOutput/RoguePersonaRoomPreset.json;",
            "ExcelOutput/RoguePersonaRoomCompType.json;",
            "TextMap/TextMapEN.json;",
        ] {
            if !row.source_ids.iter().any(|id| {
                config.du_decision_sources().get(id).is_some_and(|source| {
                    source.quality == DuDecisionEvidence::ExactStructured
                        && source.locator.starts_with(table)
                })
            }) {
                return Err(DecisionDataError::InvalidProvenance);
            }
        }
        result.push(RewardOccurrenceDefinition {
            id: RewardOccurrenceId::new(row.stable_key.clone())?,
            preset_source: row.preset_source.clone().into(),
            level: u16::try_from(row.level).map_err(|_| DecisionDataError::InvalidPolicy)?,
            variant: occurrence.variant.clone(),
            policy: match row.policy {
                DuRewardOccurrencePolicy::SingleExplicitRewardOnlySubstituteNoPoolSampling => {
                    RewardOccurrencePolicy::VersionedProjectPolicySingleExplicitRewardOnlySubstituteNoPoolSampling
                }
            },
            summary_en: row.summary_en.clone().into(),
            summary_zh_cn: row.summary_zh_cn.clone().into(),
            policy_note: row.policy_note.clone().into(),
            replacement_condition: row.replacement_condition.clone().into(),
            sources,
        });
    }
    result.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(result.into_boxed_slice())
}
