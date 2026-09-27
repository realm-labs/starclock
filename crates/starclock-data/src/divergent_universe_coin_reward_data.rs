//! Reviewed Wealth preset joins with independently replaceable chest policy.

use super::{DecisionDataError, key, validation::source_keys};
use crate::divergent_universe_decisions_generated::{
    SoraConfig, du_coin_reward_policy::DuCoinRewardPolicy,
    du_decision_evidence::DuDecisionEvidence, du_domain_card_kind::DuDomainCardKind,
    du_domain_slot_kind::DuDomainSlotKind,
};
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CoinRewardPolicy {
    VersionedProjectPolicyFixedSingleChestCreditNoAmusementFacilities,
}

/// Canonical positive i64-range aggregate chest amount at one reviewed preset.
/// Level/kind joins are current source facts; amount/count/timing are policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoinRewardDefinition {
    pub key: Box<str>,
    pub preset_source: Box<str>,
    pub level: u16,
    pub amount: u64,
    pub policy: CoinRewardPolicy,
    pub summary_en: Box<str>,
    pub summary_zh_cn: Box<str>,
    pub policy_note: Box<str>,
    pub replacement_condition: Box<str>,
    pub sources: Box<[Box<str>]>,
}

pub(super) fn compile(
    config: &SoraConfig,
) -> Result<Box<[CoinRewardDefinition]>, DecisionDataError> {
    let mut seen = BTreeSet::new();
    let mut result = Vec::new();
    for row in config.du_coin_rewards().ordered_rows() {
        if row.id <= 0 || !seen.insert(&row.preset_source) {
            return Err(DecisionDataError::InvalidIdentity);
        }
        let card = config.du_domain_cards().ordered_rows().any(|card| {
            card.preset_source == row.preset_source
                && card.level == row.level
                && card.kind == DuDomainCardKind::Coin
        });
        let fixed = config.du_domain_layout().ordered_rows().any(|position| {
            position.preset_source.as_ref() == Some(&row.preset_source)
                && position.level == Some(row.level)
                && position.kind == DuDomainSlotKind::Coin
        });
        if !card && !fixed {
            return Err(DecisionDataError::InvalidReference);
        }
        let amount = row
            .amount
            .parse::<u64>()
            .map_err(|_| DecisionDataError::InvalidPolicy)?;
        if amount == 0
            || i64::try_from(amount).is_err()
            || amount.to_string() != row.amount
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
        result.push(CoinRewardDefinition {
            key: key(&row.stable_key)?,
            preset_source: row.preset_source.clone().into(),
            level: u16::try_from(row.level).map_err(|_| DecisionDataError::InvalidPolicy)?,
            amount,
            policy: match row.policy {
                DuCoinRewardPolicy::FixedSingleChestCreditNoAmusementFacilities => {
                    CoinRewardPolicy::VersionedProjectPolicyFixedSingleChestCreditNoAmusementFacilities
                }
            },
            summary_en: row.summary_en.clone().into(),
            summary_zh_cn: row.summary_zh_cn.clone().into(),
            policy_note: row.policy_note.clone().into(),
            replacement_condition: row.replacement_condition.clone().into(),
            sources,
        });
    }
    result.sort_by(|left, right| left.key.cmp(&right.key));
    Ok(result.into_boxed_slice())
}
