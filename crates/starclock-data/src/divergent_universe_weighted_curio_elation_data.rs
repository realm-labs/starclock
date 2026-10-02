//! Sapient Pen's released operands and reviewed mode-owned execution policy.

use crate::{
    divergent_universe::DivergentUniverseBundleCandidate,
    divergent_universe_curio_catalog::DivergentUniverseWeightedCurioId,
    divergent_universe_decisions::{DecisionDataError, key, validation::source_keys},
    divergent_universe_decisions_generated::{
        SoraConfig, du_curio_elation_policy::DuCurioElationPolicy,
        du_decision_evidence::DuDecisionEvidence,
    },
};

/// Authored choices bound by the mode's command-level execution fixtures.
/// This is not an observed original-game scheduling contract.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WeightedCurioElationPolicy {
    VersionedProjectPolicyActionResolvedOriginalPartyRefresh,
}

/// Exact current operands, independent from shared-meter assembly and execution.
/// Loading this definition does not make its equipment admissible to a battle.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WeightedCurioElationDefinition {
    pub key: Box<str>,
    pub weighted_curio: DivergentUniverseWeightedCurioId,
    pub maze_buff_id: Box<str>,
    pub punchline_gain: u16,
    pub elation_bonus_millionths: i64,
    pub duration_turns: u16,
    pub policy: WeightedCurioElationPolicy,
    pub summary_en: Box<str>,
    pub summary_zh_cn: Box<str>,
    pub policy_note: Box<str>,
    pub replacement_condition: Box<str>,
    pub sources: Box<[Box<str>]>,
}

pub(super) fn compile(
    config: &SoraConfig,
    reference: &DivergentUniverseBundleCandidate,
) -> Result<Box<[WeightedCurioElationDefinition]>, DecisionDataError> {
    let mut definitions = Vec::new();
    for row in config.du_weighted_curio_elations().ordered_rows() {
        let curio = reference
            .curio_catalog()
            .weighted_curios()
            .iter()
            .find(|curio| curio.id.as_str() == row.weighted_curio_key)
            .ok_or(DecisionDataError::InvalidReference)?;
        let eligible = reference
            .curio_catalog()
            .weighted_curio_eligibility()
            .iter()
            .find(|eligible| curio.eligibility_rules.contains(&eligible.id))
            .ok_or(DecisionDataError::InvalidReference)?;
        if row.id <= 0
            || curio.id.as_str() != "divergent-universe.weighted-curio.1015"
            || row.maze_buff_id != "633415"
            || curio.maze_buff_id.as_ref() != row.maze_buff_id
            || row
                .character_paths
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>()
                != ["Elation"]
            || eligible
                .character_paths
                .iter()
                .map(AsRef::as_ref)
                .collect::<Vec<&str>>()
                != ["Elation"]
            || !eligible.elements.is_empty()
            || (
                row.gain_parameter,
                row.bonus_parameter,
                row.duration_parameter,
            ) != (1, 2, 3)
        {
            return Err(DecisionDataError::InvalidReference);
        }
        if row.punchline_gain != 2 || row.elation_bonus != "0.5" || row.duration_turns != 2 {
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
        if sources.len() != 4 {
            return Err(DecisionDataError::InvalidProvenance);
        }
        for (prefix, locator, digest) in [
            (
                "ExcelOutput/RogueTournHex.json;",
                "HexID=1015;",
                "51e91a6f53ab0545330e477d806a165108f6c7e5e70ad52b02f92c1dc4d36455",
            ),
            (
                "ExcelOutput/MazeBuff.json;",
                "ID=633415;",
                "2fab98b723ee20d8798c68e200cc9c3704abc347b0983b2955cbd40b85bfeeac",
            ),
            (
                "TextMap/TextMapEN.json;",
                "hash=7976287788830292485;",
                "afc6d0ff9eeb20d09042e61c311e60d4ed56e69757f1ac42466ca4840e6ed789",
            ),
            (
                "TextMap/TextMapCHS.json;",
                "hash=7976287788830292485;",
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
        definitions.push(WeightedCurioElationDefinition {
            key: key(&row.stable_key)?,
            weighted_curio: curio.id.clone(),
            maze_buff_id: row.maze_buff_id.clone().into(),
            punchline_gain: 2,
            elation_bonus_millionths: 500_000,
            duration_turns: 2,
            policy: match row.policy {
                DuCurioElationPolicy::ActionResolvedOriginalPartyRefresh => {
                    WeightedCurioElationPolicy::VersionedProjectPolicyActionResolvedOriginalPartyRefresh
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
