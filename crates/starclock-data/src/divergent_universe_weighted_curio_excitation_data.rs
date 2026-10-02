//! Genius' Confusion operands; loading is not battle execution or admission.

use crate::{
    divergent_universe::DivergentUniverseBundleCandidate,
    divergent_universe_curio_catalog::DivergentUniverseWeightedCurioId,
    divergent_universe_decisions::{DecisionDataError, key, validation::source_keys},
    divergent_universe_decisions_generated::{
        SoraConfig, du_curio_excitation_policy::DuCurioExcitationPolicy,
        du_decision_evidence::DuDecisionEvidence,
    },
};

/// Replaceable authored behavior, not observed original-game scheduling.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WeightedCurioExcitationPolicy {
    VersionedProjectPolicyEffectiveGainActionResolvedTeamConsumption,
}

/// Immutable exact operands and separate policy cap; execution belongs to the mode.
/// Quantum qualification is validated against the current reference eligibility.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WeightedCurioExcitationDefinition {
    pub key: Box<str>,
    pub weighted_curio: DivergentUniverseWeightedCurioId,
    pub maze_buff_id: Box<str>,
    pub stack_gain: u16,
    pub stack_consumption: u16,
    pub attack_multiplier_millionths: i64,
    pub base_chance_millionths: i64,
    pub duration_turns: u16,
    /// Project-policy safety bound, not an operand disclosed by released text.
    pub policy_maximum_stacks: u16,
    pub policy: WeightedCurioExcitationPolicy,
    pub summary_en: Box<str>,
    pub summary_zh_cn: Box<str>,
    pub policy_note: Box<str>,
    pub replacement_condition: Box<str>,
    pub sources: Box<[Box<str>]>,
}

pub(super) fn compile(
    config: &SoraConfig,
    reference: &DivergentUniverseBundleCandidate,
) -> Result<Box<[WeightedCurioExcitationDefinition]>, DecisionDataError> {
    let mut definitions = Vec::new();
    for row in config.du_weighted_curio_excitations().ordered_rows() {
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
            || curio.id.as_str() != "divergent-universe.weighted-curio.1006"
            || row.maze_buff_id != "633406"
            || curio.maze_buff_id.as_ref() != row.maze_buff_id
            || row
                .character_elements
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>()
                != ["Quantum"]
            || eligible
                .elements
                .iter()
                .map(AsRef::as_ref)
                .collect::<Vec<&str>>()
                != ["Quantum"]
            || !eligible.character_paths.is_empty()
            || (
                row.gain_parameter,
                row.consume_parameter,
                row.damage_parameter,
                row.chance_parameter,
                row.duration_parameter,
            ) != (1, 2, 3, 4, 5)
        {
            return Err(DecisionDataError::InvalidReference);
        }
        if (row.stack_gain, row.stack_consumption, row.duration_turns) != (2, 1, 1)
            || row.attack_multiplier != "2.5"
            || row.base_chance != "0.5"
        {
            return Err(DecisionDataError::InvalidReward);
        }
        if row.policy_maximum_stacks != 65535
            || !row.policy_note.starts_with("VersionedProjectPolicy:")
            || [
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
                "HexID=1006;",
                "51e91a6f53ab0545330e477d806a165108f6c7e5e70ad52b02f92c1dc4d36455",
            ),
            (
                "ExcelOutput/MazeBuff.json;",
                "ID=633406;",
                "2fab98b723ee20d8798c68e200cc9c3704abc347b0983b2955cbd40b85bfeeac",
            ),
            (
                "TextMap/TextMapEN.json;",
                "hash=332970425552739522;",
                "afc6d0ff9eeb20d09042e61c311e60d4ed56e69757f1ac42466ca4840e6ed789",
            ),
            (
                "TextMap/TextMapCHS.json;",
                "hash=332970425552739522;",
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
        definitions.push(WeightedCurioExcitationDefinition {
            key: key(&row.stable_key)?,
            weighted_curio: curio.id.clone(),
            maze_buff_id: row.maze_buff_id.clone().into(),
            stack_gain: 2,
            stack_consumption: 1,
            attack_multiplier_millionths: 2_500_000,
            base_chance_millionths: 500_000,
            duration_turns: 1,
            policy_maximum_stacks: 65535,
            policy: match row.policy {
                DuCurioExcitationPolicy::EffectiveGainActionResolvedTeamConsumption => {
                    WeightedCurioExcitationPolicy::VersionedProjectPolicyEffectiveGainActionResolvedTeamConsumption
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
