//! Mode-owned resource assembly policy, not observed game lifecycle parity.

use crate::divergent_universe_decisions::{DecisionDataError, key, validation::source_keys};
use crate::divergent_universe_decisions_generated::{
    SoraConfig, du_battle_team_resource_policy::DuBattleTeamResourcePolicy,
    du_decision_evidence::DuDecisionEvidence,
};

/// The initial value, field-to-meter mapping and lifecycle are replaceable policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BattleTeamResourcePolicy {
    VersionedProjectPolicyOriginalElationZeroClampPersist,
}

/// Immutable authoring lowered without exposing generated transport records.
/// Battle composition must evaluate the original mapped roster, never live actors.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BattleTeamResourceDefinition {
    pub key: Box<str>,
    pub resource_key: Box<str>,
    pub initial_value: u16,
    pub maximum_value: u16,
    pub policy: BattleTeamResourcePolicy,
    pub summary_en: Box<str>,
    pub summary_zh_cn: Box<str>,
    pub policy_note: Box<str>,
    pub replacement_condition: Box<str>,
    pub sources: Box<[Box<str>]>,
}

pub(super) fn compile(
    config: &SoraConfig,
) -> Result<Box<[BattleTeamResourceDefinition]>, DecisionDataError> {
    let mut definitions = Vec::new();
    for row in config.du_battle_team_resources().ordered_rows() {
        if row.id <= 0
            || row.resource_key != "shared.punchline"
            || row
                .character_paths
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>()
                != ["Elation"]
        {
            return Err(DecisionDataError::InvalidReference);
        }
        if row.initial_value != 0 || row.maximum_value != 9999 {
            return Err(DecisionDataError::InvalidReward);
        }
        if [
            &row.summary_en,
            &row.summary_zh_cn,
            &row.policy_note,
            &row.replacement_condition,
        ]
        .iter()
        .any(|value| value.trim().is_empty())
        {
            return Err(DecisionDataError::InvalidPolicy);
        }
        let sources = source_keys(config, &row.source_ids)?;
        if sources.len() != 3 {
            return Err(DecisionDataError::InvalidProvenance);
        }
        for (prefix, locator, digest) in [
            (
                "Config/GlobalConfig/GameCoreConstValue.json;",
                "ElationPointMax=9999",
                "5511ff36c631da99925c8aadae8ae46f50d620f4f937dbbed952fd61b16b80e3",
            ),
            (
                "TextMap/TextMapEN.json;",
                "hash=8389201339365092983;",
                "afc6d0ff9eeb20d09042e61c311e60d4ed56e69757f1ac42466ca4840e6ed789",
            ),
            (
                "TextMap/TextMapCHS.json;",
                "hash=8389201339365092983;",
                "ec49e07932b4ed8aba9679e247d8e554ca69e831fea7931d94682551de567147",
            ),
        ] {
            if !row.source_ids.iter().any(|id| {
                config.du_decision_sources().get(id).is_some_and(|source| {
                    source.quality == DuDecisionEvidence::ExactStructured
                        && source.url == "https://gitlab.com/Dimbreath/turnbasedgamedata"
                        && source.revision == "fd978d6ef09f941fba644c731ab54abd6f7c3568"
                        && source.game_version == "4.4"
                        && source.access_date == "2026-10-02"
                        && source.sha256 == digest
                        && source.locator.starts_with(prefix)
                        && source.locator.contains(locator)
                })
            }) {
                return Err(DecisionDataError::InvalidProvenance);
            }
        }
        definitions.push(BattleTeamResourceDefinition {
            key: key(&row.stable_key)?,
            resource_key: row.resource_key.clone().into(),
            initial_value: u16::try_from(row.initial_value)
                .map_err(|_| DecisionDataError::InvalidReward)?,
            maximum_value: u16::try_from(row.maximum_value)
                .map_err(|_| DecisionDataError::InvalidReward)?,
            policy: match row.policy {
                DuBattleTeamResourcePolicy::OriginalElationZeroClampPersist => {
                    BattleTeamResourcePolicy::VersionedProjectPolicyOriginalElationZeroClampPersist
                }
            },
            summary_en: row.summary_en.clone().into(),
            summary_zh_cn: row.summary_zh_cn.clone().into(),
            policy_note: row.policy_note.clone().into(),
            replacement_condition: row.replacement_condition.clone().into(),
            sources,
        });
    }
    if definitions.len() != 1 {
        return Err(DecisionDataError::InvalidReference);
    }
    Ok(definitions.into_boxed_slice())
}
