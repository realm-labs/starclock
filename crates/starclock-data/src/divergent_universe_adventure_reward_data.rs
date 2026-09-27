//! Reviewed current Adventure card with explicitly external challenge results.

use super::{DecisionDataError, key, validation::source_keys};
use crate::divergent_universe_decisions_generated::{
    SoraConfig, du_adventure_reward_policy::DuAdventureRewardPolicy,
    du_decision_evidence::DuDecisionEvidence, du_domain_card_kind::DuDomainCardKind,
};
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdventureRewardPolicy {
    VersionedProjectPolicyExternalEarnedChestCountAggregateFragmentsOnly,
}

/// Known level-one chest bound plus replaceable positive i64-range base credit.
/// Counts are external inputs, not inferred score tiers or simulated challenges.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdventureRewardDefinition {
    pub key: Box<str>,
    pub preset_source: Box<str>,
    pub level: u16,
    pub maximum_chests: u16,
    pub fragments_per_chest: u64,
    pub policy: AdventureRewardPolicy,
    pub summary_en: Box<str>,
    pub summary_zh_cn: Box<str>,
    pub policy_note: Box<str>,
    pub replacement_condition: Box<str>,
    pub sources: Box<[Box<str>]>,
}

pub(super) fn compile(
    config: &SoraConfig,
) -> Result<Box<[AdventureRewardDefinition]>, DecisionDataError> {
    let mut seen = BTreeSet::new();
    let mut result = Vec::new();
    for row in config.du_adventure_rewards().ordered_rows() {
        if row.id <= 0 || !seen.insert(&row.preset_source) {
            return Err(DecisionDataError::InvalidIdentity);
        }
        if row.level != 1
            || row.maximum_chests != 3
            || !config.du_domain_cards().ordered_rows().any(|card| {
                card.preset_source == row.preset_source
                    && card.level == row.level
                    && card.kind == DuDomainCardKind::Adventure
            })
        {
            return Err(DecisionDataError::InvalidReference);
        }
        let amount = row
            .fragments_per_chest
            .parse::<u64>()
            .map_err(|_| DecisionDataError::InvalidPolicy)?;
        if amount == 0
            || amount.to_string() != row.fragments_per_chest
            || amount
                .checked_mul(3)
                .and_then(|v| i64::try_from(v).ok())
                .is_none()
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
                        && (table != "TextMap/TextMapEN.json;"
                            || source.locator.contains("10649614652576832174"))
                })
            }) {
                return Err(DecisionDataError::InvalidProvenance);
            }
        }
        result.push(AdventureRewardDefinition {
            key: key(&row.stable_key)?, preset_source: row.preset_source.clone().into(),
            level: 1, maximum_chests: 3, fragments_per_chest: amount,
            policy: match row.policy {
                DuAdventureRewardPolicy::ExternalEarnedChestCountAggregateFragmentsOnly =>
                    AdventureRewardPolicy::VersionedProjectPolicyExternalEarnedChestCountAggregateFragmentsOnly,
            },
            summary_en: row.summary_en.clone().into(), summary_zh_cn: row.summary_zh_cn.clone().into(),
            policy_note: row.policy_note.clone().into(), replacement_condition: row.replacement_condition.clone().into(),
            sources,
        });
    }
    result.sort_by(|a, b| a.key.cmp(&b.key));
    Ok(result.into_boxed_slice())
}
